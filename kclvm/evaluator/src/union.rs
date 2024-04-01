//! Copyright The KCL Authors. All rights reserved.

use crate::*;
use kclvm_runtime::unification::value_subsume;
use kclvm_runtime::{ConfigEntryOperationKind, DictValue, UnionContext, UnionOptions, Value};

use self::ty::resolve_schema;

fn do_union(
    s: &Evaluator,
    p: &mut ValueRef,
    x: &ValueRef,
    opts: &UnionOptions,
    union_context: &mut UnionContext,
) -> ValueRef {
    if p.is_same_ref(x) {
        return p.clone();
    }

    let mut union_fn = |obj: &mut DictValue, delta: &DictValue| {
        // Update potential schema type
        obj.potential_schema = delta.potential_schema.clone();
        // Update attribute map
        for (k, v) in &delta.ops {
            obj.ops.insert(k.clone(), v.clone());
        }
        // Update index map
        for (k, v) in &delta.insert_indexs {
            obj.insert_indexs.insert(k.clone(), *v);
        }
        // Update values
        for (k, v) in &delta.values {
            let operation = if let Some(op) = delta.ops.get(k) {
                op
            } else {
                &ConfigEntryOperationKind::Union
            };
            let index = if let Some(idx) = delta.insert_indexs.get(k) {
                *idx
            } else {
                -1
            };
            if !obj.values.contains_key(k) {
                obj.values.insert(k.clone(), v.clone());
            } else {
                match operation {
                    ConfigEntryOperationKind::Union => {
                        let obj_value = obj.values.get_mut(k).unwrap();
                        if opts.idempotent_check && !value_subsume(v, obj_value, false) {
                            union_context.conflict = true;
                            union_context.path_backtrace.push(k.clone());
                            union_context.obj_json = if obj_value.is_config() {
                                "{...}".to_string()
                            } else if obj_value.is_list() {
                                "[...]".to_string()
                            } else {
                                obj_value.to_json_string()
                            };

                            union_context.delta_json = if v.is_config() {
                                "{...}".to_string()
                            } else if v.is_list() {
                                "[...]".to_string()
                            } else {
                                v.to_json_string()
                            };
                            return;
                        }
                        union(s, obj_value, v, false, opts, union_context);
                        if union_context.conflict {
                            union_context.path_backtrace.push(k.clone());
                            return;
                        }
                    }
                    ConfigEntryOperationKind::Override => {
                        if index < 0 {
                            obj.values.insert(k.clone(), v.clone());
                        } else {
                            let origin_value = obj.values.get_mut(k).unwrap();
                            if !origin_value.is_list() {
                                panic!("only list attribute can be inserted value");
                            }
                            if v.is_none_or_undefined() {
                                origin_value.list_remove_at(index as usize);
                            } else {
                                origin_value.list_set(index as usize, v);
                            }
                        }
                    }
                    ConfigEntryOperationKind::Insert => {
                        let origin_value = obj.values.get_mut(k).unwrap();
                        if origin_value.is_none_or_undefined() {
                            let list = ValueRef::list(None);
                            obj.values.insert(k.to_string(), list);
                        }
                        let origin_value = obj.values.get_mut(k).unwrap();
                        if origin_value.is_same_ref(v) {
                            continue;
                        }
                        match (&mut *origin_value.rc.borrow_mut(), &*v.rc.borrow()) {
                            (Value::list_value(origin_value), Value::list_value(value)) => {
                                if index == -1 {
                                    for elem in value.values.iter() {
                                        origin_value.values.push(elem.clone());
                                    }
                                } else if index >= 0 {
                                    let mut insert_index = index;
                                    for v in &value.values {
                                        origin_value
                                            .values
                                            .insert(insert_index as usize, v.clone());
                                        insert_index += 1;
                                    }
                                }
                            }
                            _ => panic!("only list attribute can be inserted value"),
                        };
                    }
                }
            }
        }
    };

    //union schema vars
    let mut union_schema = false;
    let mut pkgpath: String = "".to_string();
    let mut name: String = "".to_string();
    let mut common_keys: Vec<String> = vec![];
    let mut args = None;
    let mut kwargs = None;
    let mut valid = true;
    match (&mut *p.rc.borrow_mut(), &*x.rc.borrow()) {
        (Value::list_value(obj), Value::list_value(delta)) => {
            if !opts.list_override {
                let length = if obj.values.len() > delta.values.len() {
                    obj.values.len()
                } else {
                    delta.values.len()
                };
                let obj_len = obj.values.len();
                let delta_len = delta.values.len();
                for idx in 0..length {
                    if idx >= obj_len {
                        obj.values.push(delta.values[idx].clone());
                    } else if idx < delta_len {
                        union(
                            s,
                            &mut obj.values[idx],
                            &delta.values[idx],
                            false,
                            opts,
                            union_context,
                        );
                        if union_context.conflict {
                            union_context.path_backtrace.push(format!("list[{idx}]"));
                        }
                    }
                }
            }
        }
        (Value::dict_value(obj), Value::dict_value(delta)) => union_fn(obj, delta),
        (Value::schema_value(obj), Value::dict_value(delta)) => {
            name = obj.name.clone();
            pkgpath = obj.pkgpath.clone();
            let obj_value = obj.config.as_mut();
            union_fn(obj_value, delta);
            common_keys = obj.config_keys.clone();
            let mut other_keys: Vec<String> = delta.values.keys().cloned().collect();
            common_keys.append(&mut other_keys);
            args = Some(obj.args.clone());
            kwargs = Some(obj.kwargs.clone());
            union_schema = true;
        }
        (Value::schema_value(obj), Value::schema_value(delta)) => {
            name = obj.name.clone();
            pkgpath = obj.pkgpath.clone();
            let obj_value = obj.config.as_mut();
            let delta_value = delta.config.as_ref();
            union_fn(obj_value, delta_value);
            common_keys = obj.config_keys.clone();
            let mut other_keys: Vec<String> = delta.config_keys.clone();
            common_keys.append(&mut other_keys);
            args = Some(delta.args.clone());
            kwargs = Some(delta.kwargs.clone());
            union_schema = true;
        }
        (Value::dict_value(obj), Value::schema_value(delta)) => {
            name = delta.name.clone();
            pkgpath = delta.pkgpath.clone();
            let delta_value = delta.config.as_ref();
            union_fn(obj, delta_value);
            common_keys = delta.config_keys.clone();
            let mut other_keys: Vec<String> = obj.values.keys().cloned().collect();
            common_keys.append(&mut other_keys);
            args = Some(delta.args.clone());
            kwargs = Some(delta.kwargs.clone());
            union_schema = true;
        }
        _ => valid = false,
    }
    if !valid {
        panic!(
            "union failure, expect {:?}, got {:?}",
            p.type_str(),
            x.type_str()
        )
    }
    if union_context.conflict {
        return p.clone();
    }
    if union_schema {
        // Override schema arguments and keyword arguments.
        let mut result = p.clone();
        if let (Some(args), Some(kwargs)) = (&args, &kwargs) {
            result.set_schema_args(args, kwargs);
        }
        let optional_mapping = if p.is_schema() {
            p.schema_optional_mapping()
        } else {
            x.schema_optional_mapping()
        };
        let schema = result.dict_to_schema(
            name.as_str(),
            pkgpath.as_str(),
            &common_keys,
            &x.schema_config_meta(),
            &optional_mapping,
            args,
            kwargs,
        );
        if opts.config_resolve {
            *p = resolve_schema(s, &schema, &common_keys);
        } else {
            *p = schema;
        }
    }
    p.clone()
}

