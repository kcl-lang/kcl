//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    pub fn bin_add(&self, ctx: &mut Context, x: &Self) -> Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_add(*a, *b) {
                    panic_i32_overflow!(ctx, *a as i128 + *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_add(*a, *b) {
                    panic_i64_overflow!(ctx, *a as i128 + *b as i128);
                }

                Self::int(*a + *b)
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_add(*a, *b) {
                    panic_f32_overflow!(ctx, *a + *b);
                }
                Self::float(*a + *b)
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if is_f32_overflow_add(*a as f64, *b) {
                    panic_f32_overflow!(ctx, *a as f64 + *b);
                }
                Self::float(*a as f64 + *b)
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if is_f32_overflow_add(*a, *b as f64) {
                    panic_f32_overflow!(ctx, *a + *b as f64);
                }
                Self::float(*a + *b as f64)
            }

            (Value::str_value(a), Value::str_value(b)) => {
                Self::str(format!("{}{}", *a, *b).as_ref())
            }
            (Value::list_value(a), _) => {
                if x.is_list() {
                    let mut list = a.clone();
                    let b = x.as_list_ref();
                    for x in b.values.iter() {
                        list.values.push(x.clone());
                    }
                    Self::from(Value::list_value(list))
                } else {
                    let msg = format!(
                        "can only concatenate list (not \"{}\") to list",
                        x.type_str()
                    );
                    panic!("{}", msg);
                }
            }
            _ => panic_unsupported_bin_op!("+", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_sub(&self, ctx: &mut Context, x: &Self) -> Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_sub(*a, *b) {
                    panic_i32_overflow!(ctx, *a as i128 - *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_sub(*a, *b) {
                    panic_i32_overflow!(ctx, *a as i128 - *b as i128);
                }
                Self::int(*a - *b)
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a, *b) {
                    panic_f32_overflow!(ctx, *a - *b);
                }
                Self::float(*a - *b)
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a as f64, *b) {
                    panic_f32_overflow!(ctx, *a as f64 - *b);
                }
                Self::float(*a as f64 - *b)
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a, *b as f64) {
                    panic_f32_overflow!(ctx, *a - *b as f64);
                }
                Self::float(*a - *b as f64)
            }
            _ => panic_unsupported_bin_op!("-", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_mul(&self, ctx: &mut Context, x: &Self) -> Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_mul(*a, *b) {
                    panic_i32_overflow!(ctx, *a as i128 * *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_mul(*a, *b) {
                    panic_i64_overflow!(ctx, *a as i128 * *b as i128);
                }
                Self::int(*a * *b)
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a, *b) {
                    panic_f32_overflow!(ctx, *a * *b);
                }
                Self::float(*a * *b)
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a as f64, *b) {
                    panic_f32_overflow!(ctx, *a as f64 * *b);
                }
                Self::float(*a as f64 * *b)
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a, *b as f64) {
                    panic_f32_overflow!(ctx, *a * *b as f64);
                }
                Self::float(*a * *b as f64)
            }

            (Value::str_value(a), Value::int_value(b)) => Self::str(a.repeat(*b as usize).as_ref()),
            (Value::int_value(b), Value::str_value(a)) => Self::str(a.repeat(*b as usize).as_ref()),
            (Value::list_value(a), Value::int_value(b)) => {
                let mut list = ListValue::default();
                for _ in 0..(*b as usize) {
                    for x in a.values.iter() {
                        list.values.push(x.clone());
                    }
                }
                Self::from(Value::list_value(Box::new(list)))
            }
            (Value::int_value(b), Value::list_value(a)) => {
                let mut list = ListValue::default();
                for _ in 0..(*b as usize) {
                    for x in a.values.iter() {
                        list.values.push(x.clone());
                    }
                }
                Self::from(Value::list_value(Box::new(list)))
            }
            _ => panic_unsupported_bin_op!("*", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_div(&self, x: &Self) -> Self {
        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => Self::float((*a as f64) / (*b as f64)),
            (Value::float_value(a), Value::float_value(b)) => Self::float(*a / *b),
            (Value::int_value(a), Value::float_value(b)) => Self::float(*a as f64 / *b),
            (Value::float_value(a), Value::int_value(b)) => Self::float(*a / *b as f64),
            _ => panic_unsupported_bin_op!("/", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_mod(&self, x: &Self) -> Self {
        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                let x = *a;
                let y = *b;
                if (x < 0) != (y < 0) && x % y != 0 {
                    Self::int(x % y + y)
                } else {
                    Self::int(x % y)
                }
            }
            (Value::float_value(a), Value::float_value(b)) => Self::float(*a % *b),
            (Value::int_value(a), Value::float_value(b)) => Self::float(*a as f64 % *b),
            (Value::float_value(a), Value::int_value(b)) => Self::float(*a % *b as f64),
            _ => panic_unsupported_bin_op!("%", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_pow(&self, ctx: &mut Context, x: &Self) -> Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(ref a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_pow(*a, *b) {
                    panic_i32_overflow!(ctx, (*a as i128).pow(*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_pow(*a, *b) {
                    panic_i64_overflow!(ctx, (*a as i128).pow(*b as u32));
                }
                Self::int(a.pow(*b as u32))
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a, *b) {
                    panic_f32_overflow!(ctx, a.powf(*b));
                }
                Self::float(a.powf(*b))
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a as f64, *b) {
                    panic_f32_overflow!(ctx, (*a as f64).powf(*b));
                }
                Self::float((*a as f64).powf(*b))
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a, *b as f64) {
                    panic_f32_overflow!(ctx, a.powf(*b as f64));
                }
                Self::float(a.powf(*b as f64))
            }
            _ => panic_unsupported_bin_op!("**", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_floor_div(&self, x: &Self) -> Self {
        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                let x = *a;
                let y = *b;
                if (x < 0) != (y < 0) && x % y != 0 {
                    Self::int(x / y - 1)
                } else {
                    Self::int(x / y)
                }
            }
            (Value::float_value(a), Value::float_value(b)) => Self::float((*a / *b).floor()),
            (Value::int_value(a), Value::float_value(b)) => Self::float((*a as f64 / *b).floor()),
            (Value::float_value(a), Value::int_value(b)) => Self::float((*a / *b as f64).floor()),
            _ => panic_unsupported_bin_op!("//", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_bit_lshift(&self, ctx: &mut Context, x: &Self) -> Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_shl(*a, *b) {
                    panic_i32_overflow!(ctx, (*a as i128) << (*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_shl(*a, *b) {
                    panic_i64_overflow!(ctx, (*a as i128) << (*b as u32));
                }
                Self::int(*a << *b)
            }
            _ => panic_unsupported_bin_op!("<<", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_bit_rshift(&self, ctx: &mut Context, x: &Self) -> Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_shr(*a, *b) {
                    panic_i32_overflow!(ctx, (*a as i128) >> (*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_shr(*a, *b) {
                    panic_i64_overflow!(ctx, (*a as i128) >> (*b as u32));
                }
                Self::int(*a >> *b)
            }
            _ => panic_unsupported_bin_op!(">>", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_bit_and(&self, x: &Self) -> Self {
        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => Self::int(*a & *b),
            _ => panic_unsupported_bin_op!("&", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_bit_xor(&self, x: &Self) -> Self {
        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => Self::int(*a ^ *b),
            _ => panic_unsupported_bin_op!("^", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_bit_or(&self, ctx: &mut Context, x: &Self) -> Self {
        if let (Value::int_value(a), Value::int_value(b)) = (&*self.rc.borrow(), &*x.rc.borrow()) {
            return Self::int(*a | *b);
        };
        self.deep_copy()
            .union_entry(ctx, x, true, &UnionOptions::default())
    }

    pub fn bin_subscr(&self, x: &Self) -> Self {
        match (&*self.rc.borrow(), &*x.rc.borrow()) {
            (Value::str_value(a), Value::int_value(b)) => {
                let str_len = a.chars().count();
                let index = *b;
                let index = if index < 0 {
                    (index + str_len as i64) as usize
                } else {
                    index as usize
                };
                if index < a.len() {
                    let ch = a.chars().nth(index).unwrap();
                    Self::str(ch.to_string().as_ref())
                } else {
                    panic!("string index out of range: {b}");
                }
            }
            (Value::list_value(a), Value::int_value(b)) => {
                let index = *b;
                let index = if index < 0 {
                    (index + a.values.len() as i64) as usize
                } else {
                    index as usize
                };
                if index < a.values.len() {
                    a.values[index].clone()
                } else {
                    panic!("list index out of range: {b}");
                }
            }
            (Value::dict_value(a), Value::str_value(b)) => match a.values.get(b) {
                Some(x) => (*x).clone(),
                _ => Self::undefined(),
            },
            (Value::dict_value(_), _) => Self::undefined(),
            (Value::schema_value(a), Value::str_value(b)) => match a.config.values.get(b) {
                Some(x) => (*x).clone(),
                _ => Self::undefined(),
            },
            _ => panic!(
                "'{}' object is not subscriptable with '{}'",
                self.type_str(),
                x.type_str()
            ),
        }
    }

    pub fn bin_subscr_option(&self, x: &Self) -> Self {
        if self.is_truthy() {
            self.bin_subscr(x)
        } else {
            Self::none()
        }
    }
}

#[cfg(test)]
mod test_value_bin {

    use crate::*;

    #[test]
    fn test_int_bin() {
        let mut ctx = Context::new();
        let cases = [
            (0, 0, "+", 0),
            (1, 1, "-", 0),
            (-1, 2, "*", -2),
            (4, 2, "/", 2),
            (-2, 4, "//", -1),
            (-2, 5, "%", 3),
            (3, 2, "**", 9),
            (2, 1, ">>", 1),
            (3, 2, "<<", 12),
            (5, 9, "&", 1),
            (5, 10, "|", 15),
            (7, 11, "^", 12),
        ];
        for (left, right, op, expected) in cases {
            let left = ValueRef::int(left);
            let right = ValueRef::int(right);
            let result = match op {
                "+" => left.bin_add(&mut ctx, &right),
                "-" => left.bin_sub(&mut ctx, &right),
                "*" => left.bin_mul(&mut ctx, &right),
                "/" => left.bin_div(&right),
                "//" => left.bin_floor_div(&right),
                "%" => left.bin_mod(&right),
                "**" => left.bin_pow(&mut ctx, &right),
                "<<" => left.bin_bit_lshift(&mut ctx, &right),
                ">>" => left.bin_bit_rshift(&mut ctx, &right),
                "&" => left.bin_bit_and(&right),
                "|" => left.bin_bit_or(&mut ctx, &right),
                "^" => left.bin_bit_xor(&right),
                _ => panic!("invalid op {}", op),
            };
            assert_eq!(result.as_int(), expected as i64)
        }
    }

    #[test]
    fn test_str_subscr() {
        let data = ValueRef::str("Hello world");
        let cases = [(0, "H"), (1, "e"), (-1, "d"), (-2, "l")];
        for (index, expected) in cases {
            let index = ValueRef::int(index as i64);
            let result = data.bin_subscr(&index).as_str();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_list_subscr() {
        let data = ValueRef::list_int(&[1, 2, 3, 4]);
        let cases = [(0, 1), (1, 2), (-1, 4), (-2, 3)];
        for (index, expected) in cases {
            let index = ValueRef::int(index as i64);
            let result = data.bin_subscr(&index).as_int();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_dict_subscr() {
        let data = ValueRef::dict_int(&[("k1", 1), ("k2", 2)]);
        assert_eq!(
            data.bin_subscr(&ValueRef::str("err_key")),
            ValueRef::undefined()
        );
        let cases = [("k1", 1), ("k2", 2)];
        for (key, expected) in cases {
            let key = ValueRef::str(key);
            let result = data.bin_subscr(&key).as_int();
            assert_eq!(result, expected);
        }
    }
}
