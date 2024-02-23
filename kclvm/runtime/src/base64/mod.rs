//! Copyright The KCL Authors. All rights reserved.

extern crate base64;
use base64::{decode, encode};

use crate::*;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_base64_encode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);
    let p = args.arg_0().unwrap();
    match &*p.rc.borrow() {
        Value::str_value(x) => {
            let s = encode(x.clone());
            return ValueRef::str(s.as_str()).into_raw(ctx);
        }
        _ => {
            ctx.set_err_type(&RuntimeErrorType::TypeError);

            panic!("a string object is required, not '{}'", p.as_str());
        }
    };
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_base64_decode(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let ctx = mut_ptr_as_ref(ctx);
    let p = args.arg_0().unwrap();
    match &*p.rc.borrow() {
        Value::str_value(x) => {
            let de_str = decode(x.clone()).unwrap();
            return ValueRef::str(std::str::from_utf8(&de_str).unwrap()).into_raw(ctx);
        }
        _ => {
            ctx.set_err_type(&RuntimeErrorType::TypeError);

            panic!("argument should be a string object, not '{}'", p.as_str());
        }
    };
}