fn union(
    s: &Evaluator,
    p: &mut ValueRef,
    x: &ValueRef,
    or_mode: bool,
    opts: &UnionOptions,
    union_context: &mut UnionContext,
) -> ValueRef {
    if p.is_none_or_undefined() {
        *p = x.clone();
        return p.clone();
    }
    if x.is_none_or_undefined() {
        return p.clone();
    }
    if p.is_list_or_config() && x.is_list_or_config() {
        do_union(s, p, x, opts, union_context);
    } else if or_mode {
        if let (Value::int_value(a), Value::int_value(b)) =
            (&mut *p.rc.borrow_mut(), &*x.rc.borrow())
        {
            *a |= *b;
            return p.clone();
        };
        panic!(
            "unsupported operand type(s) for |: '{:?}' and '{:?}'",
            p.type_str(),
            x.type_str()
        )
    } else {
        *p = x.clone();
    }
    p.clone()
}

pub fn union_entry(
    s: &Evaluator,
    p: &mut ValueRef,
    x: &ValueRef,
    or_mode: bool,
    opts: &UnionOptions,
) -> ValueRef {
    let mut union_context = UnionContext::default();
    let ret = union(s, p, x, or_mode, opts, &mut union_context);
    if union_context.conflict {
        union_context.path_backtrace.reverse();
        let conflict_key = union_context.path_backtrace.last().unwrap();
        let path_string = union_context.path_backtrace.join(".");

        // build note
        // it will be like:
        // {...} | {
        //         ...
        //         b = {...}
        //         ...
        // }

        let note = format!(
            "    {{...}} | {{\n            ...\n            {} = {}\n            ...\n    }}",
            conflict_key, union_context.delta_json
        );
        if conflict_key.is_empty() {
            panic!(
                "conflicting values between {} and {}",
                union_context.delta_json, union_context.obj_json
            );
        } else {
            panic!(
                    "conflicting values on the attribute '{}' between :\n    {}\nand\n    {}\nwith union path :\n    {}\ntry operator '=' to override the attribute, like:\n{}",
                    conflict_key,
                    union_context.obj_json,
                    union_context.delta_json,
                    path_string,
                    note,
                );
        }
    }
    ret
}
