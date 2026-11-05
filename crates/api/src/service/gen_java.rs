//! Java DTO emitter from the shared package schema mapping.
//!
//! Unlike Go, an unused Java import is not a compile error — `javac`
//! silently ignores it (confirmed empirically — it is not even a lint
//! warning by default) — but this emitter still tracks which optional
//! pieces a given file's fields/`check:` expressions actually used
//! ([`JavaImports`]), purely to keep generated output free of an import or
//! helper class the KCL schemas don't need, mirroring
//! [`gen_go`](super::gen_go)'s `GoImports`.
//!
//! Java also imposes a constraint Go/Rust/Python don't have: a single
//! `.java` *file* may declare at most one `public` top-level type, and that
//! type's name must match the filename. Since this emitter has no control
//! over what filename a caller saves its output under, every generated
//! class is package-private (no `public` modifier) so the file compiles
//! under any filename.
//!
//! The `KclRegexUtil` helper is extendable and overridable on purpose: its
//! `match` method is an *instance* method (not `static`, which Java can
//! only hide, never override), dispatched through a reassignable
//! `KclRegexUtil.INSTANCE` field — the same pluggable-backend idiom used by
//! e.g. SLF4J. A consumer can swap in custom matching behavior globally by
//! assigning `KclRegexUtil.INSTANCE` to a subclass instance that overrides
//! `match`, without regenerating or editing the DTO classes themselves.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use kcl_ast::ast;

use crate::gpyrpc::{KclType, SchemaTypes};
use crate::service::gen_ident::{is_keyword, to_camel_case, to_pascal_case};
use crate::service::gen_schema::{self, SchemaNameResolver};

/// Optional pieces of a generated file: the `java.util.Map` import (needed
/// only if some field is a KCL `dict`) and the `KclRegexUtil` helper class
/// (needed only if some `check:` uses `regex.match`). Both default to
/// unused and are marked on demand while rendering field types and check
/// expressions, mirroring [`gen_go`](super::gen_go)'s `GoImports`.
#[derive(Default)]
struct JavaImports {
    uses_map: Cell<bool>,
    uses_regex: Cell<bool>,
}

impl JavaImports {
    fn mark_map(&self) {
        self.uses_map.set(true);
    }

    fn mark_regex(&self) {
        self.uses_regex.set(true);
    }
}

const REGEX_UTIL_HELPER: &str = "class KclRegexUtil {\n    // Overridable: assign a subclass instance (e.g.\n    // `KclRegexUtil.INSTANCE = new KclRegexUtil() { ... };`) to customize\n    // regex matching for every generated `validate()` method.\n    static KclRegexUtil INSTANCE = new KclRegexUtil();\n\n    boolean match(String pattern, String value) {\n        return java.util.regex.Pattern.compile(pattern).matcher(value).find();\n    }\n}\n\n";

pub(crate) fn generate_java(
    mapping: &HashMap<String, SchemaTypes>,
    package: &str,
) -> anyhow::Result<String> {
    let imports = JavaImports::default();
    let body = gen_schema::generate_source(
        mapping,
        "Java",
        java_type_name,
        |out, name, ty, resolver| emit_schema(out, name, ty, resolver, &imports),
    )?;

    let mut out = String::new();
    if !package.is_empty() {
        out.push_str(&format!("package {package};\n\n"));
    }
    out.push_str("import java.util.ArrayList;\n");
    out.push_str("import java.util.List;\n");
    if imports.uses_map.get() {
        out.push_str("import java.util.Map;\n");
    }
    out.push_str("import java.util.Objects;\n\n");
    if imports.uses_regex.get() {
        out.push_str(REGEX_UTIL_HELPER);
    }
    out.push_str(&body);
    Ok(out)
}

