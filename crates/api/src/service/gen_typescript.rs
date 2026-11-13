//! TypeScript DTO emitter from the shared package schema mapping.
//!
//! TypeScript has no equivalent of Go's compile-time unused-import error,
//! nor even a default `tsc` diagnostic for an unused top-level declaration
//! (confirmed empirically) — but this emitter still tracks which of the
//! three shared helper functions a given file's `check:` expressions
//! actually used ([`TypeScriptHelpers`]), purely to keep generated output
//! free of helpers the KCL schemas don't need, mirroring
//! [`gen_go`](super::gen_go)'s `GoImports` and
//! [`gen_java`](super::gen_java)'s `JavaImports`.
//!
//! Unlike Go/Java, there is no package/file-visibility constraint to work
//! around: a `.ts` module may freely export any number of top-level
//! classes, so every generated class is exported directly and no
//! `--package`-style argument is needed (`generate_typescript` takes only
//! the schema mapping, like `generate_rust`/`generate_python`).
//!
//! Design points
//! - Required fields (no KCL default rendered as a real initializer — see
//!   [`gen_rust`](super::gen_rust)/[`gen_go`](super::gen_go)/
//!   [`gen_java`](super::gen_java), none of which render an actual default
//!   *value* either, only a doc comment) use TypeScript's definite
//!   assignment assertion (`field!: T;`) rather than an initializer, since
//!   a consumer is expected to populate every field right after
//!   construction (e.g. via `Object.assign(new X(), {...})`), the same
//!   assumption Java's `public T field;` (implicitly null) and Go's
//!   zero-valued struct fields already make.
//! - After a `this.field != null` guard, TypeScript's control-flow
//!   narrowing already removes `undefined` from `this.field`'s type within
//!   the rest of the same `&&`-chained condition — unlike Go, no further
//!   dereference/unwrap step is needed in the generated expression.
//! - `**` is TypeScript's own exponentiation operator (ES2016+), so KCL's
//!   `**` needs no helper or cast — simpler here than Rust (`.powf` plus a
//!   cast), Go (`math.Pow` plus casts) or Java (`Math.pow` plus casts).

use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use kcl_ast::ast;

use crate::gpyrpc::{KclType, SchemaTypes};
use crate::service::gen_ident::{is_keyword, to_camel_case, to_pascal_case};
use crate::service::gen_schema::{self, SchemaNameResolver};

/// The three helper functions a generated file might need. `kclContains`
/// (for `in`/`not in`) depends on `kclDeepEqual` internally, so
/// [`mark_contains`](TypeScriptHelpers::mark_contains) also marks
/// `kclDeepEqual` as used — emitting one without the other would be a
/// compile error (an unresolved reference), not just redundant output.
#[derive(Default)]
struct TypeScriptHelpers {
    uses_deep_equal: Cell<bool>,
    uses_contains: Cell<bool>,
    uses_regex: Cell<bool>,
}

impl TypeScriptHelpers {
    fn mark_deep_equal(&self) {
        self.uses_deep_equal.set(true);
    }

    fn mark_contains(&self) {
        self.uses_contains.set(true);
        self.uses_deep_equal.set(true);
    }

    fn mark_regex(&self) {
        self.uses_regex.set(true);
    }
}

const DEEP_EQUAL_HELPER: &str = r#"function kclDeepEqual(a: unknown, b: unknown): boolean {
    if (a === b) {
        return true;
    }
    if (Array.isArray(a) && Array.isArray(b)) {
        return a.length === b.length && a.every((value, index) => kclDeepEqual(value, b[index]));
    }
    if (a !== null && b !== null && typeof a === "object" && typeof b === "object") {
        const aKeys = Object.keys(a as object);
        const bKeys = Object.keys(b as object);
        return (
            aKeys.length === bKeys.length &&
            aKeys.every((key) =>
                kclDeepEqual((a as Record<string, unknown>)[key], (b as Record<string, unknown>)[key])
            )
        );
    }
    return false;
}

"#;

const CONTAINS_HELPER: &str = r#"function kclContains(array: readonly unknown[], value: unknown): boolean {
    return array.some((item) => kclDeepEqual(item, value));
}

"#;

const REGEX_MATCH_HELPER: &str = r#"function kclRegexMatch(pattern: string, value: string): boolean {
    return new RegExp(pattern).test(value);
}

"#;

