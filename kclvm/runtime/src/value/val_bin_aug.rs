//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    pub fn bin_aug_add(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_add(*a, *b) {
                    panic_i32_overflow!(ctx, *a as i128 + *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_add(*a, *b) {
                    panic_i64_overflow!(ctx, *a as i128 + *b as i128);
                }
                *a += *b;
                true
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_add(*a, *b) {
                    panic_f32_overflow!(ctx, *a + *b);
                }
                *a += *b;
                true
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if is_f32_overflow_add(*a as f64, *b) {
                    panic_f32_overflow!(ctx, *a as f64 + *b);
                }
                *a += *b as i64;
                true
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if is_f32_overflow_add(*a, *b as f64) {
                    panic_f32_overflow!(ctx, *a + *b as f64);
                }
                *a += *b as f64;
                true
            }
            (Value::str_value(a), Value::str_value(b)) => {
                *a = format!("{}{}", *a, *b);
                true
            }
            (Value::list_value(a), _) => match &*x.rc.borrow() {
                Value::list_value(ref b) => {
                    for x in b.values.iter() {
                        a.values.push(x.clone());
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("+", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_sub(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_sub(*a, *b) {
                    panic_i32_overflow!(ctx, *a as i128 - *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_sub(*a, *b) {
                    {
                        panic_i32_overflow!(ctx, *a as i128 - *b as i128);
                    }
                }
                *a -= *b;
                true
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a, *b) {
                    panic_f32_overflow!(ctx, *a - *b);
                }
                *a -= *b;
                true
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a as f64, *b) {
                    panic_f32_overflow!(ctx, *a as f64 - *b);
                }
                *a -= *b as i64;
                true
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a, *b as f64) {
                    panic_f32_overflow!(ctx, *a - *b as f64);
                }
                *a -= *b as f64;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("-", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_mul(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_mul(*a, *b) {
                    panic_i32_overflow!(ctx, *a as i128 * *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_mul(*a, *b) {
                    panic_i64_overflow!(ctx, *a as i128 * *b as i128);
                }
                *a *= *b;
                true
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a, *b) {
                    panic_f32_overflow!(ctx, *a * *b);
                }
                *a *= *b;
                true
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a as f64, *b) {
                    panic_f32_overflow!(ctx, *a as f64 * *b);
                }
                *a *= *b as i64;
                true
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a, *b as f64) {
                    panic_f32_overflow!(ctx, *a * *b as f64);
                }
                *a *= *b as f64;
                true
            }
            (Value::str_value(a), Value::int_value(b)) => {
                *a = a.repeat(*b as usize);
                true
            }
            (Value::list_value(list), _) => match &*x.rc.borrow() {
                Value::int_value(ref b) => {
                    let n = list.values.len();
                    for _ in 1..(*b as usize) {
                        for i in 0..n {
                            list.values.push(list.values[i].clone());
                        }
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("*", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_div(&mut self, x: &Self) -> &mut Self {
        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                *a /= *b;
                true
            }
            (Value::int_value(a), Value::float_value(b)) => {
                *a /= *b as i64;
                true
            }
            (Value::float_value(a), Value::int_value(b)) => {
                *a /= *b as f64;
                true
            }
            (Value::float_value(a), Value::float_value(b)) => {
                *a /= *b;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("/", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_mod(&mut self, x: &Self) -> &mut Self {
        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                let x = *a;
                let y = *b;
                if (x < 0) != (y < 0) && x % y != 0 {
                    *a = *a % *b + *b;
                } else {
                    *a %= *b
                }
                true
            }
            (Value::int_value(a), Value::float_value(b)) => {
                *a %= *b as i64;
                true
            }
            (Value::float_value(a), Value::int_value(b)) => {
                *a %= *b as f64;
                true
            }
            (Value::float_value(a), Value::float_value(b)) => {
                *a %= *b;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("%", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_pow(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_pow(*a, *b) {
                    panic_i32_overflow!(ctx, (*a as i128).pow(*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_pow(*a, *b) {
                    panic_i64_overflow!(ctx, (*a as i128).pow(*b as u32));
                }
                *a = a.pow(*b as u32);
                true
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a, *b) {
                    panic_f32_overflow!(ctx, a.powf(*b));
                }
                *a = a.powf(*b);
                true
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a as f64, *b) {
                    panic_f32_overflow!(ctx, (*a as f64).powf(*b));
                }
                *a = a.pow(*b as u32);
                true
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a, *b as f64) {
                    panic_f32_overflow!(ctx, a.powf(*b as f64));
                }
                *a = a.powf(*b as f64);
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("**", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_floor_div(&mut self, x: &Self) -> &mut Self {
        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                let x = *a;
                let y = *b;
                if (x < 0) != (y < 0) && x % y != 0 {
                    *a = *a / *b - 1
                } else {
                    *a /= *b
                }
                true
            }
            (Value::int_value(a), Value::float_value(b)) => {
                *a = (*a as f64 / *b) as i64;
                true
            }
            (Value::float_value(a), Value::int_value(b)) => {
                *a /= *b as f64;
                true
            }
            (Value::float_value(a), Value::float_value(b)) => {
                *a /= *b;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("//", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_bit_lshift(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_shl(*a, *b) {
                    panic_i32_overflow!(ctx, (*a as i128) << (*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_shl(*a, *b) {
                    panic_i64_overflow!(ctx, (*a as i128) << (*b as u32));
                }
                *a <<= *b as usize;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("<<", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_bit_rshift(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_shr(*a, *b) {
                    panic_i32_overflow!(ctx, (*a as i128) >> (*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_shr(*a, *b) {
                    panic_i64_overflow!(ctx, (*a as i128) >> (*b as u32));
                }
                *a >>= *b as usize;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!(">>", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_bit_and(&mut self, x: &Self) -> &mut Self {
        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                *a &= *b;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("^", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_bit_xor(&mut self, x: &Self) -> &mut Self {
        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                *a ^= *b;
                true
            }
            _ => false,
        };
        if !valid {
            panic_unsupported_bin_op!("^", self.type_str(), x.type_str())
        }
        self
    }

    pub fn bin_aug_bit_or(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        let valid = match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::int_value(a), Value::int_value(b)) => {
                *a |= *b;
                true
            }
            _ => false,
        };
        if !valid {
            if self.is_list_or_config() || x.is_list_or_config() {
                self.union_entry(ctx, x, true, &UnionOptions::default());
            } else {
                panic_unsupported_bin_op!("|", self.type_str(), x.type_str());
            }
        }
        self
    }

    /// Binary aug union a | b
    pub fn bin_aug_union_with(&mut self, ctx: &mut Context, x: &Self) -> &mut Self {
        self.bin_aug_bit_or(ctx, x)
    }
}

#[cfg(test)]
mod test_value_bin_aug {
    use crate::*;

    #[test]
    fn test_int_bin() {
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
        let mut ctx = Context::new();
        for (left, right, op, expected) in cases {
            let mut left = ValueRef::int(left);
            let right = ValueRef::int(right);
            let result = match op {
                "+" => left.bin_aug_add(&mut ctx, &right),
                "-" => left.bin_aug_sub(&mut ctx, &right),
                "*" => left.bin_aug_mul(&mut ctx, &right),
                "/" => left.bin_aug_div(&right),
                "//" => left.bin_aug_floor_div(&right),
                "%" => left.bin_aug_mod(&right),
                "**" => left.bin_aug_pow(&mut ctx, &right),
                "<<" => left.bin_aug_bit_lshift(&mut ctx, &right),
                ">>" => left.bin_aug_bit_rshift(&mut ctx, &right),
                "&" => left.bin_aug_bit_and(&right),
                "|" => left.bin_aug_bit_or(&mut ctx, &right),
                "^" => left.bin_aug_bit_xor(&right),
                _ => panic!("invalid op {}", op),
            };
            assert_eq!(result.as_int(), expected as i64)
        }
    }

    #[test]
    fn test_aug_add() {
        let mut ctx = Context::new();
        // int
        assert_eq!(
            ValueRef::int(1)
                .bin_aug_add(&mut ctx, &ValueRef::int(2))
                .as_int(),
            1 + 2
        );

        // float
        assert_eq!(
            ValueRef::float(1.5)
                .bin_aug_add(&mut ctx, &ValueRef::float(2.0))
                .as_float(),
            3.5
        );

        // str

        // list

        // int + float => int
        assert_eq!(
            ValueRef::int(1)
                .bin_aug_add(&mut ctx, &ValueRef::float(2.5))
                .as_int(),
            1 + 2
        );

        // float + int => float
        assert_eq!(
            ValueRef::float(1.5)
                .bin_aug_add(&mut ctx, &ValueRef::int(2))
                .as_float(),
            1.5 + 2.0
        );
    }

    #[test]
    fn test_aug_sub() {
        // int
        // float
    }
}
