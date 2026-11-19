//! Copyright The KCL Authors. All rights reserved.
//!
//! Duration/interval helpers for the `duration` system package.
//!
//! Values are strings. The canonical form is a compact
//! `"<N>d<N>h<N>m<N>s"` encoding (`"1h30m"`, `"2d"`, `"45s"`, `"0.5s"`).
//! Zero components are omitted. Parsing also accepts Go subsecond units
//! (`ms`, `us` / `µs` / `μs`, `ns`) and fixed-length ISO 8601 durations
//! (`PT1H30M`, `P1DT2H`, `P1W`). Years and months are rejected because
//! they do not have a fixed length. Resolution is one nanosecond.
//!
//! Datetime helpers accept RFC 3339 and the strings produced by
//! `datetime.today()`, `datetime.date()`, and `datetime.now()` with its
//! default format. Arithmetic uses [`chrono::TimeDelta`], the same clock
//! as the `datetime` package. Adding a duration moves an instant by that
//! length; it does not overload `+` or `-`.
//!
//! See kcl-lang/kcl#1907.

use std::cmp::Ordering;
use std::fmt::Write;

use chrono::{
    DateTime, FixedOffset, Local, LocalResult, NaiveDateTime, Offset, TimeDelta, TimeZone,
};

use crate::*;

const NANOS_PER_SECOND: i128 = 1_000_000_000;
const NANOS_PER_MILLISECOND: i128 = 1_000_000;
const NANOS_PER_MICROSECOND: i128 = 1_000;
const NANOS_PER_MINUTE: i128 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: i128 = 60 * NANOS_PER_MINUTE;
const NANOS_PER_DAY: i128 = 24 * NANOS_PER_HOUR;
const NANOS_PER_WEEK: i128 = 7 * NANOS_PER_DAY;

// Rank of compact units. A later unit may repeat the same rank (`1h1h`)
// but may not go backwards (`30m1h`), matching the original parser.
const RANK_DAY: i32 = 0;
const RANK_HOUR: i32 = 1;
const RANK_MINUTE: i32 = 2;
const RANK_SECOND: i32 = 3;
const RANK_MILLI: i32 = 4;
const RANK_MICRO: i32 = 5;
const RANK_NANO: i32 = 6;

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse a duration into nanoseconds.
///
/// Compact form (`1h30m`, `500ms`, `1d2h3m4.5s`) and fixed-length ISO 8601
/// (`PT1H30M`, `P1W`) are accepted. Returns `None` when the text is not a
/// duration of fixed length.
pub(crate) fn parse_nanos(input: &str) -> Option<i128> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    if s.as_bytes().first() == Some(&b'P') {
        parse_iso8601_nanos(s)
    } else {
        parse_compact_nanos(s)
    }
}

fn parse_compact_nanos(s: &str) -> Option<i128> {
    let mut idx = 0;
    let mut total: i128 = 0;
    let mut last_rank: i32 = -1;
    let mut seen = false;
    while idx < s.len() {
        let (num, next) = read_compact_number(s, idx)?;
        let (unit_nanos, rank, next) = read_compact_unit(s, next)?;
        if seen && rank < last_rank {
            return None;
        }
        last_rank = rank;
        seen = true;
        idx = next;
        total = total.checked_add(decimal_to_nanos(num, unit_nanos)?)?;
    }
    if seen { Some(total) } else { None }
}

fn parse_iso8601_nanos(s: &str) -> Option<i128> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'P') {
        return None;
    }
    let mut i = 1;
    if i >= bytes.len() {
        return None;
    }
    let mut total: i128 = 0;
    let mut seen_any = false;
    let mut in_time = false;
    let mut seen_time = false;
    let mut last_rank: i32 = -1;
    while i < bytes.len() {
        if bytes[i] == b'T' {
            // `T` introduces the time part and must be followed by a component.
            if in_time || i + 1 >= bytes.len() {
                return None;
            }
            in_time = true;
            last_rank = -1;
            i += 1;
            continue;
        }
        let (num, next) = read_iso_number(s, i)?;
        i = next;
        if i >= bytes.len() {
            return None;
        }
        let unit = bytes[i];
        i += 1;
        let (unit_nanos, rank) = iso_unit(in_time, unit)?;
        // ISO 8601 allows each designator once, in order. `datetime.is_iso8601`
        // uses the same shape, so duplicates and reversed units are rejected.
        if rank <= last_rank {
            return None;
        }
        last_rank = rank;
        total = total.checked_add(decimal_to_nanos(num, unit_nanos)?)?;
        seen_any = true;
        if in_time {
            seen_time = true;
        }
    }
    if !seen_any || (in_time && !seen_time) {
        return None;
    }
    Some(total)
}

