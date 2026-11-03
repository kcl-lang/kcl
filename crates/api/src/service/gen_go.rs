//! Go DTO emitter from the shared package schema mapping.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use kcl_ast::ast;

use crate::gpyrpc::{KclType, SchemaTypes};
use crate::service::gen_ident::to_pascal_case;
use crate::service::gen_schema::{self, SchemaNameResolver};

/// Package-level stdlib imports a generated file might need. Go treats an
/// unused import as a hard compile error, so (unlike Rust/Python, where
/// unused imports are at most a warning) we track which stdlib packages a
/// `check:` expression actually used while rendering and only emit those.
#[derive(Default)]
struct GoImports {
    used: RefCell<HashSet<&'static str>>,
}

impl GoImports {
    fn mark(&self, import: &'static str) {
        self.used.borrow_mut().insert(import);
    }
}

const REGEX_MATCH_HELPER: &str = "func kclRegexMatch(pattern string, value string) bool {\n\tre, err := regexp.Compile(pattern)\n\tif err != nil {\n\t\treturn false\n\t}\n\treturn re.MatchString(value)\n}\n\n";

pub(crate) fn generate_go(
    mapping: &HashMap<String, SchemaTypes>,
    package: &str,
) -> anyhow::Result<String> {
    let package = if package.is_empty() {
        "generated"
    } else {
        package
    };
    let imports = GoImports::default();
    let body =
        gen_schema::generate_source(mapping, "Go", go_type_name, |out, name, ty, resolver| {
            emit_schema(out, name, ty, resolver, &imports)
        })?;

    let mut out = format!("package {package}\n\n");
    let used = imports.used.borrow();
    if !used.is_empty() {
        let mut names: Vec<&str> = used.iter().copied().collect();
        names.sort();
        out.push_str("import (\n");
        for name in names {
            out.push_str(&format!("\t\"{name}\"\n"));
        }
        out.push_str(")\n\n");
    }
    if used.contains("regexp") {
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
    imports: &GoImports,
) -> anyhow::Result<()> {
    for line in ty.schema_doc.lines() {
        out.push_str("//");
        if !line.is_empty() {
            out.push(' ');
            out.push_str(line);
        }
        out.push('\n');
    }
    out.push_str(&format!("type {} struct {{\n", go_type_name(name)));

    let mut properties: Vec<&String> = ty.properties.keys().collect();
    properties.sort();
    let mut used_field_names = HashSet::new();
    for property in &properties {
        let field = &ty.properties[*property];
        if field.r#type == "function" {
            anyhow::bail!("Go DTO generation does not support function field {property:?}");
        }
        if !field.description.is_empty() {
            for line in field.description.lines() {
                out.push_str(&format!("\t// {line}\n"));
            }
        }
        if !field.default.is_empty() {
            for line in field.default.lines() {
                out.push_str(&format!("\t// KCL default: {line}\n"));
            }
        }
        let field_name = go_field_name(property);
        if !used_field_names.insert(field_name.clone()) {
            anyhow::bail!("Go field name collision after normalization: {field_name:?}");
        }
        let optional = !ty.required.iter().any(|required| required == *property);
        let field_ty = go_type(field, resolver, property, optional)?;
        let tag = if optional {
            format!("`json:\"{property},omitempty\"`")
        } else {
            format!("`json:\"{property}\"`")
        };
        out.push_str(&format!("\t{field_name} {field_ty} {tag}\n"));
    }
    out.push_str("}\n");
    emit_validator(out, name, ty, imports)?;
    Ok(())
}

fn emit_validator(
    out: &mut String,
    name: &str,
    ty: &KclType,
    imports: &GoImports,
) -> anyhow::Result<()> {
    let fields = ty
        .properties
        .iter()
        .map(|(name, field)| {
            (
                name.clone(),
                (
                    go_field_name(name),
                    field.clone(),
                    !ty.required.contains(name),
                ),
            )
        })
        .collect::<HashMap<_, _>>();

    out.push_str(&format!(
        "func (s *{}) Validate() []string {{\n",
        go_type_name(name)
    ));
    out.push_str("\terrors := []string{}\n");
    for check in gen_schema::decode_checks(ty)? {
        out.push_str(&render_check(&check, &fields, imports)?);
    }
    out.push_str("\treturn errors\n");
    out.push_str("}\n");
    Ok(())
}

