//! Copyright The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use crate::*;
use std::os::raw::c_char;

use self::eval::LazyEvalScope;

#[allow(dead_code, non_camel_case_types)]
type kclvm_context_t = Context;

#[allow(dead_code, non_camel_case_types)]
type kclvm_eval_scope_t = LazyEvalScope;

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
pub unsafe extern "C" fn kclvm_context_set_kcl_workdir(
    p: *mut kclvm_context_t,
    workdir: *const c_char,
) {
    let p = mut_ptr_as_ref(p);
    if !workdir.is_null() {
        p.set_kcl_workdir(c2str(workdir));
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
// Global values and evaluation scope.
// ----------------------------------------------------------------------------

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_scope_new() -> *mut kclvm_eval_scope_t {
    Box::into_raw(Box::default())
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_scope_free(scope: *mut kclvm_eval_scope_t) {
    drop(Box::from_raw(scope));
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_scope_add_setter(
    _ctx: *mut kclvm_context_t,
    scope: *mut kclvm_eval_scope_t,
    pkg: *const c_char,
    name: *const c_char,
    setter: *const u64,
) {
    let scope = mut_ptr_as_ref(scope);
    let pkg = c2str(pkg);
    let name = c2str(name);
    let key = format!("{}.{}", pkg, name);
    if !scope.setters.contains_key(&key) {
        scope.setters.insert(key.clone(), vec![]);
    }
    if let Some(setters) = scope.setters.get_mut(&key) {
        setters.push(setter as u64);
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_scope_set(
    _ctx: *mut kclvm_context_t,
    scope: *mut kclvm_eval_scope_t,
    pkg: *const c_char,
    name: *const c_char,
    value: *const kclvm_value_ref_t,
) {
    let scope = mut_ptr_as_ref(scope);
    let value = ptr_as_ref(value);
    let pkg = c2str(pkg);
    let name = c2str(name);
    let key = format!("{}.{}", pkg, name);
    scope.set_value(&key, value);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_scope_get(
    ctx: *mut kclvm_context_t,
    scope: *mut kclvm_eval_scope_t,
    pkg: *const c_char,
    name: *const c_char,
    target: *const c_char,
    default: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let scope = mut_ptr_as_ref(scope);
    let pkg = c2str(pkg);
    let name = c2str(name);
    let target = format!("{}.{}", pkg, c2str(target));
    let key = format!("{}.{}", pkg, name);
    // Existing values or existing but not yet calculated values.
    if scope.contains_key(&key) || scope.setters.contains_key(&key) {
        scope.get_value(ctx, &key, &target).into_raw(ctx)
    } else {
        default
    }
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
    p.plan_opts.disable_none = v != 0;
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
