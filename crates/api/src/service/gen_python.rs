//! Python DTO emitter from the shared package schema mapping.

use std::collections::{HashMap, HashSet};

use kcl_ast::ast;

use crate::gpyrpc::{KclType, SchemaTypes};
use crate::service::gen_schema::{
    SchemaNameResolver, emit_schema_list, ordered_schemas, require_schemas,
};

pub(crate) fn generate_python(mapping: &HashMap<String, SchemaTypes>) -> anyhow::Result<String> {
    let ordered = ordered_schemas(mapping);
    require_schemas(&ordered)?;
    let resolver = SchemaNameResolver::new(&ordered);
    let schemas = emit_schema_list(&ordered);
    let mut used_schema_names = HashSet::new();
    for schema in &schemas {
        let python_name = python_type_name(&schema.name);
        if !used_schema_names.insert(python_name.clone()) {
            anyhow::bail!("Python schema name collision after normalization: {python_name:?}");
        }
    }

    let mut out = String::new();
    out.push_str("import dataclasses\n");
    out.push_str("import re\n");
    out.push_str("import typing\n\n\n");
    for (index, schema) in schemas.iter().enumerate() {
        emit_schema(&mut out, &schema.name, schema.ty, &resolver)?;
        if index + 1 != schemas.len() {
            out.push('\n');
        }
    }
    Ok(out)
}

fn emit_schema(
    out: &mut String,
    name: &str,
    ty: &KclType,
    resolver: &SchemaNameResolver,
) -> anyhow::Result<()> {
    out.push_str("@dataclasses.dataclass(kw_only=True)\n");
    out.push_str(&format!("class {}:\n", python_type_name(name)));
    if !ty.schema_doc.is_empty() {
        out.push_str("    \"\"\"");
        out.push_str(&ty.schema_doc.replace("\"\"\"", "\\\"\\\"\\\""));
        out.push_str("\"\"\"\n");
    }

    let mut properties: Vec<&String> = ty.properties.keys().collect();
    properties.sort();
    let mut used_field_names = HashSet::new();
    for property in &properties {
        let field = &ty.properties[*property];
        if field.r#type == "function" {
            anyhow::bail!("Python DTO generation does not support function field {property:?}");
        }
        if !field.description.is_empty() {
            for line in field.description.lines() {
                out.push_str(&format!("    # {line}\n"));
            }
        }
        if !field.default.is_empty() {
            for line in field.default.lines() {
                out.push_str(&format!("    # KCL default: {line}\n"));
            }
        }
        let field_name = python_field_name(property);
        if !used_field_names.insert(field_name.clone()) {
            anyhow::bail!("Python field name collision after normalization: {field_name:?}");
        }
        if field_name != **property {
            out.push_str(&format!("    # KCL name: {property}\n"));
        }
        let optional = !ty.required.iter().any(|required| required == *property);
        let field_ty = python_type(field, resolver, property)?;
        if optional {
            out.push_str(&format!(
                "    {field_name}: typing.Optional[{field_ty}] = None\n"
            ));
        } else {
            out.push_str(&format!("    {field_name}: {field_ty}\n"));
        }
    }
    if properties.is_empty() && ty.schema_doc.is_empty() {
        out.push_str("    pass\n");
    }
    out.push('\n');
    emit_validator(out, ty)?;
    out.push('\n');
    Ok(())
}

