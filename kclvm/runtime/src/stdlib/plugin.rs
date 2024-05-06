//! Copyright The KCL Authors. All rights reserved.

#![allow(clippy::missing_safety_doc)]

use crate::*;

use lazy_static::lazy_static;
use std::os::raw::c_char;
use std::sync::Mutex;

lazy_static! {
    static ref PLUGIN_HANDLER_FN_PTR: Mutex<
        Option<
            extern "C" fn(
                method: *const c_char,
                args_json: *const c_char,
                kwargs_json: *const c_char,
            ) -> *const c_char,
        >,
    > = Mutex::new(None);
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_plugin_init(
    fn_ptr: extern "C" fn(
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
// => return kclvm_plugin_invoke("kcl_plugin.hello.say_hello", args, kwargs)

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_plugin_invoke(
    ctx: *mut kclvm_context_t,
    method: *const c_char,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args_s = kclvm_value_to_json_value_with_null(ctx, args);
    let kwargs_s = kclvm_value_to_json_value_with_null(ctx, kwargs);

    let args_json = kclvm_value_Str_ptr(args_s);
    let kwargs_json = kclvm_value_Str_ptr(kwargs_s);

    let result_json = kclvm_plugin_invoke_json(method, args_json, kwargs_json);

    // Value delete by context.
    // kclvm_value_delete(args_s);
    // kclvm_value_delete(kwargs_s);

    let ptr = kclvm_value_from_json(ctx, result_json);
    {
        if let Some(msg) = ptr_as_ref(ptr).dict_get_value("__kcl_PanicInfo__") {
            let ctx = mut_ptr_as_ref(ctx);
            ctx.set_err_type(&RuntimeErrorType::EvaluationError);

            panic!("{}", msg.as_str());
        }
    }

    ptr
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_plugin_invoke_json(
    method: *const c_char,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    let fn_ptr_guard = PLUGIN_HANDLER_FN_PTR.lock().unwrap();
    if let Some(fn_ptr) = *fn_ptr_guard {
        fn_ptr(method, args, kwargs)
    } else {
        panic!("plugin handler is nil, should call kclvm_plugin_init at first");
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_plugin_invoke_json(
    method: *const c_char,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    unsafe {
        return kclvm_plugin_invoke_json_wasm(method, args, kwargs);
    }
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn kclvm_plugin_invoke_json_wasm(
        method: *const c_char,
        args: *const c_char,
        kwargs: *const c_char,
    ) -> *const c_char;
}