fn render_check(
    check: &ast::CheckExpr,
    fields: &HashMap<String, (String, KclType, bool)>,
    imports: &GoImports,
) -> anyhow::Result<String> {
    let mut context = GoExprContext {
        fields,
        locals: HashMap::new(),
        unwrapped_optional_fields: HashSet::new(),
        imports,
    };
    if let Some(condition) = &check.if_cond {
        collect_guarded_optionals(
            &condition.node,
            fields,
            &mut context.unwrapped_optional_fields,
        );
    }
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
        "\tif {condition} {{\n\t\terrors = append(errors, {message})\n\t}}\n"
    ))
}

struct GoExprContext<'a> {
    fields: &'a HashMap<String, (String, KclType, bool)>,
    locals: HashMap<String, (String, KclType)>,
    unwrapped_optional_fields: HashSet<String>,
    imports: &'a GoImports,
}

/// Render a KCL `if` guard. A bare optional-field identifier (the common
/// `if field` guard pattern) becomes an explicit nil check, since an
/// optional field is represented as a Go pointer.
fn render_condition(expr: &ast::Expr, context: &GoExprContext<'_>) -> anyhow::Result<String> {
    if let ast::Expr::Identifier(identifier) = expr
        && identifier.names.len() == 1
    {
        let name = &identifier.names[0].node;
        if let Some((field_name, _, true)) = context.fields.get(name) {
            return Ok(format!("s.{field_name} != nil"));
        }
    }
    render_expr(expr, context)
}