/// Years (`Y`) and months (`M` before `T`) are calendar units.
fn iso_unit(in_time: bool, unit: u8) -> Option<(i128, i32)> {
    if in_time {
        match unit {
            b'H' => Some((NANOS_PER_HOUR, 0)),
            b'M' => Some((NANOS_PER_MINUTE, 1)),
            b'S' => Some((NANOS_PER_SECOND, 2)),
            _ => None,
        }
    } else {
        match unit {
            b'W' => Some((NANOS_PER_WEEK, 0)),
            b'D' => Some((NANOS_PER_DAY, 1)),
            _ => None,
        }
    }
}

fn read_compact_number(s: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    let mut idx = start;
    let mut seen_dot = false;
    let mut seen_digit = false;
    while idx < bytes.len() {
        if bytes[idx].is_ascii_digit() {
            seen_digit = true;
            idx += 1;
        } else if bytes[idx] == b'.' && !seen_dot {
            seen_dot = true;
            idx += 1;
        } else {
            break;
        }
    }
    if !seen_digit {
        return None;
    }
    Some((&s[start..idx], idx))
}

/// ISO numbers match `\d+(?:\.\d+)?`, the same shape `datetime.is_iso8601` accepts.
fn read_iso_number(s: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = s.as_bytes();
    let mut idx = start;
    if idx >= bytes.len() || !bytes[idx].is_ascii_digit() {
        return None;
    }
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
    }
    if idx < bytes.len() && bytes[idx] == b'.' {
        let after_dot = idx + 1;
        if after_dot >= bytes.len() || !bytes[after_dot].is_ascii_digit() {
            return None;
        }
        idx = after_dot + 1;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    }
    Some((&s[start..idx], idx))
}

fn read_compact_unit(s: &str, idx: usize) -> Option<(i128, i32, usize)> {
    let rest = s.get(idx..)?;
    // Longer suffixes first so `ms` is not read as `m`.
    const LONG: &[(&str, i128, i32)] = &[
        ("ms", NANOS_PER_MILLISECOND, RANK_MILLI),
        ("us", NANOS_PER_MICROSECOND, RANK_MICRO),
        ("µs", NANOS_PER_MICROSECOND, RANK_MICRO),
        ("μs", NANOS_PER_MICROSECOND, RANK_MICRO),
        ("ns", 1, RANK_NANO),
    ];
    for (prefix, nanos, rank) in LONG {
        if rest.starts_with(prefix) {
            return Some((*nanos, *rank, idx + prefix.len()));
        }
    }
    match rest.as_bytes().first()? {
        b'd' => Some((NANOS_PER_DAY, RANK_DAY, idx + 1)),
        b'h' => Some((NANOS_PER_HOUR, RANK_HOUR, idx + 1)),
        b'm' => Some((NANOS_PER_MINUTE, RANK_MINUTE, idx + 1)),
        b's' => Some((NANOS_PER_SECOND, RANK_SECOND, idx + 1)),
        _ => None,
    }
}

/// Convert a decimal string times `unit_nanos` into nanoseconds, rounding
/// half away from zero. The fraction is decimal, not binary, so `0.1s` is
/// exactly 100_000_000ns.
fn decimal_to_nanos(num: &str, unit_nanos: i128) -> Option<i128> {
    let (int_str, frac_str) = match num.split_once('.') {
        Some((int_part, frac_part)) => (int_part, frac_part),
        None => (num, ""),
    };
    if int_str.is_empty() && frac_str.is_empty() {
        return None;
    }
    if !int_str.bytes().all(|b| b.is_ascii_digit()) || !frac_str.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    if int_str.len() > 30 || frac_str.len() > 18 {
        return None;
    }
    let int_part: i128 = if int_str.is_empty() {
        0
    } else {
        int_str.parse().ok()?
    };
    let whole = int_part.checked_mul(unit_nanos)?;
    if frac_str.is_empty() {
        return Some(whole);
    }
    let frac_val: i128 = frac_str.parse().ok()?;
    let mut scale: i128 = 1;
    for _ in 0..frac_str.len() {
        scale = scale.checked_mul(10)?;
    }
    let numer = frac_val.checked_mul(unit_nanos)?;
    let rounded = numer.checked_add(scale / 2)?.checked_div(scale)?;
    whole.checked_add(rounded)
}

fn f64_to_nanos(value: f64, unit_nanos: i128) -> Option<i128> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    if value == 0.0 {
        return Some(0);
    }
    // Whole numbers that fit in i64 stay exact. Everything else rounds to
    // the nearest nanosecond.
    if value.fract() == 0.0 && value < i64::MAX as f64 {
        return (value as i64 as i128).checked_mul(unit_nanos);
    }
    let product = value * unit_nanos as f64;
    if !product.is_finite() || product < 0.0 || product >= i128::MAX as f64 {
        return None;
    }
    Some(product.round() as i128)
}

