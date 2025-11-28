//! Copyright The KCL Authors. All rights reserved.

use std::os::raw::c_char;

use crate::{Context, ValueRef, kcl_size_t};

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
/// # Safety
/// The caller must ensure `p` is a valid pointer obtained from `new_mut_ptr`.
pub unsafe fn free_mut_ptr<T>(p: *mut T) {
    if !p.is_null() {
        unsafe {
            drop(Box::from_raw(p));
        }
    }
}

/// Convert a const raw pointer to a immutable borrow.
/// # Safety
/// The caller must ensure that `p` is a valid pointer.
pub unsafe fn ptr_as_ref<'a, T>(p: *const T) -> &'a T {
    assert!(!p.is_null());
    unsafe { &*p }
}

/// Convert a mutable raw pointer to a mutable borrow.
/// # Safety
/// The caller must ensure that `p` is a valid pointer.
pub unsafe fn mut_ptr_as_ref<'a, T>(p: *mut T) -> &'a mut T {
    assert!(!p.is_null());

    unsafe { &mut *p }
}

/// Copy str to mutable pointer with length
///
/// Copies the byte contents of a Rust string to a C-compatible mutable char pointer,
/// and updates the provided size pointer with the actual string length (in bytes).
///
/// # Safety
/// The caller **must** ensure all of the following conditions are met:
/// 1. `p` is a **non-null, valid, writable pointer** to a contiguous block of memory.
/// 2. `size` is a **non-null, valid, writable pointer** to a `kcl_size_t` value.
/// 3. The memory block pointed to by `p` has a capacity of at least `*size` bytes (before the call).
/// 4. The memory referenced by `p` and `size` remains **valid and unmodified** for the entire duration of this function call.
/// 5. The memory block at `p` does **not overlap** with the memory of the input string `v` (violating this causes undefined behavior for `ptr::copy`).
pub unsafe fn copy_str_to(v: &str, p: *mut c_char, size: *mut kcl_size_t) {
    assert!(!p.is_null() || !size.is_null());

    let c_str_ptr = v.as_ptr() as *const c_char;
    let c_str_len = v.len() as i32;
    if c_str_len <= unsafe { *size } {
        unsafe { std::ptr::copy(c_str_ptr, p, c_str_len as usize) };
        unsafe { *size = c_str_len }
    }
}

/// Convert a C str pointer to a Rust &str.
///
/// # Safety
/// The caller must ensure all of the following conditions are met:
/// 1. `p` is a **non-null, valid pointer** to a null-terminated C string.
/// 2. The memory referenced by `p` remains **valid and unmodified** for the entire lifetime `'a`.
/// 3. The C string pointed to by `p` is encoded in **valid UTF-8** (otherwise this function will panic).
pub unsafe fn c2str<'a>(p: *const c_char) -> &'a str {
    assert!(!p.is_null());

    unsafe { std::ffi::CStr::from_ptr(p) }.to_str().unwrap() as _
}

/// Convert a C str pointer pointer to a Rust Vec<String>.
///
/// # Safety
/// The caller must ensure all of the following conditions are met:
/// 1. `ptr_array` is a **non-null, valid pointer** to an array of `*const c_char` (C string pointers).
/// 2. The array pointed to by `ptr_array` is **null-terminated** (the end of the array is marked by a `null` pointer).
/// 3. Each non-null `*const c_char` in the array points to a **valid, null-terminated C string** (UTF-8 or compatible).
/// 4. The memory referenced by `ptr_array` and all inner C string pointers remains **valid and unmodified** for the duration of this function call.
/// 5. The pointers in the array are properly **aligned** for `*const c_char` (which is always true for C-compatible pointers).
pub unsafe fn c2str_vec(ptr_array: *const *const c_char) -> Vec<String> {
    assert!(!ptr_array.is_null());

    let mut result = Vec::new();
    let mut index = 0;

    loop {
        let current_ptr = unsafe { *ptr_array.offset(index) };
        if current_ptr.is_null() {
            break;
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(current_ptr) };
        let rust_string = c_str.to_string_lossy().to_string();
        result.push(rust_string);
        index += 1;
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
