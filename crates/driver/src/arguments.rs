use anyhow::Result;
use kclvm_ast::ast;
use kclvm_config::settings::KeyValuePair;
use kclvm_parser::parse_expr;

/// Parse key value pair string k=v to [KeyValuePair], note the value will be convert a json string.
pub fn parse_key_value_pair(spec: &str) -> Result<KeyValuePair> {
    let split_values = spec.split('=').collect::<Vec<&str>>();
    if split_values.len() == 2
        && !split_values[0].trim().is_empty()
        && !split_values[1].trim().is_empty()
    {
        Ok(KeyValuePair {
            key: split_values[0].to_string(),
            value: val_to_json(split_values[1]).into(),
        })
    } else {
        Err(anyhow::anyhow!("Invalid value for top level arguments"))
    }
}

/// Convert the value string to the json string.
fn val_to_json(value: &str) -> String {
    // If it is a json string, returns it.
    if serde_json::from_str::<serde_json::Value>(value).is_ok() {
        return value.to_string();
    }
    // Is a KCL value, eval KCL value to a json string.
    let expr = parse_expr(value);
    match expr {
        Some(expr) => match expr.node {
            ast::Expr::NameConstantLit(lit) => lit.value.json_value().to_string(),
            ast::Expr::NumberLit(_) | ast::Expr::StringLit(_) => value.to_string(),
            // Not a normal literal,  regard it as a string value.
            _ => format!("{:?}", value),
        },
        // Invalid value, regard it as a string value.
        None => format!("{:?}", value),
    }
}
