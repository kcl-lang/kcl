#![allow(clippy::missing_safety_doc)]

use kclvm_runner::runner::KCL_RUNTIME_PANIC_RECORD;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::c_char;
use std::ffi::{CStr, CString};
use std::{mem, ptr};

use crate::{intern_fmt, intern_run};

/// Exposes an allocation function to the WASM host.
///
/// _This implementation is copied from wasm-bindgen_
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_malloc(size: usize) -> *mut u8 {
    let align = mem::align_of::<usize>();
    let layout = Layout::from_size_align(size, align).expect("Invalid layout");
    if layout.size() > 0 {
        let ptr = alloc(layout);
        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, size);
            ptr
        } else {
            std::alloc::handle_alloc_error(layout);
        }
    } else {
        align as *mut u8
    }
}

/// Expose a deallocation function to the WASM host.
///
/// _This implementation is copied from wasm-bindgen_
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_free(ptr: *mut u8, size: usize) {
    // This happens for zero-length slices, and in that case `ptr` is
    // likely bogus so don't actually send this to the system allocator
    if size == 0 {
        return;
    }
    let align = mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(size, align);
    dealloc(ptr, layout);
}

#[repr(C)]
pub struct ExecProgramResult {
    json_result: *const c_char,
    yaml_result: *const c_char,
    log_message: *const c_char,
    err_message: *const c_char,
}

/// Execute KCL file with arguments and return the JSON/YAML result.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_exec_program(
    filename_ptr: *const c_char,
    src_ptr: *const c_char,
) -> *const ExecProgramResult {
    if filename_ptr.is_null() || src_ptr.is_null() {
        return std::ptr::null();
    }
    let filename = unsafe { CStr::from_ptr(filename_ptr).to_str().unwrap() };
    let src = unsafe { CStr::from_ptr(src_ptr).to_str().unwrap() };

    match intern_run(filename, src) {
        Ok(result) => {
            let json = CString::new(result.json_result).unwrap().into_raw();
            let yaml = CString::new(result.yaml_result).unwrap().into_raw();
            let log = CString::new(result.log_message).unwrap().into_raw();
            let err = CString::new(result.err_message).unwrap().into_raw();

            let exec_result = ExecProgramResult {
                json_result: json,
                yaml_result: yaml,
                log_message: log,
                err_message: err,
            };

            Box::into_raw(Box::new(exec_result)) as *const ExecProgramResult
        }
        Err(err) => {
            let result = ExecProgramResult {
                err_message: CString::new(err).unwrap().into_raw(),
                json_result: std::ptr::null(),
                yaml_result: std::ptr::null(),
                log_message: std::ptr::null(),
            };
            Box::into_raw(Box::new(result)) as *const ExecProgramResult
        }
    }
}

/// Free memory allocated for the ExecProgramResult.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_free_exec_program_result(result: *const ExecProgramResult) {
    if result.is_null() {
        return;
    }

    let result = Box::from_raw(result as *mut ExecProgramResult);

    if !result.json_result.is_null() {
        let _ = CString::from_raw(result.json_result as *mut c_char); // Free the C string
    }
    if !result.yaml_result.is_null() {
        let _ = CString::from_raw(result.yaml_result as *mut c_char); // Free the C string
    }
    if !result.log_message.is_null() {
        let _ = CString::from_raw(result.log_message as *mut c_char); // Free the C string
    }
    if !result.err_message.is_null() {
        let _ = CString::from_raw(result.err_message as *mut c_char); // Free the C string
    }

    // Result itself will be freed when going out of scope
}

/// Get the YAML result from ExecProgramResult.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_result_get_yaml_result(
    result: *const ExecProgramResult,
) -> *const c_char {
    if result.is_null() {
        return std::ptr::null();
    }

    let result = &*result;
    if result.yaml_result.is_null() {
        return std::ptr::null();
    }

    result.yaml_result
}

/// Get the JSON result from ExecProgramResult.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_result_get_json_result(
    result: *const ExecProgramResult,
) -> *const c_char {
    if result.is_null() {
        return std::ptr::null();
    }

    let result = &*result;
    if result.json_result.is_null() {
        return std::ptr::null();
    }

    result.json_result
}

/// Get the error message from ExecProgramResult.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_result_get_err_message(
    result: *const ExecProgramResult,
) -> *const c_char {
    if result.is_null() {
        return std::ptr::null();
    }

    let result = &*result;
    if result.err_message.is_null() {
        return std::ptr::null();
    }

    result.err_message
}

/// Get the log message from ExecProgramResult.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_result_get_log_message(
    result: *const ExecProgramResult,
) -> *const c_char {
    if result.is_null() {
        return std::ptr::null();
    }

    let result = &*result;
    if result.log_message.is_null() {
        return std::ptr::null();
    }

    result.log_message
}

/// Exposes a normal kcl run function to the WASM host.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_run(
    filename_ptr: *const c_char,
    src_ptr: *const c_char,
) -> *const c_char {
    if filename_ptr.is_null() || src_ptr.is_null() {
        return std::ptr::null();
    }
    let filename = unsafe { CStr::from_ptr(filename_ptr).to_str().unwrap() };
    let src = unsafe { CStr::from_ptr(src_ptr).to_str().unwrap() };

    match intern_run(filename, src) {
        Ok(result) => CString::new(result.yaml_result).unwrap().into_raw(),
        Err(err) => CString::new(format!("ERROR:{err}")).unwrap().into_raw(),
    }
}

/// Exposes a normal kcl run function with the log message to the WASM host.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_run_with_log_message(
    filename_ptr: *const c_char,
    src_ptr: *const c_char,
) -> *const c_char {
    if filename_ptr.is_null() || src_ptr.is_null() {
        return std::ptr::null();
    }
    let filename = unsafe { CStr::from_ptr(filename_ptr).to_str().unwrap() };
    let src = unsafe { CStr::from_ptr(src_ptr).to_str().unwrap() };

    match intern_run(filename, src) {
        Ok(result) => CString::new(result.log_message + &result.yaml_result)
            .unwrap()
            .into_raw(),
        Err(err) => CString::new(format!("ERROR:{err}")).unwrap().into_raw(),
    }
}

/// Exposes a normal kcl fmt function to the WASM host.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_fmt(src_ptr: *const c_char) -> *const c_char {
    if src_ptr.is_null() {
        return std::ptr::null();
    }
    let src = unsafe { CStr::from_ptr(src_ptr).to_str().unwrap() };

    match intern_fmt(src) {
        Ok(result) => CString::new(result).unwrap().into_raw(),
        Err(err) => CString::new(format!("ERROR:{err}")).unwrap().into_raw(),
    }
}

/// Exposes a normal kcl version function to the WASM host.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_version() -> *const c_char {
    CString::new(kclvm_version::VERSION).unwrap().into_raw()
}

/// Exposes a normal kcl runtime error function to the WASM host.
#[no_mangle]
pub unsafe extern "C-unwind" fn kcl_runtime_err(buffer: *mut u8, length: usize) -> isize {
    KCL_RUNTIME_PANIC_RECORD.with(|e| {
        let message = &e.borrow().message;
        if !message.is_empty() {
            let bytes = message.as_bytes();
            let copy_len = std::cmp::min(bytes.len(), length);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, copy_len);
            }
            copy_len as isize
        } else {
            0
        }
    })
}
