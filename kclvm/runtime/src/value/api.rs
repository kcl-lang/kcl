// Copyright 2021 The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use std::mem::transmute_copy;

use crate::*;

#[allow(non_camel_case_types)]
pub type kclvm_buffer_t = Buffer;

#[allow(non_camel_case_types)]
pub type kclvm_context_t = Context;

#[allow(non_camel_case_types)]
pub type kclvm_decorator_value_t = DecoratorValue;

#[allow(non_camel_case_types)]
pub type kclvm_kind_t = Kind;

#[allow(non_camel_case_types)]
pub type kclvm_type_t = Type;

#[allow(non_camel_case_types)]
pub type kclvm_value_ref_t = ValueRef;

#[allow(non_camel_case_types)]
pub type kclvm_iterator_t = ValueIterator;

#[allow(non_camel_case_types)]
pub type kclvm_char_t = i8;

#[allow(non_camel_case_types)]
pub type kclvm_size_t = i32;

#[allow(non_camel_case_types)]
type kclvm_bool_t = i8;

#[allow(non_camel_case_types)]
pub type kclvm_int_t = i64;

#[allow(non_camel_case_types)]
pub type kclvm_float_t = f64;

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_import_names(
    p: *mut kclvm_context_t,
    import_names: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let import_names = ptr_as_ref(import_names);

    let import_names_dict = import_names.as_dict_ref();
    for (k, v) in &import_names_dict.values {
        let mut map = IndexMap::default();
        let v_dict = v.as_dict_ref();
        for (pkgname, pkgpath) in &v_dict.values {
            map.insert(pkgname.to_string(), pkgpath.as_str());
        }
        p.import_names.insert(k.to_string(), map);
    }
}

// ----------------------------------------------------------------------------
// values: new
// ----------------------------------------------------------------------------

// singleton

#[allow(non_camel_case_types, non_upper_case_globals)]
static mut kclvm_value_Undefined_obj: usize = 0;

#[allow(non_camel_case_types, non_upper_case_globals)]
static mut kclvm_value_None_obj: usize = 0;

#[allow(non_camel_case_types, non_upper_case_globals)]
static mut kclvm_value_Bool_true_obj: usize = 0;

#[allow(non_camel_case_types, non_upper_case_globals)]
static mut kclvm_value_Bool_false_obj: usize = 0;

#[allow(non_camel_case_types, non_upper_case_globals)]
static mut kclvm_value_Int_0_obj: usize = 0;

#[allow(non_camel_case_types, non_upper_case_globals)]
static mut kclvm_value_Float_0_obj: usize = 0;

