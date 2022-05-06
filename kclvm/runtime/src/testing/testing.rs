//! KCL testing system module
//!
//! Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = ValueRef;

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_testing_arguments(
    _ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) {
    // Nothing to do
}

#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_testing_setting_file(
    _ctx: *mut kclvm_context_t,
    _args: *const kclvm_value_ref_t,
    _kwargs: *const kclvm_value_ref_t,
) {
    // Nothing to do
}
