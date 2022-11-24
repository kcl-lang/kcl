//! KCL yaml system module
//!
//! Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

// def KMANGLED_encode(data, sort_keys=False, ignore_private=False, ignore_none=False):

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_yaml_encode(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);

    let mut opt = YamlEncodeOptions::default();
    if let Some(sort_keys) = kwargs.kwarg_bool("sort_keys", None) {
        opt.sort_keys = sort_keys;
    }
    if let Some(ignore_private) = kwargs.kwarg_bool("ignore_private", None) {
        opt.ignore_private = ignore_private;
    }
    if let Some(ignore_none) = kwargs.kwarg_bool("ignore_none", None) {
        opt.ignore_none = ignore_none;
    }

    if let Some(arg0) = args.arg_i(0) {
        let s = ValueRef::str(arg0.to_yaml_string_with_options(&opt).as_ref());
        return s.into_raw();
    }
    panic!("encode() missing 1 required positional argument: 'value'")
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_yaml_decode(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(arg0) = args.arg_i(0) {
        match ValueRef::from_yaml(arg0.as_str().as_ref()) {
            Ok(x) => return x.into_raw(),
            Err(err) => panic!("{}", err),
        }
    }
    panic!("decode() missing 1 required positional argument: 'value'")
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_yaml_dump_to_file(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(data) = args.arg_i(0) {
        if let Some(filename) = args.arg_i(0) {
            let yaml = data.to_yaml_string();
            let filename = filename.as_str();

            std::fs::write(filename, yaml).expect("Unable to write file");
        }
    }
    panic!("dump_to_file() missing 2 required positional arguments: 'data' and 'filename'")
}
