//! Copyright The KCL Authors. All rights reserved.

extern crate fancy_regex;

use crate::*;
use std::mem::transmute_copy;

pub const BUILTIN_TYPE_INT: &str = "int";
pub const BUILTIN_TYPE_FLOAT: &str = "float";
pub const BUILTIN_TYPE_STR: &str = "str";
pub const BUILTIN_TYPE_BOOL: &str = "bool";
pub const BUILTIN_TYPES: [&str; 4] = [
    BUILTIN_TYPE_STR,
    BUILTIN_TYPE_BOOL,
    BUILTIN_TYPE_INT,
    BUILTIN_TYPE_FLOAT,
];
pub const KCL_TYPE_NONE: &str = "NoneType";
pub const KCL_TYPE_UNDEFINED: &str = "UndefinedType";
pub const KCL_TYPE_ANY: &str = "any";
pub const KCL_TYPE_LIST: &str = "list";
pub const KCL_TYPE_DICT: &str = "dict";
pub const KCL_TYPE_FUNCTION: &str = "function";
pub const KCL_TYPE_NUMBER_MULTIPLY: &str = "number_multiplier";
pub const KCL_NAME_CONSTANT_NONE: &str = "None";
pub const KCL_NAME_CONSTANT_UNDEFINED: &str = "Undefined";
pub const KCL_NAME_CONSTANT_TRUE: &str = "True";
pub const KCL_NAME_CONSTANT_FALSE: &str = "False";
pub const KCL_NAME_CONSTANTS: [&str; 4] = [
    KCL_NAME_CONSTANT_NONE,
    KCL_NAME_CONSTANT_UNDEFINED,
    KCL_NAME_CONSTANT_TRUE,
    KCL_NAME_CONSTANT_FALSE,
];
pub const NUMBER_MULTIPLIER_TYPE: &str = "units.NumberMultiplier";
pub const NUMBER_MULTIPLIER_REGEX: &str =
    r"^([1-9][0-9]{0,63})(E|P|T|G|M|K|k|m|u|n|Ei|Pi|Ti|Gi|Mi|Ki)$";

