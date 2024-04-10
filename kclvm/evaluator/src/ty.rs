use kclvm_runtime::{
    check_type, dereference_type, is_dict_type, is_list_type, is_type_union, schema_config_meta,
    schema_runtime_type, separate_kv, split_type_union, ConfigEntryOperationKind, ValueRef,
    BUILTIN_TYPES, KCL_TYPE_ANY, PKG_PATH_PREFIX,
};

use crate::error as kcl_error;
use crate::schema::SchemaEvalContext;
use crate::{proxy::Proxy, Evaluator};

/// Use the schema instance to build a new schema instance using the schema construct function
pub fn resolve_schema(s: &Evaluator, schema: &ValueRef, keys: &[String]) -> ValueRef {
    if !schema.is_schema() {
        return schema.clone();
    }
    let schema_value = schema.as_schema();
    let schema_type_name = schema_runtime_type(&schema_value.name, &schema_value.pkgpath);
    if let Some(index) = s.schemas.borrow().get(&schema_type_name) {
        let keys = keys.iter().map(|v| v.as_str()).collect();
        let config_value = schema.dict_get_entries(keys);
        let config_meta = {
            let ctx = s.runtime_ctx.borrow();
            schema_config_meta(
                &ctx.panic_info.kcl_file,
                ctx.panic_info.kcl_line as u64,
                ctx.panic_info.kcl_col as u64,
            )
        };

        let frame = {
            let frames = s.frames.borrow();
            frames
                .get(*index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        let schema = if let Proxy::Schema(caller) = &frame.proxy {
            s.push_pkgpath(&frame.pkgpath);
            s.push_backtrace(&frame);
            let value = (caller.body)(
                s,
                &caller.ctx.borrow().snapshot(config_value, config_meta),
                &schema_value.args,
                &schema_value.kwargs,
            );
            s.pop_backtrace();
            s.pop_pkgpath();
            value
        } else {
            schema.clone()
        };
        // ctx.panic_info = now_panic_info;
        return schema;
    }
    // ctx.panic_info = now_panic_info;
    schema.clone()
}

/// Type pack and check ValueRef with the expected type vector
pub fn type_pack_and_check(s: &Evaluator, value: &ValueRef, expected_types: Vec<&str>) -> ValueRef {
    if value.is_none_or_undefined() || expected_types.is_empty() {
        return value.clone();
    }
    let is_schema = value.is_schema();
    let value_tpe = value.type_str();
    let mut checked = false;
    let mut converted_value = value.clone();
    let expected_type = &expected_types.join(" | ").replace('@', "");
    for tpe in expected_types {
        if !is_schema {
            converted_value = convert_collection_value(s, value, tpe);
        }
        // Runtime type check
        checked = check_type(&converted_value, tpe);
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
pub fn convert_collection_value(s: &Evaluator, value: &ValueRef, tpe: &str) -> ValueRef {
    if tpe.is_empty() || tpe == KCL_TYPE_ANY {
        return value.clone();
    }
    let is_collection = value.is_list() || value.is_dict();
    let invalid_match_dict = is_dict_type(tpe) && !value.is_dict();
    let invalid_match_list = is_list_type(tpe) && !value.is_list();
    let invalid_match = invalid_match_dict || invalid_match_list;
    if !is_collection || invalid_match {
        return value.clone();
    }
    // Convert a value to union types e.g., {a: 1} => A | B
    if is_type_union(tpe) {
        let types = split_type_union(tpe);
        convert_collection_value_with_union_types(s, value, &types)
    } else if is_dict_type(tpe) {
        //let (key_tpe, value_tpe) = separate_kv(tpe);
        let (_, value_tpe) = separate_kv(&dereference_type(tpe));
        let mut expected_dict = ValueRef::dict(None);
        let dict_ref = value.as_dict_ref();
        expected_dict
            .set_potential_schema_type(&dict_ref.potential_schema.clone().unwrap_or_default());
        for (k, v) in &dict_ref.values {
            let expected_value = convert_collection_value(s, v, &value_tpe);
            let op = dict_ref
                .ops
                .get(k)
                .unwrap_or(&ConfigEntryOperationKind::Union);
            let index = dict_ref.insert_indexs.get(k).unwrap_or(&-1);
            expected_dict.dict_update_entry(k, &expected_value, op, index)
        }
        expected_dict
    } else if is_list_type(tpe) {
        let expected_type = dereference_type(tpe);
        let mut expected_list = ValueRef::list(None);
        let list_ref = value.as_list_ref();
        for v in &list_ref.values {
            let expected_value = convert_collection_value(s, v, &expected_type);
            expected_list.list_append(&expected_value)
        }
        expected_list
    } else if BUILTIN_TYPES.contains(&tpe) {
        value.clone()
    } else {
        // Get the type form @pkg.Schema
        let schema_type_name = if tpe.contains('.') {
            if tpe.starts_with(PKG_PATH_PREFIX) {
                tpe.to_string()
            } else {
                format!("{PKG_PATH_PREFIX}{tpe}")
            }
        } else {
            format!("{}.{}", s.current_pkgpath(), tpe)
        };
        if let Some(index) = s.schemas.borrow().get(&schema_type_name) {
            let config_meta = {
                let ctx = s.runtime_ctx.borrow();
                schema_config_meta(
                    &ctx.panic_info.kcl_file,
                    ctx.panic_info.kcl_line as u64,
                    ctx.panic_info.kcl_col as u64,
                )
            };
            let frame = {
                let frames = s.frames.borrow();
                frames
                    .get(*index)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            let schema = if let Proxy::Schema(caller) = &frame.proxy {
                // Try convert the  config to schema, if failed, return the config
                if !SchemaEvalContext::is_fit_config(s, &caller.ctx, value) {
                    return value.clone();
                }
                s.push_pkgpath(&frame.pkgpath);
                s.push_backtrace(&frame);
                let value = (caller.body)(
                    s,
                    &caller.ctx.borrow().snapshot(value.clone(), config_meta),
                    &s.list_value(),
                    &s.dict_value(),
                );
                s.pop_backtrace();
                s.pop_pkgpath();
                value
            } else {
                value.clone()
            };
            // ctx.panic_info = now_panic_info;
            return schema.clone();
        }
        // ctx.panic_info = now_meta_info;
        value.clone()
    }
}

/// Convert collection value including dict/list to the potential schema and return errors.
pub fn convert_collection_value_with_union_types(
    s: &Evaluator,
    value: &ValueRef,
    types: &[&str],
) -> ValueRef {
    if value.is_schema() {
        value.clone()
    } else {
        for tpe in types {
            // Try match every type and convert the value, if matched, return the value.
            let value = convert_collection_value(s, value, tpe);
            if check_type(&value, tpe) {
                return value;
            }
        }
        value.clone()
    }
}