fn nanos_to_unit(nanos: i128, unit_nanos: i128) -> f64 {
    nanos as f64 / unit_nanos as f64
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

fn format_compact_nanos(mut nanos: i128) -> String {
    if nanos < 0 {
        panic!("duration format_compact got a negative duration: {nanos}");
    }
    let mut out = String::new();
    let days = nanos / NANOS_PER_DAY;
    if days > 0 {
        write!(&mut out, "{days}d").unwrap();
        nanos %= NANOS_PER_DAY;
    }
    let hours = nanos / NANOS_PER_HOUR;
    if hours > 0 {
        write!(&mut out, "{hours}h").unwrap();
        nanos %= NANOS_PER_HOUR;
    }
    let minutes = nanos / NANOS_PER_MINUTE;
    if minutes > 0 {
        write!(&mut out, "{minutes}m").unwrap();
        nanos %= NANOS_PER_MINUTE;
    }
    let secs = nanos / NANOS_PER_SECOND;
    let frac = nanos % NANOS_PER_SECOND;
    if secs > 0 || frac > 0 || out.is_empty() {
        push_fraction(&mut out, secs, frac, 's');
    }
    out
}

/// Normalized ISO 8601. Weeks are emitted as days (`P7D`), and a zero
/// duration is `PT0S`.
fn format_iso8601(nanos: i128) -> String {
    if nanos == 0 {
        return "PT0S".to_string();
    }
    let days = nanos / NANOS_PER_DAY;
    let mut rem = nanos % NANOS_PER_DAY;
    let hours = rem / NANOS_PER_HOUR;
    rem %= NANOS_PER_HOUR;
    let minutes = rem / NANOS_PER_MINUTE;
    rem %= NANOS_PER_MINUTE;
    let secs = rem / NANOS_PER_SECOND;
    let frac = rem % NANOS_PER_SECOND;
    let mut out = String::from("P");
    if days > 0 {
        write!(&mut out, "{days}D").unwrap();
    }
    if hours > 0 || minutes > 0 || secs > 0 || frac > 0 {
        out.push('T');
        if hours > 0 {
            write!(&mut out, "{hours}H").unwrap();
        }
        if minutes > 0 {
            write!(&mut out, "{minutes}M").unwrap();
        }
        if secs > 0 || frac > 0 {
            push_fraction(&mut out, secs, frac, 'S');
        }
    }
    out
}

fn push_fraction(out: &mut String, whole: i128, frac: i128, unit: char) {
    if frac == 0 {
        write!(out, "{whole}{unit}").unwrap();
        return;
    }
    let mut digits = format!("{frac:09}");
    while digits.ends_with('0') {
        digits.pop();
    }
    write!(out, "{whole}.{digits}{unit}").unwrap();
}

// ---------------------------------------------------------------------------
// Arithmetic
// ---------------------------------------------------------------------------

pub(crate) fn add_durations(a: &str, b: &str) -> Option<String> {
    let av = parse_nanos(a)?;
    let bv = parse_nanos(b)?;
    let sum = av
        .checked_add(bv)
        .unwrap_or_else(|| panic!("duration.add() overflow: {a} + {b}"));
    Some(format_compact_nanos(sum))
}

pub(crate) fn sub_durations(a: &str, b: &str) -> Option<String> {
    let av = parse_nanos(a)?;
    let bv = parse_nanos(b)?;
    if bv > av {
        panic!("duration.sub() result would be negative: {a} - {b}");
    }
    Some(format_compact_nanos(av - bv))
}

fn scale_nanos(nanos: i128, factor: f64) -> Option<i128> {
    if !factor.is_finite() || factor < 0.0 {
        return None;
    }
    if factor == 0.0 {
        return Some(0);
    }
    if factor.fract() == 0.0 && factor < i64::MAX as f64 {
        return nanos.checked_mul(factor as i128);
    }
    let product = (nanos as f64) * factor;
    if !product.is_finite() || product < 0.0 || product >= i128::MAX as f64 {
        return None;
    }
    Some(product.round() as i128)
}

fn div_nanos(nanos: i128, factor: f64) -> Option<i128> {
    if !factor.is_finite() || factor <= 0.0 {
        return None;
    }
    if factor.fract() == 0.0 && factor < i64::MAX as f64 {
        let divisor = factor as i128;
        if divisor == 0 {
            return None;
        }
        let half = divisor / 2;
        return nanos.checked_add(half)?.checked_div(divisor);
    }
    scale_nanos(nanos, 1.0 / factor)
}

fn cmp_nanos(a: i128, b: i128) -> i64 {
    match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

// ---------------------------------------------------------------------------
// Datetime
// ---------------------------------------------------------------------------

/// Accept the datetime strings this runtime actually produces, plus RFC 3339.
fn parse_kcl_datetime(s: &str) -> Option<DateTime<FixedOffset>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt);
    }
    // `datetime.today()` is `"%Y-%m-%d %H:%M:%S.{ticks}"`. The suffix is
    // seconds since the epoch and is the instant; the wall-clock prefix is
    // what `today()` printed in the local zone.
    if s.contains('.') {
        return parse_today(s);
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return naive_as_local(naive);
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%a %b %d %H:%M:%S %Y") {
        return naive_as_local(naive);
    }
    None
}

