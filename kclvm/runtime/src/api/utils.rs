//! Copyright The KCL Authors. All rights reserved.

use std::os::raw::c_char;

use crate::{kclvm_size_t, Context, ValueRef};

/// New a mutable raw pointer.
/// Safety: The caller must ensure that `ctx` lives longer than the returned pointer
/// and that the pointer is properly deallocated by calling `free_mut_ptr`.
pub fn new_mut_ptr(ctx: &mut Context, x: ValueRef) -> *mut ValueRef {
    let ptr = Box::into_raw(Box::new(x));
    // Store the object pointer address to
    // drop it it after execution is complete
    ctx.objects.insert(ptr as usize);
    ptr
}

/// Free a mutable raw pointer.
/// Safety: The caller must ensure `p` is a valid pointer obtained from `new_mut_ptr`.
pub(crate) fn free_mut_ptr<T>(p: *mut T) {
    if !p.is_null() {
        unsafe {
            drop(Box::from_raw(p));
        }
    }
}

/// Convert a const raw pointer to a immutable borrow.
/// Safety: The caller must ensure that `p` is valid for the lifetime `'a`.
pub(crate) fn ptr_as_ref<'a, T>(p: *const T) -> &'a T {
    assert!(!p.is_null());
    unsafe { &*p }
}

/// Convert a mutable raw pointer to a mutable borrow.
/// Safety: The caller must ensure that `p` is valid for the lifetime `'a`.
pub(crate) fn mut_ptr_as_ref<'a, T>(p: *mut T) -> &'a mut T {
    assert!(!p.is_null());

    unsafe { &mut *p }
}

/// Copy str to mutable pointer with length
pub(crate) fn copy_str_to(v: &str, p: *mut c_char, size: *mut kclvm_size_t) {
    unsafe {
        let c_str_ptr = v.as_ptr() as *const c_char;
        let c_str_len = v.len() as i32;
        if c_str_len <= *size {
            std::ptr::copy(c_str_ptr, p, c_str_len as usize);
            *size = c_str_len
        }
    }
}

/// Convert a C str pointer to a Rust &str.
/// Safety: The caller must ensure that `s` is a valid null-terminated C string.
pub fn c2str<'a>(p: *const c_char) -> &'a str {
    let s = unsafe { std::ffi::CStr::from_ptr(p) }.to_str().unwrap();
    s
}

/// Convert a C str pointer pointer to a Rust Vec<String>.
pub fn c2str_vec(ptr_array: *const *const c_char) -> Vec<String> {
    let mut result = Vec::new();
    let mut index = 0;

    unsafe {
        loop {
            let current_ptr = *ptr_array.offset(index);
            if current_ptr.is_null() {
                break;
            }
            let c_str = std::ffi::CStr::from_ptr(current_ptr);
            let rust_string = c_str.to_string_lossy().to_string();
            result.push(rust_string);
            index += 1;
        }
    }

    result
}

pub fn assert_panic<F: FnOnce() + std::panic::UnwindSafe>(msg: &str, func: F) {
    match std::panic::catch_unwind(func) {
        Ok(_v) => {
            panic!("not panic, expect={msg}");
        }
        Err(e) => match e.downcast::<String>() {
            Ok(v) => {
                let got = v.to_string();
                assert!(got.contains(msg), "expect={msg}, got={got}");
            }
            Err(e) => match e.downcast::<&str>() {
                Ok(v) => {
                    let got = v.to_string();
                    assert!(got.contains(msg), "expect={msg}, got={got}");
                }
                _ => unreachable!(),
            },
        },
    };
}
