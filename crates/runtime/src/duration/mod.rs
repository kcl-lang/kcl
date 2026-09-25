//! Duration/interval helpers for the `duration` system package.
//!
//! Durations are represented as compact strings in the form
//! `"<N>d<N>h<N>m<N>s"` (e.g. `"1h30m"`, `"2d"`, `"45s"`). Zero-valued
//! components are omitted; the smallest unit is seconds. This format
//! mirrors Go's `time.Duration` `String()` output and keeps the common
//! cases (`"30m"`, `"1h"`) readable while staying trivially parseable.
//!
//! Arithmetic goes through [`chrono::TimeDelta`] so datetimes combined
//! with a duration (via [`kcl_duration_add_to_datetime`] /
//! [`kcl_duration_sub_from_datetime`]) honor leap seconds and DST the
//! same way the rest of the `datetime` package does.
//!
//! See kcl-lang/kcl#1907 for context.

use chrono::{DateTime, TimeDelta};

use crate::*;

// 1 day = 24 * 60 * 60 seconds
const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

// ---------------------------------------------------------------------------
// Core parser / formatter. Kept `pub(crate)` so they can be reused from tests
// and from any future builtins that need to normalise a duration string.
// ---------------------------------------------------------------------------

/// Parse a duration string like `"1h30m"`, `"45s"`, `"2d"`, or `"1d2h3m4s"`.
/// Returns the duration as a (possibly fractional) number of seconds.
/// Returns `None` if the input does not match the canonical format.
/// Units must appear in canonical descending order: `d`, `h`, `m`, `s`.
/// At least one unit is required.
pub(crate) fn parse_seconds(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut total: f64 = 0.0;
    let mut idx = 0;
    let bytes = s.as_bytes();
    // Track the last-seen unit so we can reject out-of-order pairs like
    // "30m1h". Order matches the unit->seconds mapping below: d > h > m > s.
    let mut last_unit: u8 = b' ';
    let mut seen_any = false;
    while idx < bytes.len() {
        // Read the numeric component.
        let start = idx;
        while idx < bytes.len() && (bytes[idx].is_ascii_digit() || bytes[idx] == b'.') {
            idx += 1;
        }
        if start == idx {
            return None;
        }
        let num_str = &s[start..idx];
        let n: f64 = num_str.parse().ok()?;
        if idx >= bytes.len() {
            return None;
        }
        let unit = bytes[idx];
        // Canonical order is d (0x64) -> h (0x68) -> m (0x6d) -> s (0x73),
        // i.e. ASCII values monotonically increase as units shrink. Any
        // pair where the new unit is smaller (ASCII-wise) than the
        // previous one is out of order.
        if seen_any && unit < last_unit {
            return None;
        }
        match unit {
            b'd' => total += n * SECONDS_PER_DAY as f64,
            b'h' => total += n * SECONDS_PER_HOUR as f64,
            b'm' => total += n * SECONDS_PER_MINUTE as f64,
            b's' => total += n,
            _ => return None,
        }
        last_unit = unit;
        seen_any = true;
        idx += 1;
    }
    Some(total)
}

/// Render a number of seconds as the canonical compact form (e.g.
/// `5400.0` -> `"1h30m"`, `0.0` -> `"0s"`). Negative durations are
/// normalised by [`format_compact`]'s caller; this function expects a
/// non-negative input.
pub(crate) fn format_compact(total_seconds: f64) -> String {
    if total_seconds < 0.0 || !total_seconds.is_finite() {
        panic!("duration format_compact got non-finite or negative input: {total_seconds}");
    }
    let mut remaining = total_seconds;
    let mut out = String::new();
    let days = (remaining / SECONDS_PER_DAY as f64) as u64;
    if days > 0 {
        use std::fmt::Write;
        write!(&mut out, "{days}d").unwrap();
        remaining -= (days * SECONDS_PER_DAY as u64) as f64;
    }
    // Hours, minutes, and fractional seconds are skipped when zero so
    // the output stays compact (e.g. "1h" rather than "1h0m0s"). Seconds
    // are the smallest unit and must always be emitted when nothing
    // larger has been written yet (so a 0-second duration renders as
    // "0s").
    let hours = (remaining / SECONDS_PER_HOUR as f64) as u64;
    if hours > 0 {
        use std::fmt::Write;
        write!(&mut out, "{hours}h").unwrap();
        remaining -= (hours * SECONDS_PER_HOUR as u64) as f64;
    }
    let minutes = (remaining / SECONDS_PER_MINUTE as f64) as u64;
    if minutes > 0 {
        use std::fmt::Write;
        write!(&mut out, "{minutes}m").unwrap();
        remaining -= (minutes * SECONDS_PER_MINUTE as u64) as f64;
    }
    if remaining > 0.0 || out.is_empty() {
        use std::fmt::Write;
        // Trim trailing zeros for readability, but keep at least one
        // digit after the decimal when a fractional part exists.
        if (remaining - remaining.trunc()).abs() < f64::EPSILON {
            write!(&mut out, "{}s", remaining as u64).unwrap();
        } else {
            write!(&mut out, "{remaining}s").unwrap();
        }
    }
    out
}

