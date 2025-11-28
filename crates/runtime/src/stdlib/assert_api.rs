//! Copyright The KCL Authors. All rights reserved.

use crate::*;

/// The function is to implement the assert statement in KCL.
/// # Safety
/// The caller must ensure that `ctx`, `value`, and `msg` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_assert(
    ctx: *mut kcl_context_t,
    value: *const kcl_value_ref_t,
    msg: *const kcl_value_ref_t,
) {
    let value = unsafe { ptr_as_ref(value) };
    let msg = unsafe { ptr_as_ref(msg) };

    if !value.is_truthy() {
        let ctx = unsafe { mut_ptr_as_ref(ctx) };
        ctx.set_err_type(&RuntimeErrorType::AssertionError);

        let msg = msg.as_str();
        panic!("{}", msg);
    }
}