fn parse_today(s: &str) -> Option<DateTime<FixedOffset>> {
    let (head, tail) = s.rsplit_once('.')?;
    if tail.is_empty() || tail.len() > 18 || !tail.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    NaiveDateTime::parse_from_str(head, "%Y-%m-%d %H:%M:%S").ok()?;
    let ticks: i64 = tail.parse().ok()?;
    let utc = DateTime::from_timestamp(ticks, 0)?;
    Some(as_fixed(utc))
}

fn naive_as_local(naive: NaiveDateTime) -> Option<DateTime<FixedOffset>> {
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(dt) => Some(as_fixed(dt)),
        _ => None,
    }
}

fn as_fixed<Tz: TimeZone>(dt: DateTime<Tz>) -> DateTime<FixedOffset> {
    let offset = dt.offset().fix();
    dt.with_timezone(&offset)
}

fn nanos_to_timedelta(nanos: i128) -> Option<TimeDelta> {
    let negative = nanos < 0;
    let abs = nanos.unsigned_abs();
    let secs_u = abs / NANOS_PER_SECOND as u128;
    if secs_u > i64::MAX as u128 {
        return None;
    }
    let delta = TimeDelta::new(secs_u as i64, (abs % NANOS_PER_SECOND as u128) as u32)?;
    Some(if negative { -delta } else { delta })
}

fn timedelta_to_nanos(delta: TimeDelta) -> Option<i128> {
    let secs = delta.num_seconds() as i128;
    let sub = delta.subsec_nanos() as i128;
    secs.checked_mul(NANOS_PER_SECOND)?.checked_add(sub)
}

fn apply_duration_to_datetime(dt_str: &str, dur_str: &str, sign: i64, func: &str) -> String {
    let dt = parse_kcl_datetime(dt_str).unwrap_or_else(|| {
        panic!(
            "{func}() invalid datetime {dt_str:?}: expected RFC 3339 or the output of datetime.today(), datetime.date(), or datetime.now()"
        )
    });
    let nanos =
        parse_nanos(dur_str).unwrap_or_else(|| panic!("{func}() invalid duration: {dur_str:?}"));
    let signed = if sign < 0 { -nanos } else { nanos };
    let delta = nanos_to_timedelta(signed)
        .unwrap_or_else(|| panic!("{func}() duration is out of range: {dur_str:?}"));
    (dt + delta).to_rfc3339()
}

fn duration_between(start: &str, end: &str) -> String {
    let a = parse_kcl_datetime(start)
        .unwrap_or_else(|| panic!("between() invalid datetime: {start:?}"));
    let b =
        parse_kcl_datetime(end).unwrap_or_else(|| panic!("between() invalid datetime: {end:?}"));
    let nanos = timedelta_to_nanos(b.signed_duration_since(a))
        .unwrap_or_else(|| panic!("between() duration is out of range"));
    if nanos < 0 {
        panic!("duration.between() result would be negative: {start} .. {end}");
    }
    format_compact_nanos(nanos)
}

// ---------------------------------------------------------------------------
// FFI
// ---------------------------------------------------------------------------

fn require_str(args: &ValueRef, kwargs: &ValueRef, index: usize, name: &str, func: &str) -> String {
    get_call_arg_str(args, kwargs, index, Some(name))
        .unwrap_or_else(|| panic!("{func}() missing 1 required positional argument: '{name}'"))
}

fn require_number(
    ctx: &mut Context,
    args: &ValueRef,
    kwargs: &ValueRef,
    index: usize,
    name: &str,
    func: &str,
) -> f64 {
    let n = get_call_arg(args, kwargs, index, Some(name))
        .unwrap_or_else(|| panic!("{func}() missing 1 required positional argument: '{name}'"));
    n.convert_to_float(ctx).as_float()
}

fn number_to_duration(value: f64, unit_nanos: i128, func: &str) -> String {
    if !value.is_finite() || value < 0.0 {
        panic!("{func}() requires a non-negative finite number, got {value}");
    }
    let nanos = f64_to_nanos(value, unit_nanos)
        .unwrap_or_else(|| panic!("{func}() result is out of range: {value}"));
    format_compact_nanos(nanos)
}

