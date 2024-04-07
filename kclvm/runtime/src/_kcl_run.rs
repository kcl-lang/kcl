//! Copyright The KCL Authors. All rights reserved.
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
pub struct RuntimePanicRecord {
    pub kcl_panic_info: bool,
    pub message: String,
    pub rust_file: String,
    pub rust_line: i32,
    pub rust_col: i32,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct FFIRunOptions {
    pub strict_range_check: i32,
    pub disable_none: i32,
    pub disable_schema_check: i32,
    pub debug_mode: i32,
    pub show_hidden: i32,
    pub sort_keys: i32,
    pub include_schema_type_path: i32,
    pub disable_empty_list: i32,
}

thread_local! {
    static KCL_RUNTIME_PANIC_RECORD: std::cell::RefCell<RuntimePanicRecord>  = std::cell::RefCell::new(RuntimePanicRecord::default())
}

fn new_ctx_with_opts(opts: FFIRunOptions, path_selector: &[String]) -> Context {
    let mut ctx = Context::new();
    // Config
    ctx.cfg.strict_range_check = opts.strict_range_check != 0;
    ctx.cfg.disable_schema_check = opts.disable_schema_check != 0;
    ctx.cfg.debug_mode = opts.debug_mode != 0;
    // Plan options
    ctx.plan_opts.disable_none = opts.disable_none != 0;
    ctx.plan_opts.show_hidden = opts.show_hidden != 0;
    ctx.plan_opts.sort_keys = opts.sort_keys != 0;
    ctx.plan_opts.include_schema_type_path = opts.include_schema_type_path != 0;
    ctx.plan_opts.disable_empty_list = opts.disable_empty_list != 0;
    ctx.plan_opts.query_paths = path_selector.to_vec();
    ctx
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn _kcl_run(
    kclvm_main_ptr: u64, // main.k => kclvm_main
    option_len: kclvm_size_t,
    option_keys: *const *const kclvm_char_t,
    option_values: *const *const kclvm_char_t,
    opts: FFIRunOptions,
    path_selector: *const *const kclvm_char_t,
    json_result_buffer_len: *mut kclvm_size_t,
    json_result_buffer: *mut kclvm_char_t,
    yaml_result_buffer_len: *mut kclvm_size_t,
    yaml_result_buffer: *mut kclvm_char_t,
    err_buffer_len: *mut kclvm_size_t,
    err_buffer: *mut kclvm_char_t,
    log_buffer_len: *mut kclvm_size_t,
    log_buffer: *mut kclvm_char_t,
) -> kclvm_size_t {
    // Init runtime context with options
    let ctx = Box::new(new_ctx_with_opts(opts, &c2str_vec(path_selector))).into_raw();
    let option_keys = std::slice::from_raw_parts(option_keys, option_len as usize);
    let option_values = std::slice::from_raw_parts(option_values, option_len as usize);
    for i in 0..(option_len as usize) {
        kclvm_builtin_option_init(ctx, option_keys[i], option_values[i]);
    }
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
    let result = std::panic::catch_unwind(|| _kcl_run_in_closure(ctx, kclvm_main_ptr));
    std::panic::set_hook(prev_hook);
    KCL_RUNTIME_PANIC_RECORD.with(|record| {
        let record = record.borrow();
        let ctx = mut_ptr_as_ref(ctx);
        ctx.set_panic_info(&record);
    });
    // Get the runtime context.
    let ctx_ref = ptr_as_ref(ctx);
    // Copy planned result and log message
    copy_str_to(
        &ctx_ref.json_result,
        json_result_buffer,
        json_result_buffer_len,
    );
    copy_str_to(
        &ctx_ref.yaml_result,
        yaml_result_buffer,
        yaml_result_buffer_len,
    );
    copy_str_to(&ctx_ref.log_message, log_buffer, log_buffer_len);
    // Copy JSON panic info message pointer
    let json_panic_info = if result.is_err() {
        ctx_ref.get_panic_info_json_string().unwrap_or_default()
    } else {
        "".to_string()
    };
    copy_str_to(&json_panic_info, err_buffer, err_buffer_len);
    // Delete the context
    kclvm_context_delete(ctx);
    result.is_err() as kclvm_size_t
}

#[allow(clippy::too_many_arguments)]
unsafe fn _kcl_run_in_closure(
    ctx: *mut Context,
    kclvm_main_ptr: u64, // main.k => kclvm_main
) {
    let kclvm_main = (&kclvm_main_ptr as *const u64) as *const ()
        as *const extern "C" fn(ctx: *mut kclvm_context_t) -> *mut kclvm_value_ref_t;

    unsafe {
        if kclvm_main.is_null() {
            panic!("kcl program main function not found");
        }
        (*kclvm_main)(ctx);
    }
}