fn collect_guarded_optionals(
    expr: &ast::Expr,
    fields: &HashMap<String, (String, KclType, bool)>,
    guarded: &mut HashSet<String>,
) {
    let identifier = match expr {
        ast::Expr::Identifier(identifier) => Some(identifier),
        ast::Expr::Compare(compare)
            if compare.ops.len() == 1
                && compare.ops[0] == ast::CmpOp::IsNot
                && matches!(
                    compare.comparators[0].node,
                    ast::Expr::NameConstantLit(ast::NameConstantLit {
                        value: ast::NameConstant::None
                    })
                ) =>
        {
            match &compare.left.node {
                ast::Expr::Identifier(identifier) => Some(identifier),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(identifier) = identifier
        && identifier.names.len() == 1
    {
        let name = &identifier.names[0].node;
        if fields.get(name).is_some_and(|(_, _, optional)| *optional) {
            guarded.insert(name.clone());
        }
    }
}

fn render_expr(expr: &ast::Expr, context: &GoExprContext<'_>) -> anyhow::Result<String> {
    Ok(match expr {
        ast::Expr::Identifier(identifier) => render_identifier(identifier, context)?,
        ast::Expr::Target(target) => {
            if !target.paths.is_empty() {
                anyhow::bail!("indexed KCL targets are not supported in Go validation")
            }
            render_field(target.name.node.as_str(), context)?
        }
        ast::Expr::NumberLit(number) => {
            if number.binary_suffix.is_some() {
                anyhow::bail!("number multipliers are not supported in Go validation")
            }
            number.to_string()
        }
        ast::Expr::StringLit(string) => format!("{:?}", string.value),
        ast::Expr::NameConstantLit(value) => match value.value {
            ast::NameConstant::True => "true".to_string(),
            ast::NameConstant::False => "false".to_string(),
            ast::NameConstant::None => "nil".to_string(),
            ast::NameConstant::Undefined => {
                anyhow::bail!("Undefined is not supported in Go validation")
            }
        },
        ast::Expr::Paren(paren) => format!("({})", render_expr(&paren.expr.node, context)?),
        ast::Expr::Unary(unary) => {
            let op = match unary.op {
                ast::UnaryOp::Not => "!",
                ast::UnaryOp::USub => "-",
                ast::UnaryOp::UAdd => "+",
                ast::UnaryOp::Invert => {
                    anyhow::bail!("bitwise inversion is not supported in Go validation")
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
                ast::BinOp::Pow => {
                    context.imports.mark("math");
                    format!("math.Pow(float64({left}), float64({right}))")
                }
                _ => anyhow::bail!(
                    "KCL operator {:?} is not supported in Go validation",
                    binary.op
                ),
            }
        }
        ast::Expr::Compare(compare) => render_compare(compare, context)?,
        ast::Expr::Selector(selector) => {
            let receiver = render_expr(&selector.value.node, context)?;
            let attr = selector.attr.node.get_name();
            if attr == "upper" {
                context.imports.mark("strings");
                format!("strings.ToUpper({receiver})")
            } else {
                format!("{receiver}.{}", go_field_name(&attr))
            }
        }
        ast::Expr::Call(call) => render_call(call, context)?,
        ast::Expr::Quant(quant) => render_quant(quant, context)?,
        other => anyhow::bail!(
            "KCL expression {} is not supported in Go validation",
            other.get_expr_name()
        ),
    })
}

fn render_identifier(
    identifier: &ast::Identifier,
    context: &GoExprContext<'_>,
) -> anyhow::Result<String> {
    let names = identifier.get_names();
    let name = names
        .first()
        .ok_or_else(|| anyhow::anyhow!("empty identifier in Go validation"))?;
    // Unlike Rust's `.iter()` (which yields references needing the same
    // dereference treatment as a guarded `Option`), a Go `range` over a
    // slice always yields plain values, so a quantifier loop variable never
    // needs dereferencing — only a guarded-optional struct field does.
    if let Some((local, _)) = context.locals.get(name) {
        return Ok(local.clone());
    }
    if context.fields.contains_key(name) {
        let mut result = render_field(name, context)?;
        for member in names.iter().skip(1) {
            result.push('.');
            result.push_str(&go_field_name(member));
        }
        return Ok(result);
    }
    anyhow::bail!("unknown identifier {name:?} in Go validation")
}

fn render_field(name: &str, context: &GoExprContext<'_>) -> anyhow::Result<String> {
    let (field_name, ty, optional) = context
        .fields
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("unknown identifier {name:?} in Go validation"))?;
    let field = format!("s.{field_name}");
    if *optional && context.unwrapped_optional_fields.contains(name) {
        return Ok(render_dereferenced_value(&field, ty));
    }
    Ok(field)
}

/// Optional scalar/schema fields are represented as Go pointers; once a
/// check has guarded one as non-nil, using its value requires an explicit
/// dereference. Optional lists/dicts/any/union are never pointer-wrapped
/// (see [`needs_pointer_when_optional`]), so they need no dereference.
fn render_dereferenced_value(value: &str, ty: &KclType) -> String {
    if needs_pointer_when_optional(ty) {
        format!("*{value}")
    } else {
        value.to_string()
    }
}

fn render_compare(compare: &ast::Compare, context: &GoExprContext<'_>) -> anyhow::Result<String> {
    if compare.ops.len() != compare.comparators.len() {
        anyhow::bail!("malformed KCL comparison in Go validation")
    }
    let mut left_expr = &compare.left.node;
    let mut comparisons = Vec::new();
    for (op, comparator) in compare.ops.iter().zip(&compare.comparators) {
        let left = render_expr(left_expr, context)?;
        let right = render_expr(&comparator.node, context)?;
        let comparison = match op {
            ast::CmpOp::Eq => format!("{left} == {right}"),
            ast::CmpOp::NotEq => format!("{left} != {right}"),
            ast::CmpOp::Lt => format!("{left} < {right}"),
            ast::CmpOp::LtE => format!("{left} <= {right}"),
            ast::CmpOp::Gt => format!("{left} > {right}"),
            ast::CmpOp::GtE => format!("{left} >= {right}"),
            ast::CmpOp::In => {
                context.imports.mark("slices");
                format!("slices.Contains({right}, {left})")
            }
            ast::CmpOp::NotIn => {
                context.imports.mark("slices");
                format!("!slices.Contains({right}, {left})")
            }
            ast::CmpOp::Is | ast::CmpOp::IsNot => {
                if let ast::Expr::NameConstantLit(value) = &comparator.node
                    && value.value == ast::NameConstant::None
                    && let Some((field_name, _, true)) =
                        optional_field_for_expr(&compare.left.node, context)
                {
                    let cmp_op = if *op == ast::CmpOp::Is { "==" } else { "!=" };
                    format!("s.{field_name} {cmp_op} nil")
                } else {
                    anyhow::bail!("KCL identity checks are only supported against None")
                }
            }
            _ => anyhow::bail!("KCL comparison {:?} is not supported in Go validation", op),
        };
        comparisons.push(comparison);
        left_expr = &comparator.node;
    }
    Ok(comparisons.join(" && "))
}

fn optional_field_for_expr<'a>(
    expr: &ast::Expr,
    context: &'a GoExprContext<'_>,
) -> Option<&'a (String, KclType, bool)> {
    let ast::Expr::Identifier(identifier) = expr else {
        return None;
    };
    if identifier.names.len() != 1 {
        return None;
    }
    context.fields.get(&identifier.names[0].node)
}

fn render_call(call: &ast::CallExpr, context: &GoExprContext<'_>) -> anyhow::Result<String> {
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
            _ => anyhow::bail!("unsupported receiver for upper() in Go validation"),
        };
        context.imports.mark("strings");
        return Ok(format!("strings.ToUpper({receiver})"));
    }
    match function.as_str() {
        "len" if call.args.len() == 1 => {
            let arg = &call.args[0].node;
            let value = render_expr(arg, context)?;
            if expression_is_string(arg, context) {
                context.imports.mark("unicode/utf8");
                Ok(format!("utf8.RuneCountInString({value})"))
            } else {
                Ok(format!("len({value})"))
            }
        }
        "regex.match" if call.args.len() == 2 => {
            let value = render_expr(&call.args[0].node, context)?;
            let pattern = render_expr(&call.args[1].node, context)?;
            context.imports.mark("regexp");
            Ok(format!("kclRegexMatch({pattern}, {value})"))
        }
        _ => {
            if let ast::Expr::Selector(selector) = &call.func.node
                && selector.attr.node.get_name() == "upper"
                && call.args.is_empty()
            {
                context.imports.mark("strings");
                return Ok(format!(
                    "strings.ToUpper({})",
                    render_expr(&selector.value.node, context)?
                ));
            }
            anyhow::bail!("KCL call {function:?} is not supported in Go validation")
        }
    }
}

