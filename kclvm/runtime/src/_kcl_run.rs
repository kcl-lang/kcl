// Copyright 2021 The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use crate::*;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_buffer_t = Buffer;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_context_t = Context;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_kind_t = Kind;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_type_t = Type;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_value_ref_t = ValueRef;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_iterator_t = ValueIterator;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_char_t = i8;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_size_t = i32;

#[allow(dead_code, non_camel_case_types)]
type kclvm_bool_t = i8;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_int_t = i64;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_float_t = f64;

// const SHOULD_PROFILE: bool = false;

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn _kcl_run(
    kclvm_main_ptr: u64, // main.k => kclvm_main
    option_len: kclvm_size_t,
    option_keys: *const *const kclvm_char_t,
    option_values: *const *const kclvm_char_t,
    strict_range_check: i32,
    disable_none: i32,
    disable_schema_check: i32,
    list_option_mode: i32,
    debug_mode: i32,
    result_buffer_len: kclvm_size_t,
    result_buffer: *mut kclvm_char_t,
    warn_buffer_len: kclvm_size_t,
    warn_buffer: *mut kclvm_char_t,
) -> kclvm_size_t {
    let ctx = kclvm_context_new();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
        let ctx = Context::current_context_mut();
        ctx.set_panic_info(info);
        let _ = ctx;
    }));

    let result = std::panic::catch_unwind(|| {
        _kcl_run_in_closure(
            kclvm_main_ptr,
            option_len,
            option_keys,
            option_values,
            strict_range_check,
            disable_none,
            disable_schema_check,
            list_option_mode,
            debug_mode,
            result_buffer_len,
            result_buffer,
        )
    });
    std::panic::set_hook(prev_hook);
    match result {
        Ok(n) => {
            let json_panic_info = Context::current_context().get_panic_info_json_string();

            let c_str_ptr = json_panic_info.as_ptr() as *const i8;
            let c_str_len = json_panic_info.len() as i32;

            unsafe {
                if c_str_len <= warn_buffer_len {
                    std::ptr::copy(c_str_ptr, warn_buffer, c_str_len as usize);
                }
            }

            kclvm_context_delete(ctx);
            n
        }
        Err(_) => {
            let json_panic_info = Context::current_context().get_panic_info_json_string();

            let c_str_ptr = json_panic_info.as_ptr() as *const i8;
            let c_str_len = json_panic_info.len() as i32;

            let mut return_len = c_str_len;

            unsafe {
                if return_len <= result_buffer_len {
                    std::ptr::copy(c_str_ptr, result_buffer, return_len as usize);
                } else {
                    *result_buffer = '\0' as kclvm_char_t;
                    return_len = 0 - return_len;
                }
            }

            kclvm_context_delete(ctx);
            return_len
        }
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn _kcl_run_in_closure(
    kclvm_main_ptr: u64, // main.k => kclvm_main
    option_len: kclvm_size_t,
    option_keys: *const *const kclvm_char_t,
    option_values: *const *const kclvm_char_t,
    strict_range_check: i32,
    disable_none: i32,
    disable_schema_check: i32,
    list_option_mode: i32,
    debug_mode: i32,
    result_buffer_len: kclvm_size_t,
    result_buffer: *mut kclvm_char_t,
) -> kclvm_size_t {
    let ctx = kclvm_context_current();

    let kclvm_main = (&kclvm_main_ptr as *const u64) as *const ()
        as *const extern "C" fn(ctx: *mut kclvm_context_t) -> *mut kclvm_value_ref_t;

    kclvm_context_set_strict_range_check(ctx, strict_range_check as kclvm_bool_t);
    kclvm_context_set_disable_none(ctx, disable_none as kclvm_bool_t);
    kclvm_context_set_disable_schema_check(ctx, disable_schema_check as kclvm_bool_t);
    kclvm_context_set_list_option_mode(ctx, list_option_mode as kclvm_bool_t);
    kclvm_context_set_debug_mode(ctx, debug_mode as kclvm_bool_t);

    unsafe {
        let option_keys = std::slice::from_raw_parts(option_keys, option_len as usize);
        let option_values = std::slice::from_raw_parts(option_values, option_len as usize);

        for i in 0..(option_len as usize) {
            kclvm_builtin_option_init(ctx, option_keys[i], option_values[i]);
        }

        let value = if kclvm_main.is_null() {
            kclvm_value_Str(b"{}\0" as *const u8 as *const kclvm_char_t)
        } else {
            kclvm_context_main_begin_hook(ctx);
            let x = (*kclvm_main)(ctx);
            kclvm_context_main_end_hook(ctx, x)
        };

        let c_str_ptr = kclvm_value_Str_ptr(value);
        let c_str_len = kclvm_value_len(value);

        let mut return_len = c_str_len;

        if return_len <= result_buffer_len {
            std::ptr::copy(c_str_ptr, result_buffer, return_len as usize);
        } else {
            *result_buffer = '\0' as kclvm_char_t;
            return_len = 0 - return_len;
        }

        // Delete by context to ignore pointer double free.
        return_len
    }
}
