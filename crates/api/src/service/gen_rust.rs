//! Rust DTO emitter from the shared package schema mapping.

use std::collections::{HashMap, HashSet};

use kcl_ast::ast;

use crate::gpyrpc::{KclType, SchemaTypes};
use crate::service::gen_schema::{
    SchemaNameResolver, emit_schema_list, ordered_schemas, require_schemas,
};

pub(crate) fn generate_rust(mapping: &HashMap<String, SchemaTypes>) -> anyhow::Result<String> {
    let ordered = ordered_schemas(mapping);
    require_schemas(&ordered)?;
    let resolver = SchemaNameResolver::new(&ordered);
    let schemas = emit_schema_list(&ordered);
    let mut used_schema_names = HashSet::new();
    for schema in &schemas {
        let rust_name = rust_type_name(&schema.name);
        if !used_schema_names.insert(rust_name.clone()) {
            anyhow::bail!("Rust schema name collision after normalization: {rust_name:?}");
        }
    }

    let mut out = String::new();
    for schema in schemas {
        emit_schema(&mut out, &schema.name, schema.ty, &resolver)?;
        out.push('\n');
    }
    Ok(out)
}

fn emit_schema(
    out: &mut String,
    name: &str,
    ty: &KclType,
    resolver: &SchemaNameResolver,
) -> anyhow::Result<()> {
    for line in ty.schema_doc.lines() {
        out.push_str("///");
        if !line.is_empty() {
            out.push(' ');
            out.push_str(line);
        }
        out.push('\n');
    }
    out.push_str("#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]\n");
    out.push_str(&format!("pub struct {} {{\n", rust_type_name(name)));

    let mut properties: Vec<&String> = ty.properties.keys().collect();
    properties.sort();
    let mut used_field_names = HashSet::new();
    for property in properties {
        let field = &ty.properties[property];
        if field.r#type == "function" {
            anyhow::bail!("Rust DTO generation does not support function field {property:?}");
        }
        if !field.description.is_empty() {
            for line in field.description.lines() {
                out.push_str(&format!("    /// {line}\n"));
            }
        }
        if !field.default.is_empty() {
            for line in field.default.lines() {
                out.push_str(&format!("    /// KCL default: {line}\n"));
            }
        }
        let field_name = rust_field_name(property);
        if !used_field_names.insert(field_name.clone()) {
            anyhow::bail!("Rust field name collision after normalization: {field_name:?}");
        }
        if field_name != *property {
            out.push_str(&format!("    #[serde(rename = {:?})]\n", property));
        }
        let optional = !ty.required.iter().any(|required| required == property);
        if optional {
            out.push_str("    #[serde(default)]\n");
        }
        let field_ty = rust_type(field, resolver, property)?;
        let field_ty = if optional {
            format!("Option<{field_ty}>")
        } else {
            field_ty
        };
        out.push_str(&format!("    pub {field_name}: {field_ty},\n"));
    }
    out.push_str("}\n");
    emit_validator(out, name, ty)?;
    Ok(())
}

