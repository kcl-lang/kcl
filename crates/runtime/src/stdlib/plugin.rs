//! Copyright The KCL Authors. All rights reserved.

#![allow(clippy::missing_safety_doc)]

use crate::*;

use lazy_static::lazy_static;
use std::os::raw::c_char;
use std::sync::Mutex;

lazy_static! {
    static ref PLUGIN_HANDLER_FN_PTR: Mutex<
        Option<
            extern "C-unwind" fn(
                method: *const c_char,
                args_json: *const c_char,
                kwargs_json: *const c_char,
            ) -> *const c_char,
        >,
    > = Mutex::new(None);
}

/// KCL plugin module prefix
pub const PLUGIN_MODULE_PREFIX: &str = "kcl_plugin.";

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_plugin_init(
    fn_ptr: extern "C-unwind" fn(
        method: *const c_char,
        args_json: *const c_char,
        kwargs_json: *const c_char,
    ) -> *const c_char,
) {
    let mut fn_ptr_guard = PLUGIN_HANDLER_FN_PTR.lock().unwrap();
    *fn_ptr_guard = Some(fn_ptr);
}

// import kcl_plugin.hello
// hello.say_hello()
//
// => return kcl_plugin_invoke("kcl_plugin.hello.say_hello", args, kwargs)

#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_plugin_invoke(
    ctx: *mut kcl_context_t,
    method: *const c_char,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx_ref = unsafe { mut_ptr_as_ref(ctx) };
    let method_ref = unsafe { c2str(method) };
    let plugin_short_method = match method_ref.strip_prefix(PLUGIN_MODULE_PREFIX) {
        Some(s) => s,
        None => method_ref,
    };
    if let Some(func) = ctx_ref.plugin_functions.get(plugin_short_method) {
        let args = unsafe { ptr_as_ref(args) };
        let kwargs = unsafe { ptr_as_ref(kwargs) };
        let result = func(ctx_ref, args, kwargs);
        return result.unwrap().into_raw(ctx_ref);
    }
    let args_s = unsafe { kcl_value_to_json_value_with_null(ctx, args) };
    let kwargs_s = unsafe { kcl_value_to_json_value_with_null(ctx, kwargs) };

    let args_json = unsafe { kcl_value_Str_ptr(args_s) };
    let kwargs_json = unsafe { kcl_value_Str_ptr(kwargs_s) };

    let result_json = unsafe { kcl_plugin_invoke_json(method, args_json, kwargs_json) };

    // Value delete by context.
    // kcl_value_delete(args_s);
    // kcl_value_delete(kwargs_s);

    let ptr = unsafe { kcl_value_from_json(ctx, result_json) };
    {
        if let Some(msg) = unsafe { ptr_as_ref(ptr).dict_get_value("__kcl_PanicInfo__") } {
            let ctx = unsafe { mut_ptr_as_ref(ctx) };
            ctx.set_err_type(&RuntimeErrorType::EvaluationError);

            panic!("{}", msg.as_str());
        }
    }

    ptr
}

#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_plugin_invoke_json(
    method: *const c_char,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    let fn_ptr_guard = PLUGIN_HANDLER_FN_PTR.lock().unwrap();
    if let Some(fn_ptr) = *fn_ptr_guard {
        fn_ptr(method, args, kwargs)
    } else {
        panic!("plugin handler is nil, should call kcl_plugin_init at first");
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_plugin_invoke_json(
    method: *const c_char,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    unsafe {
        return kcl_plugin_invoke_json_wasm(method, args, kwargs);
    }
}

// `wasm32-wasip1` expects a wasmtime host (or any WASI host) to
// link `kcl_plugin_invoke_json_wasm` at instantiation — that's how
// akua's render worker ferries plugin callouts back to host
// handlers. On `wasm32-unknown-unknown` (e.g. a JS-loaded SDK
// bundle) there is no host, so the extern would leave an
// unresolved `env.*` import that every JS loader stumbles over.
// Provide a self-contained no-op stub on that target; callers that
// need plugins stick with `wasm32-wasip1` + a wasmtime host.
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
unsafe extern "C-unwind" {
    pub fn kcl_plugin_invoke_json_wasm(
        method: *const c_char,
        args: *const c_char,
        kwargs: *const c_char,
    ) -> *const c_char;
}

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
pub unsafe fn kcl_plugin_invoke_json_wasm(
    _method: *const c_char,
    _args: *const c_char,
    _kwargs: *const c_char,
) -> *const c_char {
    // `__kcl_PanicInfo__` shape — KCL's plugin-invoke glue treats
    // this response as a runtime panic, so Packages that call a
    // plugin surface a clean evaluator error with the message
    // below rather than a silent misevaluation.
    c"{\"__kcl_PanicInfo__\":\"plugin callouts are not available in this KCL build (wasm32-unknown-unknown) — use a wasmtime-hosted build for helm.template / kustomize.build / pkg.render\"}"
        .as_ptr() as *const c_char
}