pub(crate) fn generate_typescript(
    mapping: &HashMap<String, SchemaTypes>,
) -> anyhow::Result<String> {
    let helpers = TypeScriptHelpers::default();
    let body = gen_schema::generate_source(
        mapping,
        "TypeScript",
        ts_type_name,
        |out, name, ty, resolver| emit_schema(out, name, ty, resolver, &helpers),
    )?;

    let mut out = String::new();
    if helpers.uses_deep_equal.get() {
        out.push_str(DEEP_EQUAL_HELPER);
    }
    if helpers.uses_contains.get() {
        out.push_str(CONTAINS_HELPER);
    }
    if helpers.uses_regex.get() {
        out.push_str(REGEX_MATCH_HELPER);
    }
    out.push_str(&body);
    Ok(out)
}

fn emit_schema(
    out: &mut String,
    name: &str,
    ty: &KclType,
    resolver: &SchemaNameResolver,
    helpers: &TypeScriptHelpers,
) -> anyhow::Result<()> {
    for line in ty.schema_doc.lines() {
        out.push_str("//");
        if !line.is_empty() {
            out.push(' ');
            out.push_str(line);
        }
        out.push('\n');
    }
    out.push_str(&format!("export class {} {{\n", ts_type_name(name)));

    let mut properties: Vec<&String> = ty.properties.keys().collect();
    properties.sort();
    let mut used_field_names = HashSet::new();
    for property in &properties {
        let field = &ty.properties[*property];
        if field.r#type == "function" {
            anyhow::bail!("TypeScript DTO generation does not support function field {property:?}");
        }
        if !field.description.is_empty() {
            for line in field.description.lines() {
                out.push_str(&format!("    // {line}\n"));
            }
        }
        if !field.default.is_empty() {
            for line in field.default.lines() {
                out.push_str(&format!("    // KCL default: {line}\n"));
            }
        }
        let field_name = ts_field_name(property);
        if !used_field_names.insert(field_name.clone()) {
            anyhow::bail!("TypeScript field name collision after normalization: {field_name:?}");
        }
        if field_name != **property {
            out.push_str(&format!("    // KCL name: {property}\n"));
        }
        let optional = !ty.required.iter().any(|required| required == *property);
        let field_ty = ts_type(field, resolver, property)?;
        if optional {
            out.push_str(&format!("    {field_name}?: {field_ty};\n"));
        } else {
            out.push_str(&format!("    {field_name}!: {field_ty};\n"));
        }
    }
    out.push('\n');
    emit_validator(out, ty, helpers)?;
    out.push_str("}\n");
    Ok(())
}

fn emit_validator(
    out: &mut String,
    ty: &KclType,
    helpers: &TypeScriptHelpers,
) -> anyhow::Result<()> {
    let fields = ty
        .properties
        .iter()
        .map(|(name, field)| {
            (
                name.clone(),
                (
                    ts_field_name(name),
                    field.clone(),
                    !ty.required.contains(name),
                ),
            )
        })
        .collect::<HashMap<_, _>>();

    out.push_str("    validate(): string[] {\n");
    out.push_str("        const errors: string[] = [];\n");
    for check in gen_schema::decode_checks(ty)? {
        out.push_str(&render_check(&check, &fields, helpers)?);
    }
    out.push_str("        return errors;\n");
    out.push_str("    }\n");
    Ok(())
}

fn render_check(
    check: &ast::CheckExpr,
    fields: &HashMap<String, (String, KclType, bool)>,
    helpers: &TypeScriptHelpers,
) -> anyhow::Result<String> {
    let context = TypeScriptExprContext {
        fields,
        locals: HashMap::new(),
        helpers,
    };
    let condition = check
        .if_cond
        .as_ref()
        .map(|condition| render_condition(&condition.node, &context))
        .transpose()?;
    let test = render_expr(&check.test.node, &context)?;
    let message = check
        .msg
        .as_ref()
        .map(|message| render_expr(&message.node, &context))
        .transpose()?
        .unwrap_or_else(|| gen_schema::default_check_message(&test));
    let condition = match condition {
        Some(condition) => format!("{condition} && !({test})"),
        None => format!("!({test})"),
    };
    Ok(format!(
        "        if ({condition}) {{\n            errors.push({message});\n        }}\n"
    ))
}

struct TypeScriptExprContext<'a> {
    fields: &'a HashMap<String, (String, KclType, bool)>,
    locals: HashMap<String, (String, KclType)>,
    helpers: &'a TypeScriptHelpers,
}

