//! Copyright The KCL Authors. All rights reserved.

extern crate base64;
use base64::{decode, encode};

use crate::*;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_base64_encode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("value")) {
        let s = encode(s);
        return ValueRef::str(s.as_str()).into_raw(ctx);
    }
    panic!("encode() missing 1 required positional argument: 'value'");
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_base64_decode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("value")) {
        let de_str = decode(s).unwrap();
        return ValueRef::str(std::str::from_utf8(&de_str).unwrap()).into_raw(ctx);
    }
    panic!("decode() missing 1 required positional argument: 'value'");
}
