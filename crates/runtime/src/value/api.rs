//! Copyright The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use std::{mem::transmute_copy, os::raw::c_char};

use kcl_primitives::IndexMap;

use crate::*;

use self::{eval::LazyEvalScope, walker::walk_value_mut};

#[allow(non_camel_case_types)]
pub type kcl_context_t = Context;

#[allow(non_camel_case_types)]
pub type kcl_eval_scope_t = LazyEvalScope;

#[allow(non_camel_case_types)]
pub type kcl_decorator_value_t = DecoratorValue;

#[allow(non_camel_case_types)]
pub type kcl_kind_t = Kind;

#[allow(non_camel_case_types)]
pub type kcl_type_t = Type;

#[allow(non_camel_case_types)]
pub type kcl_value_ref_t = ValueRef;

#[allow(non_camel_case_types)]
pub type kcl_iterator_t = ValueIterator;

#[allow(non_camel_case_types)]
pub type kcl_char_t = c_char;

#[allow(non_camel_case_types)]
pub type kcl_size_t = i32;

#[allow(non_camel_case_types)]
pub type kcl_bool_t = i8;

#[allow(non_camel_case_types)]
pub type kcl_int_t = i64;

#[allow(non_camel_case_types)]
pub type kcl_float_t = f64;

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_import_names(
    p: *mut kcl_context_t,
    import_names: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let import_names = unsafe { ptr_as_ref(import_names) };

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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Undefined(
    ctx: *mut kcl_context_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::undefined())
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_None(ctx: *mut kcl_context_t) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::none())
}

// bool/int/float/str

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_True(ctx: *mut kcl_context_t) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    unsafe { kcl_value_Bool(ctx, 1) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_False(ctx: *mut kcl_context_t) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    unsafe { kcl_value_Bool(ctx, 0) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Bool(
    ctx: *mut kcl_context_t,
    v: kcl_bool_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if v != 0 {
        ValueRef::bool(true).into_raw(ctx)
    } else {
        ValueRef::bool(false).into_raw(ctx)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Int(
    ctx: *mut kcl_context_t,
    v: kcl_int_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::int(v))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Float(
    ctx: *mut kcl_context_t,
    v: kcl_float_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::float(v))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Unit(
    ctx: *mut kcl_context_t,
    v: kcl_float_t,
    raw: kcl_int_t,
    unit: *const kcl_char_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let unit = unsafe { c2str(unit) };
    new_mut_ptr(ctx, ValueRef::unit(v, raw, unit))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Str(
    ctx: *mut kcl_context_t,
    v: *const kcl_char_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    unsafe {
        if v.is_null() || *v == '\0' as c_char {
            return new_mut_ptr(ctx, ValueRef::str(""));
        }
    }
    new_mut_ptr(ctx, ValueRef::str(unsafe { c2str(v) }))
}

// list/dict/schema

/// # Safety
/// The caller must ensure that `ctx` is a valid pointer
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_List(ctx: *mut kcl_context_t) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::list(None))
}

/// # Safety
/// The caller must ensure that `ctx` is a valid pointer
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_List6(
    ctx: *mut kcl_context_t,
    v1: *const kcl_value_ref_t,
    v2: *const kcl_value_ref_t,
    v3: *const kcl_value_ref_t,
    v4: *const kcl_value_ref_t,
    v5: *const kcl_value_ref_t,
    v6: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let values: Vec<&ValueRef> = vec![v1, v2, v3, v4, v5, v6]
        .into_iter()
        .map(|ptr| unsafe { ptr_as_ref(ptr) })
        .collect();
    new_mut_ptr(ctx, ValueRef::list(Some(values.as_slice())))
}

/// # Safety
/// The caller must ensure that `ctx` is a valid pointer
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_List10(
    ctx: *mut kcl_context_t,
    v1: *const kcl_value_ref_t,
    v2: *const kcl_value_ref_t,
    v3: *const kcl_value_ref_t,
    v4: *const kcl_value_ref_t,
    v5: *const kcl_value_ref_t,
    v6: *const kcl_value_ref_t,
    v7: *const kcl_value_ref_t,
    v8: *const kcl_value_ref_t,
    v9: *const kcl_value_ref_t,
    v10: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let values: Vec<&ValueRef> = vec![v1, v2, v3, v4, v5, v6, v7, v8, v9, v10]
        .into_iter()
        .map(|ptr| unsafe { ptr_as_ref(ptr) })
        .collect();

    new_mut_ptr(ctx, ValueRef::list(Some(values.as_slice())))
}

/// # Safety
/// The caller must ensure that `ctx` is a valid pointer
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Dict(ctx: *mut kcl_context_t) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::dict(None))
}

/// # Safety
/// The caller must ensure that `ctx` is a valid pointer
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Schema(ctx: *mut kcl_context_t) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::schema())
}