/// Render a KCL `if` guard. A bare optional-field identifier (the common
/// `if field` guard pattern) becomes an explicit loose-inequality check
/// against `null`, which also catches `undefined` — the actual runtime
/// value of a `field?: T` property that was never assigned.
fn render_condition(
    expr: &ast::Expr,
    context: &TypeScriptExprContext<'_>,
) -> anyhow::Result<String> {
    if let ast::Expr::Identifier(identifier) = expr
        && identifier.names.len() == 1
    {
        let name = &identifier.names[0].node;
        if let Some((field_name, _, true)) = context.fields.get(name) {
            return Ok(format!("this.{field_name} != null"));
        }
    }
    render_expr(expr, context)
}

fn render_expr(expr: &ast::Expr, context: &TypeScriptExprContext<'_>) -> anyhow::Result<String> {
    Ok(match expr {
        ast::Expr::Identifier(identifier) => render_identifier(identifier, context)?,
        ast::Expr::Target(target) => {
            if !target.paths.is_empty() {
                anyhow::bail!("indexed KCL targets are not supported in TypeScript validation")
            }
            render_field(target.name.node.as_str(), context)?
        }
        ast::Expr::NumberLit(number) => {
            if number.binary_suffix.is_some() {
                anyhow::bail!("number multipliers are not supported in TypeScript validation")
            }
            number.to_string()
        }
        ast::Expr::StringLit(string) => format!("{:?}", string.value),
        ast::Expr::NameConstantLit(value) => match value.value {
            ast::NameConstant::True => "true".to_string(),
            ast::NameConstant::False => "false".to_string(),
            ast::NameConstant::None => "null".to_string(),
            ast::NameConstant::Undefined => {
                anyhow::bail!("Undefined is not supported in TypeScript validation")
            }
        },
        ast::Expr::Paren(paren) => format!("({})", render_expr(&paren.expr.node, context)?),
        ast::Expr::Unary(unary) => {
            let op = match unary.op {
                ast::UnaryOp::Not => "!",
                ast::UnaryOp::USub => "-",
                ast::UnaryOp::UAdd => "+",
                ast::UnaryOp::Invert => {
                    anyhow::bail!("bitwise inversion is not supported in TypeScript validation")
                }
            };
            format!("{op}({})", render_expr(&unary.operand.node, context)?)
        }
        ast::Expr::Binary(binary) => {
            let left = render_expr(&binary.left.node, context)?;
            let right = render_expr(&binary.right.node, context)?;
            match binary.op {
                ast::BinOp::Add => format!("({left} + {right})"),
                ast::BinOp::Sub => format!("({left} - {right})"),
                ast::BinOp::Mul => format!("({left} * {right})"),
                ast::BinOp::Div => format!("({left} / {right})"),
                ast::BinOp::Mod => format!("({left} % {right})"),
                ast::BinOp::And => format!("({left} && {right})"),
                ast::BinOp::Or => format!("({left} || {right})"),
                ast::BinOp::Pow => format!("(({left}) ** ({right}))"),
                _ => anyhow::bail!(
                    "KCL operator {:?} is not supported in TypeScript validation",
                    binary.op
                ),
            }
        }
        ast::Expr::Compare(compare) => render_compare(compare, context)?,
        ast::Expr::Selector(selector) => {
            let receiver = render_expr(&selector.value.node, context)?;
            let attr = selector.attr.node.get_name();
            if attr == "upper" {
                format!("{receiver}.toUpperCase()")
            } else {
                format!("{receiver}.{}", ts_field_name(&attr))
            }
        }
        ast::Expr::Call(call) => render_call(call, context)?,
        ast::Expr::Quant(quant) => render_quant(quant, context)?,
        other => anyhow::bail!(
            "KCL expression {} is not supported in TypeScript validation",
            other.get_expr_name()
        ),
    })
}

fn render_identifier(
    identifier: &ast::Identifier,
    context: &TypeScriptExprContext<'_>,
) -> anyhow::Result<String> {
    let names = identifier.get_names();
    let name = names
        .first()
        .ok_or_else(|| anyhow::anyhow!("empty identifier in TypeScript validation"))?;
    if let Some((local, _)) = context.locals.get(name) {
        return Ok(local.clone());
    }
    if context.fields.contains_key(name) {
        let mut result = render_field(name, context)?;
        for member in names.iter().skip(1) {
            result.push('.');
            result.push_str(&ts_field_name(member));
        }
        return Ok(result);
    }
    anyhow::bail!("unknown identifier {name:?} in TypeScript validation")
}

