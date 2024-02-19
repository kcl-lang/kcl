//! Copyright The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

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
type kclvm_char_t = c_char;

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

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_new() -> *mut kclvm_context_t {
    Box::into_raw(Box::new(Context::new()))
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_delete(p: *mut kclvm_context_t) {
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
pub unsafe extern "C" fn kclvm_context_main_begin_hook(p: *mut kclvm_context_t) {
    let p = mut_ptr_as_ref(p);
    p.main_begin_hook();
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_main_end_hook(
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
pub unsafe extern "C" fn kclvm_context_set_kcl_location(
    p: *mut kclvm_context_t,
    filename: *const c_char,
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
pub unsafe extern "C" fn kclvm_context_set_kcl_pkgpath(
    p: *mut kclvm_context_t,
    pkgpath: *const c_char,
) {
    let p = mut_ptr_as_ref(p);
    if !pkgpath.is_null() {
        p.set_kcl_pkgpath(c2str(pkgpath));
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_kcl_modpath(
    p: *mut kclvm_context_t,
    module_path: *const c_char,
) {
    let p = mut_ptr_as_ref(p);
    if !module_path.is_null() {
        p.set_kcl_module_path(c2str(module_path));
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_kcl_filename(
    ctx: *mut kclvm_context_t,
    filename: *const c_char,
) {
    let ctx = mut_ptr_as_ref(ctx);
    if !filename.is_null() {
        ctx.set_kcl_filename(c2str(filename));
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_kcl_line_col(
    ctx: *mut kclvm_context_t,
    line: i32,
    col: i32,
) {
    let ctx = mut_ptr_as_ref(ctx);
    ctx.set_kcl_line_col(line, col);
}

// ----------------------------------------------------------------------------
// CLI config
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_debug_mode(p: *mut kclvm_context_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.debug_mode = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_strict_range_check(
    p: *mut kclvm_context_t,
    v: kclvm_bool_t,
) {
    let p = mut_ptr_as_ref(p);
    p.cfg.strict_range_check = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_disable_none(p: *mut kclvm_context_t, v: kclvm_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.disable_none = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_disable_schema_check(
    p: *mut kclvm_context_t,
    v: kclvm_bool_t,
) {
    let p = mut_ptr_as_ref(p);
    p.cfg.disable_schema_check = v != 0;
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_set_list_option_mode(
    p: *mut kclvm_context_t,
    v: kclvm_bool_t,
) {
    let p = mut_ptr_as_ref(p);
    p.cfg.list_option_mode = v != 0;
}

// ----------------------------------------------------------------------------
// invoke
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_context_invoke(
    p: *mut kclvm_context_t,
    method: *const c_char,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    let p = mut_ptr_as_ref(p);
    let method = c2str(method);

    let args = kclvm_value_from_json(p, args);
    let kwargs = kclvm_value_from_json(p, kwargs);
    let result = _kclvm_context_invoke(p, method, args, kwargs);

    p.buffer.kclvm_context_invoke_result = ptr_as_ref(result).to_json_string_with_null();
    let result_json = p.buffer.kclvm_context_invoke_result.as_ptr() as *const c_char;

    kclvm_value_delete(args);
    kclvm_value_delete(kwargs);
    kclvm_value_delete(result);

    result_json
}

unsafe fn _kclvm_context_invoke(
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
pub unsafe extern "C" fn kclvm_context_pkgpath_is_imported(
    ctx: *mut kclvm_context_t,
    pkgpath: *const kclvm_char_t,
) -> kclvm_bool_t {
    let pkgpath = c2str(pkgpath);
    let ctx = mut_ptr_as_ref(ctx);
    let result = ctx.imported_pkgpath.contains(pkgpath);
    ctx.imported_pkgpath.insert(pkgpath.to_string());
    result as kclvm_bool_t
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