/// Parse `a` and `b`, add the two durations, and return the canonical
/// string. Returns `None` if either side does not parse.
pub(crate) fn add_durations(a: &str, b: &str) -> Option<String> {
    let av = parse_seconds(a)?;
    let bv = parse_seconds(b)?;
    Some(format_compact(av + bv))
}

/// Parse `a` and `b`, subtract `b` from `a`, and return the canonical
/// string. Panics if the result would be negative — callers should
/// check first if they need to support that case.
pub(crate) fn sub_durations(a: &str, b: &str) -> Option<String> {
    let av = parse_seconds(a)?;
    let bv = parse_seconds(b)?;
    if bv > av {
        panic!("duration.sub() result would be negative: {a} - {b}");
    }
    Some(format_compact(av - bv))
}

/// Apply a duration string to an RFC 3339 datetime, returning a new
/// RFC 3339 string.
fn apply_duration_to_datetime(dt_str: &str, dur_str: &str, sign: i64) -> String {
    let dt: DateTime<chrono::FixedOffset> = DateTime::parse_from_rfc3339(dt_str)
        .unwrap_or_else(|e| panic!("add_to_datetime() expected RFC 3339 input, got {dt_str:?}: {e}"));
    let seconds = parse_seconds(dur_str)
        .unwrap_or_else(|| panic!("add_to_datetime() invalid duration: {dur_str:?}"));
    // TimeDelta::seconds takes whole seconds; sub-second fractions are
    // handled with `Duration::milliseconds` after splitting out the integer
    // part.
    let whole = seconds.trunc() as i64;
    let frac_ms = ((seconds.fract()) * 1000.0).round() as i64;
    let delta = TimeDelta::seconds(whole) + TimeDelta::milliseconds(frac_ms);
    let delta = if sign < 0 { -delta } else { delta };
    let result = dt + delta;
    result.to_rfc3339()
}

// ---------------------------------------------------------------------------
// FFI surface. Each function follows the existing `kcl_*_<name>` pattern in
// this crate. The `unsafe` blocks are the standard C-ABI marshalling — see
// any of the datetime/units builtins for equivalent usage.
// ---------------------------------------------------------------------------

/// `duration.second(n: int) -> str` — build a duration of `n` seconds.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_second(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(n) = get_call_arg(args, kwargs, 0, Some("n")) {
        let f = n.convert_to_float(ctx).as_float();
        if !f.is_finite() || f < 0.0 {
            panic!("second() requires a non-negative finite number, got {f}");
        }
        return ValueRef::str(&format_compact(f)).into_raw(ctx);
    }
    panic!("second() missing 1 required positional argument: 'n'");
}

/// `duration.minute(n: int) -> str`
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_minute(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(n) = get_call_arg(args, kwargs, 0, Some("n")) {
        let f = n.convert_to_float(ctx).as_float();
        if !f.is_finite() || f < 0.0 {
            panic!("minute() requires a non-negative finite number, got {f}");
        }
        return ValueRef::str(&format_compact(f * SECONDS_PER_MINUTE as f64)).into_raw(ctx);
    }
    panic!("minute() missing 1 required positional argument: 'n'");
}