fn render_field(name: &str, context: &TypeScriptExprContext<'_>) -> anyhow::Result<String> {
    let (field_name, _, _) = context
        .fields
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("unknown identifier {name:?} in TypeScript validation"))?;
    Ok(format!("this.{field_name}"))
}

fn render_compare(
    compare: &ast::Compare,
    context: &TypeScriptExprContext<'_>,
) -> anyhow::Result<String> {
    if compare.ops.len() != compare.comparators.len() {
        anyhow::bail!("malformed KCL comparison in TypeScript validation")
    }
    let mut left_expr = &compare.left.node;
    let mut comparisons = Vec::new();
    for (op, comparator) in compare.ops.iter().zip(&compare.comparators) {
        let left = render_expr(left_expr, context)?;
        let right = render_expr(&comparator.node, context)?;
        let comparison = match op {
            // Routed through `kclDeepEqual` rather than `===`/`!==` for
            // every operand type, not just arrays/objects: JS reference
            // equality on two schema instances (or two arrays) with equal
            // *content* would otherwise be `false`, the same reference-vs-
            // value pitfall Java's raw `==` had before `Objects.equals`.
            ast::CmpOp::Eq => {
                context.helpers.mark_deep_equal();
                format!("kclDeepEqual({left}, {right})")
            }
            ast::CmpOp::NotEq => {
                context.helpers.mark_deep_equal();
                format!("!kclDeepEqual({left}, {right})")
            }
            ast::CmpOp::Lt => format!("{left} < {right}"),
            ast::CmpOp::LtE => format!("{left} <= {right}"),
            ast::CmpOp::Gt => format!("{left} > {right}"),
            ast::CmpOp::GtE => format!("{left} >= {right}"),
            ast::CmpOp::In => {
                context.helpers.mark_contains();
                format!("kclContains({right}, {left})")
            }
            ast::CmpOp::NotIn => {
                context.helpers.mark_contains();
                format!("!kclContains({right}, {left})")
            }
            ast::CmpOp::Is | ast::CmpOp::IsNot => {
                if let ast::Expr::NameConstantLit(value) = &comparator.node
                    && value.value == ast::NameConstant::None
                    && optional_field_for_expr(&compare.left.node, context).is_some()
                {
                    let cmp_op = if *op == ast::CmpOp::Is { "==" } else { "!=" };
                    format!("{left} {cmp_op} null")
                } else {
                    anyhow::bail!("KCL identity checks are only supported against None")
                }
            }
            _ => anyhow::bail!(
                "KCL comparison {:?} is not supported in TypeScript validation",
                op
            ),
        };
        comparisons.push(comparison);
        left_expr = &comparator.node;
    }
    Ok(comparisons.join(" && "))
}

fn optional_field_for_expr<'a>(
    expr: &ast::Expr,
    context: &'a TypeScriptExprContext<'_>,
) -> Option<&'a (String, KclType, bool)> {
    let ast::Expr::Identifier(identifier) = expr else {
        return None;
    };
    if identifier.names.len() != 1 {
        return None;
    }
    context
        .fields
        .get(&identifier.names[0].node)
        .filter(|(_, _, optional)| *optional)
}