fn emit_schema(
    out: &mut String,
    name: &str,
    ty: &KclType,
    resolver: &SchemaNameResolver,
    imports: &JavaImports,
) -> anyhow::Result<()> {
    for line in ty.schema_doc.lines() {
        out.push_str("//");
        if !line.is_empty() {
            out.push(' ');
            out.push_str(line);
        }
        out.push('\n');
    }
    out.push_str(&format!("class {} {{\n", java_type_name(name)));

    let mut properties: Vec<&String> = ty.properties.keys().collect();
    properties.sort();
    let mut used_field_names = HashSet::new();
    let mut fields = Vec::with_capacity(properties.len());
    for property in &properties {
        let field = &ty.properties[*property];
        if field.r#type == "function" {
            anyhow::bail!("Java DTO generation does not support function field {property:?}");
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
        let field_name = java_field_name(property);
        if !used_field_names.insert(field_name.clone()) {
            anyhow::bail!("Java field name collision after normalization: {field_name:?}");
        }
        if field_name != **property {
            out.push_str(&format!("    // KCL name: {property}\n"));
        }
        let optional = !ty.required.iter().any(|required| required == *property);
        let field_ty = java_type(field, resolver, property, optional, imports)?;
        out.push_str(&format!("    public {field_ty} {field_name};\n"));
        fields.push(field_name);
    }
    out.push('\n');
    emit_equals_and_hash_code(out, name, &fields);
    emit_validator(out, ty, imports)?;
    out.push_str("}\n");
    Ok(())
}

/// `equals`/`hashCode` are generated for every schema (mirroring Rust's
/// `derive(PartialEq)` and Python's dataclass auto-equality) so that
/// `List.contains(...)` — used to render KCL's `in`/`not in` checks —
/// compares schema instances structurally rather than by reference.
fn emit_equals_and_hash_code(out: &mut String, name: &str, fields: &[String]) {
    let class = java_type_name(name);
    out.push_str("    @Override\n");
    out.push_str("    public boolean equals(Object o) {\n");
    out.push_str("        if (this == o) return true;\n");
    out.push_str(&format!(
        "        if (!(o instanceof {class})) return false;\n"
    ));
    out.push_str(&format!("        {class} other = ({class}) o;\n"));
    if fields.is_empty() {
        out.push_str("        return true;\n");
    } else {
        out.push_str("        return ");
        let comparisons: Vec<String> = fields
            .iter()
            .map(|field| format!("Objects.equals(this.{field}, other.{field})"))
            .collect();
        out.push_str(&comparisons.join("\n            && "));
        out.push_str(";\n");
    }
    out.push_str("    }\n\n");
    out.push_str("    @Override\n");
    out.push_str("    public int hashCode() {\n");
    out.push_str(&format!(
        "        return Objects.hash({});\n",
        fields.join(", ")
    ));
    out.push_str("    }\n\n");
}

fn emit_validator(out: &mut String, ty: &KclType, imports: &JavaImports) -> anyhow::Result<()> {
    let fields = ty
        .properties
        .iter()
        .map(|(name, field)| {
            (
                name.clone(),
                (
                    java_field_name(name),
                    field.clone(),
                    !ty.required.contains(name),
                ),
            )
        })
        .collect::<HashMap<_, _>>();

    out.push_str("    public List<String> validate() {\n");
    out.push_str("        List<String> errors = new ArrayList<>();\n");
    for check in gen_schema::decode_checks(ty)? {
        out.push_str(&render_check(&check, &fields, imports)?);
    }
    out.push_str("        return errors;\n");
    out.push_str("    }\n");
    Ok(())
}

fn render_check(
    check: &ast::CheckExpr,
    fields: &HashMap<String, (String, KclType, bool)>,
    imports: &JavaImports,
) -> anyhow::Result<String> {
    let context = JavaExprContext {
        fields,
        locals: HashMap::new(),
        imports,
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
        "        if ({condition}) {{\n            errors.add({message});\n        }}\n"
    ))
}

struct JavaExprContext<'a> {
    fields: &'a HashMap<String, (String, KclType, bool)>,
    locals: HashMap<String, (String, KclType)>,
    imports: &'a JavaImports,
}

