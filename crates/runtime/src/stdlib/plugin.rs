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
    // Release the dispatch lock before invoking the handler. The fn pointer is
    // `Copy`, so copy it out and drop the guard first: holding the lock across
    // the user callback deadlocks any nested KCL evaluation the handler triggers
    // (e.g. a plugin that renders another Package), because `std::sync::Mutex`
    // is not reentrant.
    let fn_ptr = { *PLUGIN_HANDLER_FN_PTR.lock().unwrap() };
    if let Some(fn_ptr) = fn_ptr {
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

#[cfg(target_arch = "wasm32")]
unsafe extern "C-unwind" {
    pub fn kcl_plugin_invoke_json_wasm(
        method: *const c_char,
        args: *const c_char,
        kwargs: *const c_char,
    ) -> *const c_char;
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::ptr;
    use std::sync::atomic::{AtomicBool, Ordering};

    static LOCK_FREE_DURING_HANDLER: AtomicBool = AtomicBool::new(false);

    /// Records whether the dispatch lock is available while the handler runs.
    /// With the lock released before dispatch (the fix), `try_lock` succeeds;
    /// if the lock were still held (the pre-fix bug) it would fail here — and a
    /// genuinely reentrant handler would deadlock instead of returning.
    extern "C-unwind" fn probe_handler(
        _method: *const c_char,
        _args: *const c_char,
        _kwargs: *const c_char,
    ) -> *const c_char {
        LOCK_FREE_DURING_HANDLER.store(PLUGIN_HANDLER_FN_PTR.try_lock().is_ok(), Ordering::SeqCst);
        ptr::null()
    }

    #[test]
    fn dispatch_lock_released_before_handler() {
        unsafe {
            kcl_plugin_init(probe_handler);
            kcl_plugin_invoke_json(ptr::null(), ptr::null(), ptr::null());
        }
        assert!(
            LOCK_FREE_DURING_HANDLER.load(Ordering::SeqCst),
            "dispatch lock must be released before invoking the plugin handler; \
             holding it across the callback deadlocks reentrant/nested KCL evaluation"
        );
    }
}
