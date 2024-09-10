//! Copyright The KCL Authors. All rights reserved.

use crate::*;

// data, sort_keys=False, indent=None, ignore_private=False, ignore_none=False

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_json_encode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);
    let kwargs = ptr_as_ref(kwargs);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("data")) {
        let s = ValueRef::str(
            arg0.to_json_string_with_options(&args_to_opts(args, kwargs, 1))
                .as_ref(),
        );
        return s.into_raw(ctx);
    }
    panic!("encode() missing 1 required positional argument: 'value'")
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_json_decode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("value")) {
        match ValueRef::from_json(ctx, arg0.as_str().as_ref()) {
            Ok(x) => return x.into_raw(ctx),
            Err(err) => panic!("{}", err),
        }
    }
    panic!("decode() missing 1 required positional argument: 'value'")
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_json_validate(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(arg0) = get_call_arg(args, kwargs, 0, Some("value")) {
        match ValueRef::from_json(ctx, arg0.as_str().as_ref()) {
            Ok(_) => return kclvm_value_True(ctx),
            Err(_) => return kclvm_value_False(ctx),
        }
    }
    panic!("validate() missing 1 required positional argument: 'value'")
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_json_dump_to_file(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let data = args.arg_i(0).or(kwargs.get_by_key("data"));
    let filename = args.arg_i(1).or(kwargs.get_by_key("filename"));
    match (data, filename) {
        (Some(data), Some(filename)) => {
            let filename = filename.as_str();
            let json = data.to_json_string_with_options(&args_to_opts(args, kwargs, 2));
            std::fs::write(&filename, json)
                .unwrap_or_else(|e| panic!("Unable to write file '{}': {}", filename, e));
            kclvm_value_Undefined(ctx)
        }
        _ => {
            panic!("dump_to_file() missing 2 required positional arguments: 'data' and 'filename'")
        }
    }
}

fn args_to_opts(args: &ValueRef, kwargs: &ValueRef, index: usize) -> JsonEncodeOptions {
    let mut opts = JsonEncodeOptions::default();
    if let Some(sort_keys) = get_call_arg_bool(args, kwargs, index, Some("sort_keys")) {
        opts.sort_keys = sort_keys;
    }
    if let Some(indent) = get_call_arg_int(args, kwargs, index + 1, Some("indent")) {
        opts.indent = indent;
    }
    if let Some(ignore_private) = get_call_arg_bool(args, kwargs, index + 2, Some("ignore_private"))
    {
        opts.ignore_private = ignore_private;
    }
    if let Some(ignore_none) = get_call_arg_bool(args, kwargs, index + 3, Some("ignore_none")) {
        opts.ignore_none = ignore_none;
    }
    opts
}