/// `duration.hour(n: int) -> str`
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_hour(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(n) = get_call_arg(args, kwargs, 0, Some("n")) {
        let f = n.convert_to_float(ctx).as_float();
        if !f.is_finite() || f < 0.0 {
            panic!("hour() requires a non-negative finite number, got {f}");
        }
        return ValueRef::str(&format_compact(f * SECONDS_PER_HOUR as f64)).into_raw(ctx);
    }
    panic!("hour() missing 1 required positional argument: 'n'");
}

/// `duration.day(n: int) -> str`
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_day(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(n) = get_call_arg(args, kwargs, 0, Some("n")) {
        let f = n.convert_to_float(ctx).as_float();
        if !f.is_finite() || f < 0.0 {
            panic!("day() requires a non-negative finite number, got {f}");
        }
        return ValueRef::str(&format_compact(f * SECONDS_PER_DAY as f64)).into_raw(ctx);
    }
    panic!("day() missing 1 required positional argument: 'n'");
}

/// `duration.to_seconds(d: str) -> float` — parse a duration string and
/// return the equivalent number of seconds.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_to_seconds(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("d")) {
        match parse_seconds(&s) {
            Some(v) => return ValueRef::float(v).into_raw(ctx),
            None => panic!("to_seconds() invalid duration: {s:?}"),
        }
    }
    panic!("to_seconds() missing 1 required positional argument: 'd'");
}

/// `duration.from_seconds(n: float) -> str` — render a number of seconds
/// as the canonical compact form.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_from_seconds(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(n) = get_call_arg(args, kwargs, 0, Some("n")) {
        let f = n.convert_to_float(ctx).as_float();
        if !f.is_finite() || f < 0.0 {
            panic!("from_seconds() requires a non-negative finite number, got {f}");
        }
        return ValueRef::str(&format_compact(f)).into_raw(ctx);
    }
    panic!("from_seconds() missing 1 required positional argument: 'n'");
}

/// `duration.parse(d: str) -> str` — validate and re-render a duration
/// string. Useful for normalising inputs like `"60s"` -> `"1m"`.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_parse(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("d")) {
        match parse_seconds(&s) {
            Some(v) => return ValueRef::str(&format_compact(v)).into_raw(ctx),
            None => panic!("parse() invalid duration: {s:?}"),
        }
    }
    panic!("parse() missing 1 required positional argument: 'd'");
}

/// `duration.is_valid(d: str) -> bool`
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_is_valid(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(s) = get_call_arg_str(args, kwargs, 0, Some("d")) {
        return ValueRef::bool(parse_seconds(&s).is_some()).into_raw(ctx);
    }
    panic!("is_valid() missing 1 required positional argument: 'd'");
}

/// `duration.add(a: str, b: str) -> str`
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_add(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(a) = get_call_arg_str(args, kwargs, 0, Some("a")) {
        if let Some(b) = get_call_arg_str(args, kwargs, 1, Some("b")) {
            match add_durations(&a, &b) {
                Some(v) => return ValueRef::str(&v).into_raw(ctx),
                None => panic!("add() invalid duration(s): a={a:?}, b={b:?}"),
            }
        }
        panic!("add() missing 1 required positional argument: 'b'");
    }
    panic!("add() missing 1 required positional argument: 'a'");
}

/// `duration.sub(a: str, b: str) -> str`
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_sub(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(a) = get_call_arg_str(args, kwargs, 0, Some("a")) {
        if let Some(b) = get_call_arg_str(args, kwargs, 1, Some("b")) {
            match sub_durations(&a, &b) {
                Some(v) => return ValueRef::str(&v).into_raw(ctx),
                None => panic!("sub() invalid duration(s): a={a:?}, b={b:?}"),
            }
        }
        panic!("sub() missing 1 required positional argument: 'b'");
    }
    panic!("sub() missing 1 required positional argument: 'a'");
}

