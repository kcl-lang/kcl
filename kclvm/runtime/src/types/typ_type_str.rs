// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

impl Type {
    pub fn type_str(&self) -> String {
        match self {
            Type::any_type => KCL_TYPE_ANY.to_string(),
            Type::bool_type => BUILTIN_TYPE_BOOL.to_string(),
            Type::bool_lit_type(ref v) => format!("{BUILTIN_TYPE_BOOL}({v})"),
            Type::int_type => BUILTIN_TYPE_INT.to_string(),
            Type::int_lit_type(ref v) => format!("{BUILTIN_TYPE_INT}({v})"),
            Type::float_type => BUILTIN_TYPE_FLOAT.to_string(),
            Type::float_lit_type(ref v) => format!("{BUILTIN_TYPE_FLOAT}({v})"),
            Type::str_type => BUILTIN_TYPE_STR.to_string(),
            Type::str_lit_type(ref v) => format!("{BUILTIN_TYPE_STR}({v})"),
            Type::list_type(ref v) => format!("[{}]", v.elem_type.type_str()),
            Type::dict_type(ref v) => {
                format!("{{{}:{}}}", v.key_type.type_str(), v.elem_type.type_str())
            }
            Type::union_type(ref v) => match v.elem_types.len() {
                0 => String::new(),
                1 => v.elem_types[0].type_str(),
                _ => {
                    let mut types = Vec::new();
                    let _ = v.elem_types.iter().map(|e| types.push(e.type_str()));
                    types.join("|")
                }
            },
            Type::schema_type(ref v) => v.name.to_string(),
            Type::func_type(ref _v) => "func".to_string(),
        }
    }
}