fn emit_validator(out: &mut String, name: &str, ty: &KclType) -> anyhow::Result<()> {
    let fields = ty
        .properties
        .iter()
        .map(|(name, field)| {
            (
                name.clone(),
                (
                    rust_field_name(name),
                    field.clone(),
                    !ty.required.contains(name),
                ),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut errors = Vec::new();
    if let Some(base) = &ty.base_schema {
        collect_checks(base, &fields, &mut errors)?;
    }
    collect_checks(ty, &fields, &mut errors)?;

    out.push_str(&format!("impl {} {{\n", rust_type_name(name)));
    out.push_str("    pub fn validate(&self) -> Result<(), Vec<String>> {\n");
    if errors.is_empty() {
        out.push_str("        let errors = Vec::new();\n");
    } else {
        out.push_str("        let mut errors = Vec::new();\n");
    }
    for error in errors {
        out.push_str(&error);
    }
    out.push_str("        if errors.is_empty() {\n");
    out.push_str("            Ok(())\n");
    out.push_str("        } else {\n");
    out.push_str("            Err(errors)\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    Ok(())
}

fn collect_checks(
    ty: &KclType,
    fields: &HashMap<String, (String, KclType, bool)>,
    errors: &mut Vec<String>,
) -> anyhow::Result<()> {
    for check_json in &ty.checks_json {
        let check: ast::CheckExpr = serde_json::from_str(check_json).map_err(|error| {
            anyhow::anyhow!(
                "cannot decode KCL check AST for schema {:?}: {error}",
                ty.schema_name
            )
        })?;
        let mut context = RustExprContext {
            fields,
            locals: HashMap::new(),
            unwrapped_optional_fields: HashSet::new(),
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
            .unwrap_or_else(|| format!("{:?}", format!("KCL check failed: {test}")));
        let condition = condition.map_or_else(String::new, |condition| {
            format!("if {condition} && !({test})")
        });
        let condition = if condition.is_empty() {
            format!("if !({test})")
        } else {
            condition
        };
        errors.push(format!(
            "        {condition} {{ errors.push(({message}).to_string()); }}\n"
        ));
    }
    Ok(())
}

struct RustExprContext<'a> {
    fields: &'a HashMap<String, (String, KclType, bool)>,
    locals: HashMap<String, (String, KclType)>,
    unwrapped_optional_fields: HashSet<String>,
}

fn render_condition(expr: &ast::Expr, context: &RustExprContext<'_>) -> anyhow::Result<String> {
    if let ast::Expr::Identifier(identifier) = expr {
        if identifier.names.len() == 1 {
            let name = &identifier.names[0].node;
            if let Some((field_name, _, true)) = context.fields.get(name) {
                return Ok(format!("self.{field_name}.is_some()"));
            }
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

fn render_expr(expr: &ast::Expr, context: &RustExprContext<'_>) -> anyhow::Result<String> {
    Ok(match expr {
        ast::Expr::Identifier(identifier) => render_identifier(identifier, context)?,
        ast::Expr::Target(target) => {
            let name = target.name.node.as_str();
            if !target.paths.is_empty() {
                anyhow::bail!("indexed KCL targets are not supported in Rust validation")
            }
            render_field(name, context)?
        }
        ast::Expr::NumberLit(number) => {
            if number.binary_suffix.is_some() {
                anyhow::bail!("number multipliers are not supported in Rust validation")
            }
            number.to_string()
        }
        ast::Expr::StringLit(string) => format!("{:?}", string.value),
        ast::Expr::NameConstantLit(value) => match value.value {
            ast::NameConstant::True => "true".to_string(),
            ast::NameConstant::False => "false".to_string(),
            ast::NameConstant::None => "None".to_string(),
            ast::NameConstant::Undefined => {
                anyhow::bail!("Undefined is not supported in Rust validation")
            }
        },
        ast::Expr::Paren(paren) => format!("({})", render_expr(&paren.expr.node, context)?),
        ast::Expr::Unary(unary) => {
            let op = match unary.op {
                ast::UnaryOp::Not => "!",
                ast::UnaryOp::USub => "-",
                ast::UnaryOp::UAdd => "+",
                ast::UnaryOp::Invert => {
                    anyhow::bail!("bitwise inversion is not supported in Rust validation")
                }
            };
            format!("{op}({})", render_expr(&unary.operand.node, context)?)
        }
        ast::Expr::Binary(binary) => {
            let op = match binary.op {
                ast::BinOp::Add => "+",
                ast::BinOp::Sub => "-",
                ast::BinOp::Mul => "*",
                ast::BinOp::Div => "/",
                ast::BinOp::Mod => "%",
                ast::BinOp::Pow => ".pow",
                ast::BinOp::And => "&&",
                ast::BinOp::Or => "||",
                _ => anyhow::bail!(
                    "KCL operator {:?} is not supported in Rust validation",
                    binary.op
                ),
            };
            let left = render_expr(&binary.left.node, context)?;
            let right = render_expr(&binary.right.node, context)?;
            if binary.op == ast::BinOp::Pow {
                format!("({left}).powf({right} as f64)")
            } else {
                format!("({left} {op} {right})")
            }
        }
        ast::Expr::Compare(compare) => render_compare(compare, context)?,
        ast::Expr::Selector(selector) => {
            let receiver = render_expr(&selector.value.node, context)?;
            let attr = selector.attr.node.get_name();
            if attr == "upper" {
                format!("{receiver}.to_uppercase()")
            } else {
                format!("{receiver}.{}", rust_field_name(&attr))
            }
        }
        ast::Expr::Call(call) => render_call(call, context)?,
        ast::Expr::Quant(quant) => render_quant(quant, context)?,
        ast::Expr::List(list) => format!(
            "vec![{}]",
            list.elts
                .iter()
                .map(|element| render_expr(&element.node, context))
                .collect::<anyhow::Result<Vec<_>>>()?
                .join(", ")
        ),
        other => anyhow::bail!(
            "KCL expression {} is not supported in Rust validation",
            other.get_expr_name()
        ),
    })
}

fn render_identifier(
    identifier: &ast::Identifier,
    context: &RustExprContext<'_>,
) -> anyhow::Result<String> {
    let names = identifier.get_names();
    let name = names
        .first()
        .ok_or_else(|| anyhow::anyhow!("empty identifier in Rust validation"))?;
    if let Some((local, ty)) = context.locals.get(name) {
        return Ok(render_borrowed_value(local, ty));
    }
    if context.fields.contains_key(name) {
        let mut result = render_field(name, context)?;
        for member in names.iter().skip(1) {
            result.push('.');
            result.push_str(&rust_field_name(member));
        }
        return Ok(result);
    }
    anyhow::bail!("unknown identifier {name:?} in Rust validation")
}

fn render_field(name: &str, context: &RustExprContext<'_>) -> anyhow::Result<String> {
    let (field_name, ty, optional) = context
        .fields
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("unknown identifier {name:?} in Rust validation"))?;
    let field = format!("self.{field_name}");
    if *optional && context.unwrapped_optional_fields.contains(name) {
        let value = format!("{field}.as_ref().expect(\"guarded optional field\")");
        return Ok(render_borrowed_value(&value, ty));
    }
    if *optional {
        return Ok(field);
    }
    Ok(match ty.r#type.as_str() {
        "str" => format!("{field}.as_str()"),
        "schema" | "list" | "dict" | "any" | "union" => format!("&{field}"),
        _ => field,
    })
}

fn render_borrowed_value(value: &str, ty: &KclType) -> String {
    match ty.r#type.as_str() {
        "str" => format!("{value}.as_str()"),
        "int" | "float" | "bool" | "number_multiplier" => format!("*{value}"),
        _ => value.to_string(),
    }
}

fn render_compare(compare: &ast::Compare, context: &RustExprContext<'_>) -> anyhow::Result<String> {
    if compare.ops.len() != compare.comparators.len() {
        anyhow::bail!("malformed KCL comparison in Rust validation")
    }
    let mut left_expr = &compare.left.node;
    let mut left_type = expression_type(left_expr, context);
    let mut comparisons = Vec::new();
    for (op, comparator) in compare.ops.iter().zip(&compare.comparators) {
        let right_type = expression_type(&comparator.node, context);
        let left_expected = (right_type.as_deref() == Some("float")
            && is_integer_literal(left_expr))
        .then_some("float");
        let left = render_compare_operand(left_expr, context, left_expected)?;
        let right_expected = (left_type.as_deref() == Some("float")
            && is_integer_literal(&comparator.node))
        .then_some("float");
        let right = render_compare_operand(&comparator.node, context, right_expected)?;
        let comparison = match op {
            ast::CmpOp::Eq => format!("{left} == {right}"),
            ast::CmpOp::NotEq => format!("{left} != {right}"),
            ast::CmpOp::Lt => format!("{left} < {right}"),
            ast::CmpOp::LtE => format!("{left} <= {right}"),
            ast::CmpOp::Gt => format!("{left} > {right}"),
            ast::CmpOp::GtE => format!("{left} >= {right}"),
            ast::CmpOp::In => {
                let value = if expression_is_reference(&compare.left.node, context) {
                    left.clone()
                } else {
                    format!("&({left})")
                };
                format!("{right}.contains({value})")
            }
            ast::CmpOp::NotIn => {
                let value = if expression_is_reference(&compare.left.node, context) {
                    left.clone()
                } else {
                    format!("&({left})")
                };
                format!("!{right}.contains({value})")
            }
            ast::CmpOp::Is | ast::CmpOp::IsNot => {
                if let ast::Expr::NameConstantLit(value) = &comparator.node
                    && value.value == ast::NameConstant::None
                    && let Some((field_name, _, true)) =
                        optional_field_for_expr(&compare.left.node, context)
                {
                    let method = if *op == ast::CmpOp::Is {
                        "is_none"
                    } else {
                        "is_some"
                    };
                    format!("self.{field_name}.{method}()")
                } else {
                    anyhow::bail!("KCL identity checks are only supported against None")
                }
            }
            _ => anyhow::bail!(
                "KCL comparison {:?} is not supported in Rust validation",
                op
            ),
        };
        comparisons.push(comparison);
        left_expr = &comparator.node;
        left_type = right_type;
    }
    Ok(comparisons.join(" && "))
}

fn render_compare_operand(
    expr: &ast::Expr,
    context: &RustExprContext<'_>,
    expected_type: Option<&str>,
) -> anyhow::Result<String> {
    if let ast::Expr::NumberLit(number) = expr
        && expected_type == Some("float")
        && matches!(number.value, ast::NumberLitValue::Int(_))
    {
        return Ok(format!("{}.0", number));
    }
    render_expr(expr, context)
}

fn is_integer_literal(expr: &ast::Expr) -> bool {
    matches!(
        expr,
        ast::Expr::NumberLit(ast::NumberLit {
            value: ast::NumberLitValue::Int(_),
            ..
        })
    )
}

fn expression_type(expr: &ast::Expr, context: &RustExprContext<'_>) -> Option<String> {
    match expr {
        ast::Expr::Identifier(identifier) if !identifier.names.is_empty() => {
            let name = &identifier.names[0].node;
            context
                .fields
                .get(name)
                .map(|(_, ty, _)| ty.r#type.clone())
                .or_else(|| context.locals.get(name).map(|(_, ty)| ty.r#type.clone()))
        }
        ast::Expr::Target(target) => context
            .fields
            .get(target.name.node.as_str())
            .map(|(_, ty, _)| ty.r#type.clone()),
        ast::Expr::NumberLit(number) => Some(match number.value {
            ast::NumberLitValue::Int(_) => "int".to_string(),
            ast::NumberLitValue::Float(_) => "float".to_string(),
        }),
        ast::Expr::Call(call) => match &call.func.node {
            ast::Expr::Identifier(identifier) if identifier.get_name() == "len" => {
                Some("int".to_string())
            }
            ast::Expr::Identifier(identifier) if identifier.get_name().ends_with(".upper") => {
                Some("str".to_string())
            }
            _ => Some("bool".to_string()),
        },
        ast::Expr::Compare(_) => Some("bool".to_string()),
        _ => None,
    }
}

fn expression_is_reference(expr: &ast::Expr, context: &RustExprContext<'_>) -> bool {
    match expr {
        ast::Expr::Identifier(identifier) if identifier.names.len() == 1 => {
            let name = &identifier.names[0].node;
            context.locals.contains_key(name)
                || context.fields.get(name).is_some_and(|(_, _, optional)| {
                    *optional && context.unwrapped_optional_fields.contains(name)
                })
        }
        ast::Expr::Identifier(identifier) if identifier.names.len() > 1 => {
            let name = &identifier.names[0].node;
            context.fields.get(name).is_some_and(|(_, _, optional)| {
                *optional && context.unwrapped_optional_fields.contains(name)
            })
        }
        _ => false,
    }
}

fn optional_field_for_expr<'a>(
    expr: &ast::Expr,
    context: &'a RustExprContext<'_>,
) -> Option<&'a (String, KclType, bool)> {
    let ast::Expr::Identifier(identifier) = expr else {
        return None;
    };
    if identifier.names.len() != 1 {
        return None;
    }
    context.fields.get(&identifier.names[0].node)
}

fn render_call(call: &ast::CallExpr, context: &RustExprContext<'_>) -> anyhow::Result<String> {
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
            _ => anyhow::bail!("unsupported receiver for upper() in Rust validation"),
        };
        return Ok(format!("({receiver}).to_uppercase()"));
    }
    match function.as_str() {
        "len" if call.args.len() == 1 => {
            let arg = &call.args[0].node;
            let value = render_expr(arg, context)?;
            if expression_is_string(arg, context) {
                Ok(format!("({value}).chars().count()"))
            } else {
                Ok(format!("({value}).len()"))
            }
        }
        "regex.match" if call.args.len() == 2 => {
            let value = render_expr(&call.args[0].node, context)?;
            let pattern = render_expr(&call.args[1].node, context)?;
            Ok(format!(
                "regex::Regex::new(&({pattern})).is_ok_and(|regex| regex.is_match({value}))"
            ))
        }
        _ => {
            if let ast::Expr::Selector(selector) = &call.func.node
                && selector.attr.node.get_name() == "upper"
                && call.args.is_empty()
            {
                return Ok(format!(
                    "({}).to_uppercase()",
                    render_expr(&selector.value.node, context)?
                ));
            }
            anyhow::bail!("KCL call {function:?} is not supported in Rust validation")
        }
    }
}

fn expression_is_string(expr: &ast::Expr, context: &RustExprContext<'_>) -> bool {
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

fn render_quant(quant: &ast::QuantExpr, context: &RustExprContext<'_>) -> anyhow::Result<String> {
    if quant.variables.len() != 1 {
        anyhow::bail!("Rust validation supports quantifiers with one variable")
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
    let mut nested = RustExprContext {
        fields: context.fields,
        locals: context.locals.clone(),
        unwrapped_optional_fields: context.unwrapped_optional_fields.clone(),
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
                "conditional map/filter quantifiers are not supported in Rust validation"
            ),
        }
    } else {
        test
    };
    let method = match quant.op {
        ast::QuantOperation::All => "all",
        ast::QuantOperation::Any => "any",
        _ => anyhow::bail!("only all/any quantifiers are supported in Rust validation"),
    };
    Ok(format!("({target}).iter().{method}(|{variable}| {body})"))
}

fn rust_type(ty: &KclType, resolver: &SchemaNameResolver, field: &str) -> anyhow::Result<String> {
    Ok(match ty.r#type.as_str() {
        "str" => "String".to_string(),
        "int" | "number_multiplier" => "i64".to_string(),
        "float" => "f64".to_string(),
        "bool" => "bool".to_string(),
        "any" | "union" => "serde_json::Value".to_string(),
        "schema" => resolver.name_of_ty(ty),
        "list" => {
            let item = ty
                .item
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("list field {field:?} has no item type"))?;
            format!("Vec<{}>", rust_type(item, resolver, field)?)
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
                "std::collections::HashMap<{}, {}>",
                rust_type(key, resolver, field)?,
                rust_type(item, resolver, field)?
            )
        }
        other => anyhow::bail!("unsupported KCL type {other:?} for Rust field {field:?}"),
    })
}

fn rust_type_name(name: &str) -> String {
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

fn rust_field_name(name: &str) -> String {
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
    if is_rust_keyword(&result) {
        format!("r#{result}")
    } else {
        result
    }
}

fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
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
    fn emits_deterministic_serde_dtos_with_optional_and_nested_types() {
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
        let generated = generate_rust(&mapping).unwrap();

        assert!(generated.contains("pub struct Customer {"));
        assert!(generated.contains("pub addresses: Vec<Address>,"));
        assert!(generated.contains("pub age: Option<i64>,"));
        assert!(generated.contains("#[serde(rename = \"zipCode\")]"));
        assert!(generated.contains("pub zip_code: String,"));
        assert!(
            generated.find("pub struct Address").unwrap()
                < generated.find("pub struct Customer").unwrap()
        );
    }

    #[test]
    fn converts_rust_keywords_to_raw_field_identifiers() {
        assert_eq!(rust_field_name("type"), "r#type");
        assert_eq!(rust_field_name("zipCode"), "zip_code");
    }

    #[test]
    fn rejects_fields_that_normalize_to_the_same_rust_identifier() {
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
        let error = generate_rust(&mapping).unwrap_err();
        assert!(error.to_string().contains("field name collision"));
    }
}
