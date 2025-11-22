//! Copyright The KCL Authors. All rights reserved.

extern crate base32;
use crate::*;
use base32::{Alphabet, decode, encode};

#[unsafe(no_mangle)]

pub extern "C-unwind" fn kclvm_base32_encode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("value")) {
        let s = encode(Alphabet::RFC4648 { padding: true }, s.as_bytes());
        return ValueRef::str(s.as_str()).into_raw(ctx);
    }
    panic!("encode() missing 1 required positional argument: 'value'");
}

#[unsafe(no_mangle)]

pub extern "C-unwind" fn kclvm_base32_decode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("value")) {
        if let Some(de_str) = decode(Alphabet::RFC4648 { padding: true }, &s) {
            if let Ok(s) = std::str::from_utf8(&de_str) {
                return ValueRef::str(s).into_raw(ctx);
            }
        }
        // Handle decoding errors
        return ValueRef::none().into_raw(ctx);
    }
    panic!("decode() missing 1 required positional argument: 'value'");
}