fn render_call(
    call: &ast::CallExpr,
    context: &TypeScriptExprContext<'_>,
) -> anyhow::Result<String> {
    let function = match &call.func.node {
        ast::Expr::Identifier(identifier) => identifier.get_name(),
        ast::Expr::Selector(selector) => {
            let receiver = match &selector.value.node {
                ast::Expr::Identifier(identifier) => identifier.get_name(),
                _ => String::new(),
            };
            format!("{receiver}.{}", selector.attr.node.get_name())
        }
        _ => String::new(),
    };
    if call.args.is_empty() && function.ends_with(".upper") {
        let receiver = match &call.func.node {
            ast::Expr::Identifier(identifier) => {
                let mut receiver = identifier.clone();
                receiver.names.pop();
                render_expr(&ast::Expr::Identifier(receiver), context)?
            }
            ast::Expr::Selector(selector) => render_expr(&selector.value.node, context)?,
            _ => anyhow::bail!("unsupported receiver for upper() in TypeScript validation"),
        };
        return Ok(format!("{receiver}.toUpperCase()"));
    }
    match function.as_str() {
        "len" if call.args.len() == 1 => {
            let arg = &call.args[0].node;
            let value = render_expr(arg, context)?;
            match expression_type_tag(arg, context) {
                // A JS/TS string is a sequence of UTF-16 code units, so
                // `.length` would miscount any character outside the Basic
                // Multilingual Plane — spreading through `Array.from`
                // iterates by Unicode code point instead, matching KCL's
                // (and Rust's/Go's/Java's) `len()` semantics on a string.
                Some("str") => Ok(format!("Array.from({value}).length")),
                // A `Record<K, V>` has no `.length` of its own; count its
                // own enumerable keys instead.
                Some("dict") => Ok(format!("Object.keys({value}).length")),
                _ => Ok(format!("({value}).length")),
            }
        }
        "regex.match" if call.args.len() == 2 => {
            let value = render_expr(&call.args[0].node, context)?;
            let pattern = render_expr(&call.args[1].node, context)?;
            context.helpers.mark_regex();
            Ok(format!("kclRegexMatch({pattern}, {value})"))
        }
        _ => {
            if let ast::Expr::Selector(selector) = &call.func.node
                && selector.attr.node.get_name() == "upper"
                && call.args.is_empty()
            {
                return Ok(format!(
                    "{}.toUpperCase()",
                    render_expr(&selector.value.node, context)?
                ));
            }
            anyhow::bail!("KCL call {function:?} is not supported in TypeScript validation")
        }
    }
}

/// The KCL type tag (`"str"`, `"dict"`, ...) of a `len()` argument, looked
/// up through locals first (quantifier loop variables) then fields —
/// `render_call`'s `len` branch uses this to choose between
/// `Array.from(...).length` (string), `Object.keys(...).length` (dict) and
/// a plain `.length` (array).
fn expression_type_tag<'a>(
    expr: &ast::Expr,
    context: &'a TypeScriptExprContext<'_>,
) -> Option<&'a str> {
    match expr {
        ast::Expr::StringLit(_) => Some("str"),
        ast::Expr::Identifier(identifier) => identifier.names.first().and_then(|name| {
            context
                .locals
                .get(&name.node)
                .map(|(_, ty)| ty.r#type.as_str())
                .or_else(|| {
                    context
                        .fields
                        .get(&name.node)
                        .map(|(_, ty, _)| ty.r#type.as_str())
                })
        }),
        _ => None,
    }
}

/// Render an `all`/`any` quantifier using `Array.prototype.every`/`.some`
/// — already expressions in JS/TS, unlike Go's statement-only closures
/// (which need an immediately-invoked-function-expression wrapper) and
/// simpler even than Java's `Stream.allMatch`/`anyMatch` (no
/// `.stream()` conversion needed first).
fn render_quant(
    quant: &ast::QuantExpr,
    context: &TypeScriptExprContext<'_>,
) -> anyhow::Result<String> {
    if quant.variables.len() != 1 {
        anyhow::bail!("TypeScript validation supports quantifiers with one variable")
    }
    let target = render_expr(&quant.target.node, context)?;
    let variable = quant.variables[0].node.get_name();
    let ts_variable = ts_field_name(&variable);
    let item_type = match &quant.target.node {
        ast::Expr::Identifier(identifier) => identifier
            .names
            .first()
            .and_then(|name| context.fields.get(&name.node))
            .and_then(|(_, ty, _)| ty.item.as_deref())
            .cloned()
            .unwrap_or_default(),
        _ => KclType::default(),
    };
    let mut nested = TypeScriptExprContext {
        fields: context.fields,
        locals: context.locals.clone(),
        helpers: context.helpers,
    };
    nested
        .locals
        .insert(variable.clone(), (ts_variable.clone(), item_type));
    let test = render_expr(&quant.test.node, &nested)?;
    let body = if let Some(condition) = &quant.if_cond {
        let condition = render_expr(&condition.node, &nested)?;
        match quant.op {
            ast::QuantOperation::All => format!("!({condition}) || ({test})"),
            ast::QuantOperation::Any => format!("({condition}) && ({test})"),
            _ => anyhow::bail!(
                "conditional map/filter quantifiers are not supported in TypeScript validation"
            ),
        }
    } else {
        test
    };
    let method = match quant.op {
        ast::QuantOperation::All => "every",
        ast::QuantOperation::Any => "some",
        _ => anyhow::bail!("only all/any quantifiers are supported in TypeScript validation"),
    };
    Ok(format!("{target}.{method}(({ts_variable}) => {body})"))
}

fn ts_type(ty: &KclType, resolver: &SchemaNameResolver, field: &str) -> anyhow::Result<String> {
    Ok(match ty.r#type.as_str() {
        "str" => "string".to_string(),
        "int" | "number_multiplier" | "float" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "any" | "union" => "any".to_string(),
        "schema" => resolver.name_of_ty(ty),
        "list" => {
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("list field {field:?} has no item type"))?;
            format!("{}[]", ts_type(item, resolver, field)?)
        }
        "dict" => {
            let key = ty
                .key
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("dict field {field:?} has no key type"))?;
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("dict field {field:?} has no value type"))?;
            format!(
                "Record<{}, {}>",
                ts_type(key, resolver, field)?,
                ts_type(item, resolver, field)?
            )
        }
        other => anyhow::bail!("unsupported KCL type {other:?} for TypeScript field {field:?}"),
    })
}

