//! Copyright The KCL Authors. All rights reserved.

extern crate chrono;

use chrono::{prelude::Local, NaiveDate, NaiveDateTime, NaiveTime};

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

/// Return the local time format. e.g. 'Sat Jun 06 16:26:11 1998' or format the combined date and time per the specified format string,
/// and the default date format is "%a %b %d %H:%M:%S %Y".
/// `now() -> str`
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_now(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let s = Local::now();
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let format = get_call_arg_str(args, kwargs, 0, Some("format"))
        .unwrap_or_else(|| "%a %b %d %H:%M:%S %Y".to_string());
    ValueRef::str(&s.format(&format).to_string()).into_raw(ctx)
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

/// Validates whether the provided date string matches the specified format.
/// `validate(str, str) -> bool`
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_datetime_validate(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    if let Some(date) = get_call_arg_str(args, kwargs, 0, Some("date")) {
        if let Some(format) = get_call_arg_str(args, kwargs, 1, Some("format")) {
            let result = validate_date(&date, &format);
            return ValueRef::bool(result).into_raw(ctx);
        }
        panic!("validate() takes 2 positional arguments (1 given)");
    }
    panic!("validate() takes 2 positional arguments (0 given)");
}

/// Validates whether the provided date string matches the specified format.
///
/// # Parameters
/// - `date`: A string slice representing the date to be validated.
/// - `format`: A string slice representing the expected format for the date.
///
/// # Returns
/// - Returns `true` if the date string successfully parses according to the specified format,
///   otherwise, returns `false`.
#[inline]
fn validate_date(date: &str, format: &str) -> bool {
    NaiveDateTime::parse_from_str(date, format)
        .map(|_| true)
        .or_else(|_| NaiveDate::parse_from_str(date, format).map(|_| true))
        .or_else(|_| NaiveTime::parse_from_str(date, format).map(|_| true))
        .is_ok()
}
