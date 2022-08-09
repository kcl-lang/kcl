// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;
use std::os::raw::c_char;

#[allow(dead_code, non_camel_case_types)]
type kclvm_context_t = Context;

#[allow(dead_code, non_camel_case_types)]
type kclvm_error_t = KclError;

#[allow(dead_code, non_camel_case_types)]
type kclvm_kind_t = Kind;

#[allow(dead_code, non_camel_case_types)]
type kclvm_type_t = Type;

#[allow(dead_code, non_camel_case_types)]
type kclvm_value_t = Value;

#[allow(dead_code, non_camel_case_types)]
type kclvm_char_t = i8;

#[allow(dead_code, non_camel_case_types)]
type kclvm_size_t = i32;

#[allow(dead_code, non_camel_case_types)]
type kclvm_bool_t = i8;

#[allow(dead_code, non_camel_case_types)]
type kclvm_int_t = i64;

#[allow(dead_code, non_camel_case_types)]
type kclvm_float_t = f64;

// ----------------------------------------------------------------------------
// new/delete
// ----------------------------------------------------------------------------

// singleton

#[allow(non_camel_case_types, non_upper_case_globals)]
static mut _kclvm_context_current: u64 = 0;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_current() -> *mut kclvm_context_t {
    unsafe {
        if _kclvm_context_current == 0 {
            _kclvm_context_current = kclvm_context_new() as u64;
        }
        _kclvm_context_current as *mut kclvm_context_t
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_new() -> *mut kclvm_context_t {
    let p = Box::into_raw(Box::new(Context::new()));
    unsafe {
        _kclvm_context_current = p as u64;
    }
    p
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_delete(p: *mut kclvm_context_t) {
    let ctx = mut_ptr_as_ref(p);
    for o in &ctx.objects {
        let ptr = (*o) as *mut kclvm_value_ref_t;
        kclvm_value_delete(ptr);
    }
    free_mut_ptr(p);
}

// ----------------------------------------------------------------------------
// main begin/end
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_main_begin_hook(p: *mut kclvm_context_t) {
    let p = mut_ptr_as_ref(p);
    p.main_begin_hook();
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_main_end_hook(
    p: *mut kclvm_context_t,
    return_value: *mut kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let p = mut_ptr_as_ref(p);
    p.main_end_hook(return_value)
}

// ----------------------------------------------------------------------------
// panic_info
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_kcl_location(
    p: *mut kclvm_context_t,
    filename: *const i8,
    line: i32,
    col: i32,
) {
    let p = mut_ptr_as_ref(p);
    if !filename.is_null() {
        p.set_kcl_location_info(None, Some(c2str(filename)), Some(line), Some(col));
    } else {
        p.set_kcl_location_info(None, None, Some(line), Some(col));
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_kcl_pkgpath(p: *mut kclvm_context_t, pkgpath: *const i8) {
    let p = mut_ptr_as_ref(p);
    if !pkgpath.is_null() {
        p.set_kcl_pkgpath(c2str(pkgpath));
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_kcl_filename(filename: *const i8) {
    let p = Context::current_context_mut();
    if !filename.is_null() {
        p.set_kcl_filename(c2str(filename));
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_kcl_line_col(line: i32, col: i32) {
    let p = Context::current_context_mut();
    p.set_kcl_line_col(line, col);
}

// ----------------------------------------------------------------------------
// manage types
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_put_type(p: *mut kclvm_context_t, typ: *const kclvm_type_t) {
    let p = mut_ptr_as_ref(p);
    let typ = ptr_as_ref(typ);

    p.all_types.push(typ.clone());
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_clear_all_types(p: *mut kclvm_context_t) {
    let p = mut_ptr_as_ref(p);
    p.all_types.clear();
}

// ----------------------------------------------------------------------------
// symbol
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_symbol_init(
    p: *mut kclvm_context_t,
    n: kclvm_size_t,
    symbol_names: *const *const kclvm_char_t,
) {
    let p = mut_ptr_as_ref(p);

    unsafe {
        p.symbol_names.clear();
        p.symbol_values.clear();

        let _ = std::slice::from_raw_parts(symbol_names, n as usize)
            .iter()
            .map(|arg| {
                p.symbol_names.push(c2str(*arg).to_string());
                p.symbol_values.push(Value::default());
            });
    }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_symbol_num(p: *const kclvm_context_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);

    p.symbol_names.len() as kclvm_size_t
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_symbol_name(
    p: *const kclvm_context_t,
    i: kclvm_size_t,
) -> *const kclvm_char_t {
    let p = ptr_as_ref(p);
    return match p.symbol_names.get(i as usize) {
        Some(value) => value.as_bytes().as_ptr() as *const kclvm_char_t,
        None => std::ptr::null(),
    };
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_symbol_value(
    p: *const kclvm_context_t,
    i: kclvm_size_t,
) -> *const kclvm_value_t {
    let p = ptr_as_ref(p);
    match p.symbol_values.get(i as usize) {
        Some(v) => v as *const kclvm_value_t,
        None => std::ptr::null(),
    }
}

// ----------------------------------------------------------------------------
// args
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_args_get(
    _p: *const kclvm_context_t,
    _key: *const kclvm_char_t,
) -> *const kclvm_char_t {
    //let p = ptr_as_ref(p);
    //match p.app_args.get(c2str(key)) {
    //    Some(value) => (*value).as_bytes().as_ptr() as *const kclvm_char_t,
    //    None => std::ptr::null(),
    //};
    std::ptr::null()
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_args_set(
    _p: *mut kclvm_context_t,
    _key: *const kclvm_char_t,
    _value: *const kclvm_char_t,
) {
    //let p = mut_ptr_as_ref(p);
    //p.app_args
    //    .insert(c2str(key).to_string(), c2str(value).to_string());
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_args_clear(p: *mut kclvm_context_t) {
    let p = mut_ptr_as_ref(p);
    p.app_args.clear();
}

// ----------------------------------------------------------------------------
// CLI config
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_debug_mode(p: *mut kclvm_context_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.debug_mode = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_strict_range_check(p: *mut kclvm_context_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.strict_range_check = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_disable_none(p: *mut kclvm_context_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.disable_none = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_disable_schema_check(p: *mut kclvm_context_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.disable_schema_check = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_set_list_option_mode(p: *mut kclvm_context_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.list_option_mode = v != 0;
}

// ----------------------------------------------------------------------------
// invoke
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_invoke(
    p: *mut kclvm_context_t,
    method: *const c_char,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    let p = mut_ptr_as_ref(p);
    let method = c2str(method);

    let args = kclvm_value_from_json(args);
    let kwargs = kclvm_value_from_json(kwargs);
    let result = _kclvm_context_invoke(p, method, args, kwargs);

    p.buffer.kclvm_context_invoke_result = ptr_as_ref(result).to_json_string_with_null();
    let result_json = p.buffer.kclvm_context_invoke_result.as_ptr() as *const i8;

    kclvm_value_delete(args);
    kclvm_value_delete(kwargs);
    kclvm_value_delete(result);

    result_json
}

fn _kclvm_context_invoke(
    ctx: *mut kclvm_context_t,
    method: &str,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);

    let fn_addr = _kclvm_get_fn_ptr_by_name(method);
    if fn_addr == 0 {
        panic!("null fn ptr");
    }

    let ptr = (&fn_addr as *const u64) as *const ()
        as *const extern "C" fn(
            ctx: *mut kclvm_context_t,
            args: *const kclvm_value_ref_t,
            kwargs: *const kclvm_value_ref_t,
        ) -> *mut kclvm_value_ref_t;

    unsafe { (*ptr)(ctx, args, kwargs) }
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_context_pkgpath_is_imported(pkgpath: *const kclvm_char_t) -> kclvm_bool_t {
    let pkgpath = c2str(pkgpath);
    let ctx = Context::current_context_mut();
    let result = ctx.imported_pkgpath.contains(pkgpath);
    ctx.imported_pkgpath.insert(pkgpath.to_string());
    result as kclvm_bool_t
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
