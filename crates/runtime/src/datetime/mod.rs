//! Copyright The KCL Authors. All rights reserved.

extern crate chrono;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, prelude::Local};

use crate::*;

/// Return the "%Y-%m-%d %H:%M:%S.%{ticks}" format date.
/// `today() -> str`
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_datetime_today(
    ctx: *mut kcl_context_t,
    _args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let s = Local::now();
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    ValueRef::str(&(s.format("%Y-%m-%d %H:%M:%S").to_string() + "." + &s.timestamp().to_string()))
        .into_raw(ctx)
}

/// Return the local time format. e.g. 'Sat Jun 06 16:26:11 1998' or format the combined date and time per the specified format string,
/// and the default date format is "%a %b %d %H:%M:%S %Y".
///
/// When the optional `ticks` argument (seconds since the Unix epoch, as returned
/// by `ticks()`) is provided, that instant is formatted instead of the current
/// time. This allows rendering arbitrary past or future dates. The instant is
/// rendered in the local time zone, matching the no-argument behavior.
/// `now(format: str = "%a %b %d %H:%M:%S %Y", ticks: float = None) -> str`
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_datetime_now(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let format = get_call_arg_str(args, kwargs, 0, Some("format"))
        .unwrap_or_else(|| "%a %b %d %H:%M:%S %Y".to_string());
    let formatted = match get_call_arg_num(args, kwargs, 1, Some("ticks")) {
        Some(ticks) => match ticks_to_local(ticks) {
            Some(dt) => dt.format(&format).to_string(),
            None => panic!("now() got an out-of-range 'ticks' value: {ticks}"),
        },
        None => Local::now().format(&format).to_string(),
    };
    ValueRef::str(&formatted).into_raw(ctx)
}

/// Convert `ticks` (seconds since the Unix epoch, with optional fractional
/// seconds) into a local-time `DateTime`. Returns `None` when the value cannot
/// be represented as a valid date-time.
#[inline]
fn ticks_to_local(ticks: f64) -> Option<DateTime<Local>> {
    let secs = ticks.trunc() as i64;
    let nsecs = (ticks.fract().abs() * 1_000_000_000.0).round() as u32;
    DateTime::from_timestamp(secs, nsecs).map(|dt| dt.with_timezone(&Local))
}

/// Return the current time in seconds since the Epoch. Fractions of a second may be present if the system clock provides them.
/// `ticks() -> float`
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_datetime_ticks(
    ctx: *mut kcl_context_t,
    _args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let x = Local::now().timestamp();
    ValueRef::float(x as f64).into_raw(ctx)
}

/// Return the %Y-%m-%d %H:%M:%S format date.
/// `date() -> str`
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_datetime_date(
    ctx: *mut kcl_context_t,
    _args: *const kcl_value_ref_t,
    _kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let s = Local::now();
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    ValueRef::str(&s.format("%Y-%m-%d %H:%M:%S").to_string()).into_raw(ctx)
}

/// Validates whether the provided date string matches the specified format.
/// `validate(str, str) -> bool`
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_datetime_validate(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
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

/// Validates whether the provided string is a valid RFC 3339 date-time.
/// `is_rfc3339(str) -> bool`
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_datetime_is_rfc3339(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(date) = get_call_arg_str(args, kwargs, 0, Some("date")) {
        let is_valid = chrono::DateTime::parse_from_rfc3339(&date).is_ok();
        return ValueRef::bool(is_valid).into_raw(ctx);
    }
    panic!("is_rfc3339() missing 1 required positional argument: 'date'");
}

/// Validates whether the provided string is a valid ISO 8601 duration or date-time.
/// `is_iso8601(str) -> bool`
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_datetime_is_iso8601(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(date) = get_call_arg_str(args, kwargs, 0, Some("date")) {
        // Check if it is a valid RFC 3339 (which is an ISO 8601 profile)
        if chrono::DateTime::parse_from_rfc3339(&date).is_ok() {
            return ValueRef::bool(true).into_raw(ctx);
        }
        // Fallback: Check if it is an ISO 8601 Duration
        // A standard regex for ISO 8601 durations (e.g., P3Y6M4DT12H30M5.5S)
        let re = fancy_regex::Regex::new(r"^P(?!$)(?:\d+(?:\.\d+)?Y)?(?:\d+(?:\.\d+)?M)?(?:\d+(?:\.\d+)?W)?(?:\d+(?:\.\d+)?D)?(?:T(?=\d)(?:\d+(?:\.\d+)?H)?(?:\d+(?:\.\d+)?M)?(?:\d+(?:\.\d+)?S)?)?$").unwrap();
        let is_valid = re.is_match(&date).unwrap_or(false);
        return ValueRef::bool(is_valid).into_raw(ctx);
    }
    panic!("is_iso8601() missing 1 required positional argument: 'date'");
}

#[cfg(test)]
mod tests {
    use super::ticks_to_local;

    #[test]
    fn test_ticks_to_local_roundtrips_epoch() {
        // `timestamp()` is the absolute instant (UTC epoch), independent of the
        // local time zone, so this assertion is deterministic on any machine.
        for ticks in [0.0_f64, 1_000_000_000.0, 1_700_000_000.0] {
            let dt = ticks_to_local(ticks).expect("valid epoch must convert");
            assert_eq!(dt.timestamp(), ticks as i64);
        }
    }

    #[test]
    fn test_ticks_to_local_fractional_seconds() {
        let dt = ticks_to_local(1_000_000_000.5).expect("valid epoch must convert");
        assert_eq!(dt.timestamp(), 1_000_000_000);
        assert_eq!(dt.timestamp_subsec_millis(), 500);
    }
}