fn ts_type_name(name: &str) -> String {
    to_pascal_case(name)
}

fn ts_field_name(name: &str) -> String {
    let result = to_camel_case(name);
    if is_keyword(&result, TS_KEYWORDS) {
        format!("{result}_")
    } else {
        result
    }
}

const TS_KEYWORDS: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    // Strict-mode / module-scope reserved words. KCL schemas are always
    // compiled to ES module syntax (`export class ...`), which is
    // implicitly strict mode, so these are reserved wherever our output
    // runs even though they aren't reserved in old-style sloppy-mode
    // scripts.
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "await",
    // Not reserved words at all, but each would collide with a name this
    // emitter itself declares or relies on, so escaping them here avoids a
    // confusing (if technically legal) shadowing field.
    "constructor",
    "validate",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn schema(name: &str, pkg: &str) -> KclType {
        KclType {
            r#type: "schema".to_string(),
            schema_name: name.to_string(),
            pkg_path: pkg.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn emits_deterministic_classes_with_optional_and_nested_types() {
        let mut address = schema("Address", "main");
        address.properties.insert(
            "zipCode".to_string(),
            KclType {
                r#type: "str".to_string(),
                description: "Postal code".to_string(),
                ..Default::default()
            },
        );
        address.required.push("zipCode".to_string());

        let mut customer = schema("Customer", "main");
        customer.properties.insert(
            "addresses".to_string(),
            KclType {
                r#type: "list".to_string(),
                item: Some(Box::new(schema("Address", "main"))),
                ..Default::default()
            },
        );
        customer.properties.insert(
            "age".to_string(),
            KclType {
                r#type: "int".to_string(),
                ..Default::default()
            },
        );
        customer.required.push("addresses".to_string());

        let mapping = HashMap::from([(
            "main".to_string(),
            SchemaTypes {
                schema_type: vec![address, customer],
            },
        )]);
        let generated = generate_typescript(&mapping).unwrap();

        assert!(generated.contains("export class Customer {"));
        assert!(generated.contains("addresses!: Address[];"));
        assert!(generated.contains("age?: number;"));
        assert!(generated.contains("zipCode!: string;"));
        assert!(
            generated.find("export class Address").unwrap()
                < generated.find("export class Customer").unwrap()
        );
        // Neither schema has a `check:` block, so none of the three
        // tracked helpers should be emitted.
        assert!(!generated.contains("function kclDeepEqual"));
        assert!(!generated.contains("function kclContains"));
        assert!(!generated.contains("function kclRegexMatch"));
    }

    #[test]
    fn rejects_fields_that_normalize_to_the_same_typescript_identifier() {
        let mut ty = schema("Collision", "main");
        ty.properties.insert(
            "zipCode".to_string(),
            KclType {
                r#type: "str".to_string(),
                ..Default::default()
            },
        );
        ty.properties.insert(
            "ZipCode".to_string(),
            KclType {
                r#type: "str".to_string(),
                ..Default::default()
            },
        );
        let mapping = HashMap::from([(
            "main".to_string(),
            SchemaTypes {
                schema_type: vec![ty],
            },
        )]);
        let error = generate_typescript(&mapping).unwrap_err();
        assert!(error.to_string().contains("field name collision"));
    }

    #[test]
    fn escapes_reserved_words_as_suffixed_field_identifiers() {
        assert_eq!(ts_field_name("class"), "class_");
        assert_eq!(ts_field_name("zipCode"), "zipCode");
    }
}
