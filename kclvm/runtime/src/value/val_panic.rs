// Copyright 2021 The KCL Authors. All rights reserved.

#[macro_export]
macro_rules! panic_i32_overflow {
    ($v: expr) => {
        let v = $v as i128;
        let ctx = $crate::Context::current_context_mut();
        ctx.set_err_type(&ErrType::IntOverflow_TYPE);
        panic!("{}: A 32 bit integer overflow", v)
    };
}

#[macro_export]
macro_rules! panic_i64_overflow {
    ($v: expr) => {
        let v = $v as i128;
        let ctx = $crate::Context::current_context_mut();
        ctx.set_err_type(&ErrType::IntOverflow_TYPE);
        panic!("{}: A 64 bit integer overflow", v)
    };
}
#[macro_export]
macro_rules! panic_f32_overflow {
    ($v: expr) => {
        let v = $v as f64;

        let ctx = $crate::Context::current_context_mut();
        ctx.set_err_type(&ErrType::FloatOverflow_TYPE);

        let mut s = format!("{:e}: A 32-bit floating point number overflow", v);
        if !s.contains("e-") {
            s = s.replacen("e", "e+", 1);
        }
        panic!("{}", s)
    };
}

#[macro_export]
macro_rules! panic_unsupported_bin_op {
    ($op:expr, $left_type: expr, $right_type: expr) => {
        panic!(
            "unsupported operand type(s) for {}: '{}' and '{}'",
            $op, $left_type, $right_type
        )
    };
}
