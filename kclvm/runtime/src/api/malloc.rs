// Copyright 2021 The KCL Authors. All rights reserved.
#![allow(clippy::missing_safety_doc)]

use crate::*;

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_malloc(n: i32) -> *mut u8 {
    Buffer::malloc(n as usize)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_free(ptr: *mut u8) {
    Buffer::free(ptr);
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_strlen(ptr: *mut u8) -> kclvm_size_t {
    unsafe {
        let mut p = ptr;
        while *p != b'\0' {
            p = p.add(1);
        }
        (p as i32) - (ptr as i32)
    }
}