fn expression_is_string(expr: &ast::Expr, context: &GoExprContext<'_>) -> bool {
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

/// Render an `all`/`any` quantifier as an immediately-invoked Go closure —
/// Go has no expression-level comprehension syntax, so the loop has to be a
/// statement, and a closure is the only way to use it in an expression
/// position (e.g. directly inside the enclosing `if` condition).
fn render_quant(quant: &ast::QuantExpr, context: &GoExprContext<'_>) -> anyhow::Result<String> {
    if quant.variables.len() != 1 {
        anyhow::bail!("Go validation supports quantifiers with one variable")
    }
    let target = render_expr(&quant.target.node, context)?;
    let variable = quant.variables[0].node.get_name();
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
    let mut nested = GoExprContext {
        fields: context.fields,
        locals: context.locals.clone(),
        unwrapped_optional_fields: context.unwrapped_optional_fields.clone(),
        imports: context.imports,
    };
    nested
        .locals
        .insert(variable.clone(), (variable.clone(), item_type));
    let test = render_expr(&quant.test.node, &nested)?;
    let body = if let Some(condition) = &quant.if_cond {
        let condition = render_expr(&condition.node, &nested)?;
        match quant.op {
            ast::QuantOperation::All => format!("!({condition}) || ({test})"),
            ast::QuantOperation::Any => format!("({condition}) && ({test})"),
            _ => anyhow::bail!(
                "conditional map/filter quantifiers are not supported in Go validation"
            ),
        }
    } else {
        test
    };
    Ok(match quant.op {
        ast::QuantOperation::All => format!(
            "func() bool {{\n\t\tfor _, {variable} := range {target} {{\n\t\t\tif !({body}) {{\n\t\t\t\treturn false\n\t\t\t}}\n\t\t}}\n\t\treturn true\n\t}}()"
        ),
        ast::QuantOperation::Any => format!(
            "func() bool {{\n\t\tfor _, {variable} := range {target} {{\n\t\t\tif {body} {{\n\t\t\t\treturn true\n\t\t\t}}\n\t\t}}\n\t\treturn false\n\t}}()"
        ),
        _ => anyhow::bail!("only all/any quantifiers are supported in Go validation"),
    })
}

/// Whether an optional field of this KCL type is represented as a Go
/// pointer. Lists and maps are already nil-able in Go, and `any` already
/// represents absence as a nil interface, so only scalars and schema
/// references need the extra indirection.
fn needs_pointer_when_optional(ty: &KclType) -> bool {
    !matches!(ty.r#type.as_str(), "list" | "dict" | "any" | "union")
}

fn go_type(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    field: &str,
    optional: bool,
) -> anyhow::Result<String> {
    let base = go_base_type(ty, resolver, field)?;
    Ok(if optional && needs_pointer_when_optional(ty) {
        format!("*{base}")
    } else {
        base
    })
}

fn go_base_type(
    ty: &KclType,
    resolver: &SchemaNameResolver,
    field: &str,
) -> anyhow::Result<String> {
    Ok(match ty.r#type.as_str() {
        "str" => "string".to_string(),
        "int" | "number_multiplier" => "int64".to_string(),
        "float" => "float64".to_string(),
        "bool" => "bool".to_string(),
        "any" | "union" => "any".to_string(),
        "schema" => resolver.name_of_ty(ty),
        "list" => {
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("list field {field:?} has no item type"))?;
            format!("[]{}", go_base_type(item, resolver, field)?)
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
                "map[{}]{}",
                go_base_type(key, resolver, field)?,
                go_base_type(item, resolver, field)?
            )
        }
        other => anyhow::bail!("unsupported KCL type {other:?} for Go field {field:?}"),
    })
}

