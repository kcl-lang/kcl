// Copyright 2021 The KCL Authors. All rights reserved.
extern crate base64;
use base64::{decode, encode};

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_base64_encode(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let p = args.arg_0().unwrap();
    match &*p.rc.borrow() {
        Value::str_value(x) => {
            let s = encode(x.clone());
            return ValueRef::str(s.as_str()).into_raw();
        }
        _ => {
            let ctx = Context::current_context_mut();
            ctx.set_err_type(&ErrType::TypeError_Runtime_TYPE);

            panic!("a bytes-like object is required, not '{}'", p.as_str());
        }
    };
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_base64_decode(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *mut kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let p = args.arg_0().unwrap();
    match &*p.rc.borrow() {
        Value::str_value(x) => {
            let de_str = decode(x.clone()).unwrap();
            return ValueRef::str(std::str::from_utf8(&de_str).unwrap()).into_raw();
        }
        _ => {
            let ctx = Context::current_context_mut();
            ctx.set_err_type(&ErrType::TypeError_Runtime_TYPE);

            panic!(
                "argument should be a bytes-like object or ASCII string, not '{}'",
                p.as_str()
            );
        }
    };
}