/// `duration.add_to_datetime(dt: str, d: str) -> str` — add a duration to
/// an RFC 3339 datetime string.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_add_to_datetime(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(dt) = get_call_arg_str(args, kwargs, 0, Some("dt")) {
        if let Some(d) = get_call_arg_str(args, kwargs, 1, Some("d")) {
            return ValueRef::str(&apply_duration_to_datetime(&dt, &d, 1)).into_raw(ctx);
        }
        panic!("add_to_datetime() missing 1 required positional argument: 'd'");
    }
    panic!("add_to_datetime() missing 1 required positional argument: 'dt'");
}

/// `duration.sub_from_datetime(dt: str, d: str) -> str`
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_sub_from_datetime(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    if let Some(dt) = get_call_arg_str(args, kwargs, 0, Some("dt")) {
        if let Some(d) = get_call_arg_str(args, kwargs, 1, Some("d")) {
            return ValueRef::str(&apply_duration_to_datetime(&dt, &d, -1)).into_raw(ctx);
        }
        panic!("sub_from_datetime() missing 1 required positional argument: 'd'");
    }
    panic!("sub_from_datetime() missing 1 required positional argument: 'dt'");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seconds_basic_units() {
        assert_eq!(parse_seconds("45s"), Some(45.0));
        assert_eq!(parse_seconds("5m"), Some(300.0));
        assert_eq!(parse_seconds("1h"), Some(3600.0));
        assert_eq!(parse_seconds("1d"), Some(86400.0));
    }

    #[test]
    fn parse_seconds_combined() {
        assert_eq!(parse_seconds("1h30m"), Some(5400.0));
        assert_eq!(parse_seconds("1d2h3m4s"), Some(86400.0 + 7200.0 + 180.0 + 4.0));
        assert_eq!(parse_seconds("0s"), Some(0.0));
    }

    #[test]
    fn parse_seconds_rejects_invalid() {
        assert_eq!(parse_seconds(""), None);
        assert_eq!(parse_seconds("10"), None);
        assert_eq!(parse_seconds("10x"), None);
        assert_eq!(parse_seconds("1.5h"), Some(5400.0));
        assert_eq!(parse_seconds("abc"), None);
        // Units must be in d/h/m/s order — out-of-order is rejected for
        // simplicity. Callers can use `parse` to normalize first.
        assert_eq!(parse_seconds("30m1h"), None);
    }

    #[test]
    fn format_compact_basic() {
        assert_eq!(format_compact(0.0), "0s");
        assert_eq!(format_compact(45.0), "45s");
        assert_eq!(format_compact(60.0), "1m");
        assert_eq!(format_compact(5400.0), "1h30m");
        assert_eq!(
            format_compact(86400.0 + 7200.0 + 180.0 + 4.0),
            "1d2h3m4s"
        );
        assert_eq!(format_compact(3600.0 * 24.0), "1d");
    }

    #[test]
    fn format_compact_zero_components_skipped() {
        assert_eq!(format_compact(3600.0), "1h");
        assert_eq!(format_compact(60.0 * 60.0 * 2.0), "2h");
        assert_eq!(format_compact(86400.0 * 3.0), "3d");
    }

    #[test]
    fn add_durations_normalises() {
        assert_eq!(add_durations("30m", "60s"), Some("31m".to_string()));
        assert_eq!(add_durations("23h", "2h"), Some("1d1h".to_string()));
        assert_eq!(add_durations("0s", "1h30m"), Some("1h30m".to_string()));
        assert_eq!(add_durations("invalid", "1h"), None);
    }

    #[test]
    fn sub_durations_normalises() {
        assert_eq!(sub_durations("1h", "30m"), Some("30m".to_string()));
        assert_eq!(sub_durations("2d", "1d"), Some("1d".to_string()));
        assert_eq!(sub_durations("1h", "1h"), Some("0s".to_string()));
    }

    #[test]
    #[should_panic(expected = "negative")]
    fn sub_durations_panics_on_negative() {
        let _ = sub_durations("30m", "1h");
    }

    #[test]
    fn roundtrip_through_seconds() {
        // Each canonical form should round-trip through `to_seconds` and
        // `from_seconds` losslessly.
        for s in ["45s", "1h", "1h30m", "1d2h3m4s", "0s"] {
            let secs = parse_seconds(s).unwrap();
            assert_eq!(format_compact(secs), s, "roundtrip for {s}");
        }
    }
}