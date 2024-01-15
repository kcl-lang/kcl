// Copyright 2021 The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_char;

use crate::*;

#[allow(dead_code, non_camel_case_types)]
type kclvm_context_t = Context;

#[allow(dead_code, non_camel_case_types)]
type kclvm_kind_t = Kind;

#[allow(dead_code, non_camel_case_types)]
type kclvm_type_t = Type;

#[allow(dead_code, non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[allow(dead_code, non_camel_case_types)]
type kclvm_iterator_t = ValueIterator;

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

#[derive(Debug, Default)]
pub(crate) struct RuntimePanicRecord {
    pub kcl_panic_info: bool,
    pub message: String,
    pub rust_file: String,
    pub rust_line: i32,
    pub rust_col: i32,
}

thread_local! {
    static KCL_RUNTIME_PANIC_RECORD: std::cell::RefCell<RuntimePanicRecord>  = std::cell::RefCell::new(RuntimePanicRecord::default())
}

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
    result_buffer_len: *mut kclvm_size_t,
    result_buffer: *mut kclvm_char_t,
    warn_buffer_len: *mut kclvm_size_t,
    warn_buffer: *mut kclvm_char_t,
    log_buffer_len: *mut kclvm_size_t,
    log_buffer: *mut kclvm_char_t,
) -> kclvm_size_t {
    let ctx = kclvm_context_new();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info: &std::panic::PanicInfo| {
        KCL_RUNTIME_PANIC_RECORD.with(|record| {
            let mut record = record.borrow_mut();
            record.kcl_panic_info = true;

            record.message = if let Some(s) = info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = info.payload().downcast_ref::<&String>() {
                (*s).clone()
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                (*s).clone()
            } else {
                "".to_string()
            };
            if let Some(location) = info.location() {
                record.rust_file = location.file().to_string();
                record.rust_line = location.line() as i32;
                record.rust_col = location.column() as i32;
            }
        })
    }));

    let result = std::panic::catch_unwind(|| {
        _kcl_run_in_closure(
            ctx,
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
    KCL_RUNTIME_PANIC_RECORD.with(|record| {
        let record = record.borrow();
        let ctx = mut_ptr_as_ref(ctx);
        ctx.set_panic_info(&record);
    });
    // Get the runtime context.
    let ctx_ref = ptr_as_ref(ctx);
    // Copy log message pointer
    let c_str_ptr = ctx_ref.log_message.as_ptr() as *const c_char;
    let c_str_len = ctx_ref.log_message.len() as i32;
    if c_str_len <= *log_buffer_len {
        std::ptr::copy(c_str_ptr, log_buffer, c_str_len as usize);
        *log_buffer_len = c_str_len
    }
    // Copy panic info message pointer
    let json_panic_info = ctx_ref.get_panic_info_json_string();
    let c_str_ptr = json_panic_info.as_ptr() as *const c_char;
    let c_str_len = json_panic_info.len() as i32;
    match result {
        Ok(n) => {
            unsafe {
                if c_str_len <= *warn_buffer_len {
                    std::ptr::copy(c_str_ptr, warn_buffer, c_str_len as usize);
                    *warn_buffer_len = c_str_len
                }
            }
            kclvm_context_delete(ctx);
            n
        }
        Err(_) => {
            let mut return_len = c_str_len;
            unsafe {
                if return_len <= *result_buffer_len {
                    std::ptr::copy(c_str_ptr, result_buffer, return_len as usize);
                    *result_buffer_len = return_len
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
    ctx: *mut Context,
    kclvm_main_ptr: u64, // main.k => kclvm_main
    option_len: kclvm_size_t,
    option_keys: *const *const kclvm_char_t,
    option_values: *const *const kclvm_char_t,
    strict_range_check: i32,
    disable_none: i32,
    disable_schema_check: i32,
    list_option_mode: i32,
    debug_mode: i32,
    result_buffer_len: *mut kclvm_size_t,
    result_buffer: *mut kclvm_char_t,
) -> kclvm_size_t {
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
            kclvm_value_Str(ctx, b"{}\0" as *const u8 as *const kclvm_char_t)
        } else {
            kclvm_context_main_begin_hook(ctx);
            let x = (*kclvm_main)(ctx);
            kclvm_context_main_end_hook(ctx, x)
        };

        let c_str_ptr = kclvm_value_Str_ptr(value);
        let c_str_len = kclvm_value_len(value);

        let mut return_len = c_str_len;

        if return_len <= *result_buffer_len {
            std::ptr::copy(c_str_ptr, result_buffer, return_len as usize);
            *result_buffer_len = return_len;
        } else {
            *result_buffer = '\0' as kclvm_char_t;
            return_len = 0 - return_len;
        }

        // Delete by context to ignore pointer double free.
        return_len
    }
}
