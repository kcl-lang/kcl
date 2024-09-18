#![allow(clippy::missing_safety_doc)]

use kclvm_api::FormatCodeArgs;
use kclvm_api::{gpyrpc::ExecProgramArgs as ExecProgramOptions, API};
use kclvm_parser::ParseSession;
use kclvm_runner::exec_program;
use kclvm_runner::runner::*;
pub use kclvm_runtime::*;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, c_int};
use std::ffi::{CStr, CString};
use std::mem;
use std::process::ExitCode;
use std::sync::Arc;

/// KCL CLI run function CAPI.
///
/// args is a ExecProgramArgs JSON string.
#[no_mangle]
pub unsafe extern "C" fn kclvm_cli_run(
    args: *const c_char,
    plugin_agent: *const c_char,
) -> *const c_char {
    let prev_hook = std::panic::take_hook();

    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let kclvm_cli_run_unsafe_result =
        std::panic::catch_unwind(|| kclvm_cli_run_unsafe(args, plugin_agent));
    std::panic::set_hook(prev_hook);

    match kclvm_cli_run_unsafe_result {
        Ok(result) => match result {
            Ok(result) => {
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const c_char
            }
            Err(result) => {
                let result = format!("ERROR:{result}");
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const c_char
            }
        },
        Err(err) => {
            let err_message = kclvm_error::err_to_str(err);
            let result = format!("ERROR:{err_message:}");
            let c_string = std::ffi::CString::new(result.as_str()).expect("CString::new failed");
            let ptr = c_string.into_raw();
            ptr as *const c_char
        }
    }
}

/// KCL CLI run function CAPI.
fn kclvm_cli_run_unsafe(
    args: *const c_char,
    plugin_agent: *const c_char,
) -> Result<String, String> {
    let mut args =
        ExecProgramArgs::from_str(unsafe { std::ffi::CStr::from_ptr(args) }.to_str().unwrap());
    args.plugin_agent = plugin_agent as u64;
    exec_program(Arc::new(ParseSession::default()), &args)
        .map_err(|e| PanicInfo::from(e.to_string()).to_json_string())
        .map(|r| r.json_result)
}

/// KCL CLI main function CAPI.
#[no_mangle]
pub unsafe extern "C" fn kclvm_cli_main(argc: c_int, argv: *const *const c_char) -> *mut ExitCode {
    let prev_hook = std::panic::take_hook();

    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let kclvm_cli_main_result = std::panic::catch_unwind(|| {
        let args: Vec<&str> = unsafe {
            std::slice::from_raw_parts(argv, argc as usize)
                .iter()
                .map(|ptr| CStr::from_ptr(*ptr).to_str().unwrap())
                .collect()
        };
        kclvm_cmd::main(args.as_slice())
    });
    std::panic::set_hook(prev_hook);

    match kclvm_cli_main_result {
        Ok(result) => match result {
            Ok(()) => Box::into_raw(Box::new(ExitCode::SUCCESS)),
            Err(err) => {
                let backtrace = format!("{}", err.backtrace());
                if backtrace.is_empty() || backtrace.contains("disabled backtrace") {
                    eprintln!("{err}");
                } else {
                    eprintln!("{err}\nStack backtrace:\n{backtrace}");
                }
                Box::into_raw(Box::new(ExitCode::FAILURE))
            }
        },
        Err(err) => {
            let err_str = kclvm_error::err_to_str(err);
            if !err_str.is_empty() {
                eprintln!("{err_str}");
            }
            Box::into_raw(Box::new(ExitCode::FAILURE))
        }
    }
}

/// Exposes a normal kcl run function to the WASM host.
#[no_mangle]
pub unsafe extern "C" fn kcl_run(
    filename_ptr: *const c_char,
    src_ptr: *const c_char,
) -> *const c_char {
    if filename_ptr.is_null() || src_ptr.is_null() {
        return std::ptr::null();
    }
    let filename = unsafe { CStr::from_ptr(filename_ptr).to_str().unwrap() };
    let src = unsafe { CStr::from_ptr(src_ptr).to_str().unwrap() };

    match intern_run(filename, src) {
        Ok(result) => CString::new(result).unwrap().into_raw(),
        Err(err) => CString::new(format!("ERROR:{}", err)).unwrap().into_raw(),
    }
}

fn intern_run(filename: &str, src: &str) -> Result<String, String> {
    let api = API::default();
    let args = &ExecProgramOptions {
        k_filename_list: vec![filename.to_string()],
        k_code_list: vec![src.to_string()],
        ..Default::default()
    };
    match api.exec_program(args) {
        Ok(result) => {
            if result.err_message.is_empty() {
                Ok(result.yaml_result)
            } else {
                Err(result.err_message)
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

/// Exposes a normal kcl fmt function to the WASM host.
#[no_mangle]
pub unsafe extern "C" fn kcl_fmt(src_ptr: *const c_char) -> *const c_char {
    if src_ptr.is_null() {
        return std::ptr::null();
    }
    let src = unsafe { CStr::from_ptr(src_ptr).to_str().unwrap() };

    match intern_fmt(src) {
        Ok(result) => CString::new(result).unwrap().into_raw(),
        Err(err) => CString::new(format!("ERROR:{}", err)).unwrap().into_raw(),
    }
}

fn intern_fmt(src: &str) -> Result<String, String> {
    let api = API::default();
    let args = &FormatCodeArgs {
        source: src.to_string(),
    };
    match api.format_code(args) {
        Ok(result) => String::from_utf8(result.formatted).map_err(|err| err.to_string()),
        Err(err) => Err(err.to_string()),
    }
}

/// Exposes a normal kcl version function to the WASM host.
#[no_mangle]
pub unsafe extern "C" fn kcl_version() -> *const c_char {
    CString::new(kclvm_version::VERSION).unwrap().into_raw()
}

/// Exposes a normal kcl runtime error function to the WASM host.
#[no_mangle]
pub unsafe extern "C" fn kcl_runtime_err(buffer: *mut u8, length: usize) -> isize {
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

/// Exposes an allocation function to the WASM host.
///
/// _This implementation is copied from wasm-bindgen_
#[no_mangle]
pub unsafe extern "C" fn kcl_malloc(size: usize) -> *mut u8 {
    let align = mem::align_of::<usize>();
    let layout = Layout::from_size_align(size, align).expect("Invalid layout");
    if layout.size() > 0 {
        let ptr = alloc(layout);
        if !ptr.is_null() {
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
pub unsafe extern "C" fn kcl_free(ptr: *mut u8, size: usize) {
    // This happens for zero-length slices, and in that case `ptr` is
    // likely bogus so don't actually send this to the system allocator
    if size == 0 {
        return;
    }
    let align = mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(size, align);
    dealloc(ptr, layout);
}
