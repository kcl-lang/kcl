//! Copyright The KCL Authors. All rights reserved.

use crate::*;

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
enum to_unit_suffix {
    n,
    u,
    m,
    k,
    K,
    M,
    G,
    T,
    P,
    Ki,
    Mi,
    Gi,
    Ti,
    Pi,
}

use phf::{phf_map, Map};

pub const IEC_SUFFIX: &str = "i";
pub const EXPONENTS: Map<&str, i8> = phf_map! {
    "n" => -3,
    "u" => -2,
    "m" => -1,
    "k" => 1,
    "K" => 1,
    "M" => 2,
    "G" => 3,
    "T" => 4,
    "P" => 5,
};
pub const INVALID_UNITS: [&str; 4] = ["ni", "ui", "mi", "ki"];

// to_n(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_n(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    let num = args.arg_0().unwrap().convert_to_float(ctx).as_float();
    let s = to_unit(num, to_unit_suffix::n);
    return ValueRef::str(s.as_ref()).into_raw(ctx);
}

// to_u(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_u(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    let num = args.arg_0().unwrap().convert_to_float(ctx).as_float();
    let s = to_unit(num, to_unit_suffix::u);
    return ValueRef::str(s.as_ref()).into_raw(ctx);
}

// to_m(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_m(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    let num = args.arg_0().unwrap().convert_to_float(ctx).as_float();
    let s = to_unit(num, to_unit_suffix::m);
    return ValueRef::str(s.as_ref()).into_raw(ctx);
}

// to_K(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_K(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::K);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_K() missing 1 required positional argument: 'num'");
}

// to_M(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_M(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);
    let _kwargs = ptr_as_ref(kwargs);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::M);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_M() missing 1 required positional argument: 'num'");
}

// to_G(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_G(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::G);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_G() missing 1 required positional argument: 'num'");
}

// to_T(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_T(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::T);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_T() missing 1 required positional argument: 'num'");
}

// to_P(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_P(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::P);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_P() missing 1 required positional argument: 'num'");
}

// to_Ki(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_Ki(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::Ki);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_Ki() missing 1 required positional argument: 'num'");
}

// to_Mi(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_Mi(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::Mi);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_Mi() missing 1 required positional argument: 'num'");
}

// to_Gi(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_Gi(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::Gi);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_Gi() missing 1 required positional argument: 'num'");
}

// to_Ti(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_Ti(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::Ti);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_Ti() missing 1 required positional argument: 'num'");
}

// to_Pi(num: int) -> str

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_units_to_Pi(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let ctx = mut_ptr_as_ref(ctx);
    let args = ptr_as_ref(args);

    if let Some(num) = args.arg_i_num(0, None) {
        let s = to_unit(num, to_unit_suffix::Pi);
        return ValueRef::str(s.as_ref()).into_raw(ctx);
    }
    panic!("to_Pi() missing 1 required positional argument: 'num'");
}

fn to_unit(num: f64, suffix: to_unit_suffix) -> String {
    match suffix {
        to_unit_suffix::n => format!("{}{:?}", (num / 1e-09) as i64, suffix),
        to_unit_suffix::u => format!("{}{:?}", (num / 1e-06) as i64, suffix),
        to_unit_suffix::m => format!("{}{:?}", (num / 0.001) as i64, suffix),
        to_unit_suffix::k => format!("{}{:?}", (num / 1_000.0) as i64, suffix),
        to_unit_suffix::K => format!("{}{:?}", (num / 1_000.0) as i64, suffix),
        to_unit_suffix::M => format!("{}{:?}", (num / 1_000_000.0) as i64, suffix),
        to_unit_suffix::G => format!("{}{:?}", (num / 1_000_000_000.0) as i64, suffix),
        to_unit_suffix::T => format!("{}{:?}", (num / 1_000_000_000_000.0) as i64, suffix),
        to_unit_suffix::P => format!("{}{:?}", (num / 1_000_000_000_000_000.0) as i64, suffix),
        to_unit_suffix::Ki => format!("{}{:?}", (num / 1024.0) as i64, suffix),
        to_unit_suffix::Mi => format!("{}{:?}", (num / (1024.0 * 1024.0)) as i64, suffix),
        to_unit_suffix::Gi => format!("{}{:?}", (num / (1024.0 * 1024.0 * 1024.0)) as i64, suffix),
        to_unit_suffix::Ti => format!(
            "{}{:?}",
            (num / (1024.0 * 1024.0 * 1024.0 * 1024.0)) as i64,
            suffix
        ),
        to_unit_suffix::Pi => format!(
            "{}{:?}",
            (num / (1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0)) as i64,
            suffix
        ),
    }
}