// Undefine/None

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_Undefined() -> *mut kclvm_value_ref_t {
    new_mut_ptr(ValueRef::undefined())
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_None() -> *mut kclvm_value_ref_t {
    new_mut_ptr(ValueRef::none())
}

// bool/int/float/str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_True() -> *mut kclvm_value_ref_t {
    kclvm_value_Bool(1)
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_False() -> *mut kclvm_value_ref_t {
    kclvm_value_Bool(0)
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_Bool(v: kclvm_bool_t) -> *mut kclvm_value_ref_t {
    unsafe {
        if v != 0 {
            if kclvm_value_Bool_true_obj == 0 {
                kclvm_value_Bool_true_obj = new_mut_ptr(ValueRef::bool(true)) as usize;
            }
            kclvm_value_Bool_true_obj as *mut kclvm_value_ref_t
        } else {
            if kclvm_value_Bool_false_obj == 0 {
                kclvm_value_Bool_false_obj = new_mut_ptr(ValueRef::bool(false)) as usize;
            }
            kclvm_value_Bool_false_obj as *mut kclvm_value_ref_t
        }
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_Int(v: kclvm_int_t) -> *mut kclvm_value_ref_t {
    if v == 0 {
        unsafe {
            if kclvm_value_Int_0_obj == 0 {
                kclvm_value_Int_0_obj = new_mut_ptr(ValueRef::int(0)) as usize;
            }
            return kclvm_value_Int_0_obj as *mut kclvm_value_ref_t;
        }
    }
    new_mut_ptr(ValueRef::int(v))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_Float(v: kclvm_float_t) -> *mut kclvm_value_ref_t {
    if v == 0.0 {
        unsafe {
            if kclvm_value_Float_0_obj == 0 {
                kclvm_value_Float_0_obj = new_mut_ptr(ValueRef::float(0.0)) as usize;
            }
            return kclvm_value_Float_0_obj as *mut kclvm_value_ref_t;
        }
    }
    new_mut_ptr(ValueRef::float(v))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_Unit(
    v: kclvm_float_t,
    raw: kclvm_int_t,
    unit: *const kclvm_char_t,
) -> *mut kclvm_value_ref_t {
    let unit = c2str(unit);
    new_mut_ptr(ValueRef::unit(v, raw, unit))
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Str(v: *const kclvm_char_t) -> *mut kclvm_value_ref_t {
    unsafe {
        if v.is_null() || *v == '\0' as i8 {
            return new_mut_ptr(ValueRef::str(""));
        }
    }
    return new_mut_ptr(ValueRef::str(c2str(v)));
}

// list/dict/schema

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_List() -> *mut kclvm_value_ref_t {
    new_mut_ptr(ValueRef::list(None))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_List6(
    v1: *const kclvm_value_ref_t,
    v2: *const kclvm_value_ref_t,
    v3: *const kclvm_value_ref_t,
    v4: *const kclvm_value_ref_t,
    v5: *const kclvm_value_ref_t,
    v6: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let values: Vec<&ValueRef> = vec![v1, v2, v3, v4, v5, v6]
        .into_iter()
        .map(ptr_as_ref)
        .collect();
    new_mut_ptr(ValueRef::list(Some(values.as_slice())))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_List10(
    v1: *const kclvm_value_ref_t,
    v2: *const kclvm_value_ref_t,
    v3: *const kclvm_value_ref_t,
    v4: *const kclvm_value_ref_t,
    v5: *const kclvm_value_ref_t,
    v6: *const kclvm_value_ref_t,
    v7: *const kclvm_value_ref_t,
    v8: *const kclvm_value_ref_t,
    v9: *const kclvm_value_ref_t,
    v10: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let values: Vec<&ValueRef> = vec![v1, v2, v3, v4, v5, v6, v7, v8, v9, v10]
        .into_iter()
        .map(ptr_as_ref)
        .collect();
    new_mut_ptr(ValueRef::list(Some(values.as_slice())))
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_ListN(
    n: kclvm_int_t,
    elem_values: *const *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let mut list = ListValue::default();

    unsafe {
        for xi in std::slice::from_raw_parts(elem_values, n as usize).iter() {
            let v: &ValueRef = ptr_as_ref(*xi);
            list.values.push(v.clone());
        }

        ValueRef::from(Value::list_value(Box::new(list))).into_raw()
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_Dict() -> *mut kclvm_value_ref_t {
    new_mut_ptr(ValueRef::dict(None))
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_value_Schema() -> *mut kclvm_value_ref_t {
    new_mut_ptr(ValueRef::schema())
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_schema_with_config(
    schema_dict: *const kclvm_value_ref_t,
    config: *const kclvm_value_ref_t,
    config_meta: *const kclvm_value_ref_t,
    name: *const kclvm_char_t,
    pkgpath: *const kclvm_char_t,
    is_sub_schema: *const kclvm_value_ref_t,
    record_instance: *const kclvm_value_ref_t,
    instance_pkgpath: *const kclvm_value_ref_t,
    optional_mapping: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let schema_dict = ptr_as_ref(schema_dict);
    // Config dict
    let config = ptr_as_ref(config);
    let config_meta = ptr_as_ref(config_meta);
    let config_keys: Vec<String> = config
        .as_dict_ref()
        .values
        .keys()
        .into_iter()
        .cloned()
        .collect();
    // Schema meta
    let name = c2str(name);
    let pkgpath = c2str(pkgpath);
    let runtime_type = schema_runtime_type(name, pkgpath);
    let is_sub_schema = ptr_as_ref(is_sub_schema);
    let record_instance = ptr_as_ref(record_instance);
    let instance_pkgpath = ptr_as_ref(instance_pkgpath);
    let instance_pkgpath = instance_pkgpath.as_str();
    let optional_mapping = ptr_as_ref(optional_mapping);
    let schema =
        schema_dict.dict_to_schema(name, pkgpath, &config_keys, config_meta, optional_mapping);
    // Runtime context
    let ctx = Context::current_context();
    if record_instance.is_truthy()
        && (instance_pkgpath.is_empty() || instance_pkgpath == MAIN_PKG_PATH)
    {
        // Record schema instance in the context
        let mut instance_map = ctx.instances.borrow_mut();
        if !instance_map.contains_key(&runtime_type) {
            instance_map.insert(runtime_type.clone(), vec![]);
        }
        instance_map
            .get_mut(&runtime_type)
            .unwrap()
            .push(schema_dict.clone());
    }
    // Dict to schema
    if is_sub_schema.is_truthy() {
        schema.into_raw()
    } else {
        schema_dict.clone().into_raw()
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Function(
    fn_ptr: *const u64,
    closure: *const kclvm_value_ref_t,
    name: *const kclvm_char_t,
    is_external: kclvm_bool_t,
) -> *mut kclvm_value_ref_t {
    let closure = ptr_as_ref(closure);
    let name = c2str(name);
    new_mut_ptr(ValueRef::func(
        fn_ptr as u64,
        0,
        closure.clone(),
        name,
        "",
        is_external != 0,
    ))
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Function_using_ptr(
    fn_ptr: *const u64,
    name: *const kclvm_char_t,
) -> *mut kclvm_value_ref_t {
    let name = c2str(name);
    new_mut_ptr(ValueRef::func(
        fn_ptr as u64,
        0,
        ValueRef::none(),
        name,
        "",
        false,
    ))
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_schema_function(
    fn_ptr: *const u64,
    check_fn_ptr: *const u64,
    tpe: *const kclvm_char_t,
) -> *mut kclvm_value_ref_t {
    // Schema function closures
    let is_sub_schema = ValueRef::bool(false);
    let config_meta = ValueRef::dict(None);
    let config = ValueRef::dict(None);
    let schema = ValueRef::dict(None);
    let optional_mapping = ValueRef::dict(None);
    let cal_map = ValueRef::dict(None);
    let backtrack_level_map = ValueRef::dict(None);
    let backtrack_cache = ValueRef::dict(None);
    let record_instance = ValueRef::bool(false);
    let instance_pkgpath = ValueRef::str(MAIN_PKG_PATH);

    let mut schema_args = ValueRef::list(None);
    {
        let mut schema_args_ref = schema_args.as_list_mut_ref();
        schema_args_ref.values.push(is_sub_schema);
        schema_args_ref.values.push(config_meta);
        schema_args_ref.values.push(config);
        schema_args_ref.values.push(schema);
        schema_args_ref.values.push(optional_mapping);
        schema_args_ref.values.push(cal_map);
        schema_args_ref.values.push(backtrack_level_map);
        schema_args_ref.values.push(backtrack_cache);
        schema_args_ref.values.push(record_instance);
        schema_args_ref.values.push(instance_pkgpath);
    }
    let runtime_type = c2str(tpe);
    let schema_func = ValueRef::func(
        fn_ptr as u64,
        check_fn_ptr as u64,
        schema_args,
        "",
        runtime_type,
        false,
    );
    let ctx = Context::current_context_mut();
    let mut all_schemas = ctx.all_schemas.borrow_mut();
    all_schemas.insert(runtime_type.to_string(), schema_func.clone());
    new_mut_ptr(schema_func)
}

// ----------------------------------------------------------------------------
// values: json
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_from_json(s: *const kclvm_char_t) -> *mut kclvm_value_ref_t {
    if s.is_null() {
        return kclvm_value_Undefined();
    }
    match ValueRef::from_json(c2str(s)) {
        Ok(x) => x.into_raw(),
        _ => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_to_json(p: *const kclvm_value_ref_t) -> *mut kclvm_buffer_t {
    let p = ptr_as_ref(p);
    let x = p.to_json();
    let buf = Buffer::new_with_buf(&x);
    buf.into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_to_json_value(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if p.is_null() {
        return kclvm_value_Str(std::ptr::null());
    }

    let p = ptr_as_ref(p);
    let s = p.to_json_string();

    return new_mut_ptr(ValueRef::str(s.as_ref()));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_to_json_value_with_null(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if p.is_null() {
        return kclvm_value_Str(std::ptr::null());
    }

    let p = ptr_as_ref(p);
    let s = p.to_json_string_with_null();

    return new_mut_ptr(ValueRef::str(s.as_ref()));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_plan_to_json(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let s = p.plan_to_json_string();

    return new_mut_ptr(ValueRef::str(s.as_ref()));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_plan_to_yaml(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let s = p.plan_to_yaml_string();

    return new_mut_ptr(ValueRef::str(s.as_ref()));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_to_yaml_value(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if p.is_null() {
        return kclvm_value_Str(std::ptr::null());
    }

    let p = ptr_as_ref(p);
    let s = p.to_yaml_string();

    return new_mut_ptr(ValueRef::str(s.as_ref()));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_to_str_value(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if p.is_null() {
        return kclvm_value_Str(std::ptr::null());
    }

    let p = ptr_as_ref(p);
    let s = p.to_string();

    return new_mut_ptr(ValueRef::str(s.as_ref()));
}

// ----------------------------------------------------------------------------
// values: value pointer
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Bool_ptr(p: *const kclvm_value_ref_t) -> *const kclvm_bool_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::bool_value(ref v) => v as *const bool as *const kclvm_bool_t, // sizeof(bool) == sizeof(i8)
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Int_ptr(p: *const kclvm_value_ref_t) -> *const kclvm_int_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::int_value(ref v) => v as *const kclvm_int_t,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Float_ptr(
    p: *const kclvm_value_ref_t,
) -> *const kclvm_float_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::float_value(ref v) => v as *const kclvm_float_t,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Str_ptr(p: *const kclvm_value_ref_t) -> *const kclvm_char_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::str_value(ref v) => v.as_ptr() as *const i8,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Str_len(p: *const kclvm_value_ref_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);
    p.str_len() as kclvm_size_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Str_resize(p: *mut kclvm_value_ref_t, n: kclvm_size_t) {
    let p = mut_ptr_as_ref(p);
    p.str_resize(n as usize)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_function_ptr(p: *const kclvm_value_ref_t) -> *const u64 {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::func_value(ref v) => v.fn_ptr as *const u64,
        _ => std::ptr::null::<u64>(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_check_function_ptr(p: *const kclvm_value_ref_t) -> *const u64 {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::func_value(ref v) => v.check_fn_ptr as *const u64,
        _ => std::ptr::null::<u64>(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_function_is_external(
    p: *const kclvm_value_ref_t,
) -> kclvm_bool_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::func_value(ref v) => v.is_external as kclvm_bool_t,
        _ => false as kclvm_bool_t,
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_function_external_invoke(
    p: *const kclvm_value_ref_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::func_value(ref v) => {
            let name = format!("{}\0", v.name);
            kclvm_plugin_invoke(name.as_ptr() as *const i8, args, kwargs)
        }
        _ => kclvm_value_None(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_function_invoke(
    p: *const kclvm_value_ref_t,
    ctx: *mut kclvm_context_t,
    args: *mut kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
    pkgpath: *const kclvm_char_t,
    is_in_schema: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let func = ptr_as_ref(p);
    let args_ref = mut_ptr_as_ref(args);
    if func.is_func() {
        let func = func.as_function();
        let fn_ptr = func.fn_ptr;
        let closure = &func.closure;
        let is_schema = !func.runtime_type.is_empty();
        let ctx_ref = mut_ptr_as_ref(ctx);
        if ctx_ref.cfg.debug_mode {
            ctx_ref
                .backtrace
                .push(BacktraceFrame::from_panic_info(&ctx_ref.panic_info));
            ctx_ref.panic_info.kcl_func = func.name.clone();
        }
        let now_meta_info = ctx_ref.panic_info.clone();
        unsafe {
            let call_fn: SchemaTypeFunc = transmute_copy(&fn_ptr);
            // Call schema constructor twice
            let value = if is_schema {
                let pkgpath = c2str(pkgpath);
                // Schema function closure
                let mut args_new = args_ref.deep_copy();
                let mut closure_new = closure.deep_copy();
                let config_meta_index: isize = 1;
                let cal_map_index: isize = 5;
                let record_instance_index = closure.len() - 2;
                let instance_pkgpath_index = closure.len() - 1;
                args_ref.list_append_unpack(closure);
                let args = args_ref.clone().into_raw();
                call_fn(ctx, args, kwargs);
                let cal_map = closure.list_get(cal_map_index).unwrap();
                // is sub schema
                closure_new.list_set(0, &ValueRef::bool(true));
                // record instance
                closure_new.list_set(record_instance_index, &ValueRef::bool(true));
                // instance pkgpath
                closure_new.list_set(instance_pkgpath_index, &ValueRef::str(pkgpath));
                // cal map
                closure_new.list_set(cal_map_index as usize, &cal_map);
                // config meta
                let config_meta = schema_config_meta(
                    &ctx_ref.panic_info.kcl_file,
                    ctx_ref.panic_info.kcl_line as u64,
                    ctx_ref.panic_info.kcl_col as u64,
                );
                closure_new.list_set(config_meta_index as usize, &config_meta);
                args_new.list_append_unpack(&closure_new);
                call_fn(ctx, args_new.into_raw(), kwargs)
            // Normal kcl function, call directly
            } else if func.is_external {
                let name = format!("{}\0", func.name);
                kclvm_plugin_invoke(name.as_ptr() as *const i8, args, kwargs)
            } else {
                args_ref.list_append_unpack_first(closure);
                let args = args_ref.clone().into_raw();
                call_fn(ctx, args, kwargs)
            };
            let is_in_schema = ptr_as_ref(is_in_schema);
            if is_schema && !is_in_schema.is_truthy() {
                let schema_value = ptr_as_ref(value);
                schema_value.schema_check_attr_optional(true);
            }
            if ctx_ref.cfg.debug_mode {
                ctx_ref.backtrace.pop();
            }
            ctx_ref.panic_info = now_meta_info;
            return value;
        };
    }
    kclvm_value_None()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_function_get_closure(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::func_value(ref v) => v.closure.deep_copy().into_raw(),
        Value::none | Value::undefined => kclvm_value_None(),
        _ => panic!("invalid value of function self value function"),
    }
}

// ----------------------------------------------------------------------------
// values: method
// ----------------------------------------------------------------------------

// kind

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_kind(p: *const kclvm_value_ref_t) -> kclvm_kind_t {
    let p = ptr_as_ref(p);
    p.kind()
}

// clone

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_deep_copy(
    p: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    p.deep_copy().into_raw()
}

// delete

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_delete(p: *mut kclvm_value_ref_t) {
    if p.is_null() {
        return;
    }
    unsafe {
        if p as usize == kclvm_value_Undefined_obj {
            return;
        }
        if p as usize == kclvm_value_None_obj {
            return;
        }
        if p as usize == kclvm_value_Bool_true_obj {
            return;
        }
        if p as usize == kclvm_value_Bool_false_obj {
            return;
        }
        if p as usize == kclvm_value_Int_0_obj {
            return;
        }
        if p as usize == kclvm_value_Float_0_obj {
            return;
        }
    }
    let val = ptr_as_ref(p);
    val.from_raw();
    free_mut_ptr(p);
}

// ----------------------------------------------------------------------------
// values: iterator
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_iter(p: *const kclvm_value_ref_t) -> *mut kclvm_iterator_t {
    let p = ptr_as_ref(p);
    let iter = ValueIterator::from_value(p);
    Box::into_raw(Box::new(iter))
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_iterator_delete(p: *mut kclvm_iterator_t) {
    free_mut_ptr(p);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_iterator_is_end(p: *mut kclvm_iterator_t) -> kclvm_bool_t {
    let p = ptr_as_ref(p);
    p.is_end() as kclvm_bool_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_iterator_cur_key(
    p: *mut kclvm_iterator_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    match p.key() {
        Some(x) => x,
        None => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_iterator_cur_value(
    p: *mut kclvm_iterator_t,
) -> *const kclvm_value_ref_t {
    let p = mut_ptr_as_ref(p);
    match p.value() {
        Some(x) => x,
        None => std::ptr::null(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_iterator_next_value(
    p: *mut kclvm_iterator_t,
    host: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let p = mut_ptr_as_ref(p);
    let host = ptr_as_ref(host);

    match p.next(host) {
        Some(x) => x,
        None => std::ptr::null(),
    }
}

// ----------------------------------------------------------------------------
// values: list
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_len(p: *const kclvm_value_ref_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);
    p.len() as kclvm_size_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_resize(p: *mut kclvm_value_ref_t, newsize: kclvm_size_t) {
    let p = mut_ptr_as_ref(p);
    p.list_resize(newsize as usize);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_clear(p: *mut kclvm_value_ref_t) {
    let p = mut_ptr_as_ref(p);
    p.list_clear();
}

/// Return number of occurrences of the list value.
#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_count(
    p: *const kclvm_value_ref_t,
    item: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let item = ptr_as_ref(item);
    let count = p.list_count(item);
    let count_value = ValueRef::int(count as i64);
    count_value.into_raw()
}

/// Return first index of the list value. Panic if the value is not present.
#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_find(
    p: *const kclvm_value_ref_t,
    item: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let item = ptr_as_ref(item);
    let index = p.list_find(item);
    let index_value = ValueRef::int(index as i64);
    index_value.into_raw()
}

/// Insert object before index of the list value.
#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_insert(
    p: *mut kclvm_value_ref_t,
    index: *const kclvm_value_ref_t,
    value: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let index = ptr_as_ref(index);
    let value = ptr_as_ref(value);
    p.list_insert_at(index.as_int() as usize, value);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_get(
    p: *const kclvm_value_ref_t,
    i: kclvm_size_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    match p.list_get(i as isize) {
        Some(x) => x.into_raw(),
        _ => panic!("list index out of range"),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_get_option(
    p: *const kclvm_value_ref_t,
    i: kclvm_size_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    match p.list_get_option(i as isize) {
        Some(x) => x.into_raw(),
        _ => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_set(
    p: *mut kclvm_value_ref_t,
    i: kclvm_size_t,
    v: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    p.list_set(i as usize, v);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_pop(p: *mut kclvm_value_ref_t) -> *const kclvm_value_ref_t {
    let p = mut_ptr_as_ref(p);
    match p.list_pop() {
        Some(x) => x.into_raw(),
        _ => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_pop_first(
    p: *mut kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let p = mut_ptr_as_ref(p);
    match p.list_pop_first() {
        Some(x) => x.into_raw(),
        _ => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_append(p: *mut kclvm_value_ref_t, v: *const kclvm_value_ref_t) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    p.list_append(v);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_append_bool(p: *mut kclvm_value_ref_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.list_append(&ValueRef::bool(v != 0));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_append_int(p: *mut kclvm_value_ref_t, v: kclvm_int_t) {
    let p = mut_ptr_as_ref(p);
    p.list_append(&ValueRef::int(v));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_append_float(p: *mut kclvm_value_ref_t, v: kclvm_float_t) {
    let p = mut_ptr_as_ref(p);
    p.list_append(&ValueRef::float(v));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_append_str(p: *mut kclvm_value_ref_t, v: *const kclvm_char_t) {
    let p = mut_ptr_as_ref(p);
    p.list_append(&ValueRef::str(c2str(v)));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_append_unpack(
    p: *mut kclvm_value_ref_t,
    v: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);

    if p.is_list() {
        p.list_append_unpack(v);
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_list_remove_at(p: *mut kclvm_value_ref_t, i: kclvm_size_t) {
    let p = mut_ptr_as_ref(p);
    p.list_remove_at(i as usize);
}

// ----------------------------------------------------------------------------
// values: dict
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_len(p: *const kclvm_value_ref_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);
    match &*p.rc.borrow() {
        Value::dict_value(ref dict) => dict.values.len() as kclvm_size_t,
        _ => 0,
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_clear(p: *mut kclvm_value_ref_t) {
    let p = mut_ptr_as_ref(p);
    p.dict_clear();
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_get(
    p: *const kclvm_value_ref_t,
    key: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let key = ptr_as_ref(key);

    match p.dict_get(key) {
        Some(x) => x.into_raw(),
        None => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_has_value(
    p: *const kclvm_value_ref_t,
    key: *const kclvm_char_t,
) -> kclvm_bool_t {
    let p = ptr_as_ref(p);
    let key = c2str(key);
    match p.dict_get_value(key) {
        Some(_) => true as kclvm_bool_t,
        None => false as kclvm_bool_t,
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_get_value(
    p: *const kclvm_value_ref_t,
    key: *const kclvm_char_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let key = c2str(key);
    match p.dict_get_value(key) {
        Some(x) => x.into_raw(),
        None => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_get_entry(
    p: *const kclvm_value_ref_t,
    key: *const kclvm_char_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let key = c2str(key);
    match p.dict_get_entry(key) {
        Some(x) => x.into_raw(),
        None => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_get_value_by_path(
    p: *const kclvm_value_ref_t,
    path: *const kclvm_char_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let path = c2str(path);
    match p.get_by_path(path) {
        Some(x) => x.into_raw(),
        None => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_set_value(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_char_t,
    val: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let key = c2str(key);
    let val = ptr_as_ref(val);
    if p.is_config() {
        p.dict_update_key_value(key, val.clone());
    }
    if p.is_schema() {
        let schema: ValueRef;
        {
            let schema_value = p.as_schema();
            let mut config_keys = schema_value.config_keys.clone();
            config_keys.push(key.to_string());
            schema = resolve_schema(p, &config_keys);
        }
        p.schema_update_with_schema(&schema);
    }
    /*panic*/
}

#[no_mangle]
#[runtime_fn]
/// Return all dict keys.
pub unsafe extern "C" fn kclvm_dict_keys(p: *const kclvm_value_ref_t) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let r = p.dict_keys();
    r.into_raw()
}

#[no_mangle]
#[runtime_fn]
/// Return all dict values.
pub unsafe extern "C" fn kclvm_dict_values(
    p: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(p);
    let r = p.dict_values();
    r.into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_insert(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_char_t,
    v: *const kclvm_value_ref_t,
    op: kclvm_size_t,
    insert_index: kclvm_size_t,
) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    p.dict_insert(
        c2str(key),
        v,
        ConfigEntryOperationKind::from_i32(op),
        insert_index,
    );
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_merge(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_char_t,
    v: *const kclvm_value_ref_t,
    op: kclvm_size_t,
    insert_index: kclvm_size_t,
) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    let key = c2str(key);
    let attr_map = {
        match &*p.rc.borrow() {
            Value::dict_value(dict) => dict.attr_map.clone(),
            Value::schema_value(schema) => schema.config.attr_map.clone(),
            _ => panic!("invalid object '{}' in attr_map", p.type_str()),
        }
    };
    if attr_map.contains_key(key) {
        let v = type_pack_and_check(v, vec![attr_map.get(key).unwrap()]);
        p.dict_merge(
            key,
            &v,
            ConfigEntryOperationKind::from_i32(op),
            insert_index,
        );
    } else {
        p.dict_merge(key, v, ConfigEntryOperationKind::from_i32(op), insert_index);
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_insert_value(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_value_ref_t,
    v: *const kclvm_value_ref_t,
    op: kclvm_size_t,
    insert_index: kclvm_size_t,
) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    let key = ptr_as_ref(key);
    let key = key.attr_str();
    p.dict_insert(
        key.as_str(),
        v,
        ConfigEntryOperationKind::from_i32(op),
        insert_index,
    );
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_update_key_value(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_value_ref_t,
    v: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    let key = ptr_as_ref(key);
    let key = key.attr_str();
    p.dict_update_key_value(key.as_str(), v.clone());
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_safe_insert(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_char_t,
    v: *const kclvm_value_ref_t,
    op: kclvm_size_t,
    insert_index: kclvm_size_t,
) {
    if p.is_null() || key.is_null() || v.is_null() {
        return;
    }
    kclvm_dict_insert(p, key, v, op, insert_index);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_insert_unpack(
    p: *mut kclvm_value_ref_t,
    v: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    p.dict_insert_unpack(v);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_default_collection_insert_int_pointer(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_char_t,
    ptr: *const u64,
) {
    let p = mut_ptr_as_ref(p);
    let key = c2str(key);
    let ptr = ptr as i64;
    if p.is_dict() {
        let mut dict_ref_mut = p.as_dict_mut_ref();
        if !dict_ref_mut.values.contains_key(key) {
            let value = ValueRef::list(None);
            dict_ref_mut.values.insert(key.to_string(), value);
        }
        let values = dict_ref_mut.values.get_mut(key).unwrap();
        let value = ValueRef::int(ptr);
        if !value.r#in(values) {
            values.list_append(&value);
        }
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_default_collection_insert_value(
    p: *mut kclvm_value_ref_t,
    key: *const kclvm_char_t,
    value: *const kclvm_value_ref_t,
) {
    let p = mut_ptr_as_ref(p);
    let key = c2str(key);
    let value = ptr_as_ref(value);
    if p.is_dict() {
        let mut dict_ref_mut = p.as_dict_mut_ref();
        if !dict_ref_mut.values.contains_key(key) {
            let value = ValueRef::list(None);
            dict_ref_mut.values.insert(key.to_string(), value);
        }
        let values = dict_ref_mut.values.get_mut(key).unwrap();
        if !value.r#in(values) {
            values.list_append(value);
        }
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_remove(p: *mut kclvm_value_ref_t, key: *const kclvm_char_t) {
    let p = mut_ptr_as_ref(p);
    p.dict_remove(c2str(key));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_dict_update(p: *mut kclvm_value_ref_t, v: *const kclvm_value_ref_t) {
    let p = mut_ptr_as_ref(p);
    let v = ptr_as_ref(v);
    p.dict_update(v);
}

// ----------------------------------------------------------------------------
// values: arith
// ----------------------------------------------------------------------------

// is true

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_is_truthy(p: *const kclvm_value_ref_t) -> kclvm_bool_t {
    let p = ptr_as_ref(p);
    p.is_truthy() as kclvm_bool_t
}

// len

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_len(p: *const kclvm_value_ref_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);
    p.len() as kclvm_size_t
}

// compare

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_cmp_equal_to(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if a == b {
        return kclvm_value_Bool(1);
    }
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.cmp_equal(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_cmp_not_equal_to(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if a == b {
        return kclvm_value_Bool(0);
    }
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(!a.cmp_equal(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_cmp_less_than(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if a == b {
        return kclvm_value_Bool(0);
    }
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.cmp_less_than(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_cmp_less_than_or_equal(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.cmp_less_than_or_equal(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_cmp_greater_than(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    if a == b {
        return kclvm_value_Bool(0);
    }
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.cmp_greater_than(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_cmp_greater_than_or_equal(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.cmp_greater_than_or_equal(b)).into_raw()
}

// is/in

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_is(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    if a == b {
        return kclvm_value_Bool(1);
    }
    kclvm_value_Bool(0)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_is_not(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    if a == b {
        return kclvm_value_Bool(0);
    }
    kclvm_value_Bool(1)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_in(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.r#in(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_not_in(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.not_in(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_as(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    let ty_str = b.as_str();
    let value = type_pack_and_check(a, vec![ty_str.as_str()]);
    value.into_raw()
}

// unary-xxx

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_unary_plus(
    a: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    a.unary_plus().into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_unary_minus(
    a: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    a.unary_minus().into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_unary_not(
    a: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    a.unary_not().into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_unary_l_not(
    a: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    a.unary_l_not().into_raw()
}

// op-xxx

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_add(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_add(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_sub(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_sub(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_mul(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_mul(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_div(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_div(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_mod(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_mod(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_pow(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_pow(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_floor_div(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_floor_div(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_bit_lshift(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_bit_lshift(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_bit_rshift(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_bit_rshift(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_bit_and(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_bit_and(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_bit_xor(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_bit_xor(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_bit_or(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_bit_or(b).into_raw()
}

// op-aug-xxx

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_add(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    return a.bin_aug_add(b) as *const kclvm_value_ref_t;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_sub(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    return a.bin_aug_sub(b) as *const kclvm_value_ref_t;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_mul(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    return a.bin_aug_mul(b) as *const kclvm_value_ref_t;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_div(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    return a.bin_aug_div(b) as *const kclvm_value_ref_t;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_mod(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    return a.bin_aug_mod(b) as *const kclvm_value_ref_t;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_pow(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    return a.bin_aug_pow(b) as *const kclvm_value_ref_t;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_floor_div(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    return a.bin_aug_floor_div(b) as *const kclvm_value_ref_t;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_bit_lshift(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_aug_bit_lshift(b) as *const kclvm_value_ref_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_bit_rshift(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_aug_bit_rshift(b) as *const kclvm_value_ref_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_bit_and(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_aug_bit_and(b) as *const kclvm_value_ref_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_bit_xor(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_aug_bit_xor(b) as *const kclvm_value_ref_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_op_aug_bit_or(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_aug_bit_or(b) as *const kclvm_value_ref_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_union(
    schema: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let a = mut_ptr_as_ref(schema);
    let b = ptr_as_ref(b);
    let attr_map = match &*a.rc.borrow() {
        Value::dict_value(dict) => dict.attr_map.clone(),
        Value::schema_value(schema) => schema.config.attr_map.clone(),
        _ => panic!("invalid object '{}' in attr_map", a.type_str()),
    };
    if b.is_config() {
        let dict = b.as_dict_ref();
        let mut result = schema;
        for (k, v) in &dict.values {
            if attr_map.contains_key(k) {
                let v = type_pack_and_check(v, vec![attr_map.get(k).unwrap()]);
                let mut entry = b.dict_get_entry(k).unwrap().deep_copy();
                entry.dict_update_key_value(k, v);
                result = a
                    .union_entry(&entry, true, false, false, false)
                    .clone()
                    .into_raw();
            } else {
                let entry = b.dict_get_entry(k).unwrap();
                result = a
                    .union_entry(&entry, true, false, false, false)
                    .clone()
                    .into_raw();
            }
        }
        result
    } else {
        a.union_entry(b, true, false, false, false).into_raw()
    }
}

// logic: && ||

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_logic_and(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.logic_and(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_logic_or(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    ValueRef::bool(a.logic_or(b)).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_subscr(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_subscr(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_subscr_option(
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    a.bin_subscr_option(b).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_load_attr(
    obj: *const kclvm_value_ref_t,
    key: *const kclvm_char_t,
) -> *const kclvm_value_ref_t {
    let p = ptr_as_ref(obj);
    let key = c2str(key);
    // load_attr including str/dict/schema.
    if p.is_dict() {
        match p.dict_get_value(key) {
            Some(x) => {
                return x.into_raw();
            }
            None => {
                return kclvm_value_Undefined();
            }
        }
    } else if p.is_schema() {
        let dict = p.schema_to_dict();
        match dict.dict_get_value(key) {
            Some(x) => {
                return x.into_raw();
            }
            None => panic!("schema '{}' attribute '{}' not found", p.type_str(), key),
        }
    } else if p.is_str() {
        let function = match key {
            "lower" => kclvm_builtin_str_lower,
            "upper" => kclvm_builtin_str_upper,
            "capitalize" => kclvm_builtin_str_capitalize,
            "count" => kclvm_builtin_str_count,
            "endswith" => kclvm_builtin_str_endswith,
            "find" => kclvm_builtin_str_find,
            "format" => kclvm_builtin_str_format,
            "index" => kclvm_builtin_str_index,
            "isalnum" => kclvm_builtin_str_isalnum,
            "isalpha" => kclvm_builtin_str_isalpha,
            "isdigit" => kclvm_builtin_str_isdigit,
            "islower" => kclvm_builtin_str_islower,
            "isspace" => kclvm_builtin_str_isspace,
            "istitle" => kclvm_builtin_str_istitle,
            "isupper" => kclvm_builtin_str_isupper,
            "join" => kclvm_builtin_str_join,
            "lstrip" => kclvm_builtin_str_lstrip,
            "rstrip" => kclvm_builtin_str_rstrip,
            "replace" => kclvm_builtin_str_replace,
            "removeprefix" => kclvm_builtin_str_removeprefix,
            "removesuffix" => kclvm_builtin_str_removesuffix,
            "rfind" => kclvm_builtin_str_rfind,
            "rindex" => kclvm_builtin_str_rindex,
            "rsplit" => kclvm_builtin_str_rsplit,
            "split" => kclvm_builtin_str_split,
            "splitlines" => kclvm_builtin_str_splitlines,
            "startswith" => kclvm_builtin_str_startswith,
            "strip" => kclvm_builtin_str_strip,
            "title" => kclvm_builtin_str_title,
            _ => panic!("str object attr '{key}' not found"),
        };
        let closure = ValueRef::list(Some(&[p]));
        return new_mut_ptr(ValueRef::func(
            function as usize as u64,
            0,
            closure,
            "",
            "",
            false,
        ));
    }
    // schema instance
    else if p.is_func() {
        let function = match key {
            "instances" => kclvm_schema_instances,
            _ => panic!("schema object attr '{key}' not found"),
        };
        let closure = ValueRef::list(Some(&[p]));
        return new_mut_ptr(ValueRef::func(
            function as usize as u64,
            0,
            closure,
            "",
            "",
            false,
        ));
    }
    panic!(
        "invalid value '{}' to load attribute '{}'",
        p.type_str(),
        key
    );
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_load_attr_option(
    p: *const kclvm_value_ref_t,
    key: *const kclvm_char_t,
) -> *const kclvm_value_ref_t {
    let p_ref = ptr_as_ref(p);
    if p_ref.is_truthy() {
        kclvm_value_load_attr(p, key)
    } else {
        kclvm_value_None()
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_remove_item(
    a: *mut kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
) {
    let a = mut_ptr_as_ref(a);
    let b = ptr_as_ref(b);
    if a.is_dict() {
        a.dict_remove(&b.as_str());
    } else if a.is_list() {
        a.list_remove(b);
    } else {
        panic!("only list, dict and schema can be removed item");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_slice(
    x: *const kclvm_value_ref_t,
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
    step: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let x = ptr_as_ref(x);
    let a = ptr_as_ref(a);
    let b = ptr_as_ref(b);
    let step = ptr_as_ref(step);
    x.list_slice(a, b, step).into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_slice_option(
    x: *const kclvm_value_ref_t,
    a: *const kclvm_value_ref_t,
    b: *const kclvm_value_ref_t,
    step: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let value = ptr_as_ref(x);
    if value.is_truthy() {
        kclvm_value_slice(x, a, b, step)
    } else {
        kclvm_value_None()
    }
}

// ----------------------------------------------------------------------------
// values: schema
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_backtrack_cache(
    schema: *const kclvm_value_ref_t,
    cache: *mut kclvm_value_ref_t,
    cal_map: *const kclvm_value_ref_t,
    name: *const kclvm_char_t,
    runtime_type: *const kclvm_value_ref_t,
) {
    let schema = ptr_as_ref(schema);
    let cache = mut_ptr_as_ref(cache);
    let cal_map = ptr_as_ref(cal_map);
    let name = c2str(name);
    if let Some(v) = cal_map.dict_get_value(name) {
        if v.len() == 1 {
            if let Some(value) = schema.dict_get_value(name) {
                cache.dict_update_key_value(name, value);
            }
        } else if let (Some(cal_map_runtime_type_list), Some(cal_map_meta_line_list)) = (
            cal_map.dict_get_value(&format!("{name}_{CAL_MAP_RUNTIME_TYPE}")),
            cal_map.dict_get_value(&format!("{name}_{CAL_MAP_META_LINE}")),
        ) {
            if let (Some(cal_map_runtime_type), Some(cal_map_meta_line)) = (
                cal_map_runtime_type_list.list_get(-1),
                cal_map_meta_line_list.list_get(-1),
            ) {
                let runtime_type = ptr_as_ref(runtime_type);
                let line = Context::current_context().panic_info.kcl_line as i64;
                let cal_map_meta_line = cal_map_meta_line.as_int();
                if runtime_type == &cal_map_runtime_type && line >= cal_map_meta_line {
                    if let Some(value) = schema.dict_get_value(name) {
                        cache.dict_update_key_value(name, value);
                    }
                }
            }
        }
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_instances(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(val) = args.pop_arg_first() {
        let function = val.as_function();
        let main_pkg = args.arg_0().or_else(|| kwargs.kwarg("main_pkg"));
        let main_pkg = if let Some(v) = main_pkg {
            v.is_truthy()
        } else {
            true
        };
        let runtime_type = &function.runtime_type;
        let instance_map = ctx.instances.borrow_mut();
        if instance_map.contains_key(runtime_type) {
            let mut list = ValueRef::list(None);
            for v in instance_map.get(runtime_type).unwrap() {
                if v.is_schema() {
                    let schema = v.as_schema();
                    if main_pkg {
                        if schema.pkgpath == MAIN_PKG_PATH {
                            list.list_append(v)
                        }
                    } else {
                        list.list_append(v)
                    }
                } else if v.is_dict() {
                    let runtime_type_attr_path =
                        format!("{SCHEMA_SETTINGS_ATTR_NAME}.{SETTINGS_SCHEMA_TYPE_KEY}");
                    let runtime_type =
                        if let Some(runtime_type) = v.get_by_path(&runtime_type_attr_path) {
                            runtime_type.as_str()
                        } else {
                            runtime_type.to_string()
                        };
                    let names: Vec<&str> = runtime_type.rsplit('.').collect();
                    let name = names[0];
                    let pkgpath = names[1];
                    let v = v.dict_to_schema(
                        name,
                        pkgpath,
                        &[],
                        &ValueRef::dict(None),
                        &ValueRef::dict(None),
                    );
                    list.list_append(&v);
                }
            }
            list.into_raw()
        } else {
            kclvm_value_List()
        }
    } else {
        kclvm_value_None()
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_value_check(
    schema_value: *mut kclvm_value_ref_t,
    schema_config: *const kclvm_value_ref_t,
    _config_meta: *const kclvm_value_ref_t,
    schema_name: *const kclvm_char_t,
    index_sign_value: *const kclvm_value_ref_t,
    _key_name: *const kclvm_char_t,
    key_type: *const kclvm_char_t,
    _value_type: *const kclvm_char_t,
    _any_other: kclvm_bool_t,
    is_relaxed: kclvm_bool_t,
) {
    let schema_value = mut_ptr_as_ref(schema_value);
    let schema_config = ptr_as_ref(schema_config);
    let index_sign_value = ptr_as_ref(index_sign_value);
    let key_type = c2str(key_type);
    let has_index_signature = !key_type.is_empty();
    let should_add_attr = is_relaxed != 0 || has_index_signature;

    let ctx = Context::current_context_mut();
    if ctx.cfg.disable_schema_check {
        return;
    }

    let config = schema_config.as_dict_ref();
    for (key, value) in &config.values {
        let is_not_in_schema = schema_value.dict_get_value(key).is_none();
        if should_add_attr && is_not_in_schema {
            let value = index_sign_value
                .deep_copy()
                .union_entry(value, true, false, false, true);
            let op = config
                .ops
                .get(key)
                .unwrap_or(&ConfigEntryOperationKind::Union);
            schema_value.dict_update_entry(key.as_str(), &value.clone(), op, &-1);
        } else if !should_add_attr && is_not_in_schema {
            let schema_name = c2str(schema_name);
            panic!("{key}: No such member in the schema '{schema_name}'");
        }
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_do_check_with_index_sign_attr(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
    check_fn_ptr: *const u64,
    attr_name: *const kclvm_char_t,
) {
    let check_fn_ptr = check_fn_ptr as u64;
    let args_value = ptr_as_ref(args);
    let attr_name = c2str(attr_name);
    unsafe {
        let check_fn: SchemaTypeFunc = transmute_copy(&check_fn_ptr);
        // args_0: config_meta, args_1: config, args_2: schema, args_3: cal_map
        // Schema check function closure
        let config_meta = args_value.arg_i(0).unwrap();
        let config = args_value.arg_i(1).unwrap();
        let mut schema = args_value.arg_i(2).unwrap();
        let cal_map = args_value.arg_i(3).unwrap();
        let backtrack_level_map = args_value.arg_i(4).unwrap();
        let backtrack_cache = args_value.arg_i(5).unwrap();
        for (k, _) in &config.as_dict_ref().values {
            // relaxed keys
            if schema.attr_map_get(k).is_none() {
                let value = ValueRef::str(k);
                schema.dict_update_key_value(attr_name, value);
                let args = &mut ValueRef::list(None);
                // Schema check function closure
                args.list_append(&config_meta);
                args.list_append(&config);
                args.list_append(&schema);
                args.list_append(&cal_map);
                args.list_append(&backtrack_level_map);
                args.list_append(&backtrack_cache);
                let args = args.clone().into_raw();
                check_fn(ctx, args, kwargs);
            }
        }
        schema.dict_remove(attr_name);
    };
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_optional_check(p: *const kclvm_value_ref_t) {
    let p = ptr_as_ref(p);

    let ctx = Context::current_context();
    if !ctx.cfg.disable_schema_check {
        p.schema_check_attr_optional(true);
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_default_settings(
    schema_value: *mut kclvm_value_ref_t,
    config_value: *const kclvm_value_ref_t,
    runtime_type: *const kclvm_char_t,
) {
    let schema_value = mut_ptr_as_ref(schema_value);
    let config_value = ptr_as_ref(config_value);
    let runtime_type = c2str(runtime_type);
    schema_value.schema_default_settings(config_value, runtime_type);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_assert(
    value: *const kclvm_value_ref_t,
    msg: *const kclvm_value_ref_t,
    config_meta: *const kclvm_value_ref_t,
) {
    let value = ptr_as_ref(value);
    let msg = ptr_as_ref(msg);
    let config_meta = ptr_as_ref(config_meta);
    if !value.is_truthy() {
        let ctx = Context::current_context_mut();
        ctx.set_err_type(&ErrType::SchemaCheckFailure_TYPE);
        if let Some(config_meta_file) = config_meta.get_by_key(CONFIG_META_FILENAME) {
            let config_meta_line = config_meta.get_by_key(CONFIG_META_LINE).unwrap();
            let config_meta_column = config_meta.get_by_key(CONFIG_META_COLUMN).unwrap();
            ctx.set_kcl_config_meta_location_info(
                Some("Instance check failed"),
                Some(config_meta_file.as_str().as_str()),
                Some(config_meta_line.as_int() as i32),
                Some(config_meta_column.as_int() as i32),
            );
        }

        ctx.set_kcl_location_info(Some("Check failed on the condition"), None, None, None);

        panic!("{}", msg.as_str());
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_value_new(
    ctx: *mut kclvm_context_t,
    args: *mut kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
    schema_value_or_func: *const kclvm_value_ref_t,
    config: *const kclvm_value_ref_t,
    config_meta: *const kclvm_value_ref_t,
    pkgpath: *const kclvm_char_t,
) -> *const kclvm_value_ref_t {
    let schema_value_or_func = ptr_as_ref(schema_value_or_func);
    if schema_value_or_func.is_func() {
        let schema_func = schema_value_or_func.as_function();
        let schema_fn_ptr = schema_func.fn_ptr;
        let ctx_ref = mut_ptr_as_ref(ctx);
        let now_meta_info = ctx_ref.panic_info.clone();
        if ctx_ref.cfg.debug_mode {
            ctx_ref
                .backtrace
                .push(BacktraceFrame::from_panic_info(&ctx_ref.panic_info));
            ctx_ref.panic_info.kcl_func = schema_func.runtime_type.clone();
        }
        let value = unsafe {
            let org_args = ptr_as_ref(args).deep_copy();
            let schema_fn: SchemaTypeFunc = transmute_copy(&schema_fn_ptr);
            let cal_map = kclvm_value_Dict();
            let instance_pkgpath = kclvm_value_Str(pkgpath);
            // Schema function closures
            let values = [
                // is_sub_schema
                kclvm_value_Bool(0),
                // Config meta
                config_meta,
                // Config value
                config,
                // Schema value
                kclvm_value_Dict(),
                // optional_mapping
                kclvm_value_Dict(),
                // cal order map
                cal_map,
                // backtrack level map
                kclvm_value_Dict(),
                // backtrack cache
                kclvm_value_Dict(),
                // record instance
                kclvm_value_Bool(0),
                // instance pkgpath
                instance_pkgpath,
            ];
            for value in values {
                kclvm_list_append(args, value);
            }
            schema_fn(ctx, args, kwargs);
            // schema args
            let args = org_args.into_raw();
            let values = [
                // is_sub_schema
                kclvm_value_Bool(1),
                // Config meta
                config_meta,
                // Config value
                config,
                // Schema value
                kclvm_value_Dict(),
                // optional_mapping
                kclvm_value_Dict(),
                // cal order map
                cal_map,
                // backtrack level map
                kclvm_value_Dict(),
                // backtrack cache
                kclvm_value_Dict(),
                // record instance
                kclvm_value_Bool(1),
                // instance pkgpath
                instance_pkgpath,
            ];
            for value in values {
                kclvm_list_append(args, value);
            }
            schema_fn(ctx, args, kwargs)
        };
        ctx_ref.panic_info = now_meta_info;
        if ctx_ref.cfg.debug_mode {
            ctx_ref.backtrace.pop();
        }
        value
    } else {
        let config = ptr_as_ref(config);
        let result = schema_value_or_func
            .deep_copy()
            .union_entry(config, true, false, true, true);
        result.into_raw()
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_convert_collection_value(
    value: *const kclvm_value_ref_t,
    tpe: *const kclvm_char_t,
) -> *const kclvm_value_ref_t {
    let value = ptr_as_ref(value);
    let tpe = c2str(tpe);
    let value = type_pack_and_check(value, vec![tpe]);
    value.into_raw()
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_schema_get_value(
    p: *const kclvm_value_ref_t,
    key: *const kclvm_char_t,
    config: *const kclvm_value_ref_t,
    config_meta: *const kclvm_value_ref_t,
    cal_map: *const kclvm_value_ref_t,
    target_attr: *const kclvm_char_t,
    backtrack_level_map: *mut kclvm_value_ref_t,
    backtrack_cache: *mut kclvm_value_ref_t,
    args: *mut kclvm_value_ref_t,
    kwargs: *mut kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let schema = ptr_as_ref(p);
    let key = c2str(key);
    let cal_map = ptr_as_ref(cal_map);
    let target_attr = c2str(target_attr);
    let backtrack_level_map = mut_ptr_as_ref(backtrack_level_map);
    let backtrack_cache = mut_ptr_as_ref(backtrack_cache);
    let args_org = mut_ptr_as_ref(args);
    let kwargs = mut_ptr_as_ref(kwargs);
    let default_level = ValueRef::int(0);
    let level = backtrack_level_map
        .dict_get_value(key)
        .unwrap_or(default_level);
    let level = level.as_int();
    let is_backtracking = level > 0;
    // Deal in-place modify and return it self immediately
    if key == target_attr && !is_backtracking {
        match schema.dict_get_value(key) {
            Some(x) => return x.into_raw(),
            None => return kclvm_value_Undefined(),
        }
    }
    if let Some(v) = backtrack_cache.dict_get_value(key) {
        return v.into_raw();
    }
    if let Some(attr_code) = cal_map.dict_get_value(key) {
        let now_level = level + 1;
        backtrack_level_map.dict_update_key_value(key, ValueRef::int(now_level));
        let attr_code = attr_code.as_list_ref();
        let n = attr_code.values.len();
        let index = n - now_level as usize;
        if index >= n {
            let value = match schema.dict_get_value(key) {
                Some(x) => x,
                None => ValueRef::undefined(),
            };
            return value.into_raw();
        }
        let fn_ptr = &attr_code.values[index];
        let fn_ptr = fn_ptr.as_int();
        // When we calculate other schema attribute values, we retain
        // the row and column number information of the current schema attribute.
        let ctx = Context::current_context_mut();
        let panic_info = ctx.panic_info.clone();
        unsafe {
            let attr_fn: SchemaTypeFunc = transmute_copy(&fn_ptr);
            // args_0: config_meta, args_1: config, args_2: schema, args_3: cal_map
            let config_meta = ptr_as_ref(config_meta);
            let config = ptr_as_ref(config);
            let mut args = ValueRef::list(None);
            let args_org = args_org.as_list_ref();
            for value in &args_org.values {
                args.list_append(value);
            }
            // Schema attr function closure
            args.list_append(config_meta);
            args.list_append(config);
            args.list_append(schema);
            args.list_append(cal_map);
            args.list_append(backtrack_level_map);
            args.list_append(backtrack_cache);
            let args = args.into_raw();
            let kwargs = kwargs.clone().into_raw();
            let ctx = kclvm_context_current();
            attr_fn(ctx, args, kwargs);
        };
        // Restore the panic info of current schema attribute.
        ctx.panic_info = panic_info;
        backtrack_level_map.dict_update_key_value(key, ValueRef::int(level));
        let value = match schema.dict_get_value(key) {
            Some(x) => x,
            None => ValueRef::undefined(),
        };
        backtrack_cache.dict_update_key_value(key, value);
    }
    match schema.dict_get_value(key) {
        Some(x) => x.into_raw(),
        None => kclvm_value_Undefined(),
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_config_attr_map(
    value: *mut kclvm_value_ref_t,
    name: *const kclvm_char_t,
    type_str: *const kclvm_char_t,
) {
    let value = mut_ptr_as_ref(value);
    let name = c2str(name);
    let type_str = c2str(type_str);
    value.update_attr_map(name, type_str);
}

// ----------------------------------------------------------------------------
// values: decorators
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_value_Decorator(
    name: *const kclvm_char_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
    config_meta: *const kclvm_value_ref_t,
    attr_name: *const kclvm_char_t,
    config_value: *const kclvm_value_ref_t,
    is_schema_target: *const kclvm_value_ref_t,
) -> *const kclvm_decorator_value_t {
    let name = c2str(name);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let config_meta = ptr_as_ref(config_meta);
    let attr_name = c2str(attr_name);
    let config_value = ptr_as_ref(config_value);
    let is_schema_target = ptr_as_ref(is_schema_target);
    let decorator = DecoratorValue::new(name, args, kwargs);
    decorator.run(
        attr_name,
        is_schema_target.as_bool(),
        config_value,
        config_meta,
    );
    decorator.into_raw()
}

// ----------------------------------------------------------------------------
// values: string member functions
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_lower(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_lower().into_raw()
    } else {
        panic!("invalid self value in str_lower");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_upper(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_upper().into_raw()
    } else {
        panic!("invalid self value in str_upper");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_capitalize(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_capitalize().into_raw()
    } else {
        panic!("invalid self value in str_capitalize");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_count(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_count(&sub, start.as_ref(), end.as_ref()).into_raw()
        } else {
            panic!("count() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_count");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_endswith(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        if let Some(suffix) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_endswith(&suffix, start.as_ref(), end.as_ref())
                .into_raw()
        } else {
            panic!("endswith() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_endswith");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_find(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_find(&sub, start.as_ref(), end.as_ref()).into_raw()
        } else {
            panic!("find() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_find");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_format(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(val) = args.pop_arg_first() {
        val.str_format(args, kwargs).into_raw()
    } else {
        panic!("invalid self value in str_format");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_index(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_index(&sub, start.as_ref(), end.as_ref()).into_raw()
        } else {
            panic!("index() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_index");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_isalnum(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_isalnum().into_raw()
    } else {
        panic!("invalid self value in str_isalnum");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_isalpha(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_isalpha().into_raw()
    } else {
        panic!("invalid self value in str_isalpha");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_isdigit(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_isdigit().into_raw()
    } else {
        panic!("invalid self value in str_isdigit");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_islower(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_islower().into_raw()
    } else {
        panic!("invalid self value in str_islower");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_isspace(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_isspace().into_raw()
    } else {
        panic!("invalid self value in str_isspace");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_istitle(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_istitle().into_raw()
    } else {
        panic!("invalid self value in str_istitle");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_isupper(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_isupper().into_raw()
    } else {
        panic!("invalid self value in str_isupper");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_join(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        let iter = args.arg_i(0).unwrap();
        val.str_join(&iter).into_raw()
    } else {
        panic!("invalid self value in str_join");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_lstrip(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        let chars = args.arg_i(0);
        val.str_lstrip(chars.as_ref()).into_raw()
    } else {
        panic!("invalid self value in str_lstrip");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_rstrip(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        let chars = args.arg_i(0);
        val.str_rstrip(chars.as_ref()).into_raw()
    } else {
        panic!("invalid self value in str_rstrip");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_replace(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        let old = args.arg_i(0).unwrap();
        let new = args.arg_i(1).unwrap();
        let count = args.arg_i(2);
        val.str_replace(&old, &new, count.as_ref()).into_raw()
    } else {
        panic!("invalid self value in str_replace");
    }
}

/// If the string starts with the prefix string, return string[len(prefix):].
/// Otherwise, return a copy of the original string.
#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_removeprefix(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        let prefix = args.arg_i(0).unwrap();
        val.str_removeprefix(&prefix).into_raw()
    } else {
        panic!("invalid self value in str_removeprefix");
    }
}

/// If the string ends with the suffix string and that suffix is not empty, return string[:-len(suffix)].
/// Otherwise, return a copy of the original string.
#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_removesuffix(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        let suffix = args.arg_i(0).unwrap();
        val.str_removesuffix(&suffix).into_raw()
    } else {
        panic!("invalid self value in str_removesuffix");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_rfind(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_rfind(&sub, start.as_ref(), end.as_ref()).into_raw()
        } else {
            panic!("rfind() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_rfind");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_rindex(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_rindex(&sub, start.as_ref(), end.as_ref())
                .into_raw()
        } else {
            panic!("rindex() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_rindex");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_rsplit(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(val) = args.pop_arg_first() {
        let sep = if let Some(sep) = args.arg_i(0) {
            Some(sep)
        } else {
            kwargs.kwarg("sep")
        };
        let maxsplit = if let Some(maxsplit) = args.arg_i(1) {
            Some(maxsplit)
        } else {
            kwargs.kwarg("maxsplit")
        };
        val.str_rsplit(sep.as_ref(), maxsplit.as_ref()).into_raw()
    } else {
        panic!("invalid self value in str_rsplit");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_split(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(val) = args.pop_arg_first() {
        let sep = if let Some(sep) = args.arg_i(0) {
            Some(sep)
        } else {
            kwargs.kwarg("sep")
        };
        let maxsplit = if let Some(maxsplit) = args.arg_i(1) {
            Some(maxsplit)
        } else {
            kwargs.kwarg("maxsplit")
        };
        let x = val.str_split(sep.as_ref(), maxsplit.as_ref());
        x.into_raw()
    } else {
        panic!("invalid self value in str_split");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_splitlines(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(val) = args.pop_arg_first() {
        if let Some(keepends) = args.arg_i(0) {
            val.str_splitlines(Some(&keepends)).into_raw()
        } else if let Some(keepends) = kwargs.kwarg("keepends") {
            val.str_splitlines(Some(&keepends)).into_raw()
        } else {
            val.str_splitlines(None).into_raw()
        }
    } else {
        panic!("invalid self value in str_splitlines");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_startswith(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        if let Some(suffix) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_startswith(&suffix, start.as_ref(), end.as_ref())
                .into_raw()
        } else {
            panic!("startswith() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_startswith");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_strip(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        let chars = args.arg_i(0);
        val.str_strip(chars.as_ref()).into_raw()
    } else {
        panic!("invalid self value in str_strip");
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_builtin_str_title(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    if let Some(val) = args.pop_arg_first() {
        val.str_title().into_raw()
    } else {
        panic!("invalid self value in str_title");
    }
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
