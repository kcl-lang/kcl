// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

#[allow(dead_code, non_camel_case_types)]
type kclvm_bool_t = i8;

#[allow(dead_code, non_camel_case_types)]
type kclvm_char_t = i8;

#[allow(dead_code, non_camel_case_types)]
type kclvm_size_t = i32;

#[allow(dead_code, non_camel_case_types)]
type kclvm_buffer_t = Buffer;

#[allow(dead_code, non_camel_case_types)]
pub type kclvm_value_t = Value;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    buf: Vec<u8>,
}

impl Buffer {
    #[allow(dead_code)]
    pub fn new_with_buf(buf: &[u8]) -> Self {
        Buffer { buf: buf.to_vec() }
    }

    #[allow(dead_code)]
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    #[allow(dead_code)]
    pub fn malloc(size: usize) -> *mut u8 {
        let p = Box::into_raw(Box::new(Buffer {
            buf: vec![0u8; size + 8],
        }));

        unsafe {
            let data_ptr = (*p).buf.as_ptr() as *mut u8;
            let u64bytes = (p as u64).to_le_bytes();
            (*p).buf[..8].clone_from_slice(&u64bytes[..8]);
            data_ptr.add(8)
        }
    }

    #[allow(dead_code)]
    pub fn free(_data_ptr: *mut u8) {
        unsafe {
            let p = u64::from_le_bytes(
                std::slice::from_raw_parts(((_data_ptr as u64) - 8) as *const u8, 8)
                    .try_into()
                    .unwrap(),
            ) as *mut Self;

            drop(Box::from_raw(p));
        }
    }
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_buffer_new(size: kclvm_size_t) -> *mut kclvm_buffer_t {
    let mut p = Buffer { buf: Vec::new() };
    p.buf.resize(size as usize, 0);
    Box::into_raw(Box::new(p))
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_buffer_delete(p: *mut kclvm_buffer_t) {
    free_mut_ptr(p)
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_buffer_size(p: *const kclvm_buffer_t) -> kclvm_size_t {
    let p = ptr_as_ref(p);
    p.buf.len() as kclvm_size_t
}

#[no_mangle]
#[runtime_fn]
pub unsafe extern "C" fn kclvm_buffer_data(p: *const kclvm_buffer_t) -> *const kclvm_char_t {
    let p = ptr_as_ref(p);
    if !p.buf.is_empty() {
        p.buf.as_ptr() as *const kclvm_char_t
    } else {
        std::ptr::null()
    }
}
