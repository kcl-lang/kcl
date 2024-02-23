//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl Type {
    pub fn type_str(&self) -> String {
        match self {
            Type::Any => KCL_TYPE_ANY.to_string(),
            Type::Bool => BUILTIN_TYPE_BOOL.to_string(),
            Type::BoolLit(ref v) => format!("{BUILTIN_TYPE_BOOL}({v})"),
            Type::Int => BUILTIN_TYPE_INT.to_string(),
            Type::IntLit(ref v) => format!("{BUILTIN_TYPE_INT}({v})"),
            Type::Float => BUILTIN_TYPE_FLOAT.to_string(),
            Type::FloatLit(ref v) => format!("{BUILTIN_TYPE_FLOAT}({v})"),
            Type::Str => BUILTIN_TYPE_STR.to_string(),
            Type::StrLit(ref v) => format!("{BUILTIN_TYPE_STR}({v})"),
            Type::List(ref v) => format!("[{}]", v.elem_type.type_str()),
            Type::Dict(ref v) => {
                format!("{{{}:{}}}", v.key_type.type_str(), v.elem_type.type_str())
            }
            Type::Union(ref v) => match v.elem_types.len() {
                0 => String::new(),
                1 => v.elem_types[0].type_str(),
                _ => {
                    let mut types = Vec::new();
                    let _ = v.elem_types.iter().map(|e| types.push(e.type_str()));
                    types.join(" | ")
                }
            },
            Type::Schema(ref v) => v.name.to_string(),
            Type::Func(ref _v) => "func".to_string(),
        }
    }
}
