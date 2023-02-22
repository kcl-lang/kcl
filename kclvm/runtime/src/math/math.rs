//! KCL math system module
//!
//! Copyright 2021 The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

extern crate num_integer;

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

// https://docs.python.org/3/library/math.html
// https://doc.rust-lang.org/std/primitive.f64.html
// https://github.com/RustPython/RustPython/blob/main/stdlib/src/math.rs

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_ceil(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Int(x);
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Int(x.ceil() as i64);
    }

    panic!("ceil() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_factorial(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    fn factorial(num: i64) -> i64 {
        if num >= 21 {
            // overflow: 21! = 51090942171709440000
            // MaxInt64:       9223372036854775807
            panic!("factorial() result overflow");
        }
        match num {
            0 => 1,
            1 => 1,
            _ => factorial(num - 1) * num,
        }
    }

    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        if x >= 0 {
            return kclvm_value_Int(factorial(x));
        }
    }
    if let Some(x) = args.arg_i_float(0, None) {
        if x >= 0.0 && (x as i64 as f64) == x {
            return kclvm_value_Float(factorial(x as i64) as f64);
        }
    }
    if args.args_len() > 0 {
        panic!("factorial() only accepts integral values")
    }
    panic!("factorial() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_floor(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Int(x);
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Float(x.floor());
    }

    panic!("floor() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_gcd(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(a) = args.arg_i_int(0, None) {
        if let Some(b) = args.arg_i_int(1, None) {
            return kclvm_value_Int(num_integer::gcd(a, b));
        }
    }

    panic!(
        "gcd() takes exactly two arguments ({} given)",
        args.args_len()
    );
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_isfinite(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(_x) = args.arg_i_int(0, None) {
        return kclvm_value_Bool(true as i8);
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Bool(x.is_finite() as i8);
    }

    panic!("isfinite() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_isinf(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(_x) = args.arg_i_int(0, None) {
        return kclvm_value_Bool(false as i8);
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Bool(x.is_infinite() as i8);
    }

    panic!("isinf() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_isnan(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(_x) = args.arg_i_int(0, None) {
        return kclvm_value_Bool(false as i8);
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Bool(x.is_nan() as i8);
    }

    panic!("isnan() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_modf(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        let list = ValueRef::list_float(&[0.0, x as f64]);
        return list.into_raw();
    }
    if let Some(x) = args.arg_i_float(0, None) {
        if !x.is_finite() {
            if x.is_infinite() {
                let list = ValueRef::list_float(&[0.0_f64.copysign(x), x]);
                return list.into_raw();
            } else if x.is_nan() {
                let list = ValueRef::list_float(&[x, x]);
                return list.into_raw();
            }
        }
        let list = ValueRef::list_float(&[x.fract(), x.trunc()]);
        return list.into_raw();
    }

    panic!("modf() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_exp(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Float((x as f64).exp());
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Float(x.exp());
    }
    panic!("exp() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_expm1(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Float((x as f64).exp_m1());
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Float(x.exp_m1());
    }
    panic!("expm1() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_log(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        if let Some(base) = args.arg_i_float(1, Some(std::f64::consts::E)) {
            return kclvm_value_Int((x as f64).log(base) as i64);
        }
    }
    if let Some(x) = args.arg_i_float(0, None) {
        if let Some(base) = args.arg_i_float(1, Some(std::f64::consts::E)) {
            return kclvm_value_Float(x.log(base));
        }
    }
    panic!("log() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_log1p(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Float(((x + 1) as f64).ln_1p());
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Float((x + 1.0).ln_1p());
    }
    panic!("log1p() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_log2(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Int((x as f64).log2() as i64);
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Float(x.log2());
    }
    panic!("log2() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_log10(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Float((x as f64).log10());
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Float(x.log10());
    }
    panic!("log10() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_pow(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        if let Some(n) = args.arg_i_int(1, None) {
            if n < 0 {
                return kclvm_value_Float((x as f64).powf(n as f64));
            } else {
                return kclvm_value_Int(x.pow(n as u32));
            }
        }
        if let Some(n) = args.arg_i_float(1, None) {
            return kclvm_value_Float((x as f64).powf(n));
        }
    }
    if let Some(x) = args.arg_i_float(0, None) {
        if let Some(n) = args.arg_i_int(1, None) {
            return kclvm_value_Float(x.powi(n as i32));
        }
        if let Some(n) = args.arg_i_float(1, None) {
            return kclvm_value_Float(x.powf(n));
        }
    }
    panic!("pow() takes exactly one argument (0 given)");
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_math_sqrt(
    _ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);

    if let Some(x) = args.arg_i_int(0, None) {
        return kclvm_value_Float((x as f64).sqrt());
    }
    if let Some(x) = args.arg_i_float(0, None) {
        return kclvm_value_Float(x.sqrt());
    }
    panic!("sqrt() takes exactly one argument (0 given)");
}
