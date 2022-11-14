// Copyright 2021 The KCL Authors. All rights reserved.

use crate::{kclvm_value_Undefined, Context, ValueRef};

/// New a mutable raw pointer.
pub fn new_mut_ptr(x: ValueRef) -> *mut ValueRef {
    let ptr = Box::into_raw(Box::new(x));
    let ctx = Context::current_context_mut();
    // Store the object pointer address to
    // drop it it after execution is complete
    ctx.objects.insert(ptr as usize);
    ptr
}

/// Free a mutable raw pointer.
pub fn free_mut_ptr<T>(p: *mut T) {
    if !p.is_null() {
        unsafe {
            drop(Box::from_raw(p));
        }
    }
}

/// Convert a const raw pointer to a immutable borrow.
pub fn ptr_as_ref<'a, T>(p: *const T) -> &'a T {
    if p.is_null() {
        let v = kclvm_value_Undefined();
        ptr_as_ref(v as *const T)
    } else {
        unsafe { &*p }
    }
}

/// Convert a mutable raw pointer to a mutable borrow.
pub fn mut_ptr_as_ref<'a, T>(p: *mut T) -> &'a mut T {
    assert!(!p.is_null());

    if p.is_null() {
        let v = kclvm_value_Undefined();
        mut_ptr_as_ref(v as *mut T)
    } else {
        unsafe { &mut *p }
    }
}

/// Convert a C str pointer to a Rust &str.
pub fn c2str<'a>(s: *const i8) -> &'a str {
    let s = unsafe { std::ffi::CStr::from_ptr(s) }.to_str().unwrap();
    s
}

/// Convert a raw double pinter to a Rust Vec.
pub fn convert_double_pointer_to_vec(data: &mut &mut i8, len: usize) -> Vec<String> {
    unsafe {
        match std::slice::from_raw_parts(data, len)
            .iter()
            .map(|arg| {
                std::ffi::CStr::from_ptr(*arg)
                    .to_str()
                    .map(ToString::to_string)
            })
            .collect()
        {
            Err(_error) => Vec::<String>::new(),
            Ok(x) => x,
        }
    }
}

pub fn assert_panic<F: FnOnce() + std::panic::UnwindSafe>(msg: &str, func: F) {
    match std::panic::catch_unwind(func) {
        Ok(_v) => {
            panic!("not panic, expect={}", msg);
        }
        Err(e) => match e.downcast::<String>() {
            Ok(v) => {
                let got = v.to_string();
                assert!(got.contains(msg), "expect={}, got={}", msg, got);
            }
            Err(e) => match e.downcast::<&str>() {
                Ok(v) => {
                    let got = v.to_string();
                    assert!(got.contains(msg), "expect={}, got={}", msg, got);
                }
                _ => unreachable!(),
            },
        },
    };
}
