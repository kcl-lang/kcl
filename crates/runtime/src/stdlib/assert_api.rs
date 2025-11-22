//! Copyright The KCL Authors. All rights reserved.

use crate::*;

#[allow(non_camel_case_types)]
type kcl_value_ref_t = ValueRef;

#[unsafe(no_mangle)]

pub extern "C-unwind" fn kcl_assert(
    ctx: *mut kcl_context_t,
    value: *const kcl_value_ref_t,
    msg: *const kcl_value_ref_t,
) {
    let value = ptr_as_ref(value);
    let msg = ptr_as_ref(msg);

    if !value.is_truthy() {
        let ctx = mut_ptr_as_ref(ctx);
        ctx.set_err_type(&RuntimeErrorType::AssertionError);

        let msg = msg.as_str();
        panic!("{}", msg);
    }
}
