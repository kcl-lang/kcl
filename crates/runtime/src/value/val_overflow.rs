//! Copyright The KCL Authors. All rights reserved.

pub fn is_i32_overflow(v: i64) -> bool {
    v > i32::MAX as i64 || v < i32::MIN as i64
}
pub fn is_i32_overflow2(v1: i64, v2: i64) -> bool {
    is_i32_overflow(v1) || is_i32_overflow(v2)
}

pub fn is_f32_overflow(v: f64) -> bool {
    v > f32::MAX as f64 || v < f32::MIN as f64
}
pub fn is_f32_overflow2(v1: f64, v2: f64) -> bool {
    is_f32_overflow(v1) || is_f32_overflow(v2)
}

pub fn is_i32_overflow_add(v1: i64, v2: i64) -> bool {
    is_i32_overflow2(v1, v2) || (v1 as i32).checked_add(v2 as i32).is_none()
}
pub fn is_i32_overflow_sub(v1: i64, v2: i64) -> bool {
    is_i32_overflow2(v1, v2) || (v1 as i32).checked_sub(v2 as i32).is_none()
}
pub fn is_i32_overflow_mul(v1: i64, v2: i64) -> bool {
    is_i32_overflow2(v1, v2) || (v1 as i32).checked_mul(v2 as i32).is_none()
}
pub fn is_i32_overflow_div(v1: i64, v2: i64) -> bool {
    is_i32_overflow2(v1, v2) || (v1 as i32).checked_div(v2 as i32).is_none()
}
pub fn is_i32_overflow_pow(v1: i64, v2: i64) -> bool {
    is_i32_overflow2(v1, v2) || (v1 as i32).checked_pow(v2 as u32).is_none()
}
pub fn is_i32_overflow_shl(v1: i64, v2: i64) -> bool {
    is_i32_overflow2(v1, v2) || (v1 as i32).checked_shl(v2 as u32).is_none()
}
pub fn is_i32_overflow_shr(v1: i64, v2: i64) -> bool {
    is_i32_overflow2(v1, v2) || (v1 as i32).checked_shr(v2 as u32).is_none()
}

pub fn is_i64_overflow_add(v1: i64, v2: i64) -> bool {
    v1.checked_add(v2).is_none()
}
pub fn is_i64_overflow_sub(v1: i64, v2: i64) -> bool {
    v1.checked_sub(v2).is_none()
}
pub fn is_i64_overflow_mul(v1: i64, v2: i64) -> bool {
    v1.checked_mul(v2).is_none()
}
pub fn is_i64_overflow_div(v1: i64, v2: i64) -> bool {
    v1.checked_div(v2).is_none()
}
pub fn is_i64_overflow_pow(v1: i64, v2: i64) -> bool {
    is_i32_overflow(v2) || v1.checked_pow(v2 as u32).is_none()
}
pub fn is_i64_overflow_shl(v1: i64, v2: i64) -> bool {
    is_i32_overflow(v2) || v1.checked_shl(v2 as u32).is_none()
}
pub fn is_i64_overflow_shr(v1: i64, v2: i64) -> bool {
    is_i32_overflow(v2) || v1.checked_shr(v2 as u32).is_none()
}

pub fn is_f32_overflow_add(v1: f64, v2: f64) -> bool {
    is_f32_overflow2(v1, v2) || is_f32_overflow(v1 + v2)
}
pub fn is_f32_overflow_sub(v1: f64, v2: f64) -> bool {
    is_f32_overflow2(v1, v2) || is_f32_overflow(v1 - v2)
}
pub fn is_f32_overflow_mul(v1: f64, v2: f64) -> bool {
    is_f32_overflow2(v1, v2) || is_f32_overflow(v1 * v2)
}
pub fn is_f32_overflow_div(v1: f64, v2: f64) -> bool {
    is_f32_overflow2(v1, v2) || is_f32_overflow(v1 / v2)
}
pub fn is_f32_overflow_pow(v1: f64, v2: f64) -> bool {
    is_f32_overflow2(v1, v2) || is_i32_overflow(v2 as i64) || is_f32_overflow(v1.powf(v2))
}

#[cfg(test)]
mod test_is_int_overflow {
    use crate::*;

    #[test]
    fn test_is_i32_overflow() {
        assert!(!is_i32_overflow(2147483647));
        assert!(is_i32_overflow(2147483647 + 1));

        assert!(is_i32_overflow2(i32::MAX as i64 + 1, i32::MAX as i64 + 2));
    }
}