/// Render a KCL `if` guard. A bare optional-field identifier (the common
/// `if field` guard pattern) becomes an explicit non-null check. Unlike Go,
/// no corresponding *dereference* step is needed afterward: Java's boxed
/// wrapper types (`Long`, `Double`, `Boolean`) auto-unbox transparently
/// wherever a primitive is expected, so `this.age >= 18` after a
/// `this.age != null` guard just works without any special-cased rendering.
fn render_condition(expr: &ast::Expr, context: &JavaExprContext<'_>) -> anyhow::Result<String> {
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

fn render_expr(expr: &ast::Expr, context: &JavaExprContext<'_>) -> anyhow::Result<String> {
    Ok(match expr {
        ast::Expr::Identifier(identifier) => render_identifier(identifier, context)?,
        ast::Expr::Target(target) => {
            if !target.paths.is_empty() {
                anyhow::bail!("indexed KCL targets are not supported in Java validation")
            }
            render_field(target.name.node.as_str(), context)?
        }
        ast::Expr::NumberLit(number) => {
            if number.binary_suffix.is_some() {
                anyhow::bail!("number multipliers are not supported in Java validation")
            }
            number.to_string()
        }
        ast::Expr::StringLit(string) => format!("{:?}", string.value),
        ast::Expr::NameConstantLit(value) => match value.value {
            ast::NameConstant::True => "true".to_string(),
            ast::NameConstant::False => "false".to_string(),
            ast::NameConstant::None => "null".to_string(),
            ast::NameConstant::Undefined => {
                anyhow::bail!("Undefined is not supported in Java validation")
            }
        },
        ast::Expr::Paren(paren) => format!("({})", render_expr(&paren.expr.node, context)?),
        ast::Expr::Unary(unary) => {
            let op = match unary.op {
                ast::UnaryOp::Not => "!",
                ast::UnaryOp::USub => "-",
                ast::UnaryOp::UAdd => "+",
                ast::UnaryOp::Invert => {
                    anyhow::bail!("bitwise inversion is not supported in Java validation")
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
                ast::BinOp::Pow => format!("Math.pow((double)({left}), (double)({right}))"),
                _ => anyhow::bail!(
                    "KCL operator {:?} is not supported in Java validation",
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
                format!("{receiver}.{}", java_field_name(&attr))
            }
        }
        ast::Expr::Call(call) => render_call(call, context)?,
        ast::Expr::Quant(quant) => render_quant(quant, context)?,
        other => anyhow::bail!(
            "KCL expression {} is not supported in Java validation",
            other.get_expr_name()
        ),
    })
}

fn render_identifier(
    identifier: &ast::Identifier,
    context: &JavaExprContext<'_>,
) -> anyhow::Result<String> {
    let names = identifier.get_names();
    let name = names
        .first()
        .ok_or_else(|| anyhow::anyhow!("empty identifier in Java validation"))?;
    if let Some((local, _)) = context.locals.get(name) {
        return Ok(local.clone());
    }
    if context.fields.contains_key(name) {
        let mut result = render_field(name, context)?;
        for member in names.iter().skip(1) {
            result.push('.');
            result.push_str(&java_field_name(member));
        }
        return Ok(result);
    }
    anyhow::bail!("unknown identifier {name:?} in Java validation")
}

fn render_field(name: &str, context: &JavaExprContext<'_>) -> anyhow::Result<String> {
    let (field_name, _, _) = context
        .fields
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("unknown identifier {name:?} in Java validation"))?;
    Ok(format!("this.{field_name}"))
}

fn render_compare(compare: &ast::Compare, context: &JavaExprContext<'_>) -> anyhow::Result<String> {
    if compare.ops.len() != compare.comparators.len() {
        anyhow::bail!("malformed KCL comparison in Java validation")
    }
    let mut left_expr = &compare.left.node;
    let mut comparisons = Vec::new();
    for (op, comparator) in compare.ops.iter().zip(&compare.comparators) {
        let left = render_expr(left_expr, context)?;
        let right = render_expr(&comparator.node, context)?;
        let comparison = match op {
            // `.equals()`/`Objects.equals()` are used instead of `==`/`!=`
            // for value equality: `==` on boxed numeric wrapper types
            // (`Long`, `Double`, ...) compares object identity rather than
            // value, which is correct only by accident for small cached
            // values. `Objects.equals` is also null-safe and works
            // uniformly whether the operands are primitives (auto-boxed
            // for the call), boxed wrappers, strings, or schema types.
            ast::CmpOp::Eq => format!("Objects.equals({left}, {right})"),
            ast::CmpOp::NotEq => format!("!Objects.equals({left}, {right})"),
            ast::CmpOp::Lt => format!("{left} < {right}"),
            ast::CmpOp::LtE => format!("{left} <= {right}"),
            ast::CmpOp::Gt => format!("{left} > {right}"),
            ast::CmpOp::GtE => format!("{left} >= {right}"),
            ast::CmpOp::In => format!("{right}.contains({left})"),
            ast::CmpOp::NotIn => format!("!{right}.contains({left})"),
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
                "KCL comparison {:?} is not supported in Java validation",
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
    context: &'a JavaExprContext<'_>,
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

fn render_call(call: &ast::CallExpr, context: &JavaExprContext<'_>) -> anyhow::Result<String> {
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
            _ => anyhow::bail!("unsupported receiver for upper() in Java validation"),
        };
        return Ok(format!("{receiver}.toUpperCase()"));
    }
    match function.as_str() {
        "len" if call.args.len() == 1 => {
            let arg = &call.args[0].node;
            let value = render_expr(arg, context)?;
            if expression_is_string(arg, context) {
                Ok(format!("({value}).codePointCount(0, ({value}).length())"))
            } else {
                Ok(format!("({value}).size()"))
            }
        }
        "regex.match" if call.args.len() == 2 => {
            let value = render_expr(&call.args[0].node, context)?;
            let pattern = render_expr(&call.args[1].node, context)?;
            context.imports.mark_regex();
            Ok(format!("KclRegexUtil.INSTANCE.match({pattern}, {value})"))
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
            anyhow::bail!("KCL call {function:?} is not supported in Java validation")
        }
    }
}

fn expression_is_string(expr: &ast::Expr, context: &JavaExprContext<'_>) -> bool {
    match expr {
        ast::Expr::StringLit(_) => true,
        ast::Expr::Identifier(identifier) => identifier
            .names
            .first()
            .and_then(|name| {
                context
                    .locals
                    .get(&name.node)
                    .map(|(_, ty)| ty)
                    .or_else(|| context.fields.get(&name.node).map(|(_, ty, _)| ty))
            })
            .is_some_and(|ty| ty.r#type == "str"),
        _ => false,
    }
}

/// Render an `all`/`any` quantifier using the Java Stream API. Unlike Go
/// (whose closures are statements, needing an immediately-invoked-closure
/// trick to use as an expression), `Stream.allMatch`/`anyMatch` already
/// *are* expressions, so no IIFE-equivalent wrapper is needed.
fn render_quant(quant: &ast::QuantExpr, context: &JavaExprContext<'_>) -> anyhow::Result<String> {
    if quant.variables.len() != 1 {
        anyhow::bail!("Java validation supports quantifiers with one variable")
    }
    let target = render_expr(&quant.target.node, context)?;
    let variable = quant.variables[0].node.get_name();
    let java_variable = java_field_name(&variable);
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
    let mut nested = JavaExprContext {
        fields: context.fields,
        locals: context.locals.clone(),
        imports: context.imports,
    };
    nested
        .locals
        .insert(variable.clone(), (java_variable.clone(), item_type));
    let test = render_expr(&quant.test.node, &nested)?;
    let body = if let Some(condition) = &quant.if_cond {
        let condition = render_expr(&condition.node, &nested)?;
        match quant.op {
            ast::QuantOperation::All => format!("!({condition}) || ({test})"),
            ast::QuantOperation::Any => format!("({condition}) && ({test})"),
            _ => anyhow::bail!(
                "conditional map/filter quantifiers are not supported in Java validation"
            ),
        }
    } else {
        test
    };
    let method = match quant.op {
        ast::QuantOperation::All => "allMatch",
        ast::QuantOperation::Any => "anyMatch",
        _ => anyhow::bail!("only all/any quantifiers are supported in Java validation"),
    };
    Ok(format!(
        "{target}.stream().{method}({java_variable} -> {body})"
    ))
}

/// Java generics require boxed element types (`List<long>` is illegal), so
/// nested `list`/`dict` item/key types are always rendered boxed regardless
/// of the enclosing field's own optionality.
fn java_type(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    field: &str,
    optional: bool,
    imports: &JavaImports,
) -> anyhow::Result<String> {
    java_type_impl(ty, resolver, field, optional, imports)
}

fn java_type_impl(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    field: &str,
    boxed: bool,
    imports: &JavaImports,
) -> anyhow::Result<String> {
    Ok(match ty.r#type.as_str() {
        "str" => "String".to_string(),
        "int" | "number_multiplier" => if boxed { "Long" } else { "long" }.to_string(),
        "float" => if boxed { "Double" } else { "double" }.to_string(),
        "bool" => if boxed { "Boolean" } else { "boolean" }.to_string(),
        "any" | "union" => "Object".to_string(),
        "schema" => resolver.name_of_ty(ty),
        "list" => {
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("list field {field:?} has no item type"))?;
            format!(
                "List<{}>",
                java_type_impl(item, resolver, field, true, imports)?
            )
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
            imports.mark_map();
            format!(
                "Map<{}, {}>",
                java_type_impl(key, resolver, field, true, imports)?,
                java_type_impl(item, resolver, field, true, imports)?
            )
        }
        other => anyhow::bail!("unsupported KCL type {other:?} for Java field {field:?}"),
    })
}

fn java_type_name(name: &str) -> String {
    to_pascal_case(name)
}

fn java_field_name(name: &str) -> String {
    let result = to_camel_case(name);
    if is_keyword(&result, JAVA_KEYWORDS) {
        format!("{result}_")
    } else {
        result
    }
}

const JAVA_KEYWORDS: &[&str] = &[
    "abstract",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "final",
    "finally",
    "float",
    "for",
    "goto",
    "if",
    "implements",
    "import",
    "instanceof",
    "int",
    "interface",
    "long",
    "native",
    "new",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "try",
    "void",
    "volatile",
    "while",
    "true",
    "false",
    "null",
    "_",
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
        let generated = generate_java(&mapping, "").unwrap();

        assert!(!generated.contains("package "));
        assert!(generated.contains("class Customer {"));
        assert!(generated.contains("public List<Address> addresses;"));
        assert!(generated.contains("public Long age;"));
        assert!(generated.contains("public String zipCode;"));
        assert!(
            generated.find("class Address").unwrap() < generated.find("class Customer").unwrap()
        );
        // Neither schema uses a `dict` field or a `regex.match` check, so
        // tracking must omit both the Map import and the regex helper
        // class rather than emitting them unconditionally.
        assert!(!generated.contains("import java.util.Map;"));
        assert!(!generated.contains("KclRegexUtil"));
    }

    #[test]
    fn emits_map_import_only_when_a_dict_field_is_used() {
        let mut coordinates = schema("Coordinates", "main");
        coordinates.properties.insert(
            "values".to_string(),
            KclType {
                r#type: "dict".to_string(),
                key: Some(Box::new(KclType {
                    r#type: "str".to_string(),
                    ..Default::default()
                })),
                item: Some(Box::new(KclType {
                    r#type: "float".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        coordinates.required.push("values".to_string());

        let mapping = HashMap::from([(
            "main".to_string(),
            SchemaTypes {
                schema_type: vec![coordinates],
            },
        )]);
        let generated = generate_java(&mapping, "").unwrap();

        assert!(generated.contains("import java.util.Map;"));
        assert!(generated.contains("public Map<String, Double> values;"));
    }

    #[test]
    fn uses_the_requested_package_name() {
        let mapping = HashMap::from([(
            "main".to_string(),
            SchemaTypes {
                schema_type: vec![schema("Solo", "main")],
            },
        )]);
        let generated = generate_java(&mapping, "com.example.dto").unwrap();
        assert!(generated.starts_with("package com.example.dto;\n"));
    }

    #[test]
    fn rejects_fields_that_normalize_to_the_same_java_identifier() {
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
        let error = generate_java(&mapping, "").unwrap_err();
        assert!(error.to_string().contains("field name collision"));
    }
}
