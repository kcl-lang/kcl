//! Copyright The KCL Authors. All rights reserved.

extern crate chrono;

use chrono::prelude::Local;

use crate::*;

// today() -> str:

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_today(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let s = Local::now().to_string();
    let ctx = mut_ptr_as_ref(ctx);
    return ValueRef::str(s.as_ref()).into_raw(ctx);
}

// now() -> str:

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_now(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let s = Local::now().to_string();
    let ctx = mut_ptr_as_ref(ctx);
    return ValueRef::str(s.as_ref()).into_raw(ctx);
}

// ticks() -> float:

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_ticks(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let x = Local::now().timestamp();
    ValueRef::float(x as f64).into_raw(ctx)
}

// date() -> str:

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_date(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let s = Local::now().to_string();
    let ctx = mut_ptr_as_ref(ctx);
    return ValueRef::str(s.as_ref()).into_raw(ctx);
}
