//! Copyright The KCL Authors. All rights reserved.

extern crate chrono;

use chrono::prelude::Local;

use crate::*;

/// Return the "%Y-%m-%d %H:%M:%S.%{ticks}" format date.
/// `today() -> str`
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_today(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let s = Local::now();
    let ctx = mut_ptr_as_ref(ctx);
    ValueRef::str(&(s.format("%Y-%m-%d %H:%M:%S").to_string() + "." + &s.timestamp().to_string()))
        .into_raw(ctx)
}

/// Return the local time. e.g. 'Sat Jun 06 16:26:11 1998'
/// `now() -> str`
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_now(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let s = Local::now();
    let ctx = mut_ptr_as_ref(ctx);
    ValueRef::str(&s.format("%a %b %d %H:%M:%S %Y").to_string()).into_raw(ctx)
}

/// Return the current time in seconds since the Epoch. Fractions of a second may be present if the system clock provides them.
/// `ticks() -> float`
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

/// Return the %Y-%m-%d %H:%M:%S format date.
/// `date() -> str`
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_date(
    ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let s = Local::now();
    let ctx = mut_ptr_as_ref(ctx);
    ValueRef::str(&s.format("%Y-%m-%d %H:%M:%S").to_string()).into_raw(ctx)
}
