//! Copyright The KCL Authors. All rights reserved.

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_assert(
    ctx: *mut kclvm_context_t,
    value: *const kclvm_value_ref_t,
    msg: *const kclvm_value_ref_t,
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
