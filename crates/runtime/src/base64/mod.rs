//! Copyright The KCL Authors. All rights reserved.

extern crate base64;
use base64::{decode, encode};

use crate::*;

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_base64_encode(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("value")) {
        let s = encode(s);
        return ValueRef::str(s.as_str()).into_raw(ctx);
    }
    panic!("encode() missing 1 required positional argument: 'value'");
}

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_base64_decode(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("value")) {
        let de_str = decode(s).unwrap();
        return ValueRef::str(std::str::from_utf8(&de_str).unwrap()).into_raw(ctx);
    }
    panic!("decode() missing 1 required positional argument: 'value'");
}
