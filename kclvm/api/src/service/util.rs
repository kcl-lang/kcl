use std::fmt::Display;

use serde::Serialize;

/// Transform the str with zero value into [`Option<String>`]
#[inline]
pub(crate) fn transform_str_para(para: &str) -> Option<String> {
    if para.is_empty() {
        None
    } else {
        Some(para.to_string())
    }
}

/// Transform the [`Result<V, E>`]  into [`serde_json::Value`]
#[inline]
pub(crate) fn result_to_json_value<V, E>(val: &Result<V, E>) -> serde_json::Value
where
    V: Serialize,
    E: Display,
{
    match val {
        Ok(val) => serde_json::to_value(val).unwrap(),
        Err(err) => serde_json::Value::String(err.to_string()),
    }
}