/// Parse and return number based on input quantity.
///
/// Supported suffixes:
/// SI: n | u | m | k | K | M | G | T | P
/// IEC: Ki | Mi | Gi | Ti | Pi
///
/// Input:
/// quantity: &str.
///
/// Returns:
/// result: i64
pub fn to_quantity(quantity: &str) -> i64 {
    let quantity_len = quantity.len();
    let mut number = quantity;
    let mut suffix: Option<&str> = None;
    if quantity_len >= 2 && &quantity[quantity_len - 1..] == IEC_SUFFIX {
        if EXPONENTS.contains_key(&quantity[quantity_len - 2..quantity_len - 1]) {
            number = &quantity[..quantity_len - 2];
            suffix = Some(&quantity[quantity_len - 2..]);
        }
    } else if quantity_len >= 1 && EXPONENTS.contains_key(&quantity[quantity_len - 1..]) {
        number = &quantity[..quantity_len - 1];
        suffix = Some(&quantity[quantity_len - 1..]);
    }
    if number.is_empty() {
        panic!("number can't be empty")
    }
    let number: i64 = number.parse().unwrap();
    if suffix.is_none() {
        return number;
    }
    let suffix = suffix.unwrap();
    validate_unit(&suffix[0..1]);
    let base: i64 = if suffix.ends_with(IEC_SUFFIX) {
        1024
    } else {
        1000
    };
    let exponent = EXPONENTS.get(&suffix[0..1]).unwrap();
    number * (base.pow(*exponent as u32))
}

/// Calculate number based on value and binary suffix.
///
/// Supported suffixes:
/// SI: n | u | m | k | K | M | G | T | P
/// IEC: Ki | Mi | Gi | Ti | Pi
///
/// Input:
/// value: int.
/// suffix: str.
///
/// Returns:
/// int
///
/// Raises:
/// ValueError on invalid or unknown suffix
pub fn cal_num(value: i64, unit: &str) -> f64 {
    validate_unit(unit);
    let mut base: f64 = 1000.0;
    let mut unit = unit;
    if unit.len() > 1 && &unit[1..2] == IEC_SUFFIX {
        base = 1024.0;
        unit = &unit[0..1];
    }
    let exponent = EXPONENTS
        .get(unit)
        .unwrap_or_else(|| panic!("invalid unit {unit}"));
    value as f64 * base.powf(*exponent as f64)
}

#[inline]
pub fn real_uint_value(raw: i64, unit: &str) -> i128 {
    (raw as i128) * (u64_unit_value(unit) as i128)
}

/// Validate the unit is valid
pub fn validate_unit(unit: &str) {
    if unit.is_empty() || unit.len() > 2 {
        panic!("Invalid suffix {unit}");
    }
    if INVALID_UNITS.contains(&unit) {
        panic!("Invalid suffix {unit}");
    }
    if !EXPONENTS.contains_key(&unit[..1]) {
        panic!("Invalid suffix {unit}");
    }
}

pub fn f64_unit_value(unit: &str) -> f64 {
    match unit {
        "n" => 1e-09,
        "u" => 1e-06,
        "m" => 0.001,
        _ => 1_f64,
    }
}

pub fn u64_unit_value(unit: &str) -> u64 {
    match unit {
        "k" => 1_000,
        "K" => 1_000,
        "M" => 1_000_000,
        "G" => 1_000_000_000,
        "T" => 1_000_000_000_000,
        "P" => 1_000_000_000_000_000,
        "Ki" => 1024,
        "Mi" => 1048576,
        "Gi" => 1073741824,
        "Ti" => 1099511627776,
        "Pi" => 1125899906842624,
        _ => 1_u64,
    }
}
