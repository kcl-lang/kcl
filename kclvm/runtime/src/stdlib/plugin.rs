// Copyright 2021 The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use crate::*;

use std::os::raw::c_char;

#[allow(non_upper_case_globals)]
static mut _plugin_handler_fn_ptr: u64 = 0;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_plugin_init(
    fn_ptr: extern "C" fn(
        method: *const i8,
        args_json: *const c_char,
        kwargs_json: *const c_char,
    ) -> *const c_char,
) {
    unsafe {
        _plugin_handler_fn_ptr = fn_ptr as usize as u64;
    }
}

// import kcl_plugin.hello
// hello.SayHello()
//
// => return kclvm_plugin_invoke("kcl_plugin.hello.SayHello", args, kwarge)

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_plugin_invoke(
    method: *const i8,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args_s = kclvm_value_to_json_value_with_null(args);
    let kwargs_s = kclvm_value_to_json_value_with_null(kwargs);

    let args_json = kclvm_value_Str_ptr(args_s);
    let kwargs_json = kclvm_value_Str_ptr(kwargs_s);

    let result_json = kclvm_plugin_invoke_json(method, args_json, kwargs_json);

    // Value delete by context.
    // kclvm_value_delete(args_s);
    // kclvm_value_delete(kwargs_s);

    let ptr = kclvm_value_from_json(result_json);
    {
        if let Some(msg) = ptr_as_ref(ptr).dict_get_value("__kcl_PanicInfo__") {
            let ctx = Context::current_context_mut();
            ctx.set_err_type(&ErrType::EvaluationError_TYPE);

            panic!("{}", msg.as_str());
        }
    }

    ptr
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_plugin_invoke_json(
    method: *const i8,
    args: *const c_char,
    kwargs: *const c_char,
) -> *const c_char {
    unsafe {
        if _plugin_handler_fn_ptr == 0 {
            panic!("plugin is nil, should call kclvm_plugin_init at first");
        }

        let ptr = (&_plugin_handler_fn_ptr as *const u64) as *const ()
            as *const extern "C" fn(
                method: *const i8,
                args: *const c_char,
                kwargs: *const c_char,
            ) -> *const c_char;

        (*ptr)(method, args, kwargs)
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_plugin_invoke_json(
    method: *const i8,
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
        method: *const i8,
        args: *const c_char,
        kwargs: *const c_char,
    ) -> *const c_char;
}