fn duration_as_unit(text: &str, unit_nanos: i128, func: &str) -> f64 {
    let nanos = parse_nanos(text).unwrap_or_else(|| panic!("{func}() invalid duration: {text:?}"));
    nanos_to_unit(nanos, unit_nanos)
}

fn ffi_parts<'a>(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> (&'a mut Context, &'a ValueRef, &'a ValueRef) {
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    (ctx, args, kwargs)
}

/// `duration.second(n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_second(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let n = require_number(ctx, args, kwargs, 0, "n", "second");
    ValueRef::str(&number_to_duration(n, NANOS_PER_SECOND, "second")).into_raw(ctx)
}

/// `duration.millisecond(n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_millisecond(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let n = require_number(ctx, args, kwargs, 0, "n", "millisecond");
    ValueRef::str(&number_to_duration(n, NANOS_PER_MILLISECOND, "millisecond")).into_raw(ctx)
}

/// `duration.minute(n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_minute(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let n = require_number(ctx, args, kwargs, 0, "n", "minute");
    ValueRef::str(&number_to_duration(n, NANOS_PER_MINUTE, "minute")).into_raw(ctx)
}

/// `duration.hour(n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_hour(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let n = require_number(ctx, args, kwargs, 0, "n", "hour");
    ValueRef::str(&number_to_duration(n, NANOS_PER_HOUR, "hour")).into_raw(ctx)
}

/// `duration.day(n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_day(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let n = require_number(ctx, args, kwargs, 0, "n", "day");
    ValueRef::str(&number_to_duration(n, NANOS_PER_DAY, "day")).into_raw(ctx)
}

/// `duration.to_seconds(d) -> float`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_to_seconds(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "to_seconds");
    ValueRef::float(duration_as_unit(&d, NANOS_PER_SECOND, "to_seconds")).into_raw(ctx)
}

/// `duration.to_milliseconds(d) -> float`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_to_milliseconds(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "to_milliseconds");
    ValueRef::float(duration_as_unit(
        &d,
        NANOS_PER_MILLISECOND,
        "to_milliseconds",
    ))
    .into_raw(ctx)
}

/// `duration.to_minutes(d) -> float`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_to_minutes(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "to_minutes");
    ValueRef::float(duration_as_unit(&d, NANOS_PER_MINUTE, "to_minutes")).into_raw(ctx)
}

/// `duration.to_hours(d) -> float`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_to_hours(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "to_hours");
    ValueRef::float(duration_as_unit(&d, NANOS_PER_HOUR, "to_hours")).into_raw(ctx)
}

/// `duration.to_days(d) -> float`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_to_days(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "to_days");
    ValueRef::float(duration_as_unit(&d, NANOS_PER_DAY, "to_days")).into_raw(ctx)
}

/// `duration.from_seconds(n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_from_seconds(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let n = require_number(ctx, args, kwargs, 0, "n", "from_seconds");
    ValueRef::str(&number_to_duration(n, NANOS_PER_SECOND, "from_seconds")).into_raw(ctx)
}

/// `duration.to_iso8601(d) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_to_iso8601(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "to_iso8601");
    let nanos = parse_nanos(&d).unwrap_or_else(|| panic!("to_iso8601() invalid duration: {d:?}"));
    ValueRef::str(&format_iso8601(nanos)).into_raw(ctx)
}

/// `duration.parse(d) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_parse(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "parse");
    let nanos = parse_nanos(&d).unwrap_or_else(|| panic!("parse() invalid duration: {d:?}"));
    ValueRef::str(&format_compact_nanos(nanos)).into_raw(ctx)
}

/// `duration.is_valid(d) -> bool`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_is_valid(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "is_valid");
    ValueRef::bool(parse_nanos(&d).is_some()).into_raw(ctx)
}

/// `duration.add(a, b) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_add(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let a = require_str(args, kwargs, 0, "a", "add");
    let b = require_str(args, kwargs, 1, "b", "add");
    match add_durations(&a, &b) {
        Some(v) => ValueRef::str(&v).into_raw(ctx),
        None => panic!("add() invalid duration(s): a={a:?}, b={b:?}"),
    }
}

/// `duration.sub(a, b) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_sub(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let a = require_str(args, kwargs, 0, "a", "sub");
    let b = require_str(args, kwargs, 1, "b", "sub");
    match sub_durations(&a, &b) {
        Some(v) => ValueRef::str(&v).into_raw(ctx),
        None => panic!("sub() invalid duration(s): a={a:?}, b={b:?}"),
    }
}

/// `duration.mul(d, n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_mul(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "mul");
    let n = require_number(ctx, args, kwargs, 1, "n", "mul");
    if !n.is_finite() || n < 0.0 {
        panic!("mul() requires a non-negative finite number, got {n}");
    }
    let nanos = parse_nanos(&d).unwrap_or_else(|| panic!("mul() invalid duration: {d:?}"));
    let scaled = scale_nanos(nanos, n).unwrap_or_else(|| panic!("mul() result is out of range"));
    ValueRef::str(&format_compact_nanos(scaled)).into_raw(ctx)
}

