use crate::eval::str_literal_eval;

use super::*;

/// Parse type string
pub fn parse_type_str(ty_str: &str) -> TypeRef {
    if ty_str.is_empty() {
        return Arc::new(Type::ANY);
    }
    let ty_str = ty_str_strip(ty_str);
    match TYPES_MAPPING.get(ty_str) {
        Some(ty) => Arc::new(ty.clone()),
        None => {
            if is_union_type_str(ty_str) {
                parse_union_type_str(ty_str)
            } else if is_literal_type_str(ty_str) {
                parse_lit_type_str(ty_str)
            } else if is_number_multiplier_literal_type_str(ty_str) {
                parse_number_multiplier_literal_type_str(ty_str)
            } else if is_dict_type_str(ty_str) {
                let (key_ty_str, val_ty_str) = separate_kv(&dereference_type(ty_str));
                Arc::new(Type::dict(
                    parse_type_str(&key_ty_str),
                    parse_type_str(&val_ty_str),
                ))
            } else if is_list_type_str(ty_str) {
                Arc::new(Type::list(parse_type_str(&dereference_type(ty_str))))
            } else {
                parse_named_type_str(ty_str)
            }
        }
    }
}

/// is_literal_type returns the type string whether is a literal type
pub fn is_literal_type_str(ty_str: &str) -> bool {
    if NAME_CONSTANTS.contains(&ty_str) {
        return true;
    }
    if ty_str.starts_with('\"') {
        return ty_str.ends_with('\"');
    }
    if ty_str.starts_with('\'') {
        return ty_str.ends_with('\'');
    }
    matches!(ty_str.parse::<f64>(), Ok(_))
}

/// is_dict_type returns the type string whether is a dict type
#[inline]
pub fn is_dict_type_str(ty: &str) -> bool {
    ty.len() >= 2 && &ty[0..1] == "{" && &ty[ty.len() - 1..] == "}"
}

/// is_list_type returns the type string whether is a list type
#[inline]
pub fn is_list_type_str(ty: &str) -> bool {
    ty.len() >= 2 && &ty[0..1] == "[" && &ty[ty.len() - 1..] == "]"
}

#[inline]
pub fn is_builtin_type_str(ty: &str) -> bool {
    BUILTIN_TYPES.contains(&ty)
}

/// is schema expected type
pub fn is_schema_type_str(expected_type: &str) -> bool {
    if expected_type.is_empty() {
        return true;
    }
    !is_list_type_str(expected_type)
        && !is_dict_type_str(expected_type)
        && !is_builtin_type_str(expected_type)
        && !is_literal_type_str(expected_type)
}

