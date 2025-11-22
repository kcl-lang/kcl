//! Copyright The KCL Authors. All rights reserved.

#[macro_export]
macro_rules! panic_i32_overflow {
    ($ctx: expr,$v: expr) => {
        let v = $v as i128;
        $ctx.set_err_type(&RuntimeErrorType::IntOverflow);
        panic!("{}: A 32 bit integer overflow", v)
    };
}

#[macro_export]
macro_rules! panic_i64_overflow {
    ($ctx: expr,$v: expr) => {
        let v = $v as i128;
        $ctx.set_err_type(&RuntimeErrorType::IntOverflow);
        panic!("{}: A 64 bit integer overflow", v)
    };
}
#[macro_export]
macro_rules! panic_f32_overflow {
    ($ctx: expr,$v: expr) => {
        let v = $v as f64;

        $ctx.set_err_type(&RuntimeErrorType::FloatOverflow);

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