/// # Safety
/// The caller must ensure that `ctx`, `schema_dict`, `config`, `config_meta`,
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_schema_with_config(
    ctx: *mut kcl_context_t,
    schema_dict: *const kcl_value_ref_t,
    config: *const kcl_value_ref_t,
    config_meta: *const kcl_value_ref_t,
    name: *const kcl_char_t,
    pkgpath: *const kcl_char_t,
    is_sub_schema: *const kcl_value_ref_t,
    record_instance: *const kcl_value_ref_t,
    instance_pkgpath: *const kcl_value_ref_t,
    optional_mapping: *const kcl_value_ref_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let schema_dict = unsafe { ptr_as_ref(schema_dict) };
    // Config dict
    let config = unsafe { ptr_as_ref(config) };
    let config_meta = unsafe { ptr_as_ref(config_meta) };
    let config_keys: Vec<String> = config.as_dict_ref().values.keys().cloned().collect();
    // Schema meta
    let name = unsafe { c2str(name) };
    let pkgpath = unsafe { c2str(pkgpath) };
    let runtime_type = schema_runtime_type(name, pkgpath);
    let is_sub_schema = unsafe { ptr_as_ref(is_sub_schema) };
    let record_instance = unsafe { ptr_as_ref(record_instance) };
    let instance_pkgpath = unsafe { ptr_as_ref(instance_pkgpath) };
    let instance_pkgpath = instance_pkgpath.as_str();
    let optional_mapping = unsafe { ptr_as_ref(optional_mapping) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if record_instance.is_truthy() {
        // Record schema instance in the context
        if !ctx.instances.contains_key(&runtime_type) {
            ctx.instances
                .insert(runtime_type.clone(), IndexMap::default());
        }
        let pkg_instance_map = ctx.instances.get_mut(&runtime_type).unwrap();
        if !pkg_instance_map.contains_key(&instance_pkgpath) {
            pkg_instance_map.insert(instance_pkgpath.clone(), vec![]);
        }
        pkg_instance_map
            .get_mut(&instance_pkgpath)
            .unwrap()
            .push(schema_dict.clone());
    }
    // Dict to schema
    if is_sub_schema.is_truthy() {
        let schema = schema_dict.dict_to_schema(
            name,
            pkgpath,
            &config_keys,
            config_meta,
            optional_mapping,
            Some(args.clone()),
            Some(kwargs.clone()),
        );
        schema.into_raw(ctx)
    } else {
        schema_dict.clone().into_raw(ctx)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Function(
    ctx: *mut kcl_context_t,
    fn_ptr: *const u64,
    closure: *const kcl_value_ref_t,
    name: *const kcl_char_t,
    is_external: kcl_bool_t,
) -> *mut kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let closure = unsafe { ptr_as_ref(closure) };
    let name = unsafe { c2str(name) };
    new_mut_ptr(
        ctx,
        ValueRef::func(
            fn_ptr as u64,
            0,
            closure.clone(),
            name,
            "",
            is_external != 0,
        ),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Function_using_ptr(
    ctx: *mut kcl_context_t,
    fn_ptr: *const u64,
    name: *const kcl_char_t,
) -> *mut kcl_value_ref_t {
    let name = unsafe { c2str(name) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(
        ctx,
        ValueRef::func(fn_ptr as u64, 0, ValueRef::none(), name, "", false),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_schema_function(
    ctx: *mut kcl_context_t,
    fn_ptr: *const u64,
    check_fn_ptr: *const u64,
    attr_map: *const kcl_value_ref_t,
    tpe: *const kcl_char_t,
) -> *mut kcl_value_ref_t {
    // Schema function closures
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
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
    let runtime_type = unsafe { c2str(tpe) };
    let schema_func = ValueRef::func(
        fn_ptr as u64,
        check_fn_ptr as u64,
        schema_args,
        "",
        runtime_type,
        false,
    );
    let attr_map = unsafe { ptr_as_ref(attr_map) };
    let attr_dict = attr_map.as_dict_ref();
    let schema_ty = SchemaType {
        name: runtime_type.to_string(),
        attrs: attr_dict
            .values
            .iter()
            .map(|(k, _)| (k.to_string(), Type::any()))  // TODO: store schema attr type in the runtime.
            .collect(),
        has_index_signature: attr_dict.attr_map.contains_key(CAL_MAP_INDEX_SIGNATURE),
        func: schema_func.clone(),
    };
    ctx.all_schemas.insert(runtime_type.to_string(), schema_ty);
    new_mut_ptr(ctx, schema_func)
}

// ----------------------------------------------------------------------------
// values: json
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_from_json(
    ctx: *mut kcl_context_t,
    s: *const kcl_char_t,
) -> *mut kcl_value_ref_t {
    let ctx_ref = unsafe { mut_ptr_as_ref(ctx) };
    if s.is_null() {
        return unsafe { kcl_value_Undefined(ctx) };
    }
    match ValueRef::from_json(ctx_ref, unsafe { c2str(s) }) {
        Ok(x) => x.into_raw(ctx_ref),
        _ => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_to_json_value(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if p.is_null() {
        return unsafe { kcl_value_Str(ctx, std::ptr::null()) };
    }

    let p = unsafe { ptr_as_ref(p) };
    let s = p.to_json_string();
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::str(s.as_ref()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_to_json_value_with_null(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if p.is_null() {
        return unsafe { kcl_value_Str(ctx, std::ptr::null()) };
    }

    let p = unsafe { ptr_as_ref(p) };
    let s = p.to_json_string_with_null();
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    new_mut_ptr(ctx, ValueRef::str(s.as_ref()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_plan_to_json(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let ctx: &mut Context = unsafe { mut_ptr_as_ref(ctx) };
    let value = match ctx.buffer.custom_manifests_output.clone() {
        Some(output) => ValueRef::from_yaml_stream(ctx, &output).unwrap(),
        None => p.clone(),
    };
    let (json_string, yaml_string) = value.plan(ctx);
    ctx.json_result = json_string.clone();
    ctx.yaml_result = yaml_string.clone();
    new_mut_ptr(ctx, ValueRef::str(&json_string))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_plan_to_yaml(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let value = match ctx.buffer.custom_manifests_output.clone() {
        Some(output) => ValueRef::from_yaml_stream(ctx, &output).unwrap(),
        None => p.clone(),
    };
    let (json_string, yaml_string) = value.plan(ctx);
    ctx.json_result = json_string.clone();
    ctx.yaml_result = yaml_string.clone();
    new_mut_ptr(ctx, ValueRef::str(&yaml_string))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_to_yaml_value(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if p.is_null() {
        return unsafe { kcl_value_Str(ctx, std::ptr::null()) };
    }
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let p = unsafe { ptr_as_ref(p) };
    let s = p.to_yaml_string();

    new_mut_ptr(ctx, ValueRef::str(s.as_ref()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_to_str_value(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if p.is_null() {
        return unsafe { kcl_value_Str(ctx, std::ptr::null()) };
    }

    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let p = unsafe { ptr_as_ref(p) };
    let s = p.to_string();

    new_mut_ptr(ctx, ValueRef::str(s.as_ref()))
}

// ----------------------------------------------------------------------------
// values: value pointer
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Str_ptr(p: *const kcl_value_ref_t) -> *const kcl_char_t {
    let p = unsafe { ptr_as_ref(p) };
    match &*p.rc.borrow() {
        Value::str_value(v) => v.as_ptr() as *const c_char,
        _ => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_function_ptr(p: *const kcl_value_ref_t) -> *const u64 {
    let p = unsafe { ptr_as_ref(p) };
    match &*p.rc.borrow() {
        Value::func_value(v) => v.fn_ptr as *const u64,
        _ => std::ptr::null::<u64>(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_check_function_ptr(
    p: *const kcl_value_ref_t,
) -> *const u64 {
    let p = unsafe { ptr_as_ref(p) };
    match &*p.rc.borrow() {
        Value::func_value(v) => v.check_fn_ptr as *const u64,
        _ => std::ptr::null::<u64>(),
    }
}

// ----------------------------------------------------------------------------
// values: method
// ----------------------------------------------------------------------------

// clone

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_deep_copy(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    p.deep_copy().into_raw(ctx)
}

// delete

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_delete(p: *mut kcl_value_ref_t) {
    if p.is_null() {
        return;
    }
    let val = unsafe { ptr_as_ref(p) };
    val.from_raw();
    unsafe { free_mut_ptr(p) };
}

// ----------------------------------------------------------------------------
// values: iterator
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_iter(p: *const kcl_value_ref_t) -> *mut kcl_iterator_t {
    let p = unsafe { ptr_as_ref(p) };
    let iter = ValueIterator::from_value(p);
    Box::into_raw(Box::new(iter))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_iterator_delete(p: *mut kcl_iterator_t) {
    unsafe { free_mut_ptr(p) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_iterator_is_end(p: *mut kcl_iterator_t) -> kcl_bool_t {
    let p = unsafe { ptr_as_ref(p) };
    p.is_end() as kcl_bool_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_iterator_cur_key(
    p: *mut kcl_iterator_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    match p.key() {
        Some(x) => x,
        None => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_iterator_cur_value(
    p: *mut kcl_iterator_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { mut_ptr_as_ref(p) };
    match p.value() {
        Some(x) => x,
        None => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_iterator_next_value(
    p: *mut kcl_iterator_t,
    host: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { mut_ptr_as_ref(p) };
    let host = unsafe { ptr_as_ref(host) };

    match p.next(host) {
        Some(x) => x,
        None => std::ptr::null(),
    }
}

// ----------------------------------------------------------------------------
// values: list
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_len(p: *const kcl_value_ref_t) -> kcl_size_t {
    let p = unsafe { ptr_as_ref(p) };
    p.len() as kcl_size_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_resize(p: *mut kcl_value_ref_t, newsize: kcl_size_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.list_resize(newsize as usize);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_clear(p: *mut kcl_value_ref_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.list_clear();
}

/// Return number of occurrences of the list value.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_count(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    item: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let item = unsafe { ptr_as_ref(item) };
    let count = p.list_count(item);
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let count_value = ValueRef::int(count as i64);
    count_value.into_raw(ctx)
}

/// Return first index of the list value. Panic if the value is not present.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_find(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    item: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let item = unsafe { ptr_as_ref(item) };
    let index = p.list_find(item);
    let index_value = ValueRef::int(index as i64);
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    index_value.into_raw(ctx)
}

/// Insert object before index of the list value.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_insert(
    p: *mut kcl_value_ref_t,
    index: *const kcl_value_ref_t,
    value: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let index = unsafe { ptr_as_ref(index) };
    let value = unsafe { ptr_as_ref(value) };
    p.list_insert_at(index.as_int() as usize, value);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_get(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    i: kcl_size_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    match p.list_get(i as isize) {
        Some(x) => x.into_raw(ctx),
        _ => panic!("list index out of range"),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_get_option(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    i: kcl_size_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };

    match p.list_get_option(i as isize) {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        _ => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_set(
    p: *mut kcl_value_ref_t,
    i: kcl_size_t,
    v: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };
    p.list_set(i as usize, v);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_pop(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { mut_ptr_as_ref(p) };

    match p.list_pop() {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        _ => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_pop_first(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { mut_ptr_as_ref(p) };
    match p.list_pop_first() {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        _ => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_append(
    p: *mut kcl_value_ref_t,
    v: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };
    p.list_append(v);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_append_bool(p: *mut kcl_value_ref_t, v: kcl_bool_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.list_append(&ValueRef::bool(v != 0));
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_append_int(p: *mut kcl_value_ref_t, v: kcl_int_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.list_append(&ValueRef::int(v));
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_append_float(p: *mut kcl_value_ref_t, v: kcl_float_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.list_append(&ValueRef::float(v));
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_append_str(p: *mut kcl_value_ref_t, v: *const kcl_char_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.list_append(&ValueRef::str(unsafe { c2str(v) }));
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_append_unpack(
    p: *mut kcl_value_ref_t,
    v: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };

    if p.is_list() {
        p.list_append_unpack(v);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_list_remove_at(p: *mut kcl_value_ref_t, i: kcl_size_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.list_remove_at(i as usize);
}

// ----------------------------------------------------------------------------
// values: dict
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_len(p: *const kcl_value_ref_t) -> kcl_size_t {
    let p = unsafe { ptr_as_ref(p) };
    match &*p.rc.borrow() {
        Value::dict_value(dict) => dict.values.len() as kcl_size_t,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_clear(p: *mut kcl_value_ref_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.dict_clear();
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_is_override_attr(
    p: *const kcl_value_ref_t,
    key: *const kcl_char_t,
) -> kcl_bool_t {
    let p = unsafe { ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
    let is_override_op = matches!(
        p.dict_get_attr_operator(key),
        Some(ConfigEntryOperationKind::Override)
    );
    let without_index = matches!(p.dict_get_insert_index(key), Some(-1) | None);
    (is_override_op && without_index) as kcl_bool_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_get(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    key: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let key = unsafe { ptr_as_ref(key) };

    match p.dict_get(key) {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        None => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_has_value(
    p: *const kcl_value_ref_t,
    key: *const kcl_char_t,
) -> kcl_bool_t {
    let p = unsafe { ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
    match p.dict_get_value(key) {
        Some(_) => true as kcl_bool_t,
        None => false as kcl_bool_t,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_get_value(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    key: *const kcl_char_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
    match p.dict_get_value(key) {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        None => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_get_entry(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    key: *const kcl_char_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
    match p.dict_get_entry(key) {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        None => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_get_value_by_path(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    path: *const kcl_char_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let path = unsafe { c2str(path) };
    match p.get_by_path(path) {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        None => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_set_value(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
    key: *const kcl_char_t,
    val: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
    let val = unsafe { ptr_as_ref(val) };
    if p.is_config() {
        p.dict_update_key_value(key, val.clone());
        if p.is_schema() {
            let schema: ValueRef;
            {
                let schema_value = p.as_schema();
                let mut config_keys = schema_value.config_keys.clone();
                config_keys.push(key.to_string());
                schema = resolve_schema(unsafe { mut_ptr_as_ref(ctx) }, p, &config_keys);
            }
            p.schema_update_with_schema(&schema);
        }
    } else {
        panic!(
            "failed to update the dict. An iterable of key-value pairs was expected, but got {}. Check if the syntax for updating the dictionary with the attribute '{}' is correct",
            p.type_str(),
            key
        );
    }
}

#[unsafe(no_mangle)]
/// Return all dict keys.
pub unsafe extern "C-unwind" fn kcl_dict_keys(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let r = p.dict_keys();
    r.into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
/// Return all dict values.
pub unsafe extern "C-unwind" fn kcl_dict_values(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(p) };
    let r = p.dict_values();
    r.into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_insert(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
    key: *const kcl_char_t,
    v: *const kcl_value_ref_t,
    op: kcl_size_t,
    insert_index: kcl_size_t,
    has_insert_index: kcl_bool_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };
    p.dict_insert(
        unsafe { mut_ptr_as_ref(ctx) },
        unsafe { c2str(key) },
        v,
        ConfigEntryOperationKind::from_i32(op),
        if has_insert_index != 0 {
            Some(insert_index)
        } else {
            None
        },
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_merge(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
    key: *const kcl_char_t,
    v: *const kcl_value_ref_t,
    op: kcl_size_t,
    insert_index: kcl_size_t,
    has_insert_index: kcl_bool_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };
    let key = unsafe { c2str(key) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let attr_map = {
        match &*p.rc.borrow() {
            Value::dict_value(dict) => dict.attr_map.clone(),
            Value::schema_value(schema) => schema.config.attr_map.clone(),
            _ => panic!("invalid object '{}' in attr_map", p.type_str()),
        }
    };
    if attr_map.contains_key(key) {
        let v = type_pack_and_check(ctx, v, vec![attr_map.get(key).unwrap()], false);
        p.dict_merge(
            ctx,
            key,
            &v,
            ConfigEntryOperationKind::from_i32(op),
            if has_insert_index != 0 {
                Some(insert_index)
            } else {
                None
            },
        );
    } else {
        p.dict_merge(
            ctx,
            key,
            v,
            ConfigEntryOperationKind::from_i32(op),
            if has_insert_index != 0 {
                Some(insert_index)
            } else {
                None
            },
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_insert_value(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
    key: *const kcl_value_ref_t,
    v: *const kcl_value_ref_t,
    op: kcl_size_t,
    insert_index: kcl_size_t,
    has_insert_index: kcl_bool_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };
    let key = unsafe { ptr_as_ref(key) };
    let key = key.attr_str();
    p.dict_insert(
        unsafe { mut_ptr_as_ref(ctx) },
        key.as_str(),
        v,
        ConfigEntryOperationKind::from_i32(op),
        if has_insert_index != 0 {
            Some(insert_index)
        } else {
            None
        },
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_update_key_value(
    p: *mut kcl_value_ref_t,
    key: *const kcl_value_ref_t,
    v: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };
    let key = unsafe { ptr_as_ref(key) };
    let key = key.attr_str();
    p.dict_update_key_value(key.as_str(), v.clone());
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_safe_insert(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
    key: *const kcl_char_t,
    v: *const kcl_value_ref_t,
    op: kcl_size_t,
    insert_index: kcl_size_t,
    has_insert_index: kcl_bool_t,
) {
    if p.is_null() || key.is_null() || v.is_null() {
        return;
    }
    unsafe { kcl_dict_insert(ctx, p, key, v, op, insert_index, has_insert_index) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_insert_unpack(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
    v: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let v = unsafe { ptr_as_ref(v) };
    p.dict_insert_unpack(ctx, v);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_default_collection_insert_int_pointer(
    p: *mut kcl_value_ref_t,
    key: *const kcl_char_t,
    ptr: *const u64,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_default_collection_insert_value(
    p: *mut kcl_value_ref_t,
    key: *const kcl_char_t,
    value: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
    let value = unsafe { ptr_as_ref(value) };
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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_remove(p: *mut kcl_value_ref_t, key: *const kcl_char_t) {
    let p = unsafe { mut_ptr_as_ref(p) };
    p.dict_remove(unsafe { c2str(key) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_dict_update(
    p: *mut kcl_value_ref_t,
    v: *const kcl_value_ref_t,
) {
    let p = unsafe { mut_ptr_as_ref(p) };
    let v = unsafe { ptr_as_ref(v) };
    p.dict_update(v);
}

// ----------------------------------------------------------------------------
// values: arith
// ----------------------------------------------------------------------------

// is true

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_is_truthy(p: *const kcl_value_ref_t) -> kcl_bool_t {
    let p = unsafe { ptr_as_ref(p) };
    p.is_truthy() as kcl_bool_t
}

// len

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_len(p: *const kcl_value_ref_t) -> kcl_size_t {
    let p = unsafe { ptr_as_ref(p) };
    p.len() as kcl_size_t
}

// compare

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_cmp_equal_to(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if a == b {
        return unsafe { kcl_value_Bool(ctx, 1) };
    }
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.cmp_equal(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_cmp_not_equal_to(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if a == b {
        return unsafe { kcl_value_Bool(ctx, 0) };
    }
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(!a.cmp_equal(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_cmp_less_than(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if a == b {
        return unsafe { kcl_value_Bool(ctx, 0) };
    }
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.cmp_less_than(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_cmp_less_than_or_equal(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.cmp_less_than_or_equal(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_cmp_greater_than(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    if a == b {
        return unsafe { kcl_value_Bool(ctx, 0) };
    }
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.cmp_greater_than(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_cmp_greater_than_or_equal(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.cmp_greater_than_or_equal(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

// is/in

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_is(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    if a == b {
        return unsafe { kcl_value_Bool(ctx, 1) };
    }
    unsafe { kcl_value_Bool(ctx, 0) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_is_not(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    if a == b {
        return unsafe { kcl_value_Bool(ctx, 0) };
    }
    unsafe { kcl_value_Bool(ctx, 1) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_in(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.r#in(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_not_in(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.not_in(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_as(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ty_str = b.as_str();
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let value = type_pack_and_check(ctx, a, vec![ty_str.as_str()], true);
    value.into_raw(ctx)
}

// unary-xxx

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_unary_plus(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    a.unary_plus().into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_unary_minus(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    a.unary_minus().into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_unary_not(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    a.unary_not().into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_unary_l_not(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    a.unary_l_not().into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

// op-xxx

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_add(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_add(ctx, b).into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_sub(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_sub(ctx, b).into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_mul(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_mul(ctx, b).into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_div(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_div(b).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_mod(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_mod(b).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_pow(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_pow(ctx, b).into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_floor_div(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_floor_div(b).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_bit_lshift(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_bit_lshift(ctx, b).into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_bit_rshift(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_bit_rshift(ctx, b).into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_bit_and(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_bit_and(b).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_bit_xor(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_bit_xor(b).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_bit_or(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_bit_or(ctx, b).into_raw(ctx)
}

// op-aug-xxx

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_add(
    ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_aug_add(ctx, b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_sub(
    ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_aug_sub(ctx, b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_mul(
    ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_aug_mul(ctx, b)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_div(
    _ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_aug_div(b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_mod(
    _ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_aug_mod(b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_pow(
    ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_aug_pow(ctx, b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_floor_div(
    _ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_aug_floor_div(b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_bit_lshift(
    ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_aug_bit_lshift(ctx, b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_bit_rshift(
    ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_aug_bit_rshift(ctx, b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_bit_and(
    _ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_aug_bit_and(b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_bit_xor(
    _ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_aug_bit_xor(b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_op_aug_bit_or(
    ctx: *mut kcl_context_t,
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    a.bin_aug_bit_or(ctx, b) as *const kcl_value_ref_t
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_union(
    ctx: *mut kcl_context_t,
    schema: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let a = unsafe { mut_ptr_as_ref(schema) };
    let b = unsafe { ptr_as_ref(b) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let attr_map = match &*a.rc.borrow() {
        Value::dict_value(dict) => dict.attr_map.clone(),
        Value::schema_value(schema) => schema.config.attr_map.clone(),
        _ => panic!("invalid object '{}' in attr_map", a.type_str()),
    };
    let opts = UnionOptions {
        list_override: false,
        idempotent_check: false,
        config_resolve: true,
    };
    if b.is_config() {
        let dict = b.as_dict_ref();
        for k in dict.values.keys() {
            let entry = b.dict_get_entry(k).unwrap();
            a.union_entry(ctx, &entry, true, &opts);
            // Has type annotation
            if let Some(ty) = attr_map.get(k) {
                let value = a.dict_get_value(k).unwrap();
                a.dict_update_key_value(k, type_pack_and_check(ctx, &value, vec![ty], false));
            }
        }
        a.clone().into_raw(ctx)
    } else {
        a.union_entry(ctx, b, true, &opts).into_raw(ctx)
    }
}

// logic: && ||

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_logic_and(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.logic_and(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_logic_or(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    ValueRef::bool(a.logic_or(b)).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_subscr(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_subscr(b).into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_subscr_set(
    ctx: *mut kcl_context_t,
    p: *mut kcl_value_ref_t,
    index: *const kcl_value_ref_t,
    val: *const kcl_value_ref_t,
) {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let p = unsafe { mut_ptr_as_ref(p) };
    let index = unsafe { ptr_as_ref(index) };
    let val = unsafe { ptr_as_ref(val) };
    p.bin_subscr_set(ctx, index, val);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_subscr_option(
    ctx: *mut kcl_context_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    a.bin_subscr_option(b)
        .into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_load_attr(
    ctx: *mut kcl_context_t,
    obj: *const kcl_value_ref_t,
    key: *const kcl_char_t,
) -> *const kcl_value_ref_t {
    let p = unsafe { ptr_as_ref(obj) };
    let key = unsafe { c2str(key) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    p.load_attr(key).into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_load_attr_option(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    key: *const kcl_char_t,
) -> *const kcl_value_ref_t {
    let p_ref = unsafe { ptr_as_ref(p) };
    if p_ref.is_truthy() {
        unsafe { kcl_value_load_attr(ctx, p, key) }
    } else {
        unsafe { kcl_value_None(ctx) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_remove_item(
    a: *mut kcl_value_ref_t,
    b: *const kcl_value_ref_t,
) {
    let a = unsafe { mut_ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    if a.is_dict() {
        a.dict_remove(&b.as_str());
    } else if a.is_list() {
        a.list_remove(b);
    } else {
        panic!("only list, dict and schema can be removed item");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_slice(
    ctx: *mut kcl_context_t,
    x: *const kcl_value_ref_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
    step: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let x = unsafe { ptr_as_ref(x) };
    let a = unsafe { ptr_as_ref(a) };
    let b = unsafe { ptr_as_ref(b) };
    let step = unsafe { ptr_as_ref(step) };
    x.list_slice(a, b, step)
        .into_raw(unsafe { mut_ptr_as_ref(ctx) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_slice_option(
    ctx: *mut kcl_context_t,
    x: *const kcl_value_ref_t,
    a: *const kcl_value_ref_t,
    b: *const kcl_value_ref_t,
    step: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let value = unsafe { ptr_as_ref(x) };
    if value.is_truthy() {
        unsafe { kcl_value_slice(ctx, x, a, b, step) }
    } else {
        unsafe { kcl_value_None(ctx) }
    }
}

// ----------------------------------------------------------------------------
// values: schema
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_backtrack_cache(
    ctx: *mut kcl_context_t,
    schema: *const kcl_value_ref_t,
    cache: *mut kcl_value_ref_t,
    cal_map: *const kcl_value_ref_t,
    name: *const kcl_char_t,
    runtime_type: *const kcl_value_ref_t,
) {
    let schema = unsafe { ptr_as_ref(schema) };
    let cache = unsafe { mut_ptr_as_ref(cache) };
    let cal_map = unsafe { ptr_as_ref(cal_map) };
    let name = unsafe { c2str(name) };
    if let Some(v) = cal_map.dict_get_value(name) {
        if v.len() == 1 {
            if let Some(value) = schema.dict_get_value(name) {
                cache.dict_update_key_value(name, value);
            }
        } else if let (Some(cal_map_runtime_type_list), Some(cal_map_meta_line_list)) = (
            cal_map.dict_get_value(&format!("{name}_{CAL_MAP_RUNTIME_TYPE}")),
            cal_map.dict_get_value(&format!("{name}_{CAL_MAP_META_LINE}")),
        ) && let (Some(cal_map_runtime_type), Some(cal_map_meta_line)) = (
            cal_map_runtime_type_list.list_get(-1),
            cal_map_meta_line_list.list_get(-1),
        ) {
            let runtime_type = unsafe { ptr_as_ref(runtime_type) };
            let ctx = unsafe { mut_ptr_as_ref(ctx) };
            let line = ctx.panic_info.kcl_line as i64;
            let cal_map_meta_line = cal_map_meta_line.as_int();
            if runtime_type == &cal_map_runtime_type
                && line >= cal_map_meta_line
                && let Some(value) = schema.dict_get_value(name)
            {
                cache.dict_update_key_value(name, value);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_instances(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx_ref = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(val) = args.pop_arg_first() {
        let function = val.as_function();
        let full_pkg = args.arg_0().or_else(|| kwargs.kwarg("full_pkg"));
        let full_pkg = if let Some(v) = full_pkg {
            v.is_truthy()
        } else {
            false
        };
        let runtime_type = &function.runtime_type;
        if ctx_ref.instances.contains_key(runtime_type) {
            let mut list = ValueRef::list(None);
            let instance_map = ctx_ref.instances.get(runtime_type).unwrap();
            if full_pkg {
                for (_, v_list) in instance_map {
                    collect_schema_instances(&mut list, v_list, runtime_type)
                }
            } else {
                // Get the schema instances only located at the main package.
                if let Some(v_list) = instance_map.get(MAIN_PKG_PATH) {
                    collect_schema_instances(&mut list, v_list, runtime_type)
                }
                if let Some(v_list) = instance_map.get("") {
                    collect_schema_instances(&mut list, v_list, runtime_type)
                }
            };
            list.into_raw(ctx_ref)
        } else {
            unsafe { kcl_value_List(ctx) }
        }
    } else {
        unsafe { kcl_value_None(ctx) }
    }
}

fn collect_schema_instances(list: &mut ValueRef, v_list: &[ValueRef], runtime_type: &str) {
    for v in v_list {
        if v.is_schema() {
            list.list_append(v)
        } else if v.is_dict() {
            let runtime_type = v
                .get_potential_schema_type()
                .unwrap_or(runtime_type.to_string());
            let names: Vec<&str> = runtime_type.rsplit('.').collect();
            let name = names[0];
            let pkgpath = names[1];
            let v = v.dict_to_schema(
                name,
                pkgpath,
                &[],
                &ValueRef::dict(None),
                &ValueRef::dict(None),
                None,
                None,
            );
            list.list_append(&v);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_value_check(
    ctx: *mut kcl_context_t,
    schema_value: *mut kcl_value_ref_t,
    schema_config: *const kcl_value_ref_t,
    _config_meta: *const kcl_value_ref_t,
    schema_name: *const kcl_char_t,
    index_sign_value: *const kcl_value_ref_t,
    key_name: *const kcl_char_t,
    key_type: *const kcl_char_t,
    value_type: *const kcl_char_t,
    _any_other: kcl_bool_t,
) {
    let schema_value = unsafe { mut_ptr_as_ref(schema_value) };
    let schema_config = unsafe { ptr_as_ref(schema_config) };
    let index_sign_value = unsafe { ptr_as_ref(index_sign_value) };
    let key_type = unsafe { c2str(key_type) };
    let value_type = unsafe { c2str(value_type) };
    let index_key_name = unsafe { c2str(key_name) };
    let has_index_signature = !key_type.is_empty();

    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if ctx.cfg.disable_schema_check {
        return;
    }
    let config = schema_config.as_dict_ref();
    for (key, value) in &config.values {
        let no_such_attr = schema_value.dict_get_value(key).is_none();
        if has_index_signature && no_such_attr {
            // Allow index signature value has different values
            // related to the index signature key name.
            let should_update =
                if let Some(index_key_value) = schema_value.dict_get_value(index_key_name) {
                    index_key_value.is_str() && key == &index_key_value.as_str()
                } else {
                    true
                };
            if should_update {
                let op = config
                    .ops
                    .get(key)
                    .unwrap_or(&ConfigEntryOperationKind::Union);
                schema_value.dict_update_entry(
                    key.as_str(),
                    &index_sign_value.deep_copy(),
                    &ConfigEntryOperationKind::Override,
                    None,
                );
                schema_value.dict_insert(ctx, key.as_str(), value, op.clone(), None);
                let value = schema_value.dict_get_value(key).unwrap();
                schema_value.dict_update_key_value(
                    key.as_str(),
                    type_pack_and_check(ctx, &value, vec![value_type], false),
                );
            }
        } else if !has_index_signature && no_such_attr {
            let schema_name = unsafe { c2str(schema_name) };
            panic!("No attribute named '{key}' in the schema '{schema_name}'");
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_do_check_with_index_sign_attr(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
    check_fn_ptr: *const u64,
    attr_name: *const kcl_char_t,
) {
    let check_fn_ptr = check_fn_ptr as u64;
    let args_value = unsafe { ptr_as_ref(args) };
    let attr_name = unsafe { c2str(attr_name) };

    let check_fn: SchemaTypeFunc = unsafe { transmute_copy(&check_fn_ptr) };
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
            let args = args.clone().into_raw(unsafe { mut_ptr_as_ref(ctx) });
            unsafe { check_fn(ctx, args, kwargs) };
        }
    }
    schema.dict_remove(attr_name);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_optional_check(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
) {
    let p = unsafe { ptr_as_ref(p) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if !ctx.cfg.disable_schema_check {
        p.schema_check_attr_optional(ctx, true);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_default_settings(
    schema_value: *mut kcl_value_ref_t,
    _config_value: *const kcl_value_ref_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
    runtime_type: *const kcl_char_t,
) {
    let schema_value = unsafe { mut_ptr_as_ref(schema_value) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let runtime_type = unsafe { c2str(runtime_type) };
    schema_value.set_potential_schema_type(runtime_type);
    schema_value.set_schema_args(args, kwargs);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_assert(
    ctx: *mut kcl_context_t,
    value: *const kcl_value_ref_t,
    msg: *const kcl_value_ref_t,
    config_meta: *const kcl_value_ref_t,
) {
    let value = unsafe { ptr_as_ref(value) };
    let msg = unsafe { ptr_as_ref(msg) };
    let config_meta = unsafe { ptr_as_ref(config_meta) };
    if !value.is_truthy() {
        let ctx = unsafe { mut_ptr_as_ref(ctx) };
        ctx.set_err_type(&RuntimeErrorType::SchemaCheckFailure);
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

        let arg_msg = format!(
            "Check failed on the condition{}",
            if msg.is_empty() {
                "".to_string()
            } else {
                format!(": {msg}")
            }
        );
        ctx.set_kcl_location_info(Some(arg_msg.as_str()), None, None, None);

        panic!("{}", msg.as_str());
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_value_new(
    ctx: *mut kcl_context_t,
    args: *mut kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
    schema_value_or_func: *const kcl_value_ref_t,
    config: *const kcl_value_ref_t,
    config_meta: *const kcl_value_ref_t,
    pkgpath: *const kcl_char_t,
) -> *const kcl_value_ref_t {
    let schema_value_or_func = unsafe { ptr_as_ref(schema_value_or_func) };
    if schema_value_or_func.is_func() {
        let schema_func = schema_value_or_func.as_function();
        let schema_fn_ptr = schema_func.fn_ptr;
        let ctx_ref = unsafe { mut_ptr_as_ref(ctx) };
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
            let cal_map = kcl_value_Dict(ctx);
            let instance_pkgpath = kcl_value_Str(ctx, pkgpath);
            // Schema function closures
            let values = [
                // is_sub_schema
                kcl_value_Bool(ctx, 0),
                // Config meta
                config_meta,
                // Config value
                config,
                // Schema value
                kcl_value_Dict(ctx),
                // optional_mapping
                kcl_value_Dict(ctx),
                // cal order map
                cal_map,
                // backtrack level map
                kcl_value_Dict(ctx),
                // backtrack cache
                kcl_value_Dict(ctx),
                // record instance
                kcl_value_Bool(ctx, 0),
                // instance pkgpath
                instance_pkgpath,
            ];
            for value in values {
                kcl_list_append(args, value);
            }
            schema_fn(ctx, args, kwargs);
            // schema args
            let args = org_args.into_raw(ctx_ref);
            let values = [
                // is_sub_schema
                kcl_value_Bool(ctx, 1),
                // Config meta
                config_meta,
                // Config value
                config,
                // Schema value
                kcl_value_Dict(ctx),
                // optional_mapping
                kcl_value_Dict(ctx),
                // cal order map
                cal_map,
                // backtrack level map
                kcl_value_Dict(ctx),
                // backtrack cache
                kcl_value_Dict(ctx),
                // record instance
                kcl_value_Bool(ctx, 1),
                // instance pkgpath
                instance_pkgpath,
            ];
            for value in values {
                kcl_list_append(args, value);
            }
            schema_fn(ctx, args, kwargs)
        };
        ctx_ref.panic_info = now_meta_info;
        if ctx_ref.cfg.debug_mode {
            ctx_ref.backtrace.pop();
        }
        value
    } else {
        let config = unsafe { ptr_as_ref(config) };
        let result = schema_value_or_func.deep_copy().union_entry(
            unsafe { mut_ptr_as_ref(ctx) },
            config,
            true,
            &UnionOptions::default(),
        );
        result.into_raw(unsafe { mut_ptr_as_ref(ctx) })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_convert_collection_value(
    ctx: *mut kcl_context_t,
    value: *const kcl_value_ref_t,
    tpe: *const kcl_char_t,
    is_in_schema: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let value = unsafe { ptr_as_ref(value) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let tpe = unsafe { c2str(tpe) };
    let value = type_pack_and_check(ctx, value, vec![tpe], false);
    let is_in_schema = unsafe { ptr_as_ref(is_in_schema) };
    // Schema required attribute validating.
    if !is_in_schema.is_truthy() {
        walk_value_mut(&value, &mut |value: &ValueRef| {
            if value.is_schema() {
                value.schema_check_attr_optional(ctx, true);
            }
        })
    }
    value.into_raw(ctx)
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_schema_get_value(
    ctx: *mut kcl_context_t,
    p: *const kcl_value_ref_t,
    key: *const kcl_char_t,
    config: *const kcl_value_ref_t,
    config_meta: *const kcl_value_ref_t,
    cal_map: *const kcl_value_ref_t,
    target_attr: *const kcl_char_t,
    backtrack_level_map: *mut kcl_value_ref_t,
    backtrack_cache: *mut kcl_value_ref_t,
    args: *mut kcl_value_ref_t,
    kwargs: *mut kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let schema = unsafe { ptr_as_ref(p) };
    let key = unsafe { c2str(key) };
    let cal_map = unsafe { ptr_as_ref(cal_map) };
    let target_attr = unsafe { c2str(target_attr) };
    let backtrack_level_map = unsafe { mut_ptr_as_ref(backtrack_level_map) };
    let backtrack_cache = unsafe { mut_ptr_as_ref(backtrack_cache) };
    let args_org = unsafe { mut_ptr_as_ref(args) };
    let kwargs = unsafe { mut_ptr_as_ref(kwargs) };
    let default_level = ValueRef::int(0);
    let level = backtrack_level_map
        .dict_get_value(key)
        .unwrap_or(default_level);
    let level = level.as_int();
    let is_backtracking = level > 0;
    // Deal in-place modify and return it self immediately
    if key == target_attr && !is_backtracking {
        match schema.dict_get_value(key) {
            Some(x) => return x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
            None => return unsafe { kcl_value_Undefined(ctx) },
        }
    }
    if let Some(v) = backtrack_cache.dict_get_value(key) {
        return v.into_raw(unsafe { mut_ptr_as_ref(ctx) });
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
            return value.into_raw(unsafe { mut_ptr_as_ref(ctx) });
        }
        let fn_ptr = &attr_code.values[index];
        let fn_ptr = fn_ptr.as_int();
        // When we calculate other schema attribute values, we retain
        // the row and column number information of the current schema attribute.
        let ctx_ref = unsafe { mut_ptr_as_ref(ctx) };
        let panic_info = ctx_ref.panic_info.clone();
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
            let args = args.into_raw(ctx_ref);
            let kwargs = kwargs.clone().into_raw(ctx_ref);
            attr_fn(ctx, args, kwargs);
        };
        // Restore the panic info of current schema attribute.
        ctx_ref.panic_info = panic_info;
        backtrack_level_map.dict_update_key_value(key, ValueRef::int(level));
        let value = match schema.dict_get_value(key) {
            Some(x) => x,
            None => ValueRef::undefined(),
        };
        backtrack_cache.dict_update_key_value(key, value);
    }
    match schema.dict_get_value(key) {
        Some(x) => x.into_raw(unsafe { mut_ptr_as_ref(ctx) }),
        None => unsafe { kcl_value_Undefined(ctx) },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_config_attr_map(
    value: *mut kcl_value_ref_t,
    name: *const kcl_char_t,
    type_str: *const kcl_char_t,
) {
    let value = unsafe { mut_ptr_as_ref(value) };
    let name = unsafe { c2str(name) };
    let type_str = unsafe { c2str(type_str) };
    value.update_attr_map(name, type_str);
}

// ----------------------------------------------------------------------------
// values: decorators
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_value_Decorator(
    ctx: *mut kcl_context_t,
    name: *const kcl_char_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
    config_meta: *const kcl_value_ref_t,
    attr_name: *const kcl_char_t,
    config_value: *const kcl_value_ref_t,
    is_schema_target: *const kcl_value_ref_t,
) -> *const kcl_decorator_value_t {
    let name = unsafe { c2str(name) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let config_meta = unsafe { ptr_as_ref(config_meta) };
    let attr_name = unsafe { c2str(attr_name) };
    let config_value = unsafe { ptr_as_ref(config_value) };
    let is_schema_target = unsafe { ptr_as_ref(is_schema_target) };
    let decorator = DecoratorValue::new(name, args, kwargs);
    decorator.run(
        unsafe { mut_ptr_as_ref(ctx) },
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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_lower(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_lower().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_lower");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_upper(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_upper().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_upper");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_capitalize(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_capitalize()
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_capitalize");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_chars(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_chars().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_chars");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_count(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_count(&sub, start.as_ref(), end.as_ref())
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            panic!("count() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_count");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_endswith(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(suffix) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_endswith(&suffix, start.as_ref(), end.as_ref())
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            panic!("endswith() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_endswith");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_find(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_find(&sub, start.as_ref(), end.as_ref())
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            panic!("find() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_find");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_format(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(val) = args.pop_arg_first() {
        val.str_format(args, kwargs)
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_format");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_index(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_index(&sub, start.as_ref(), end.as_ref())
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            panic!("index() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_index");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_isalnum(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_isalnum().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_isalnum");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_isalpha(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_isalpha().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_isalpha");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_isdigit(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_isdigit().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_isdigit");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_islower(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_islower().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_islower");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_isspace(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_isspace().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_isspace");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_istitle(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_istitle().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_istitle");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_isupper(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_isupper().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_isupper");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_join(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        let iter = args.arg_i(0).unwrap();
        val.str_join(&iter).into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_join");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_lstrip(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        let chars = args.arg_i(0);
        val.str_lstrip(chars.as_ref())
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_lstrip");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_rstrip(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        let chars = args.arg_i(0);
        val.str_rstrip(chars.as_ref())
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_rstrip");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_replace(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        let old = args.arg_i(0).expect("expect 1 argument, found 0");
        let new = args.arg_i(1).expect("expect 2 arguments, found 1");
        let count = args.arg_i(2);
        val.str_replace(&old, &new, count.as_ref())
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_replace");
    }
}

/// If the string starts with the prefix string, return string[len(prefix):].
/// Otherwise, return a copy of the original string.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_removeprefix(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        let prefix = args.arg_i(0).expect("expect 1 argument, found 0");
        val.str_removeprefix(&prefix)
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_removeprefix");
    }
}

/// If the string ends with the suffix string and that suffix is not empty, return string[:-len(suffix)].
/// Otherwise, return a copy of the original string.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_removesuffix(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        let suffix = args.arg_i(0).expect("expect 1 argument, found 0");
        val.str_removesuffix(&suffix)
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_removesuffix");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_rfind(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_rfind(&sub, start.as_ref(), end.as_ref())
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            panic!("rfind() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_rfind");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_rindex(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(sub) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_rindex(&sub, start.as_ref(), end.as_ref())
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            panic!("rindex() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_rindex");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_rsplit(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
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
        val.str_rsplit(sep.as_ref(), maxsplit.as_ref())
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_rsplit");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_split(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
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
        x.into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_split");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_splitlines(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(keepends) = args.arg_i(0) {
            val.str_splitlines(Some(&keepends))
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else if let Some(keepends) = kwargs.kwarg("keepends") {
            val.str_splitlines(Some(&keepends))
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            val.str_splitlines(None)
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        }
    } else {
        panic!("invalid self value in str_splitlines");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_startswith(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        if let Some(suffix) = args.arg_0() {
            let start = args.arg_i(1);
            let end = args.arg_i(2);
            val.str_startswith(&suffix, start.as_ref(), end.as_ref())
                .into_raw(unsafe { mut_ptr_as_ref(ctx) })
        } else {
            panic!("startswith() takes at least 1 argument (0 given)");
        }
    } else {
        panic!("invalid self value in str_startswith");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_strip(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        let chars = args.arg_i(0);
        val.str_strip(chars.as_ref())
            .into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_strip");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_builtin_str_title(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    if let Some(val) = args.pop_arg_first() {
        val.str_title().into_raw(unsafe { mut_ptr_as_ref(ctx) })
    } else {
        panic!("invalid self value in str_title");
    }
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