/// is union type
pub fn is_union_type_str(ty: &str) -> bool {
    let mut stack = String::new();
    let mut i = 0;
    while i < ty.chars().count() {
        let c = ty.chars().nth(i).unwrap();
        if c == '|' && stack.is_empty() {
            return true;
        } else if c == '[' || c == '{' {
            stack.push(c);
        } else if c == ']' || c == '}' {
            stack.pop();
        } else if c == '\"' {
            let t = &ty[i..];
            let re = fancy_regex::Regex::new(r#""(?!"").*?(?<!\\)(\\\\)*?""#).unwrap();
            if let Ok(Some(v)) = re.find(t) {
                i += v.range().end - 1;
            }
        } else if c == '\'' {
            let t = &ty[i..];
            let re = fancy_regex::Regex::new(r#"'(?!'').*?(?<!\\)(\\\\)*?'"#).unwrap();
            if let Ok(Some(v)) = re.find(t) {
                i += v.range().end - 1;
            }
        }
        i += 1;
    }
    false
}

/// is number multiplier literal type
fn is_number_multiplier_literal_type_str(ty_str: &str) -> bool {
    let re = fancy_regex::Regex::new(NUMBER_MULTIPLIER_REGEX).unwrap();
    match re.is_match(ty_str) {
        Ok(ok) => ok,
        _ => false,
    }
}

/// separate_kv split the union type and do not split '|' in dict and list
/// e.g., "int|str" -> vec!["int", "str"]
pub fn split_type_union(ty_str: &str) -> Vec<&str> {
    let mut i = 0;
    let mut s_index = 0;
    let mut stack = String::new();
    let mut types: Vec<&str> = vec![];
    while i < ty_str.chars().count() {
        let c = ty_str.chars().nth(i).unwrap();
        if c == '|' && stack.is_empty() {
            types.push(&ty_str[s_index..i]);
            s_index = i + 1;
        }
        // List/Dict type
        else if c == '[' || c == '{' {
            stack.push(c);
        }
        // List/Dict type
        else if c == ']' || c == '}' {
            stack.pop();
        }
        // String literal type
        else if c == '\"' {
            let t = &ty_str[i..];
            let re = fancy_regex::Regex::new(r#""(?!"").*?(?<!\\)(\\\\)*?""#).unwrap();
            if let Ok(Some(v)) = re.find(t) {
                i += v.range().end - 1;
            }
        }
        // String literal type
        else if c == '\'' {
            let t = &ty_str[i..];
            let re = fancy_regex::Regex::new(r#"'(?!'').*?(?<!\\)(\\\\)*?'"#).unwrap();
            if let Ok(Some(v)) = re.find(t) {
                i += v.range().end - 1;
            }
        }
        i += 1;
    }
    types.push(&ty_str[s_index..]);
    types
}

/// Parse union type string.
pub fn parse_union_type_str(ty_str: &str) -> TypeRef {
    let types = split_type_union(ty_str)
        .iter()
        .map(|ty_str| parse_type_str(ty_str))
        .collect::<Vec<TypeRef>>();
    sup(&types)
}

/// Parse literal type string.
pub fn parse_lit_type_str(ty_str: &str) -> TypeRef {
    // Bool literal type.
    if ty_str == NAME_CONSTANT_TRUE {
        return Arc::new(Type::bool_lit(true));
    } else if ty_str == NAME_CONSTANT_FALSE {
        return Arc::new(Type::bool_lit(false));
    }
    match ty_str.parse::<i64>() {
        // Float literal type.
        Ok(v) => Arc::new(Type::int_lit(v)),
        Err(_) => match ty_str.parse::<f64>() {
            // Int literal type.
            Ok(v) => Arc::new(Type::float_lit(v)),
            // Maybe string literal type
            Err(_) => match str_literal_eval(ty_str, false, false) {
                Some(v) => Arc::new(Type::str_lit(&v)),
                None => bug!("invalid literal type string {}", ty_str),
            },
        },
    }
}

/// Parse number multiplier literal type.
pub fn parse_number_multiplier_literal_type_str(ty_str: &str) -> TypeRef {
    let suffix_index = if &ty_str[ty_str.len() - 1..] == kclvm_runtime::IEC_SUFFIX {
        ty_str.len() - 2
    } else {
        ty_str.len() - 1
    };
    let (value, suffix) = (
        match ty_str[..suffix_index].parse::<i64>() {
            Ok(v) => v,
            Err(_) => bug!("invalid number multiplier literal type str {}", ty_str),
        },
        &ty_str[suffix_index..],
    );
    Arc::new(Type::number_multiplier(
        kclvm_runtime::cal_num(value, suffix),
        value,
        suffix,
    ))
}

/// Please note Named type to find it in the scope (e.g. schema type, type alias).
#[inline]
pub fn parse_named_type_str(ty_str: &str) -> TypeRef {
    Arc::new(Type::named(ty_str))
}

/// separate_kv function separates key_type and value_type in the dictionary type strings,
/// e.g., "str:str" -> ("str", "str")
pub fn separate_kv(expected_type: &str) -> (String, String) {
    let mut stack = String::new();
    for (n, c) in expected_type.chars().enumerate() {
        if c == '[' || c == '{' {
            stack.push(c)
        } else if c == ']' {
            if &stack[stack.len() - 1..] != "[" {
                panic!("invalid type string {}", expected_type);
            }
            stack.pop();
        } else if c == '}' {
            if &stack[stack.len() - 1..] != "{" {
                panic!("invalid type string {}", expected_type);
            }
            stack.pop();
        } else if c == ':' {
            if !stack.is_empty() {
                panic!("invalid type string {}", expected_type);
            }
            return (
                expected_type[..n].to_string(),
                expected_type[n + 1..].to_string(),
            );
        }
    }
    ("".to_string(), "".to_string())
}

/// dereference_type function removes the first and last [] {} in the type string
/// e.g., "\[int\]" -> "int"
pub fn dereference_type(tpe: &str) -> String {
    if tpe.len() > 1
        && ((&tpe[0..1] == "[" && &tpe[tpe.len() - 1..] == "]")
            || (&tpe[0..1] == "{" && &tpe[tpe.len() - 1..] == "}"))
    {
        return tpe[1..tpe.len() - 1].to_string();
    }
    tpe.to_string()
}

#[inline]
fn ty_str_strip(ty_str: &str) -> &str {
    let chars = " \r\n";
    ty_str.trim_matches(|c| chars.contains(c))
}
