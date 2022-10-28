// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    pub fn bin_aug_add(&mut self, x: &Self) -> &mut Self {
        let ctx = crate::Context::current_context_mut();
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_add(*a, *b) {
                    panic_i32_overflow!(*a as i128 + *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_add(*a, *b) {
                    panic_i64_overflow!(*a as i128 + *b as i128);
                }
                let a: &mut i64 = get_ref_mut(a);
                *a += *b;
                self
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_add(*a, *b) {
                    panic_f32_overflow!(*a + *b);
                }
                let a: &mut f64 = get_ref_mut(a);
                *a += *b;
                self
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if is_f32_overflow_add(*a as f64, *b) {
                    panic_f32_overflow!(*a as f64 + *b);
                }
                let a: &mut i64 = get_ref_mut(a);
                *a += *b as i64;
                self
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if is_f32_overflow_add(*a, *b as f64) {
                    panic_f32_overflow!(*a + *b as f64);
                }
                let a: &mut f64 = get_ref_mut(a);
                *a += *b as f64;
                self
            }
            (Value::str_value(a), Value::str_value(b)) => {
                let a: &mut String = get_ref_mut(a);
                *a = format!("{}{}", *a, *b);
                self
            }
            (Value::list_value(a), _) => match &*x.rc {
                Value::list_value(ref b) => {
                    let list: &mut ListValue = get_ref_mut(a);
                    for x in b.values.iter() {
                        list.values.push(x.clone());
                    }
                    self
                }
                _ => panic_unsupported_bin_op!("+", self.type_str(), x.type_str()),
            },
            _ => panic_unsupported_bin_op!("+", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_sub(&mut self, x: &Self) -> &mut Self {
        let ctx = crate::Context::current_context_mut();
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_sub(*a, *b) {
                    panic_i32_overflow!(*a as i128 - *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_sub(*a, *b) {
                    {
                        panic_i32_overflow!(*a as i128 - *b as i128);
                    }
                }
                let a: &mut i64 = get_ref_mut(a);
                *a -= *b;
                self
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a, *b) {
                    panic_f32_overflow!(*a - *b);
                }
                let a: &mut f64 = get_ref_mut(a);
                *a -= *b;
                self
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a as f64, *b) {
                    panic_f32_overflow!(*a as f64 - *b);
                }
                let a: &mut i64 = get_ref_mut(a);
                *a -= *b as i64;
                self
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_sub(*a, *b as f64) {
                    panic_f32_overflow!(*a - *b as f64);
                }
                let a: &mut f64 = get_ref_mut(a);
                *a -= *b as f64;
                self
            }
            _ => panic_unsupported_bin_op!("-", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_mul(&mut self, x: &Self) -> &mut Self {
        let ctx = crate::Context::current_context_mut();
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_mul(*a, *b) {
                    panic_i32_overflow!(*a as i128 * *b as i128);
                }
                if strict_range_check_64 && is_i64_overflow_mul(*a, *b) {
                    panic_i64_overflow!(*a as i128 * *b as i128);
                }
                let a: &mut i64 = get_ref_mut(a);
                *a *= *b;
                self
            }
            (Value::float_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a, *b) {
                    panic_f32_overflow!(*a * *b);
                }
                let a: &mut f64 = get_ref_mut(a);
                *a *= *b;
                self
            }
            (Value::int_value(a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a as f64, *b) {
                    panic_f32_overflow!(*a as f64 * *b);
                }
                let a: &mut i64 = get_ref_mut(a);
                *a *= *b as i64;
                self
            }
            (Value::float_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_mul(*a, *b as f64) {
                    panic_f32_overflow!(*a * *b as f64);
                }
                let a: &mut f64 = get_ref_mut(a);
                *a *= *b as f64;
                self
            }
            (Value::str_value(a), Value::int_value(b)) => {
                let a: &mut String = get_ref_mut(a);
                *a = a.repeat(*b as usize);
                self
            }
            (Value::list_value(ref list), _) => match &*x.rc {
                Value::int_value(ref b) => {
                    let list: &mut ListValue = get_ref_mut(list);
                    let n = list.values.len();
                    for _ in 1..(*b as usize) {
                        for i in 0..n {
                            list.values.push(list.values[i].clone());
                        }
                    }
                    self
                }
                _ => panic_unsupported_bin_op!("*", self.type_str(), x.type_str()),
            },
            _ => panic_unsupported_bin_op!("*", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_div(&mut self, x: &Self) -> &mut Self {
        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                *a /= *b;
                self
            }
            (Value::int_value(a), Value::float_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                *a /= *b as i64;
                self
            }
            (Value::float_value(a), Value::int_value(b)) => {
                let a: &mut f64 = get_ref_mut(a);
                *a /= *b as f64;
                self
            }
            (Value::float_value(a), Value::float_value(b)) => {
                let a: &mut f64 = get_ref_mut(a);
                *a /= *b;
                self
            }
            _ => panic_unsupported_bin_op!("/", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_mod(&mut self, x: &Self) -> &mut Self {
        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                let x = *a;
                let y = *b;
                if (x < 0) != (y < 0) && x % y != 0 {
                    *a = *a % *b + *b;
                } else {
                    *a %= *b
                }
                self
            }
            (Value::int_value(a), Value::float_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                *a %= *b as i64;
                self
            }
            (Value::float_value(a), Value::int_value(b)) => {
                let a: &mut f64 = get_ref_mut(a);
                *a %= *b as f64;
                self
            }
            (Value::float_value(a), Value::float_value(b)) => {
                let a: &mut f64 = get_ref_mut(a);
                *a %= *b;
                self
            }
            _ => panic_unsupported_bin_op!("%", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_pow(&mut self, x: &Self) -> &mut Self {
        let ctx = crate::Context::current_context_mut();
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc, &*x.rc) {
            (Value::int_value(ref a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_pow(*a, *b) {
                    panic_i32_overflow!((*a as i128).pow(*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_pow(*a, *b) {
                    panic_i64_overflow!((*a as i128).pow(*b as u32));
                }
                let a: &mut i64 = get_ref_mut(a);
                *a = a.pow(*b as u32);
                self
            }
            (Value::float_value(ref a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a, *b) {
                    panic_f32_overflow!(a.powf(*b));
                }
                let a: &mut f64 = get_ref_mut(a);
                *a = a.powf(*b as f64);
                self
            }
            (Value::int_value(ref a), Value::float_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a as f64, *b) {
                    panic_f32_overflow!((*a as f64).powf(*b));
                }
                let a: &mut i64 = get_ref_mut(a);
                *a = a.pow(*b as u32);
                self
            }
            (Value::float_value(ref a), Value::int_value(b)) => {
                if strict_range_check_32 && is_f32_overflow_pow(*a, *b as f64) {
                    panic_f32_overflow!(a.powf(*b as f64));
                }
                let a: &mut f64 = get_ref_mut(a);
                *a = a.powf(*b as f64);
                self
            }
            _ => panic_unsupported_bin_op!("**", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_floor_div(&mut self, x: &Self) -> &mut Self {
        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                let x = *a;
                let y = *b;
                if (x < 0) != (y < 0) && x % y != 0 {
                    *a = *a / *b - 1
                } else {
                    *a /= *b
                }
                self
            }
            (Value::int_value(a), Value::float_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                *a = (*a as f64 / *b) as i64;
                self
            }
            (Value::float_value(a), Value::int_value(b)) => {
                let a: &mut f64 = get_ref_mut(a);
                *a /= *b as f64;
                self
            }
            (Value::float_value(a), Value::float_value(b)) => {
                let a: &mut f64 = get_ref_mut(a);
                *a /= *b;
                self
            }
            _ => panic_unsupported_bin_op!("//", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_bit_lshift(&mut self, x: &Self) -> &mut Self {
        let ctx = crate::Context::current_context_mut();
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_shl(*a, *b) {
                    panic_i32_overflow!((*a as i128) << (*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_shl(*a, *b) {
                    panic_i64_overflow!((*a as i128) << (*b as u32));
                }
                let a: &mut i64 = get_ref_mut(a);
                *a <<= *b as usize;
                self
            }
            _ => panic_unsupported_bin_op!("<<", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_bit_rshift(&mut self, x: &Self) -> &mut Self {
        let ctx = crate::Context::current_context_mut();
        let strict_range_check_32 = ctx.cfg.strict_range_check;
        let strict_range_check_64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;

        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                if strict_range_check_32 && is_i32_overflow_shr(*a, *b) {
                    panic_i32_overflow!((*a as i128) >> (*b as u32));
                }
                if strict_range_check_64 && is_i64_overflow_shr(*a, *b) {
                    panic_i64_overflow!((*a as i128) >> (*b as u32));
                }
                let a: &mut i64 = get_ref_mut(a);
                *a >>= *b as usize;
                self
            }
            _ => panic_unsupported_bin_op!(">>", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_bit_and(&mut self, x: &Self) -> &mut Self {
        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                *a &= *b as i64;
                self
            }
            _ => panic_unsupported_bin_op!("&", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_bit_xor(&mut self, x: &Self) -> &mut Self {
        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                *a ^= *b as i64;
                self
            }
            _ => panic_unsupported_bin_op!("^", self.type_str(), x.type_str()),
        }
    }

    pub fn bin_aug_bit_or(&mut self, x: &Self) -> &mut Self {
        match (&*self.rc, &*x.rc) {
            (Value::int_value(a), Value::int_value(b)) => {
                let a: &mut i64 = get_ref_mut(a);
                *a |= *b as i64;
                self
            }
            _ => {
                if self.is_list_or_config() || x.is_list_or_config() {
                    self.union(x, true, false, true, true);
                    return self;
                }
                panic_unsupported_bin_op!("|", self.type_str(), x.type_str());
            }
        }
    }

    /// Binary aug union a | b
    pub fn bin_aug_union_with(&mut self, x: &Self) -> &mut Self {
        self.bin_aug_bit_or(x)
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
        for (left, right, op, expected) in cases {
            let mut left = ValueRef::int(left);
            let right = ValueRef::int(right);
            let result = match op {
                "+" => left.bin_aug_add(&right),
                "-" => left.bin_aug_sub(&right),
                "*" => left.bin_aug_mul(&right),
                "/" => left.bin_aug_div(&right),
                "//" => left.bin_aug_floor_div(&right),
                "%" => left.bin_aug_mod(&right),
                "**" => left.bin_aug_pow(&right),
                "<<" => left.bin_aug_bit_lshift(&right),
                ">>" => left.bin_aug_bit_rshift(&right),
                "&" => left.bin_aug_bit_and(&right),
                "|" => left.bin_aug_bit_or(&right),
                "^" => left.bin_aug_bit_xor(&right),
                _ => panic!("invalid op {}", op),
            };
            assert_eq!(result.as_int(), expected as i64)
        }
    }

    #[test]
    fn test_aug_add() {
        // int
        assert_eq!(
            ValueRef::int(1).bin_aug_add(&ValueRef::int(2)).as_int(),
            1 + 2
        );

        // float
        assert_eq!(
            ValueRef::float(1.5)
                .bin_aug_add(&ValueRef::float(2.0))
                .as_float(),
            3.5
        );

        // str

        // list

        // int + float => int
        assert_eq!(
            ValueRef::int(1).bin_aug_add(&ValueRef::float(2.5)).as_int(),
            1 + 2
        );

        // float + int => float
        assert_eq!(
            ValueRef::float(1.5)
                .bin_aug_add(&ValueRef::int(2))
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