fn emit_validator(out: &mut String, ty: &KclType) -> anyhow::Result<()> {
    let fields = ty
        .properties
        .iter()
        .map(|(name, field)| {
            (
                name.clone(),
                (
                    python_field_name(name),
                    field.clone(),
                    !ty.required.contains(name),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut checks = Vec::new();
    if let Some(base) = &ty.base_schema {
        collect_checks(base, &fields, &mut checks)?;
    }
    collect_checks(ty, &fields, &mut checks)?;

    out.push_str("    def validate(self) -> typing.List[str]:\n");
    out.push_str("        errors: typing.List[str] = []\n");
    for check in checks {
        out.push_str(&check);
    }
    out.push_str("        return errors\n");
    Ok(())
}

fn collect_checks(
    ty: &KclType,
    fields: &HashMap<String, (String, KclType, bool)>,
    checks: &mut Vec<String>,
) -> anyhow::Result<()> {
    for check_json in &ty.checks_json {
        let check: ast::CheckExpr = serde_json::from_str(check_json).map_err(|error| {
            anyhow::anyhow!(
                "cannot decode KCL check AST for schema {:?}: {error}",
                ty.schema_name
            )
        })?;
        let context = PythonExprContext {
            fields,
            locals: HashMap::new(),
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
            .unwrap_or_else(|| format!("{:?}", format!("KCL check failed: {test}")));
        let condition = match condition {
            Some(condition) => format!("{condition} and not ({test})"),
            None => format!("not ({test})"),
        };
        checks.push(format!(
            "        if {condition}:\n            errors.append({message})\n"
        ));
    }
    Ok(())
}

struct PythonExprContext<'a> {
    fields: &'a HashMap<String, (String, KclType, bool)>,
    locals: HashMap<String, String>,
}

/// Render a KCL `if` guard. A bare optional-field identifier (the common
/// `if field` guard pattern) becomes an explicit `is not None` check since
/// Python has no notion of an "unset" value that is also falsy-by-type.
fn render_condition(expr: &ast::Expr, context: &PythonExprContext<'_>) -> anyhow::Result<String> {
    if let ast::Expr::Identifier(identifier) = expr
        && identifier.names.len() == 1
    {
        let name = &identifier.names[0].node;
        if let Some((field_name, _, true)) = context.fields.get(name) {
            return Ok(format!("self.{field_name} is not None"));
        }
    }
    render_expr(expr, context)
}

fn render_expr(expr: &ast::Expr, context: &PythonExprContext<'_>) -> anyhow::Result<String> {
    Ok(match expr {
        ast::Expr::Identifier(identifier) => render_identifier(identifier, context)?,
        ast::Expr::Target(target) => {
            if !target.paths.is_empty() {
                anyhow::bail!("indexed KCL targets are not supported in Python validation")
            }
            render_name(target.name.node.as_str(), context)?
        }
        ast::Expr::NumberLit(number) => {
            if number.binary_suffix.is_some() {
                anyhow::bail!("number multipliers are not supported in Python validation")
            }
            number.to_string()
        }
        ast::Expr::StringLit(string) => format!("{:?}", string.value),
        ast::Expr::NameConstantLit(value) => match value.value {
            ast::NameConstant::True => "True".to_string(),
            ast::NameConstant::False => "False".to_string(),
            ast::NameConstant::None => "None".to_string(),
            ast::NameConstant::Undefined => {
                anyhow::bail!("Undefined is not supported in Python validation")
            }
        },
        ast::Expr::Paren(paren) => format!("({})", render_expr(&paren.expr.node, context)?),
        ast::Expr::Unary(unary) => {
            let (op, spaced) = match unary.op {
                ast::UnaryOp::Not => ("not", true),
                ast::UnaryOp::USub => ("-", false),
                ast::UnaryOp::UAdd => ("+", false),
                ast::UnaryOp::Invert => {
                    anyhow::bail!("bitwise inversion is not supported in Python validation")
                }
            };
            let operand = render_expr(&unary.operand.node, context)?;
            if spaced {
                format!("{op} ({operand})")
            } else {
                format!("{op}({operand})")
            }
        }
        ast::Expr::Binary(binary) => {
            let op = match binary.op {
                ast::BinOp::Add => "+",
                ast::BinOp::Sub => "-",
                ast::BinOp::Mul => "*",
                ast::BinOp::Div => "/",
                ast::BinOp::Mod => "%",
                ast::BinOp::Pow => "**",
                ast::BinOp::And => "and",
                ast::BinOp::Or => "or",
                _ => anyhow::bail!(
                    "KCL operator {:?} is not supported in Python validation",
                    binary.op
                ),
            };
            let left = render_expr(&binary.left.node, context)?;
            let right = render_expr(&binary.right.node, context)?;
            format!("({left} {op} {right})")
        }
        ast::Expr::Compare(compare) => render_compare(compare, context)?,
        ast::Expr::Selector(selector) => {
            let receiver = render_expr(&selector.value.node, context)?;
            let attr = selector.attr.node.get_name();
            if attr == "upper" {
                format!("{receiver}.upper()")
            } else {
                format!("{receiver}.{}", python_field_name(&attr))
            }
        }
        ast::Expr::Call(call) => render_call(call, context)?,
        ast::Expr::Quant(quant) => render_quant(quant, context)?,
        ast::Expr::List(list) => format!(
            "[{}]",
            list.elts
                .iter()
                .map(|element| render_expr(&element.node, context))
                .collect::<anyhow::Result<Vec<_>>>()?
                .join(", ")
        ),
        other => anyhow::bail!(
            "KCL expression {} is not supported in Python validation",
            other.get_expr_name()
        ),
    })
}

fn render_identifier(
    identifier: &ast::Identifier,
    context: &PythonExprContext<'_>,
) -> anyhow::Result<String> {
    let names = identifier.get_names();
    let name = names
        .first()
        .ok_or_else(|| anyhow::anyhow!("empty identifier in Python validation"))?;
    let mut result = render_name(name, context)?;
    for member in names.iter().skip(1) {
        result.push('.');
        result.push_str(&python_field_name(member));
    }
    Ok(result)
}

fn render_name(name: &str, context: &PythonExprContext<'_>) -> anyhow::Result<String> {
    if let Some(local) = context.locals.get(name) {
        return Ok(local.clone());
    }
    if let Some((field_name, _, _)) = context.fields.get(name) {
        return Ok(format!("self.{field_name}"));
    }
    anyhow::bail!("unknown identifier {name:?} in Python validation")
}

fn render_compare(
    compare: &ast::Compare,
    context: &PythonExprContext<'_>,
) -> anyhow::Result<String> {
    if compare.ops.len() != compare.comparators.len() {
        anyhow::bail!("malformed KCL comparison in Python validation")
    }
    let mut result = render_expr(&compare.left.node, context)?;
    for (op, comparator) in compare.ops.iter().zip(&compare.comparators) {
        let op = match op {
            ast::CmpOp::Eq => "==",
            ast::CmpOp::NotEq => "!=",
            ast::CmpOp::Lt => "<",
            ast::CmpOp::LtE => "<=",
            ast::CmpOp::Gt => ">",
            ast::CmpOp::GtE => ">=",
            ast::CmpOp::In => "in",
            ast::CmpOp::NotIn => "not in",
            ast::CmpOp::Is => "is",
            ast::CmpOp::IsNot => "is not",
            ast::CmpOp::Not => {
                anyhow::bail!("KCL comparison {op:?} is not supported in Python validation")
            }
        };
        let right = render_expr(&comparator.node, context)?;
        result.push_str(&format!(" {op} {right}"));
    }
    Ok(result)
}

fn render_call(call: &ast::CallExpr, context: &PythonExprContext<'_>) -> anyhow::Result<String> {
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
            _ => anyhow::bail!("unsupported receiver for upper() in Python validation"),
        };
        return Ok(format!("({receiver}).upper()"));
    }
    match function.as_str() {
        "len" if call.args.len() == 1 => {
            let value = render_expr(&call.args[0].node, context)?;
            Ok(format!("len({value})"))
        }
        "regex.match" if call.args.len() == 2 => {
            let value = render_expr(&call.args[0].node, context)?;
            let pattern = render_expr(&call.args[1].node, context)?;
            Ok(format!("re.match({pattern}, {value}) is not None"))
        }
        _ => anyhow::bail!("KCL call {function:?} is not supported in Python validation"),
    }
}

fn render_quant(quant: &ast::QuantExpr, context: &PythonExprContext<'_>) -> anyhow::Result<String> {
    if quant.variables.len() != 1 {
        anyhow::bail!("Python validation supports quantifiers with one variable")
    }
    let target = render_expr(&quant.target.node, context)?;
    let variable = quant.variables[0].node.get_name();
    let python_variable = python_field_name(&variable);
    let mut nested = PythonExprContext {
        fields: context.fields,
        locals: context.locals.clone(),
    };
    nested
        .locals
        .insert(variable.clone(), python_variable.clone());
    let test = render_expr(&quant.test.node, &nested)?;
    let method = match quant.op {
        ast::QuantOperation::All => "all",
        ast::QuantOperation::Any => "any",
        _ => anyhow::bail!("only all/any quantifiers are supported in Python validation"),
    };
    Ok(match &quant.if_cond {
        Some(condition) => {
            let condition = render_expr(&condition.node, &nested)?;
            format!("{method}({test} for {python_variable} in {target} if {condition})")
        }
        None => format!("{method}({test} for {python_variable} in {target})"),
    })
}

fn python_type(ty: &KclType, resolver: &SchemaNameResolver, field: &str) -> anyhow::Result<String> {
    Ok(match ty.r#type.as_str() {
        "str" => "str".to_string(),
        "int" | "number_multiplier" => "int".to_string(),
        "float" => "float".to_string(),
        "bool" => "bool".to_string(),
        "any" | "union" => "typing.Any".to_string(),
        "schema" => resolver.name_of_ty(ty),
        "list" => {
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("list field {field:?} has no item type"))?;
            format!("typing.List[{}]", python_type(item, resolver, field)?)
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
                "typing.Dict[{}, {}]",
                python_type(key, resolver, field)?,
                python_type(item, resolver, field)?
            )
        }
        other => anyhow::bail!("unsupported KCL type {other:?} for Python field {field:?}"),
    })
}

