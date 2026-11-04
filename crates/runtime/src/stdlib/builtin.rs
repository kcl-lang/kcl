//! Copyright The KCL Authors. All rights reserved.

use std::{collections::HashSet, num::IntErrorKind, ops::Index};

use crate::*;

impl Context {
    pub fn builtin_option_init(&mut self, key: &str, value: &str) {
        if let Ok(x) = ValueRef::from_json(self, value) {
            self.option_values.insert(key.to_string(), x);
            return;
        }
        self.option_values
            .insert(key.to_string(), ValueRef::str(value));
    }

    pub fn builtin_option_reset(&mut self) {
        self.option_values.clear();
    }
}

/// Truncate a float towards zero for `int()`. NaN and values whose result does
/// not fit in an i64 are errors instead of saturating.
fn float_to_int(ctx: &mut Context, v: f64) -> i64 {
    if v.is_nan() {
        panic!("cannot convert float NaN to integer");
    }
    let v = v.trunc();
    // i64::MIN as f64 is exactly -2^63, and i64::MAX as f64 rounds up to 2^63.
    if v < i64::MIN as f64 || v >= i64::MAX as f64 {
        ctx.set_err_type(&RuntimeErrorType::IntOverflow);
        panic!("{v}: A 64 bit integer overflow");
    }
    v as i64
}

