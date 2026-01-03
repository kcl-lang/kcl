use anyhow::Result;
use kcl_ast::ast;
use kcl_config::settings::KeyValuePair;
use kcl_parser::parse_expr;

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
        // Check if it's a number literal with scientific notation that should be treated as a string
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(value) {
            if json_val.is_number() && looks_like_scientific_notation(value) {
                // If it looks like scientific notation (e.g., 12e1), treat it as a string
                return format!("{:?}", value);
            }
        }
        return value.to_string();
    }
    // Is a KCL value, eval KCL value to a json string.
    let expr = parse_expr(value);
    match expr {
        Some(expr) => match expr.node {
            ast::Expr::NameConstantLit(lit) => lit.value.json_value().to_string(),
            ast::Expr::NumberLit(_) | ast::Expr::StringLit(_) => {
                // For number literals, check if it looks like scientific notation
                if looks_like_scientific_notation(value) {
                    format!("{:?}", value)
                } else {
                    value.to_string()
                }
            }
            // Not a normal literal,  regard it as a string value.
            _ => format!("{:?}", value),
        },
        // Invalid value, regard it as a string value.
        None => format!("{:?}", value),
    }
}

/// Check if a string looks like scientific notation (e.g., 12e1, 1.5e-3)
fn looks_like_scientific_notation(s: &str) -> bool {
    let s = s.trim();
    // Check if it contains 'e' or 'E' (scientific notation marker)
    let has_e = s.contains('e') || s.contains('E');
    // Make sure it's not just "e" or starts with e (not a valid number)
    if !has_e {
        return false;
    }
    // Check if it looks like a valid number with scientific notation
    // Pattern: [digits][.digits][eE][+-][digits]
    let parts: Vec<&str> = s.split(|c: char| c == 'e' || c == 'E').collect();
    if parts.len() != 2 {
        return false;
    }
    let base = parts[0];
    let exponent = parts[1];
    // Base should look like a number (integer or float)
    let base_is_number = base.parse::<f64>().is_ok() || base.parse::<i64>().is_ok();
    // Exponent should look like an integer (optional +/- sign)
    let exp_is_number = exponent.parse::<i64>().is_ok();
    base_is_number && exp_is_number
}