fn python_type_name(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalize {
                result.push(ch.to_ascii_uppercase());
            } else {
                result.push(ch);
            }
            capitalize = false;
        } else {
            capitalize = true;
        }
    }
    if result.is_empty() || result.as_bytes()[0].is_ascii_digit() {
        result.insert_str(0, "Kcl");
    }
    result
}

fn python_field_name(name: &str) -> String {
    let mut chars = name.chars().peekable();
    let mut result = String::new();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_uppercase() {
            if !result.is_empty()
                && !result.ends_with('_')
                && (result
                    .chars()
                    .last()
                    .is_some_and(|prev| prev.is_ascii_lowercase())
                    || chars.peek().is_some_and(|next| next.is_ascii_lowercase()))
            {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            result.push(ch);
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }
    while result.ends_with('_') {
        result.pop();
    }
    if result.is_empty() || result.as_bytes()[0].is_ascii_digit() {
        result.insert_str(0, "field_");
    }
    if is_python_keyword(&result) {
        result.push('_');
    }
    result
}

fn is_python_keyword(name: &str) -> bool {
    matches!(
        name,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
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
    fn emits_deterministic_dataclasses_with_optional_and_nested_types() {
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
        let generated = generate_python(&mapping).unwrap();

        assert!(generated.contains("class Customer:"));
        assert!(generated.contains("addresses: typing.List[Address]"));
        assert!(generated.contains("age: typing.Optional[int] = None"));
        assert!(generated.contains("# KCL name: zipCode"));
        assert!(generated.contains("zip_code: str"));
        assert!(
            generated.find("class Address").unwrap() < generated.find("class Customer").unwrap()
        );
    }

    #[test]
    fn converts_python_keywords_to_suffixed_field_identifiers() {
        assert_eq!(python_field_name("class"), "class_");
        assert_eq!(python_field_name("zipCode"), "zip_code");
    }

    #[test]
    fn rejects_fields_that_normalize_to_the_same_python_identifier() {
        let mut ty = schema("Collision", "main");
        ty.properties.insert(
            "zipCode".to_string(),
            KclType {
                r#type: "str".to_string(),
                ..Default::default()
            },
        );
        ty.properties.insert(
            "zip_code".to_string(),
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
        let error = generate_python(&mapping).unwrap_err();
        assert!(error.to_string().contains("field name collision"));
    }
}