/// `duration.div(d, n) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_div(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let d = require_str(args, kwargs, 0, "d", "div");
    let n = require_number(ctx, args, kwargs, 1, "n", "div");
    if !n.is_finite() || n <= 0.0 {
        panic!("div() requires a positive finite number, got {n}");
    }
    let nanos = parse_nanos(&d).unwrap_or_else(|| panic!("div() invalid duration: {d:?}"));
    let scaled = div_nanos(nanos, n).unwrap_or_else(|| panic!("div() result is out of range"));
    ValueRef::str(&format_compact_nanos(scaled)).into_raw(ctx)
}

/// `duration.cmp(a, b) -> int`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_cmp(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let a = require_str(args, kwargs, 0, "a", "cmp");
    let b = require_str(args, kwargs, 1, "b", "cmp");
    let av = parse_nanos(&a).unwrap_or_else(|| panic!("cmp() invalid duration: {a:?}"));
    let bv = parse_nanos(&b).unwrap_or_else(|| panic!("cmp() invalid duration: {b:?}"));
    ValueRef::int(cmp_nanos(av, bv)).into_raw(ctx)
}

/// `duration.add_to_datetime(dt, d) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_add_to_datetime(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let dt = require_str(args, kwargs, 0, "dt", "add_to_datetime");
    let d = require_str(args, kwargs, 1, "d", "add_to_datetime");
    ValueRef::str(&apply_duration_to_datetime(&dt, &d, 1, "add_to_datetime")).into_raw(ctx)
}

/// `duration.sub_from_datetime(dt, d) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_sub_from_datetime(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let dt = require_str(args, kwargs, 0, "dt", "sub_from_datetime");
    let d = require_str(args, kwargs, 1, "d", "sub_from_datetime");
    ValueRef::str(&apply_duration_to_datetime(
        &dt,
        &d,
        -1,
        "sub_from_datetime",
    ))
    .into_raw(ctx)
}

