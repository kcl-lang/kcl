//! Copyright The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use crate::*;
use std::os::raw::c_char;

use self::eval::LazyEvalScope;

#[allow(dead_code, non_camel_case_types)]
type kcl_context_t = Context;

#[allow(dead_code, non_camel_case_types)]
type kcl_eval_scope_t = LazyEvalScope;

#[allow(dead_code, non_camel_case_types)]
type kcl_kind_t = Kind;

#[allow(dead_code, non_camel_case_types)]
type kcl_type_t = Type;

#[allow(dead_code, non_camel_case_types)]
type kcl_value_t = Value;

#[allow(dead_code, non_camel_case_types)]
type kcl_char_t = c_char;

#[allow(dead_code, non_camel_case_types)]
type kcl_size_t = i32;

#[allow(dead_code, non_camel_case_types)]
type kcl_bool_t = i8;

#[allow(dead_code, non_camel_case_types)]
type kcl_int_t = i64;

#[allow(dead_code, non_camel_case_types)]
type kcl_float_t = f64;

// ----------------------------------------------------------------------------
// new/delete
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_new() -> *mut kcl_context_t {
    Box::into_raw(Box::new(Context::new()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_delete(p: *mut kcl_context_t) {
    let ctx = mut_ptr_as_ref(p);
    for o in &ctx.objects {
        let ptr = (*o) as *mut kcl_value_ref_t;
        unsafe { kcl_value_delete(ptr) };
    }
    free_mut_ptr(p);
}

// ----------------------------------------------------------------------------
// panic_info
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_kcl_location(
    p: *mut kcl_context_t,
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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_kcl_pkgpath(
    p: *mut kcl_context_t,
    pkgpath: *const c_char,
) {
    let p = mut_ptr_as_ref(p);
    if !pkgpath.is_null() {
        p.set_kcl_pkgpath(c2str(pkgpath));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_kcl_modpath(
    p: *mut kcl_context_t,
    module_path: *const c_char,
) {
    let p = mut_ptr_as_ref(p);
    if !module_path.is_null() {
        p.set_kcl_module_path(c2str(module_path));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_kcl_workdir(
    p: *mut kcl_context_t,
    workdir: *const c_char,
) {
    let p = mut_ptr_as_ref(p);
    if !workdir.is_null() {
        p.set_kcl_workdir(c2str(workdir));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_kcl_filename(
    ctx: *mut kcl_context_t,
    filename: *const c_char,
) {
    let ctx = mut_ptr_as_ref(ctx);
    if !filename.is_null() {
        ctx.set_kcl_filename(c2str(filename));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_kcl_line_col(
    ctx: *mut kcl_context_t,
    line: i32,
    col: i32,
) {
    let ctx = mut_ptr_as_ref(ctx);
    ctx.set_kcl_line_col(line, col);
}

// ----------------------------------------------------------------------------
// Global values and evaluation scope.
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_scope_new() -> *mut kcl_eval_scope_t {
    Box::into_raw(Box::default())
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_scope_delete(scope: *mut kcl_eval_scope_t) {
    drop(unsafe { Box::from_raw(scope) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_scope_add_setter(
    _ctx: *mut kcl_context_t,
    scope: *mut kcl_eval_scope_t,
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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_scope_set(
    _ctx: *mut kcl_context_t,
    scope: *mut kcl_eval_scope_t,
    pkg: *const c_char,
    name: *const c_char,
    value: *const kcl_value_ref_t,
) {
    let scope = mut_ptr_as_ref(scope);
    let value = ptr_as_ref(value);
    let pkg = c2str(pkg);
    let name = c2str(name);
    let key = format!("{}.{}", pkg, name);
    scope.set_value(&key, value);
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_scope_get(
    ctx: *mut kcl_context_t,
    scope: *mut kcl_eval_scope_t,
    pkg: *const c_char,
    name: *const c_char,
    target: *const c_char,
    default: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
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

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_debug_mode(p: *mut kcl_context_t, v: kcl_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.cfg.debug_mode = v != 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_strict_range_check(
    p: *mut kcl_context_t,
    v: kcl_bool_t,
) {
    let p = mut_ptr_as_ref(p);
    p.cfg.strict_range_check = v != 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_disable_none(p: *mut kcl_context_t, v: kcl_bool_t) {
    let p = mut_ptr_as_ref(p);
    p.plan_opts.disable_none = v != 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_set_disable_schema_check(
    p: *mut kcl_context_t,
    v: kcl_bool_t,
) {
    let p = mut_ptr_as_ref(p);
    p.cfg.disable_schema_check = v != 0;
}

// ----------------------------------------------------------------------------
// invoke
// ----------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_invoke(
    p: *mut kcl_context_t,
    method: *const c_char,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    let p = mut_ptr_as_ref(p);
    let method = c2str(method);

    let args = unsafe { kcl_value_from_json(p, args) };
    let kwargs = unsafe { kcl_value_from_json(p, kwargs) };
    let result = unsafe { kcl_context_invoke_inner(p, method, args, kwargs) };

    p.buffer.kcl_context_invoke_result = ptr_as_ref(result).to_json_string_with_null();
    let result_json = p.buffer.kcl_context_invoke_result.as_ptr() as *const c_char;

    unsafe { kcl_value_delete(args) };
    unsafe { kcl_value_delete(kwargs) };
    unsafe { kcl_value_delete(result) };

    result_json
}

unsafe fn kcl_context_invoke_inner(
    ctx: *mut kcl_context_t,
    method: &str,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);

    let fn_addr = kcl_get_fn_ptr_by_name(method);
    if fn_addr == 0 {
        panic!("null fn ptr");
    }

    let ptr = (&fn_addr as *const u64) as *const ()
        as *const extern "C-unwind" fn(
            ctx: *mut kcl_context_t,
            args: *const kcl_value_ref_t,
            kwargs: *const kcl_value_ref_t,
        ) -> *mut kcl_value_ref_t;

    unsafe { (*ptr)(ctx, args, kwargs) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_context_pkgpath_is_imported(
    ctx: *mut kcl_context_t,
    pkgpath: *const kcl_char_t,
) -> kcl_bool_t {
    let pkgpath = c2str(pkgpath);
    let ctx = mut_ptr_as_ref(ctx);
    let result = ctx.imported_pkgpath.contains(pkgpath);
    ctx.imported_pkgpath.insert(pkgpath.to_string());
    result as kcl_bool_t
}

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