pub type SchemaTypeFunc = unsafe extern "C" fn(
    *mut kclvm_context_t,
    *const kclvm_value_ref_t,
    *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t;

// common
impl ValueRef {
    pub fn type_str(&self) -> String {
        match &*self.rc.borrow() {
            Value::undefined => String::from(KCL_TYPE_UNDEFINED),
            Value::none => String::from(KCL_TYPE_NONE),
            Value::bool_value(..) => String::from(BUILTIN_TYPE_BOOL),
            Value::int_value(..) => String::from(BUILTIN_TYPE_INT),
            Value::float_value(..) => String::from(BUILTIN_TYPE_FLOAT),
            Value::unit_value(_, raw, suffix) => {
                format!("{KCL_TYPE_NUMBER_MULTIPLY}({raw}{suffix})")
            }
            Value::str_value(..) => String::from(BUILTIN_TYPE_STR),
            Value::list_value(..) => String::from(KCL_TYPE_LIST),
            Value::dict_value(..) => String::from(KCL_TYPE_DICT),
            Value::schema_value(ref v) => v.name.clone(),
            Value::func_value(..) => String::from(KCL_TYPE_FUNCTION),
        }
    }
}

/// Use the schema instance to build a new schema instance using the schema construct function
pub fn resolve_schema(ctx: &mut Context, schema: &ValueRef, keys: &[String]) -> ValueRef {
    if !schema.is_schema() {
        return schema.clone();
    }
    let schema_value = schema.as_schema();
    let schema_type_name = schema_runtime_type(&schema_value.name, &schema_value.pkgpath);
    let now_meta_info = ctx.panic_info.clone();
    let has_schema_type = { ctx.all_schemas.contains_key(&schema_type_name) };
    if has_schema_type {
        let schema_type = { ctx.all_schemas.get(&schema_type_name).unwrap().clone() };
        let schema_type = schema_type.func.as_function();
        let schema_fn_ptr = schema_type.fn_ptr;
        let keys = keys.iter().map(|v| v.as_str()).collect();
        let config = schema.dict_get_entries(keys);
        let config_new = config.clone();
        let config_meta = schema_config_meta(
            &ctx.panic_info.kcl_file,
            ctx.panic_info.kcl_line as u64,
            ctx.panic_info.kcl_col as u64,
        );
        let config_meta_new = config_meta.clone();
        let value = unsafe {
            let schema_fn: SchemaTypeFunc = transmute_copy(&schema_fn_ptr);
            let cal_map = kclvm_value_Dict(ctx as *mut Context);
            let list = schema_value.args.clone().into_raw(ctx);
            // Schema function closures
            // is sub schema
            kclvm_list_append(list, ValueRef::bool(false).into_raw(ctx));
            // config meta
            kclvm_list_append(list, config_meta.into_raw(ctx));
            // schema
            kclvm_list_append(list, config.into_raw(ctx));
            // config
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // optional mapping
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // cal order map
            kclvm_list_append(list, cal_map);
            // backtrack level map
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // backtrack cache
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // record instance
            kclvm_list_append(list, ValueRef::bool(false).into_raw(ctx));
            // instance pkgpath
            kclvm_list_append(
                list,
                ValueRef::str(&now_meta_info.kcl_pkgpath).into_raw(ctx),
            );
            let dict = schema_value.kwargs.clone().into_raw(ctx);
            schema_fn(ctx, list, dict);
            let list = schema_value.args.clone().into_raw(ctx);
            // Schema function closures
            // is sub schema
            kclvm_list_append(list, ValueRef::bool(true).into_raw(ctx));
            // config meta
            kclvm_list_append(list, config_meta_new.into_raw(ctx));
            // schema
            kclvm_list_append(list, config_new.into_raw(ctx));
            // config
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // optional mapping
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // cal order map
            kclvm_list_append(list, cal_map);
            // backtrack level map
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // backtrack cache
            kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
            // record instance
            kclvm_list_append(list, ValueRef::bool(true).into_raw(ctx));
            // instance pkgpath
            kclvm_list_append(
                list,
                ValueRef::str(&now_meta_info.kcl_pkgpath).into_raw(ctx),
            );
            let value = schema_fn(ctx, list, dict);
            ptr_as_ref(value)
        };
        ctx.panic_info = now_meta_info;
        return value.clone();
    }
    ctx.panic_info = now_meta_info;
    schema.clone()
}

/// Type pack and check ValueRef with the expected type vector
pub fn type_pack_and_check(
    ctx: &mut Context,
    value: &ValueRef,
    expected_types: Vec<&str>,
) -> ValueRef {
    if value.is_none_or_undefined() || expected_types.is_empty() {
        return value.clone();
    }
    let is_schema = value.is_schema();
    let value_tpe = value.type_str();
    let mut checked = false;
    let mut converted_value = value.clone();
    let expected_type = &expected_types.join(" | ").replace('@', "");
    for tpe in expected_types {
        let tpe = if !tpe.contains('.') {
            match ctx.import_names.get(tpe) {
                Some(mapping) => mapping.keys().next().unwrap(),
                None => tpe,
            }
        } else {
            tpe
        }
        .to_string();
        if !is_schema {
            converted_value = convert_collection_value(ctx, value, &tpe);
        }
        // Runtime type check
        checked = check_type(&converted_value, &tpe);
        if checked {
            break;
        }
    }
    if !checked {
        panic!("expect {expected_type}, got {value_tpe}");
    }
    converted_value
}

/// Convert collection value including dict/list to the potential schema
pub fn convert_collection_value(ctx: &mut Context, value: &ValueRef, tpe: &str) -> ValueRef {
    // May be a type alias.
    let tpe = if !tpe.contains('.') {
        match ctx.import_names.get(tpe) {
            Some(mapping) => mapping.keys().next().unwrap(),
            None => tpe,
        }
    } else {
        tpe
    }
    .to_string();
    if tpe.is_empty() || tpe == KCL_TYPE_ANY {
        return value.clone();
    }
    let is_collection = value.is_list() || value.is_dict();
    let invalid_match_dict = is_dict_type(&tpe) && !value.is_dict();
    let invalid_match_list = is_list_type(&tpe) && !value.is_list();
    let invalid_match = invalid_match_dict || invalid_match_list;
    if !is_collection || invalid_match {
        return value.clone();
    }
    // Convert a vlaue to union types e.g., {a: 1} => A | B
    if is_type_union(&tpe) {
        let types = split_type_union(&tpe);
        convert_collection_value_with_union_types(ctx, value, &types)
    } else if is_dict_type(&tpe) {
        //let (key_tpe, value_tpe) = separate_kv(tpe);
        let (_, value_tpe) = separate_kv(&dereference_type(&tpe));
        let mut expected_dict = ValueRef::dict(None);
        let dict_ref = value.as_dict_ref();
        expected_dict
            .set_potential_schema_type(&dict_ref.potential_schema.clone().unwrap_or_default());
        for (k, v) in &dict_ref.values {
            let expected_value = convert_collection_value(ctx, v, &value_tpe);
            let op = dict_ref
                .ops
                .get(k)
                .unwrap_or(&ConfigEntryOperationKind::Union);
            let index = dict_ref.insert_indexs.get(k).unwrap_or(&-1);
            expected_dict.dict_update_entry(k, &expected_value, op, index)
        }
        expected_dict
    } else if is_list_type(&tpe) {
        let expected_type = dereference_type(&tpe);
        let mut expected_list = ValueRef::list(None);
        let list_ref = value.as_list_ref();
        for v in &list_ref.values {
            let expected_value = convert_collection_value(ctx, v, &expected_type);
            expected_list.list_append(&expected_value)
        }
        expected_list
    } else if BUILTIN_TYPES.contains(&tpe.as_str()) {
        value.clone()
    } else {
        let now_meta_info = ctx.panic_info.clone();
        let mut schema_type_name = if tpe.contains('.') {
            tpe.to_string()
        } else {
            format!(
                "{}.{}",
                if now_meta_info.kcl_pkgpath.is_empty() {
                    MAIN_PKG_PATH
                } else {
                    now_meta_info.kcl_pkgpath.as_str()
                },
                tpe
            )
        };

        if schema_type_name.contains('.') {
            let splits: Vec<&str> = schema_type_name.rsplitn(2, '.').collect();
            let pkgname = splits[1];
            let name = splits[0];
            match ctx.import_names.get(&now_meta_info.kcl_file) {
                Some(mapping) => {
                    if let Some(pkgpath) = mapping.get(pkgname) {
                        schema_type_name = format!("{pkgpath}.{name}");
                    }
                }
                None => {
                    for (_, mapping) in &ctx.import_names {
                        if let Some(pkgpath) = mapping.get(pkgname) {
                            schema_type_name = format!("{pkgpath}.{name}");
                            break;
                        }
                    }
                }
            }
        }
        let has_schema_type = { ctx.all_schemas.contains_key(&schema_type_name) };
        if has_schema_type {
            let schema_type = { ctx.all_schemas.get(&schema_type_name).unwrap().clone() };
            let schema_fn = schema_type.func.as_function();
            let schema_fn_ptr = schema_fn.fn_ptr;
            let value = unsafe {
                let schema_fn: SchemaTypeFunc = transmute_copy(&schema_fn_ptr);
                let cal_order = kclvm_value_Dict(ctx as *mut Context);
                let list = kclvm_value_List(ctx as *mut Context);
                // Schema function closures
                // is_sub_schema
                kclvm_list_append(list, ValueRef::bool(false).into_raw(ctx));
                // config meta
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // config
                kclvm_list_append(list, value.clone().into_raw(ctx));
                // schema
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // optional mapping
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // cal order map
                kclvm_list_append(list, cal_order);
                // backtrack level map
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // backtrack cache
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // record instance
                kclvm_list_append(list, ValueRef::bool(false).into_raw(ctx));
                // instance pkgpath
                kclvm_list_append(
                    list,
                    ValueRef::str(&now_meta_info.kcl_pkgpath).into_raw(ctx),
                );
                let dict = kclvm_value_Dict(ctx as *mut Context);
                schema_fn(ctx, list, dict);
                let list = kclvm_value_List(ctx as *mut Context);

                // Try convert the  config to schema, if failed, return the config
                if !value.is_fit_schema(&schema_type, ptr_as_ref(cal_order)) {
                    return value.clone();
                }

                // Schema function closures
                // is_sub_schema
                kclvm_list_append(list, ValueRef::bool(true).into_raw(ctx));
                // config meta
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // config
                kclvm_list_append(list, value.clone().into_raw(ctx));
                // schema
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // optional mapping
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // cal order map
                kclvm_list_append(list, cal_order);
                // backtrack level map
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // backtrack cache
                kclvm_list_append(list, kclvm_value_Dict(ctx as *mut Context));
                // record instance
                kclvm_list_append(list, ValueRef::bool(true).into_raw(ctx));
                // instance pkgpath
                kclvm_list_append(
                    list,
                    ValueRef::str(&now_meta_info.kcl_pkgpath).into_raw(ctx),
                );
                let value = schema_fn(ctx, list, dict);
                ptr_as_ref(value)
            };
            ctx.panic_info = now_meta_info;
            return value.clone();
        }
        ctx.panic_info = now_meta_info;
        value.clone()
    }
}

/// Convert collection value including dict/list to the potential schema and return errors.
pub fn convert_collection_value_with_union_types(
    ctx: &mut Context,
    value: &ValueRef,
    types: &[&str],
) -> ValueRef {
    if value.is_schema() {
        value.clone()
    } else {
        for tpe in types {
            // Try match every type and convert the value, if matched, return the value.
            let value = convert_collection_value(ctx, value, tpe);
            if check_type(&value, tpe) {
                return value;
            }
        }
        value.clone()
    }
}

/// check_type returns the value wether match the given the type string
pub fn check_type(value: &ValueRef, tpe: &str) -> bool {
    if tpe.is_empty() || tpe == KCL_TYPE_ANY {
        return true;
    }
    if value.is_none_or_undefined() {
        return true;
    }
    if is_type_union(tpe) {
        return check_type_union(value, tpe);
    }

    if check_type_literal(value, tpe) {
        return true;
    }

    if check_number_multiplier_type(value, tpe) {
        return true;
    }
    // if value type is a dict type e.g. {"k": "v"}
    else if value.is_dict() {
        return check_type_dict(value, tpe);
    }
    // if value type is a list type e.g. [1, 2, 3]
    else if value.is_list() {
        return check_type_list(value, tpe);
    } else if !value.is_none_or_undefined() {
        // if value type is a built-in type e.g. str, int, float, bool
        if match_builtin_type(value, tpe) {
            return true;
        }
        if value.is_schema() {
            // not list/dict, not built-in type, treat as user defined schema,
            // do not check user schema type because it has been checked at compile time
            return is_schema_type(tpe);
        }
        // Type error
        return false;
    }
    // Type error
    false
}

/// check_type_union returns the value wether match the given the union type string
pub fn check_type_union(value: &ValueRef, tpe: &str) -> bool {
    let expected_types = split_type_union(tpe);
    if expected_types.len() <= 1 {
        false
    } else {
        expected_types.iter().any(|tpe| check_type(value, tpe))
    }
}

/// check_type_literal returns the value wether match the given the literal type string
pub fn check_type_literal(value: &ValueRef, tpe: &str) -> bool {
    if !is_literal_type(tpe) {
        return false;
    }
    if value.is_none() {
        return tpe == KCL_NAME_CONSTANT_NONE;
    }
    if value.is_undefined() {
        return tpe == KCL_NAME_CONSTANT_UNDEFINED;
    }
    if value.is_bool() {
        let value = value.as_bool();
        if !value {
            return tpe == KCL_NAME_CONSTANT_FALSE;
        }
        if value {
            return tpe == KCL_NAME_CONSTANT_TRUE;
        }
    }
    if value.is_int() || value.is_float() {
        return value.to_string() == *tpe;
    }
    if value.is_str() {
        let value = format!("{:?}", value.as_str());
        return value == *tpe;
    }
    false
}

/// check_number_multiplier_type returns the value wether match the given the type string
pub fn check_number_multiplier_type(value: &ValueRef, tpe: &str) -> bool {
    if value.is_unit() {
        if is_number_multiplier_literal_type(tpe) {
            let (_, raw, suffix) = value.as_unit();
            return format!("{raw}{suffix}") == tpe;
        }
        return tpe == NUMBER_MULTIPLIER_TYPE;
    }
    false
}

/// check_type_dict returns the value wether match the given the dict type string
pub fn check_type_dict(value: &ValueRef, tpe: &str) -> bool {
    if tpe.is_empty() {
        return true;
    }
    if !is_dict_type(tpe) || !value.is_dict() {
        return false;
    }
    let expected_type = dereference_type(tpe);
    let (_, expected_value_type) = separate_kv(&expected_type);
    let dict_ref = value.as_dict_ref();
    for (_, v) in &dict_ref.values {
        if !check_type(v, &expected_value_type) {
            return false;
        }
    }
    true
}

/// check_type_list returns the value wether match the given the list type string
pub fn check_type_list(value: &ValueRef, tpe: &str) -> bool {
    if tpe.is_empty() {
        return true;
    }
    if !is_list_type(tpe) || !value.is_list() {
        return false;
    }
    let expected_type = dereference_type(tpe);
    let list_ref = value.as_list_ref();
    for v in &list_ref.values {
        if !check_type(v, &expected_type) {
            return false;
        }
    }
    true
}

/// match_builtin_type returns the value wether match the given the type string
#[inline]
pub fn match_builtin_type(value: &ValueRef, tpe: &str) -> bool {
    value.type_str() == *tpe || (value.type_str() == BUILTIN_TYPE_INT && tpe == BUILTIN_TYPE_FLOAT)
}

/// is_literal_type returns the type string whether is a literal type
pub fn is_literal_type(tpe: &str) -> bool {
    if KCL_NAME_CONSTANTS.contains(&tpe) {
        return true;
    }
    if tpe.starts_with('\"') {
        return tpe.ends_with('\"');
    }
    if tpe.starts_with('\'') {
        return tpe.ends_with('\'');
    }
    if ValueRef::str(tpe).str_isdigit().is_truthy() {
        return true;
    }
    if ValueRef::str(tpe.replacen('.', "", 1).as_str())
        .str_isdigit()
        .is_truthy()
        && tpe.matches('.').count() < 2
    {
        return true;
    }
    false
}

/// is_dict_type returns the type string whether is a dict type
#[inline]
pub fn is_dict_type(tpe: &str) -> bool {
    let count = tpe.chars().count();
    count >= 2
        && matches!(tpe.chars().next(), Some('{'))
        && matches!(tpe.chars().nth(count - 1), Some('}'))
}

/// is_list_type returns the type string whether is a list type
#[inline]
pub fn is_list_type(tpe: &str) -> bool {
    let count = tpe.chars().count();
    count >= 2
        && matches!(tpe.chars().next(), Some('['))
        && matches!(tpe.chars().nth(count - 1), Some(']'))
}

#[inline]
pub fn is_builtin_type(tpe: &str) -> bool {
    BUILTIN_TYPES.contains(&tpe)
}

/// is schema expected type
pub fn is_schema_type(expected_type: &str) -> bool {
    if expected_type.is_empty() {
        return true;
    }
    !is_list_type(expected_type)
        && !is_dict_type(expected_type)
        && !is_builtin_type(expected_type)
        && !is_literal_type(expected_type)
}

/// is union type
pub fn is_type_union(tpe: &str) -> bool {
    let mut stack = String::new();
    let mut i = 0;
    while i < tpe.chars().count() {
        let c = tpe.chars().nth(i).unwrap();
        if c == '|' && stack.is_empty() {
            return true;
        } else if c == '[' || c == '{' {
            stack.push(c);
        } else if c == ']' || c == '}' {
            stack.pop();
        } else if c == '\"' {
            let t: String = tpe.chars().skip(i).collect();
            let re = fancy_regex::Regex::new(r#""(?!"").*?(?<!\\)(\\\\)*?""#).unwrap();
            if let Ok(Some(v)) = re.find(&t) {
                i += v.as_str().chars().count() - 1;
            }
        } else if c == '\'' {
            let t: String = tpe.chars().skip(i).collect();
            let re = fancy_regex::Regex::new(r#"'(?!'').*?(?<!\\)(\\\\)*?'"#).unwrap();
            if let Ok(Some(v)) = re.find(&t) {
                i += v.as_str().chars().count() - 1;
            }
        }
        i += 1;
    }
    false
}

/// is number multiplier literal type
fn is_number_multiplier_literal_type(tpe: &str) -> bool {
    let re = fancy_regex::Regex::new(NUMBER_MULTIPLIER_REGEX).unwrap();
    match re.is_match(tpe) {
        Ok(ok) => ok,
        _ => false,
    }
}

#[inline]
fn ty_str_strip(ty_str: &str) -> &str {
    // Empty and tab chars.
    let chars = " \t";
    ty_str.trim_matches(|c| chars.contains(c))
}

/// separate_kv split the union type and do not split '|' in dict and list
/// e.g., "int|str" -> vec!["int", "str"], "int | str" -> vec!["int", "str"]
pub fn split_type_union(tpe: &str) -> Vec<&str> {
    let mut i = 0;
    let mut s_index = 0;
    let mut stack = String::new();
    let mut types: Vec<&str> = vec![];
    while i < tpe.chars().count() {
        let (c_idx, c) = tpe.char_indices().nth(i).unwrap();
        if c == '|' && stack.is_empty() {
            types.push(&tpe[s_index..c_idx]);
            s_index = c_idx + 1;
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
            let t: String = tpe.chars().skip(i).collect();
            let re = fancy_regex::Regex::new(r#""(?!"").*?(?<!\\)(\\\\)*?""#).unwrap();
            if let Ok(Some(v)) = re.find(&t) {
                i += v.as_str().chars().count() - 1;
            }
        }
        // String literal type
        else if c == '\'' {
            let t: String = tpe.chars().skip(i).collect();
            let re = fancy_regex::Regex::new(r#"'(?!'').*?(?<!\\)(\\\\)*?'"#).unwrap();
            if let Ok(Some(v)) = re.find(&t) {
                i += v.as_str().chars().count() - 1;
            }
        }
        i += 1;
    }
    types.push(&tpe[s_index..]);
    // Remove empty and tab chars in the type string.
    types.iter().map(|ty| ty_str_strip(ty)).collect()
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
                panic!("invalid type string {expected_type}");
            }
            stack.pop();
        } else if c == '}' {
            if &stack[stack.len() - 1..] != "{" {
                panic!("invalid type string {expected_type}");
            }
            stack.pop();
        } else if c == ':' {
            if !stack.is_empty() {
                panic!("invalid type string {expected_type}");
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

#[cfg(test)]
mod test_value_type {

    use crate::*;

    #[test]
    fn test_check_type() {
        let cases = [
            // true cases
            (ValueRef::int(0), "int", true),
            (ValueRef::float(0.0), "float", true),
            (ValueRef::bool(true), "bool", true),
            (ValueRef::str("123"), "str", true),
            (ValueRef::list_int(&[1, 2, 3]), "[int]", true),
            (ValueRef::dict_str(&[("key", "value")]), "{str:}", true),
            // false cases
            (ValueRef::int(0), "str", false),
            (ValueRef::str("0"), "int", false),
        ];
        for (value, tpe, expected) in cases {
            assert_eq!(check_type(&value, tpe), expected);
        }
    }

    #[test]
    fn test_check_type_union() {
        let cases = [
            // true cases
            (ValueRef::int(0), "int|str", true),
            (ValueRef::str("0"), "int|str", true),
            (ValueRef::int(0), "int|[int|str]", true),
            (ValueRef::list_int(&[1, 2, 3]), "int|[int|str]", true),
            (
                ValueRef::list_str(&[1.to_string(), 2.to_string(), 3.to_string()]),
                "int|[int|str]",
                true,
            ),
            // false cases
            (ValueRef::bool(true), "int|str", false),
        ];
        for (value, tpe, expected) in cases {
            assert_eq!(check_type_union(&value, tpe), expected);
        }
    }

    #[test]
    fn test_check_type_literal() {
        let cases = [
            // true cases
            (ValueRef::int(0), "0", true),
            (ValueRef::float(0.0), "0.0", true),
            (ValueRef::str("123"), "\"123\"", true),
            (ValueRef::bool(true), "True", true),
            // false cases
            (ValueRef::str("123"), "\"234\"", false),
            (ValueRef::str("True"), "True", false),
        ];
        for (value, tpe, expected) in cases {
            assert_eq!(check_type_literal(&value, tpe), expected);
        }
    }

    #[test]
    fn test_check_number_multiplier_type() {
        let cases = [
            // true cases
            (ValueRef::unit(1024.0, 1, "Ki"), "1Ki", true),
            (
                ValueRef::unit(1024.0, 1, "Ki"),
                NUMBER_MULTIPLIER_TYPE,
                true,
            ),
            // false cases
            (ValueRef::unit(1024.0, 1, "Ki"), "1Mi", false),
        ];
        for (value, tpe, expected) in cases {
            assert_eq!(check_number_multiplier_type(&value, tpe), expected);
        }
    }

    #[test]
    fn test_check_type_dict() {
        let cases = [
            // true cases
            (ValueRef::dict_str(&[("key", "value")]), "{str:}", true),
            (ValueRef::dict_int(&[("key", 1)]), "{str:int}", true),
            // false cases
            (ValueRef::int(0), "int", false),
            (ValueRef::float(0.0), "float", false),
            (ValueRef::dict_str(&[("key", "value")]), "{str:int}", false),
            (ValueRef::dict_int(&[("key", 1)]), "{str:str}", false),
        ];
        for (value, tpe, expected) in cases {
            assert_eq!(check_type_dict(&value, tpe), expected);
        }
    }

    #[test]
    fn test_check_type_list() {
        let cases = [
            // true cases
            (ValueRef::list_int(&[1, 2, 3]), "[int]", true),
            (ValueRef::list_int(&[1, 2, 3]), "[]", true),
            (ValueRef::list_int(&[1, 2, 3]), "[any]", true),
            (ValueRef::list_int(&[1, 2, 3]), "[int|str]", true),
            (
                ValueRef::list_str(&[1.to_string(), 2.to_string(), 3.to_string()]),
                "[str]",
                true,
            ),
            (
                ValueRef::list_str(&[1.to_string(), 2.to_string(), 3.to_string()]),
                "[]",
                true,
            ),
            (
                ValueRef::list_str(&[1.to_string(), 2.to_string(), 3.to_string()]),
                "[any]",
                true,
            ),
            (
                ValueRef::list_str(&[1.to_string(), 2.to_string(), 3.to_string()]),
                "[int|str]",
                true,
            ),
            // false cases
            (ValueRef::int(0), "int", false),
            (ValueRef::float(0.0), "float", false),
            (ValueRef::list_int(&[1, 2, 3]), "[str]", false),
            (ValueRef::list_int(&[1, 2, 3]), "[bool]", false),
        ];
        for (value, tpe, expected) in cases {
            assert_eq!(check_type_list(&value, tpe), expected);
        }
    }

    #[test]
    fn test_match_builtin_type() {
        let cases = [
            // true cases
            (ValueRef::int(0), "int", true),
            (ValueRef::int(1), "int", true),
            (ValueRef::float(0.0), "float", true),
            (ValueRef::float(1.0), "float", true),
            (ValueRef::bool(true), "bool", true),
            (ValueRef::bool(false), "bool", true),
            (ValueRef::str("test"), "str", true),
            (ValueRef::str("''\"''"), "str", true),
            // false cases
            (ValueRef::float(0.0), "int", false),
            (ValueRef::bool(true), "float", false),
            (ValueRef::str("test"), "bool", false),
            (ValueRef::int(1), "str", false),
        ];
        for (value, tpe, expected) in cases {
            assert_eq!(match_builtin_type(&value, tpe), expected);
        }
    }

    #[test]
    fn test_is_literal_type() {
        let cases = [
            // true cases
            ("123", true),
            ("\"123\"", true),
            ("True", true),
            ("False", true),
            ("1.0", true),
            // false cases
            ("int", false),
            ("float", false),
            ("str", false),
            ("bool", false),
            ("", false),
            ("int8", false),
            ("string", false),
            ("any", false),
            ("[", false),
            ("]", false),
        ];
        for (value, expected) in cases {
            assert_eq!(is_literal_type(value), expected);
        }
    }

    #[test]
    fn test_is_list_type() {
        let cases = [
            // true cases
            ("[]", true),
            ("[int]", true),
            ("[str]", true),
            ("[[str]]", true),
            ("[str|int]", true),
            ("[pkg.Schema]", true),
            // false cases
            ("int", false),
            ("float", false),
            ("str", false),
            ("bool", false),
            ("", false),
            ("int8", false),
            ("string", false),
            ("[", false),
            ("]", false),
        ];
        for (value, expected) in cases {
            assert_eq!(is_list_type(value), expected);
        }
    }

    #[test]
    fn test_is_dict_type() {
        let cases = [
            // true cases
            ("{}", true),
            ("{:}", true),
            ("{str:}", true),
            ("{str:str}", true),
            ("{str: str}", true),
            ("{str:int}", true),
            ("{str: int}", true),
            ("{str:{str:}}", true),
            ("{str:{str:str}}", true),
            ("{str:str|int}", true),
            ("{str:pkg.Schema}", true),
            // false cases
            ("int", false),
            ("float", false),
            ("str", false),
            ("bool", false),
            ("", false),
            ("int8", false),
            ("string", false),
            ("{", false),
            ("}", false),
        ];
        for (value, expected) in cases {
            assert_eq!(is_dict_type(value), expected);
        }
    }

    #[test]
    fn test_is_type_union() {
        let cases = [
            ("A|B|C", true),
            ("'123'|'456'|'789'", true),
            ("'|'|'||'|'|||'", true),
            ("\"aa\\\"ab|\"|\"aa\\\"abccc\"", true),
            ("[\"|\"]|\"\"", true),
            ("{str:\"|\"}|\"|\"", true),
            ("\"aa\\\"ab|\"", false),
            ("\"|aa\\\"ab|\"", false),
        ];
        for (value, expected) in cases {
            assert_eq!(is_type_union(value), expected);
        }
    }

    #[test]
    fn test_split_type_union() {
        let cases = [
            ("", vec![""]),
            ("str|int", vec!["str", "int"]),
            ("str | int", vec!["str", "int"]),
            ("str|int|bool", vec!["str", "int", "bool"]),
            ("str | int | bool", vec!["str", "int", "bool"]),
            ("str|[str]", vec!["str", "[str]"]),
            ("str|{str:int}", vec!["str", "{str:int}"]),
            ("A|B|C", vec!["A", "B", "C"]),
            ("'123'|'456'|'789'", vec!["'123'", "'456'", "'789'"]),
            ("'|'|'||'|'|||'", vec!["'|'", "'||'", "'|||'"]),
            ("{str:\"|\"}|\"|\"", vec!["{str:\"|\"}", "\"|\""]),
        ];
        for (value, expected) in cases {
            assert_eq!(split_type_union(value), expected);
        }
    }

    #[test]
    fn test_separate_kv() {
        let cases = [
            ("", ("", "")),
            ("str:str", ("str", "str")),
            ("str:[str]", ("str", "[str]")),
            ("str:[str]", ("str", "[str]")),
            ("str:{str:int}", ("str", "{str:int}")),
        ];
        for (value, expected) in cases {
            let expected = (expected.0.to_string(), expected.1.to_string());
            assert_eq!(separate_kv(value), expected);
        }
    }

    #[test]
    fn test_dereference_type() {
        let cases = [
            ("", ""),
            ("[]", ""),
            ("{}", ""),
            ("[int]", "int"),
            ("{str:str}", "str:str"),
        ];
        for (value, expected) in cases {
            assert_eq!(dereference_type(value), expected);
        }
    }
}
