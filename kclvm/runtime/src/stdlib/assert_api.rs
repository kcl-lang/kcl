// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_assert(
    value: *const kclvm_value_ref_t,
    msg: *const kclvm_value_ref_t,
) {
    let value = ptr_as_ref(value);
    let msg = ptr_as_ref(msg);

    if !value.is_truthy() {
        let ctx = Context::current_context_mut();
        ctx.set_err_type(&ErrType::AssertionError_TYPE);

        let msg = msg.as_str();
        panic!("{}", msg);
    }
}