/// `duration.between(start, end) -> str`
///
/// # Safety
/// `ctx`, `args`, and `kwargs` must be valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_duration_between(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let (ctx, args, kwargs) = ffi_parts(ctx, args, kwargs);
    let start = require_str(args, kwargs, 0, "start", "between");
    let end = require_str(args, kwargs, 1, "end", "between");
    ValueRef::str(&duration_between(&start, &end)).into_raw(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(s: &str) -> Option<f64> {
        parse_nanos(s).map(|n| nanos_to_unit(n, NANOS_PER_SECOND))
    }

    #[test]
    fn parse_seconds_basic_units() {
        assert_eq!(secs("45s"), Some(45.0));
        assert_eq!(secs("5m"), Some(300.0));
        assert_eq!(secs("1h"), Some(3600.0));
        assert_eq!(secs("1d"), Some(86400.0));
    }

    #[test]
    fn parse_seconds_combined() {
        assert_eq!(secs("1h30m"), Some(5400.0));
        assert_eq!(secs("1d2h3m4s"), Some(86400.0 + 7200.0 + 180.0 + 4.0));
        assert_eq!(secs("0s"), Some(0.0));
    }

    #[test]
    fn parse_seconds_rejects_invalid() {
        assert_eq!(secs(""), None);
        assert_eq!(secs("10"), None);
        assert_eq!(secs("10x"), None);
        assert_eq!(secs("1.5h"), Some(5400.0));
        assert_eq!(secs("abc"), None);
        assert_eq!(secs("30m1h"), None);
    }

    #[test]
    fn repeated_units_still_accumulate() {
        assert_eq!(secs("1h1h"), Some(7200.0));
        assert_eq!(format_compact_nanos(parse_nanos("1h1h").unwrap()), "2h");
    }

    #[test]
    fn parse_subsecond_and_iso8601() {
        assert_eq!(parse_nanos("500ms"), Some(500_000_000));
        assert_eq!(parse_nanos("1500ms"), Some(1_500_000_000));
        assert_eq!(parse_nanos("2us"), Some(2_000));
        assert_eq!(parse_nanos("1µs"), Some(1_000));
        assert_eq!(parse_nanos("1μs"), Some(1_000));
        assert_eq!(parse_nanos("100ns"), Some(100));
        assert_eq!(parse_nanos("1s1ms"), Some(1_001_000_000));
        assert_eq!(parse_nanos("1ms1s"), None);
        assert_eq!(parse_nanos("0.1s"), Some(100_000_000));
        assert_eq!(parse_nanos("PT1H30M"), Some(5_400_000_000_000));
        assert_eq!(parse_nanos("P1W"), Some(NANOS_PER_WEEK));
        assert_eq!(parse_nanos("P1DT2H3M4S"), parse_nanos("1d2h3m4s"));
        assert_eq!(parse_nanos("PT0.5S"), Some(500_000_000));
        assert_eq!(parse_nanos("PT1M"), Some(NANOS_PER_MINUTE));
        assert_eq!(parse_nanos("P1Y"), None);
        assert_eq!(parse_nanos("P1M"), None);
        assert_eq!(parse_nanos("PT"), None);
        assert_eq!(parse_nanos("P1DT"), None);
        assert_eq!(parse_nanos("PT1H1H"), None);
        assert_eq!(parse_nanos("P1D1W"), None);
        assert_eq!(
            format_compact_nanos(parse_nanos("PT1.5H30M").unwrap()),
            "2h"
        );
    }

    #[test]
    fn format_compact_basic() {
        assert_eq!(
            number_to_duration(0.0, NANOS_PER_SECOND, "from_seconds"),
            "0s"
        );
        assert_eq!(
            number_to_duration(45.0, NANOS_PER_SECOND, "from_seconds"),
            "45s"
        );
        assert_eq!(
            number_to_duration(60.0, NANOS_PER_SECOND, "from_seconds"),
            "1m"
        );
        assert_eq!(
            number_to_duration(5400.0, NANOS_PER_SECOND, "from_seconds"),
            "1h30m"
        );
        assert_eq!(
            number_to_duration(
                86400.0 + 7200.0 + 180.0 + 4.0,
                NANOS_PER_SECOND,
                "from_seconds"
            ),
            "1d2h3m4s"
        );
        assert_eq!(
            number_to_duration(3600.0 * 24.0, NANOS_PER_SECOND, "from_seconds"),
            "1d"
        );
        assert_eq!(
            number_to_duration(0.5, NANOS_PER_SECOND, "from_seconds"),
            "0.5s"
        );
        assert_eq!(
            number_to_duration(0.001, NANOS_PER_SECOND, "from_seconds"),
            "0.001s"
        );
    }

    #[test]
    fn format_compact_zero_components_skipped() {
        assert_eq!(
            number_to_duration(3600.0, NANOS_PER_SECOND, "from_seconds"),
            "1h"
        );
        assert_eq!(
            number_to_duration(60.0 * 60.0 * 2.0, NANOS_PER_SECOND, "from_seconds"),
            "2h"
        );
        assert_eq!(
            number_to_duration(86400.0 * 3.0, NANOS_PER_SECOND, "from_seconds"),
            "3d"
        );
    }

    #[test]
    fn format_round_trips_subsecond_text() {
        for s in [
            "0.5s",
            "1.5s",
            "0.001s",
            "0.000001s",
            "0.0000001s",
            "1h30m0.5s",
        ] {
            let nanos = parse_nanos(s).unwrap();
            assert_eq!(format_compact_nanos(nanos), s, "roundtrip for {s}");
        }
    }

    #[test]
    fn format_iso8601_normalizes() {
        assert_eq!(format_iso8601(0), "PT0S");
        assert_eq!(format_iso8601(parse_nanos("1h30m").unwrap()), "PT1H30M");
        assert_eq!(
            format_iso8601(parse_nanos("1d2h3m4s").unwrap()),
            "P1DT2H3M4S"
        );
        assert_eq!(format_iso8601(parse_nanos("1.5s").unwrap()), "PT1.5S");
        assert_eq!(format_iso8601(parse_nanos("1d").unwrap()), "P1D");
        assert_eq!(format_iso8601(parse_nanos("P1W").unwrap()), "P7D");
        assert_eq!(format_iso8601(parse_nanos("12h").unwrap()), "PT12H");
    }

    #[test]
    fn add_durations_normalises() {
        assert_eq!(add_durations("30m", "60s"), Some("31m".to_string()));
        assert_eq!(add_durations("23h", "2h"), Some("1d1h".to_string()));
        assert_eq!(add_durations("0s", "1h30m"), Some("1h30m".to_string()));
        assert_eq!(add_durations("invalid", "1h"), None);
        assert_eq!(add_durations("500ms", "500ms"), Some("1s".to_string()));
        assert_eq!(add_durations("PT30M", "30m"), Some("1h".to_string()));
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
    fn scale_and_compare() {
        assert_eq!(
            format_compact_nanos(scale_nanos(parse_nanos("30m").unwrap(), 2.0).unwrap()),
            "1h"
        );
        assert_eq!(
            format_compact_nanos(scale_nanos(parse_nanos("1h").unwrap(), 0.0).unwrap()),
            "0s"
        );
        assert_eq!(
            format_compact_nanos(div_nanos(parse_nanos("1h").unwrap(), 2.0).unwrap()),
            "30m"
        );
        assert_eq!(
            format_compact_nanos(div_nanos(parse_nanos("1s").unwrap(), 2.0).unwrap()),
            "0.5s"
        );
        assert!(div_nanos(parse_nanos("1s").unwrap(), 0.0).is_none());
        assert_eq!(
            cmp_nanos(parse_nanos("2h").unwrap(), parse_nanos("30m").unwrap()),
            1
        );
        assert_eq!(
            cmp_nanos(parse_nanos("30m").unwrap(), parse_nanos("2h").unwrap()),
            -1
        );
        assert_eq!(
            cmp_nanos(parse_nanos("PT1H").unwrap(), parse_nanos("60m").unwrap()),
            0
        );
        assert_eq!(
            cmp_nanos(parse_nanos("500ms").unwrap(), parse_nanos("0.5s").unwrap()),
            0
        );
    }

    #[test]
    fn roundtrip_through_seconds() {
        for s in ["45s", "1h", "1h30m", "1d2h3m4s", "0s"] {
            let secs = secs(s).unwrap();
            assert_eq!(
                number_to_duration(secs, NANOS_PER_SECOND, "from_seconds"),
                s,
                "roundtrip for {s}"
            );
        }
    }

    #[test]
    fn datetime_helpers_accept_rfc3339_and_today() {
        assert_eq!(
            apply_duration_to_datetime("2024-01-15T10:00:00Z", "1h30m", 1, "add_to_datetime"),
            "2024-01-15T11:30:00+00:00"
        );
        assert_eq!(
            apply_duration_to_datetime("2024-01-15T10:00:00Z", "1.5s", 1, "add_to_datetime"),
            "2024-01-15T10:00:01.500+00:00"
        );
        assert_eq!(
            apply_duration_to_datetime("2024-01-15T10:00:00Z", "500ms", 1, "add_to_datetime"),
            "2024-01-15T10:00:00.500+00:00"
        );
        assert_eq!(
            apply_duration_to_datetime("2024-02-28T23:00:00Z", "2h", 1, "add_to_datetime"),
            "2024-02-29T01:00:00+00:00"
        );
        // 1705312800 is 2024-01-15T10:00:00Z. The wall-clock prefix is ignored.
        assert_eq!(
            apply_duration_to_datetime(
                "1999-01-01 00:00:00.1705312800",
                "40m",
                1,
                "add_to_datetime"
            ),
            "2024-01-15T10:40:00+00:00"
        );
        assert_eq!(
            apply_duration_to_datetime(
                "2024-01-15 10:00:00.1705312800",
                "30m",
                -1,
                "sub_from_datetime"
            ),
            "2024-01-15T09:30:00+00:00"
        );
        assert_eq!(
            duration_between("2024-01-15T10:00:00Z", "2024-01-17T12:30:00Z"),
            "2d2h30m"
        );
        assert_eq!(
            duration_between("2024-01-15T10:00:00Z", "2024-01-15 10:00:00.1705312800"),
            "0s"
        );
        assert_eq!(
            duration_between("2024-01-15T10:00:00Z", "2024-01-15T10:00:01.500Z"),
            "1.5s"
        );
    }

    #[test]
    fn datetime_date_and_now_formats_use_local_time() {
        let date = "2024-01-15 10:00:00";
        let parsed = parse_kcl_datetime(date).expect("date() format");
        let naive = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S").unwrap();
        let expected = Local
            .from_local_datetime(&naive)
            .single()
            .expect("local date");
        assert_eq!(parsed.timestamp(), expected.timestamp());

        let now = "Mon Jan 15 10:00:00 2024";
        let parsed = parse_kcl_datetime(now).expect("now() format");
        let naive = NaiveDateTime::parse_from_str(now, "%a %b %d %H:%M:%S %Y").unwrap();
        let expected = Local
            .from_local_datetime(&naive)
            .single()
            .expect("local now");
        assert_eq!(parsed.timestamp(), expected.timestamp());
    }

    #[test]
    fn conversions_match_the_constructors() {
        assert_eq!(
            nanos_to_unit(parse_nanos("1.5s").unwrap(), NANOS_PER_MILLISECOND),
            1500.0
        );
        assert_eq!(
            nanos_to_unit(parse_nanos("1h30m").unwrap(), NANOS_PER_MINUTE),
            90.0
        );
        assert_eq!(
            nanos_to_unit(parse_nanos("1d").unwrap(), NANOS_PER_HOUR),
            24.0
        );
        assert_eq!(
            nanos_to_unit(parse_nanos("36h").unwrap(), NANOS_PER_DAY),
            1.5
        );
        assert_eq!(
            number_to_duration(500.0, NANOS_PER_MILLISECOND, "millisecond"),
            "0.5s"
        );
        assert_eq!(number_to_duration(1.5, NANOS_PER_MINUTE, "minute"), "1m30s");
    }
}