fn go_type_name(name: &str) -> String {
    to_pascal_case(name)
}

/// Go struct fields must be exported (capitalized) to be visible to
/// `encoding/json`, so this reuses the same PascalCase convention as
/// [`go_type_name`]. Unlike Rust/Python, no keyword escaping table is
/// needed: every Go keyword (`type`, `func`, `range`, ...) is lowercase, so
/// a PascalCase identifier can never collide with one.
fn go_field_name(name: &str) -> String {
    to_pascal_case(name)
}

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
    fn emits_deterministic_structs_with_optional_and_nested_types() {
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
        let generated = generate_go(&mapping, "").unwrap();

        assert!(generated.starts_with("package generated\n"));
        assert!(generated.contains("type Customer struct {"));
        assert!(generated.contains("Addresses []Address `json:\"addresses\"`"));
        assert!(generated.contains("Age *int64 `json:\"age,omitempty\"`"));
        assert!(generated.contains("ZipCode string `json:\"zipCode\"`"));
        assert!(
            generated.find("type Address struct").unwrap()
                < generated.find("type Customer struct").unwrap()
        );
    }

    #[test]
    fn uses_the_requested_package_name() {
        let mapping = HashMap::from([(
            "main".to_string(),
            SchemaTypes {
                schema_type: vec![schema("Solo", "main")],
            },
        )]);
        let generated = generate_go(&mapping, "dto").unwrap();
        assert!(generated.starts_with("package dto\n"));
    }

    #[test]
    fn rejects_fields_that_normalize_to_the_same_go_identifier() {
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
        let error = generate_go(&mapping, "").unwrap_err();
        assert!(error.to_string().contains("field name collision"));
    }
}