impl ValueRef {
    pub fn any_true(&self) -> bool {
        match &*self.rc.borrow() {
            Value::list_value(list) => {
                for x in list.values.iter() {
                    if x.is_truthy() {
                        return true;
                    }
                }
                false
            }
            Value::dict_value(dict) => {
                for (_k, x) in dict.values.iter() {
                    if x.is_truthy() {
                        return true;
                    }
                }
                false
            }
            Value::schema_value(schema) => {
                for (_k, x) in schema.config.values.iter() {
                    if x.is_truthy() {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn all_true(&self) -> bool {
        match &*self.rc.borrow() {
            Value::list_value(list) => {
                for x in list.values.iter() {
                    if !x.is_truthy() {
                        return false;
                    }
                }
                true
            }
            Value::dict_value(dict) => {
                for (_k, x) in dict.values.iter() {
                    if !x.is_truthy() {
                        return false;
                    }
                }
                true
            }
            Value::schema_value(schema) => {
                for (_k, x) in schema.config.values.iter() {
                    if x.is_truthy() {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    #[allow(clippy::mutable_key_type)]
    pub fn isunique(&self) -> bool {
        match &*self.rc.borrow() {
            Value::list_value(list) => {
                let mut set: HashSet<&ValueRef> = HashSet::new();
                for x in list.values.iter() {
                    if set.contains(x) {
                        return false;
                    }
                    set.insert(x);
                }
                true
            }
            _ => false,
        }
    }

    pub fn sorted(&self, reverse: Option<&ValueRef>) -> ValueRef {
        let reverse = if let Some(v) = reverse {
            v.as_bool()
        } else {
            false
        };
        match &*self.rc.borrow() {
            Value::str_value(s) => {
                let mut list = ListValue::default();
                for c in s.chars() {
                    list.values
                        .push(Self::from(Value::str_value(String::from(c))));
                }
                let values = &mut list.values;
                if reverse {
                    values.sort_by(|a, b| b.partial_cmp(a).unwrap());
                } else {
                    values.sort();
                }
                Self::from(Value::list_value(Box::new(list)))
            }
            Value::list_value(_) => {
                let mut list = self.deep_copy();
                {
                    let mut list_ref = list.as_list_mut_ref();
                    let values = &mut list_ref.values;
                    if reverse {
                        values.sort_by(|a, b| b.partial_cmp(a).unwrap());
                    } else {
                        values.sort();
                    }
                }
                list
            }
            Value::dict_value(dict) => {
                let mut list = ListValue::default();
                for (k, _v) in dict.values.iter() {
                    list.values
                        .push(Self::from(Value::str_value(k.to_string())));
                }
                let values = &mut list.values;
                if reverse {
                    values.sort_by(|a, b| b.partial_cmp(a).unwrap());
                } else {
                    values.sort();
                }
                Self::from(Value::list_value(Box::new(list)))
            }
            _ => panic!("sorted only for str|list|dict type"),
        }
    }

    pub fn convert_to_int(&self, ctx: &mut Context, base: Option<&ValueRef>) -> ValueRef {
        let strict_range_check_i32 = ctx.cfg.strict_range_check;
        let strict_range_check_i64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match &*self.rc.borrow() {
            Value::int_value(v) => ValueRef::int(*v),
            Value::float_value(v) => ValueRef::int(float_to_int(ctx, *v)),
            Value::unit_value(v, raw, unit) => {
                let v_i128 = crate::real_uint_value(*raw, unit);
                let int_32_overflow = strict_range_check_i32 && v_i128 != ((v_i128 as i32) as i128);
                let int_64_overflow = strict_range_check_i64 && v_i128 != ((v_i128 as i64) as i128);

                if ctx.cfg.debug_mode {
                    if int_32_overflow {
                        ctx.set_err_type(&RuntimeErrorType::IntOverflow);

                        panic!("{v_i128}: A 32 bit integer overflow");
                    }
                    if int_64_overflow {
                        ctx.set_err_type(&RuntimeErrorType::IntOverflow);

                        panic!("{v_i128}: A 64 bit integer overflow");
                    }
                }

                ValueRef::int(float_to_int(ctx, *v))
            }
            Value::bool_value(v) => ValueRef::int(*v as i64),
            Value::str_value(v) => {
                let base = if let Some(v) = base { v.as_int() } else { 10 };
                // When base is explicitly provided and not 10, don't use to_quantity
                // because it's designed for decimal-based unit suffixes (Ki, Mi, Gi, etc.)
                if base == 10 {
                    return match to_quantity(v.as_str()) {
                        Ok(value) => match i64::try_from(value) {
                            Ok(value) => ValueRef::int(value),
                            Err(_) => {
                                panic_i64_overflow!(ctx, value);
                            }
                        },
                        Err(err)
                            if matches!(
                                err.kind(),
                                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow
                            ) =>
                        {
                            ctx.set_err_type(&RuntimeErrorType::IntOverflow);
                            panic!("{v}: A 64 bit integer overflow")
                        }
                        Err(_) => panic!("invalid literal for int() with base 10: '{self}'"),
                    };
                }
                // For non-base-10 parsing, strip common prefixes (0x, 0o, 0b) if present
                // This allows parsing strings like "0xe4" with base=16
                // Handle negative numbers by stripping the sign first
                let s = v.as_str();
                let is_negative = s.starts_with('-');
                let s = if is_negative { &s[1..] } else { s };
                let s = s
                    .strip_prefix("0x")
                    .or_else(|| s.strip_prefix("0X"))
                    .or_else(|| s.strip_prefix("0o"))
                    .or_else(|| s.strip_prefix("0O"))
                    .or_else(|| s.strip_prefix("0b"))
                    .or_else(|| s.strip_prefix("0B"))
                    .unwrap_or(s);
                let number_str = if is_negative {
                    format!("-{s}")
                } else {
                    s.to_string()
                };
                let v: i64 =
                    i64::from_str_radix(number_str.as_str(), base as u32).unwrap_or_else(|_| {
                        panic!("invalid literal for int() with base {base}: '{self}'")
                    });
                ValueRef::int(v)
            }
            _ => panic!(
                "int() argument must be a string or a number, not '{}'",
                self.type_str()
            ),
        }
    }

    pub fn convert_to_float(&self, ctx: &mut Context) -> ValueRef {
        let strict_range_check_i32 = ctx.cfg.strict_range_check;
        let strict_range_check_i64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match &*self.rc.borrow() {
            Value::int_value(v) => ValueRef::float(*v as f64),
            Value::float_value(v) => {
                let float32_overflow = strict_range_check_i32 && (*v as f32).is_infinite();
                let float64_overflow = strict_range_check_i64 && (*v).is_infinite();

                if float32_overflow {
                    ctx.set_err_type(&RuntimeErrorType::FloatOverflow);

                    panic!("inf: A 32-bit floating point number overflow");
                }
                if float64_overflow {
                    ctx.set_err_type(&RuntimeErrorType::FloatOverflow);

                    panic!("inf: A 64-bit floating point number overflow");
                }
                ValueRef::float(*v)
            }
            Value::unit_value(v, _, _) => ValueRef::float(*v),
            Value::bool_value(v) => ValueRef::float((*v as i64) as f64),
            Value::str_value(v) => {
                let v: f64 = v.parse().unwrap_or_else(|_| {
                    panic!("invalid literal for float() with base 10: '{self}'")
                });
                let float32_overflow = strict_range_check_i32 && (v as f32).is_infinite();
                let float64_overflow = strict_range_check_i64 && (v).is_infinite();
                if float32_overflow {
                    ctx.set_err_type(&RuntimeErrorType::FloatOverflow);

                    panic!("inf: A 32-bit floating point number overflow");
                }
                if float64_overflow {
                    ctx.set_err_type(&RuntimeErrorType::FloatOverflow);

                    panic!("inf: A 64-bit floating point number overflow");
                }
                ValueRef::float(v)
            }
            _ => panic!("invalid literal for float() with base 10: '{self}'"),
        }
    }

    pub fn max_value(&self) -> ValueRef {
        self.filter(|x, y| x.cmp_greater_than(y))
    }

    pub fn min_value(&self) -> ValueRef {
        self.filter(|x, y| x.cmp_less_than(y))
    }

    pub fn filter(&self, filter: fn(&ValueRef, &ValueRef) -> bool) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(s) => {
                if s.is_empty() {
                    panic!("arg is an empty str");
                }
                let mut result = s.chars().next().unwrap();
                for ch in s.chars() {
                    if filter(
                        &ValueRef::str(&ch.to_string()),
                        &ValueRef::str(&result.to_string()),
                    ) {
                        result = ch;
                    }
                }
                ValueRef::str(&result.to_string())
            }
            Value::list_value(list) => {
                if list.values.is_empty() {
                    panic!("arg is an empty list");
                }
                let mut result = list.values.first().unwrap();
                for val in list.values.iter() {
                    if filter(val, result) {
                        result = val;
                    }
                }
                result.clone()
            }
            Value::dict_value(dict) => {
                if dict.values.is_empty() {
                    panic!("arg is an empty dict");
                }
                let keys: Vec<String> = dict.values.keys().map(|s| s.to_string()).collect();
                let mut result = keys.first().unwrap();
                for key in keys.iter() {
                    if filter(&ValueRef::str(key), &ValueRef::str(result)) {
                        result = key;
                    }
                }
                ValueRef::str(result)
            }
            Value::schema_value(schema) => {
                if schema.config.values.is_empty() {
                    panic!("arg is an empty dict");
                }
                let keys: Vec<String> =
                    schema.config.values.keys().map(|s| s.to_string()).collect();
                let mut result = keys.first().unwrap();
                for key in keys.iter() {
                    if filter(&ValueRef::str(key), &ValueRef::str(result)) {
                        result = key;
                    }
                }
                ValueRef::str(result)
            }
            _ => panic!("{} object is not iterable", self.type_str()),
        }
    }

    pub fn hex(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::int_value(val) => {
                if *val == i64::MIN {
                    ValueRef::str("-0x8000000000000000")
                } else if *val >= 0 {
                    ValueRef::str(format!("0x{:X}", val.abs()).to_lowercase().as_str())
                } else {
                    ValueRef::str(format!("-0x{:X}", val.abs()).to_lowercase().as_str())
                }
            }
            _ => ValueRef::undefined(),
        }
    }

    pub fn oct(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::int_value(val) => {
                if *val == i64::MIN {
                    ValueRef::str("-01000000000000000000000")
                } else if *val >= 0 {
                    ValueRef::str(format!("0o{:o}", val.abs()).as_str())
                } else {
                    ValueRef::str(format!("-0o{:o}", val.abs()).as_str())
                }
            }
            _ => ValueRef::undefined(),
        }
    }

    pub fn bin(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::int_value(val) => {
                if *val == i64::MIN {
                    ValueRef::str(
                        "-0b1000000000000000000000000000000000000000000000000000000000000000",
                    )
                } else if *val >= 0 {
                    ValueRef::str(format!("0b{:b}", val.abs()).as_str())
                } else {
                    ValueRef::str(format!("-0b{:b}", val.abs()).as_str())
                }
            }
            _ => ValueRef::undefined(),
        }
    }

    pub fn sum(&self, ctx: &mut Context, init_value: &ValueRef) -> ValueRef {
        match &*self.rc.borrow() {
            Value::list_value(list) => {
                let mut result = match &*init_value.rc.borrow() {
                    Value::str_value(_str) => panic!("sum() can't sum strings"),
                    _ => init_value.clone(),
                };
                for val in list.values.iter() {
                    //xx_bin_aug_add() might modify the value of init_value
                    result = result.bin_add(ctx, val)
                }
                result
            }
            _ => ValueRef::undefined(),
        }
    }

    pub fn abs(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::int_value(val) => ValueRef::int(val.abs()),
            Value::float_value(val) => ValueRef::float(val.abs()),
            _ => ValueRef::undefined(),
        }
    }

    pub fn ord(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::str_value(str) => {
                let string_len = str.chars().count();
                if string_len != 1 {
                    panic!(
                        " ord() expected string of length 1, but string of length {} found",
                        string_len
                    )
                }
                let ch = str.chars().next().unwrap();
                ValueRef::int(ch as i64)
            }
            _ => ValueRef::undefined(),
        }
    }

    pub fn zip(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::list_value(list) => {
                let mut iters = Vec::new();
                for val in list.values.iter() {
                    iters.push(val.iter());
                }
                let mut zip_lists = Vec::new();
                let mut pre_len = 0;
                loop {
                    for (i, iter) in iters.iter_mut().enumerate() {
                        if iter.is_end() {
                            if i != 0 {
                                zip_lists.pop();
                            }
                            let mut result = ValueRef::list(None);
                            for val in zip_lists.iter() {
                                result.list_append(val);
                            }
                            return result;
                        }
                        if zip_lists.len() <= pre_len {
                            zip_lists.push(ValueRef::list(None));
                        }
                        zip_lists[pre_len].list_append(iter.next(list.values.index(i)).unwrap());
                    }
                    pre_len += 1;
                }
            }
            _ => ValueRef::undefined(),
        }
    }
}

pub fn list(iterable: Option<&ValueRef>) -> ValueRef {
    match iterable {
        Some(val) => {
            let mut iter = val.iter();
            let mut result = ValueRef::list(None);
            while !iter.is_end() {
                result.list_append(iter.next(val).unwrap());
            }
            result
        }
        _ => ValueRef::list(None),
    }
}

pub fn dict(ctx: &mut Context, iterable: Option<&ValueRef>) -> ValueRef {
    match iterable {
        Some(val) => {
            let mut iter = val.iter();
            let mut result = ValueRef::dict(None);
            while !iter.is_end() {
                iter.next(val);
                let elem = iter.cur_val.clone();
                let k = iter.cur_key.clone();
                match &*k.rc.borrow() {
                    Value::str_value(str) => {
                        result.dict_insert(ctx, str.as_str(), &elem, Default::default(), None);
                    }
                    _ => {
                        let mut elem_iter = elem.iter();
                        if elem_iter.len != 2 {
                            panic!(
                                "dictionary update sequence element #{} has length {}; 2 is required",
                                iter.pos - 1,
                                elem_iter.len
                            );
                        }
                        let k = elem_iter.next(val).unwrap().to_string();
                        let v = elem_iter.next(val).unwrap();
                        result.dict_insert(ctx, k.as_str(), v, Default::default(), None);
                    }
                };
            }
            result
        }
        _ => ValueRef::dict(None),
    }
}

pub fn range(start: &ValueRef, stop: &ValueRef, step: &ValueRef) -> ValueRef {
    match (&*start.rc.borrow(), &*stop.rc.borrow(), &*step.rc.borrow()) {
        (Value::int_value(start), Value::int_value(stop), Value::int_value(step)) => {
            if *step == 0 {
                panic!("range() step argument must not be zero");
            }
            let mut cur = *start;
            let mut list = ValueRef::list(None);
            let cmp = if *step > 0 {
                |x, y| x < y
            } else {
                |x, y| x > y
            };
            while cmp(cur, *stop) {
                list.list_append(&ValueRef::int(cur));
                cur += step;
            }
            list
        }
        _ => ValueRef::undefined(),
    }
}

/// Check if the modular result of a and b is 0
pub fn multiplyof(a: &ValueRef, b: &ValueRef) -> ValueRef {
    match (&*a.rc.borrow(), &*b.rc.borrow()) {
        (Value::int_value(a), Value::int_value(b)) => ValueRef::bool(a % b == 0),
        _ => ValueRef::undefined(),
    }
}

pub fn pow(ctx: &mut Context, x: &ValueRef, y: &ValueRef, z: &ValueRef) -> ValueRef {
    match &*z.rc.borrow() {
        Value::int_value(z) => {
            if *z == 0 {
                panic!("pow() 3rd argument cannot be 0")
            }
            match (&*x.rc.borrow(), &*y.rc.borrow()) {
                (Value::int_value(x), Value::int_value(y)) => match (*y).cmp(&0) {
                    std::cmp::Ordering::Equal => ValueRef::int(1).bin_mod(&ValueRef::int(*z)),
                    std::cmp::Ordering::Greater => {
                        let mut ans = 1_i128;
                        let mut base = *x as i128;
                        let m = *z as i128;
                        let mut exp = *y as i128;
                        while exp > 0 {
                            if exp % 2 == 1 {
                                ans = (ans * base) % m;
                            }
                            base = (base * base) % m;
                            exp /= 2;
                        }
                        // Rust's `%` keeps the sign of the dividend, so apply KCL's `%`
                        // to give the result the sign of `z`, as `x ** y % z` does.
                        ValueRef::int(ans as i64).bin_mod(&ValueRef::int(*z))
                    }
                    std::cmp::Ordering::Less => {
                        panic!("pow() 2nd argument cannot be negative when 3rd argument specified")
                    }
                },
                _ => panic!("pow() 3rd argument not allowed unless all arguments are integers"),
            }
        }
        _ => match (&*x.rc.borrow(), &*y.rc.borrow()) {
            // Non-negative int exponents give an exact int result, as `x ** y` does.
            (Value::int_value(_), Value::int_value(exp)) if *exp >= 0 => x.bin_pow(ctx, y),
            (Value::int_value(x), Value::int_value(y)) => {
                ValueRef::float((*x as f64).powf(*y as f64))
            }
            (Value::int_value(x), Value::float_value(y)) => ValueRef::float((*x as f64).powf(*y)),
            (Value::float_value(x), Value::int_value(y)) => ValueRef::float((*x).powf(*y as f64)),
            (Value::float_value(x), Value::float_value(y)) => ValueRef::float((*x).powf(*y)),
            _ => ValueRef::undefined(),
        },
    }
}

pub fn round(number: &ValueRef, ndigits: &ValueRef) -> ValueRef {
    match &*ndigits.rc.borrow() {
        Value::int_value(ndigits) => match &*number.rc.borrow() {
            Value::int_value(number) => ValueRef::int(round_int(*number, *ndigits)),
            Value::float_value(number) => {
                if *ndigits == 0 {
                    ValueRef::float(number.round_ties_even())
                } else {
                    let (y, pow1, pow2) = if *ndigits >= 0 {
                        // according to cpython: pow1 and pow2 are each safe from overflow, but
                        //                       pow1*pow2 ~= pow(10.0, ndigits) might overflow
                        let (pow1, pow2) = if *ndigits > 22 {
                            ((10.0_f64).powf((*ndigits - 22) as f64), 1e22)
                        } else {
                            ((10.0_f64).powf(*ndigits as f64), 1.0)
                        };
                        let y = ((*number) * pow1) * pow2;
                        if !y.is_finite() {
                            return ValueRef::float(*number);
                        }
                        (y, pow1, Some(pow2))
                    } else {
                        let pow1 = (10.0_f64).powf((-ndigits) as f64);
                        ((*number) / pow1, pow1, None)
                    };
                    let z = y.round();
                    #[allow(clippy::float_cmp)]
                    let z = if (y - z).abs() == 0.5 {
                        2.0 * (y / 2.0).round()
                    } else {
                        z
                    };
                    let z = if let Some(pow2) = pow2 {
                        // ndigits >= 0
                        (z / pow2) / pow1
                    } else {
                        z * pow1
                    };
                    if !z.is_finite() {
                        // overflow
                        ValueRef::undefined()
                    } else {
                        ValueRef::float(z)
                    }
                }
            }
            _ => ValueRef::undefined(),
        },
        _ => match &*number.rc.borrow() {
            Value::int_value(number) => ValueRef::int(*number),
            Value::float_value(number) => ValueRef::int(number.round_ties_even() as i64),
            _ => ValueRef::undefined(),
        },
    }
}

/// Round an integer to `ndigits` decimal digits, rounding half to even.
/// A non-negative `ndigits` leaves the integer unchanged.
fn round_int(number: i64, ndigits: i64) -> i64 {
    if ndigits >= 0 {
        return number;
    }
    // Half of 10^20 is already larger than any i64, so every i64 rounds to 0.
    if ndigits < -19 {
        return 0;
    }
    let pow = 10_i128.pow((-ndigits) as u32);
    let number = number as i128;
    let mut quotient = number.div_euclid(pow);
    let remainder = number.rem_euclid(pow);
    if remainder * 2 > pow || (remainder * 2 == pow && quotient % 2 != 0) {
        quotient += 1;
    }
    match i64::try_from(quotient * pow) {
        Ok(result) => result,
        Err(_) => panic!("round() result overflow"),
    }
}

pub fn type_of(x: &ValueRef, full_name: &ValueRef) -> ValueRef {
    if x.is_schema() {
        if full_name.is_truthy() {
            let schema = x.as_schema();
            let full_type_str = if let Some(v) = schema.pkgpath.strip_prefix('@') {
                v
            } else {
                schema.pkgpath.as_str()
            };
            let mut result = String::new();
            if full_type_str != MAIN_PKG_PATH {
                result += full_type_str;
                result += ".";
            }
            result += &schema.name;
            return ValueRef::str(result.as_str());
        }
    } else if x.is_none() {
        return ValueRef::str("None");
    }
    ValueRef::str(x.type_str().as_str())
}

#[cfg(test)]
mod test_builtin {

    use crate::*;

    #[test]
    fn test_sorted() {
        assert_panic("sorted only for str|list|dict type", || {
            let _ = ValueRef::int(1).sorted(None);
        });
    }

    #[test]
    fn test_convert_float_to_int() {
        let mut ctx = Context::new();
        for (v, expected) in [
            (1.9, 1),
            (-1.9, -1),
            (9.2e18, 9200000000000000000),
            (-9223372036854775808.0, i64::MIN),
        ] {
            let result = ValueRef::float(v).convert_to_int(&mut ctx, None);
            assert_eq!(expected, result.as_int(), "int({v})");
        }
        for v in [9223372036854775808.0, -9.3e18, 1e300, f64::INFINITY] {
            assert_panic("A 64 bit integer overflow", || {
                let mut ctx = Context::new();
                let _ = ValueRef::float(v).convert_to_int(&mut ctx, None);
            });
        }
        assert_panic("cannot convert float NaN to integer", || {
            let mut ctx = Context::new();
            let _ = ValueRef::float(f64::NAN).convert_to_int(&mut ctx, None);
        });
    }

    #[test]
    fn test_pow() {
        let mut ctx = Context::new();
        let ctx = &mut ctx;
        assert_eq!(
            8,
            builtin::pow(ctx, &ValueRef::int(2), &ValueRef::int(3), &ValueRef::none()).as_int()
        );
        assert_eq!(
            3,
            builtin::pow(ctx, &ValueRef::int(2), &ValueRef::int(3), &ValueRef::int(5)).as_int()
        );
        assert_eq!(
            (2.0_f64).powf(0.5),
            builtin::pow(
                ctx,
                &ValueRef::int(2),
                &ValueRef::float(0.5),
                &ValueRef::none()
            )
            .as_float()
        );
        assert_eq!(
            182523810312617,
            builtin::pow(
                ctx,
                &ValueRef::int(141592653589793),
                &ValueRef::int(238462643383279),
                &ValueRef::int(502884197169399)
            )
            .as_int()
        )
    }

    #[test]
    fn test_pow_int_is_exact() {
        let mut ctx = Context::new();
        let result = builtin::pow(
            &mut ctx,
            &ValueRef::int(3),
            &ValueRef::int(39),
            &ValueRef::none(),
        );
        assert!(result.is_int());
        assert_eq!(4052555153018976267, result.as_int());
        let result = builtin::pow(
            &mut ctx,
            &ValueRef::int(2),
            &ValueRef::int(-1),
            &ValueRef::none(),
        );
        assert_eq!(0.5, result.as_float());
        assert_panic("A 64 bit integer overflow", || {
            let mut ctx = Context::new();
            let _ = builtin::pow(
                &mut ctx,
                &ValueRef::int(2),
                &ValueRef::int(64),
                &ValueRef::none(),
            );
        });
    }

    #[test]
    fn test_pow_mod_sign() {
        let mut ctx = Context::new();
        for (x, y, z, expected) in [
            (-2, 3, 5, 2),
            (2, 3, -5, -2),
            (-2, 3, -5, -3),
            (-3, 2, 5, 4),
            (7, 0, 1, 0),
            (7, 0, -3, -2),
        ] {
            let result = builtin::pow(
                &mut ctx,
                &ValueRef::int(x),
                &ValueRef::int(y),
                &ValueRef::int(z),
            );
            assert_eq!(expected, result.as_int(), "pow({x}, {y}, {z})");
        }
    }

    #[test]
    fn test_convert_str_to_int() {
        let mut ctx = Context::new();
        for (s, expected) in [("10", 10), ("1024Mi", 1073741824), ("1500m", 1)] {
            let result = ValueRef::str(s).convert_to_int(&mut ctx, None);
            assert_eq!(expected, result.as_int(), "int({s:?})");
        }
        for s in ["", "abc", "1ki", "5é"] {
            assert_panic("invalid literal for int() with base 10", || {
                let mut ctx = Context::new();
                let _ = ValueRef::str(s).convert_to_int(&mut ctx, None);
            });
        }
        for s in ["9000Pi", "99999999999999999999"] {
            assert_panic("A 64 bit integer overflow", || {
                let mut ctx = Context::new();
                let _ = ValueRef::str(s).convert_to_int(&mut ctx, None);
            });
        }
    }

    #[test]
    fn test_zip() {
        let mut list1 = ValueRef::list(Some(&[
            &ValueRef::int(1),
            &ValueRef::int(2),
            &ValueRef::int(3),
        ]));
        let mut list2 = ValueRef::list(Some(&[
            &ValueRef::int(6),
            &ValueRef::int(7),
            &ValueRef::int(8),
        ]));

        let mut lists = ValueRef::list(Some(&[&list1, &list2]));

        let zip_list1 = ValueRef::list(Some(&[&ValueRef::int(1), &ValueRef::int(6)]));

        let zip_list2 = ValueRef::list(Some(&[&ValueRef::int(2), &ValueRef::int(7)]));

        let zip_list3 = ValueRef::list(Some(&[&ValueRef::int(3), &ValueRef::int(8)]));

        let zip_lists = ValueRef::list(Some(&[&zip_list1, &zip_list2, &zip_list3]));

        assert!(zip_lists.cmp_equal(&lists.zip()));

        list1 = ValueRef::list(Some(&[
            &ValueRef::int(1),
            &ValueRef::int(2),
            &ValueRef::int(3),
            &ValueRef::int(4),
        ]));

        lists = ValueRef::list(Some(&[&list1, &list2]));

        assert!(zip_lists.cmp_equal(&lists.zip()));

        list1 = ValueRef::list(Some(&[
            &ValueRef::int(1),
            &ValueRef::int(2),
            &ValueRef::int(3),
        ]));

        list2 = ValueRef::list(Some(&[
            &ValueRef::int(6),
            &ValueRef::int(7),
            &ValueRef::int(8),
            &ValueRef::int(9),
        ]));

        lists = ValueRef::list(Some(&[&list1, &list2]));

        assert!(zip_lists.cmp_equal(&lists.zip()));
    }

    #[test]
    fn test_round() {
        assert_eq!(
            1,
            builtin::round(&ValueRef::int(1), &ValueRef::none()).as_int()
        );
        assert_eq!(
            1,
            builtin::round(&ValueRef::float(1.4), &ValueRef::none()).as_int()
        );
        assert_eq!(
            2,
            builtin::round(&ValueRef::float(1.5), &ValueRef::none()).as_int()
        );

        assert_eq!(
            1.6,
            builtin::round(&ValueRef::float(1.5555), &ValueRef::int(1)).as_float()
        );
        assert_eq!(
            1.56,
            builtin::round(&ValueRef::float(1.5555), &ValueRef::int(2)).as_float()
        );

        assert_eq!(
            2,
            builtin::round(&ValueRef::float(1.5555), &ValueRef::none()).as_int()
        );
        assert_eq!(
            2.0,
            builtin::round(&ValueRef::float(1.5555), &ValueRef::int(0)).as_float()
        );
    }

    #[test]
    fn test_round_half_to_even() {
        for (number, expected) in [(0.5, 0), (1.5, 2), (2.5, 2), (-0.5, 0), (-2.5, -2)] {
            let result = builtin::round(&ValueRef::float(number), &ValueRef::none());
            assert!(result.is_int());
            assert_eq!(expected, result.as_int());
        }
        for (number, expected) in [(0.5, 0.0), (2.5, 2.0), (3.5, 4.0), (-2.5, -2.0)] {
            assert_eq!(
                expected,
                builtin::round(&ValueRef::float(number), &ValueRef::int(0)).as_float()
            );
        }
    }

    #[test]
    fn test_round_int_ndigits() {
        for (number, ndigits, expected) in [
            (5, 2, 5),
            (5, 0, 5),
            (1234, -2, 1200),
            (1250, -2, 1200),
            (1350, -2, 1400),
            (-1250, -2, -1200),
            (-1251, -2, -1300),
            (15, -1, 20),
            (25, -1, 20),
            (4_000_000_000_000_000_000, -19, 0),
            (i64::MAX, -20, 0),
            (i64::MIN, i64::MIN, 0),
        ] {
            let result = builtin::round(&ValueRef::int(number), &ValueRef::int(ndigits));
            assert!(result.is_int());
            assert_eq!(expected, result.as_int());
        }
        assert_panic("round() result overflow", || {
            let _ = builtin::round(&ValueRef::int(i64::MAX), &ValueRef::int(-19));
        });
    }

    #[test]
    fn test_ord() {
        assert_eq!(65, ValueRef::str("A").ord().as_int());
        assert_eq!(66, ValueRef::str("B").ord().as_int());
        assert_eq!(67, ValueRef::str("C").ord().as_int());
    }

    #[test]
    fn test_hex() {
        assert_eq!("0x12", ValueRef::int(18).hex().to_string());
        assert_eq!("-0x12", ValueRef::int(-18).hex().to_string());
        assert_eq!(
            "-0x8000000000000000",
            ValueRef::int(i64::MIN).hex().to_string()
        );
    }

    #[test]
    fn test_bin() {
        assert_eq!("0b1000", ValueRef::int(8).bin().to_string());
        assert_eq!("-0b1000", ValueRef::int(-8).bin().to_string());
        assert_eq!(
            "-0b1000000000000000000000000000000000000000000000000000000000000000",
            ValueRef::int(i64::MIN).bin().to_string()
        );
    }

    #[test]
    fn test_multiplyof() {
        assert!(
            builtin::multiplyof(&ValueRef::int(25), &ValueRef::int(5))
                .cmp_equal(&ValueRef::bool(true))
        );
        assert!(
            builtin::multiplyof(&ValueRef::int(25), &ValueRef::int(7))
                .cmp_equal(&ValueRef::bool(false))
        );
    }

    #[test]
    fn test_isunique() {
        //list=[]
        let mut list = ValueRef::list(None);
        assert!(list.isunique());

        //list=[1]
        list = ValueRef::list(Some(&[&ValueRef::int(1)]));
        assert!(list.isunique());

        //list=[1, 2]
        list = ValueRef::list(Some(&[&ValueRef::int(1), &ValueRef::int(2)]));
        assert!(list.isunique());

        //list=[1, 1]
        list = ValueRef::list(Some(&[&ValueRef::int(1), &ValueRef::int(1)]));
        assert!(!list.isunique());

        //list=[1, 1.0]
        list = ValueRef::list(Some(&[&ValueRef::int(1), &ValueRef::float(1.0)]));
        assert!(!list.isunique());

        //list=[1.1, 1.1]
        list = ValueRef::list(Some(&[&ValueRef::float(1.1), &ValueRef::float(1.1)]));
        assert!(!list.isunique());

        //list=["abc","abc"]
        list = ValueRef::list(Some(&[&ValueRef::str("abc"), &ValueRef::str("abc")]));
        assert!(!list.isunique());

        //list=["abc","a${'bc'}"]
        list = ValueRef::list(Some(&[&ValueRef::str("abc"), &ValueRef::str("a${'bc'}")]));
        assert!(list.isunique());
    }

    #[test]
    fn test_range() {
        let mut list = range(&ValueRef::int(1), &ValueRef::int(5), &ValueRef::int(1));
        let mut expect_list = ValueRef::list(Some(&[
            &ValueRef::int(1),
            &ValueRef::int(2),
            &ValueRef::int(3),
            &ValueRef::int(4),
        ]));
        assert!(expect_list.cmp_equal(&list));
        list = range(&ValueRef::int(1), &ValueRef::int(5), &ValueRef::int(2));
        expect_list = ValueRef::list(Some(&[&ValueRef::int(1), &ValueRef::int(3)]));
        assert!(expect_list.cmp_equal(&list));

        list = range(&ValueRef::int(5), &ValueRef::int(1), &ValueRef::int(-1));
        expect_list = ValueRef::list(Some(&[
            &ValueRef::int(5),
            &ValueRef::int(4),
            &ValueRef::int(3),
            &ValueRef::int(2),
        ]));
        assert!(expect_list.cmp_equal(&list));
    }

    #[test]
    fn test_max() {
        let list = ValueRef::list(Some(&[
            &ValueRef::int(1),
            &ValueRef::int(8),
            &ValueRef::int(5),
            &ValueRef::int(12),
            &ValueRef::int(3),
        ]));
        assert!(list.max_value().cmp_equal(&ValueRef::int(12)));
    }

    #[test]
    fn test_min_value() {
        let list = ValueRef::list(Some(&[
            &ValueRef::int(1),
            &ValueRef::int(8),
            &ValueRef::int(5),
            &ValueRef::int(12),
            &ValueRef::int(3),
        ]));
        assert!(list.min_value().cmp_equal(&ValueRef::int(1)));
    }

    #[test]
    fn test_sorted_normal() {
        //list=[]
        let mut list = ValueRef::list(None);
        let mut sorted = ValueRef::list(None);
        for i in 1..6 {
            list.list_append(&ValueRef::int(6 - i));
            sorted.list_append(&ValueRef::int(i));
        }

        assert!(sorted.cmp_equal(&list.sorted(None)));
    }

    #[test]
    #[should_panic]
    fn test_sorted_panic() {
        let list = ValueRef::list(Some(&[&ValueRef::str("abc"), &ValueRef::int(1)]));
        list.sorted(None);
    }
}
