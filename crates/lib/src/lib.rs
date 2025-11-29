#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, c_char, c_int};
use std::process::ExitCode;

use kcl_api::FormatCodeArgs;
use kcl_api::{API, ExecProgramArgs};

mod capi;
pub use capi::*;
use kcl_parser::ParseSessionRef;
use kcl_runner::exec_program;
use kcl_runtime::PanicInfo;

/// KCL CLI run function CAPI.
///
/// args is a ExecProgramArgs JSON string.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn libkcl_run(
    args: *const c_char,
    plugin_agent: *const c_char,
) -> *const c_char {
    let prev_hook = std::panic::take_hook();

    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let libkcl_run_unsafe_result =
        std::panic::catch_unwind(|| libkcl_run_unsafe(args, plugin_agent));
    std::panic::set_hook(prev_hook);

    match libkcl_run_unsafe_result {
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
            let err_message = kcl_error::err_to_str(err);
            let result = format!("ERROR:{err_message:}");
            let c_string = std::ffi::CString::new(result.as_str()).expect("CString::new failed");
            let ptr = c_string.into_raw();
            ptr as *const c_char
        }
    }
}

/// KCL CLI run function CAPI.
fn libkcl_run_unsafe(args: *const c_char, plugin_agent: *const c_char) -> Result<String, String> {
    let mut args = kcl_runner::ExecProgramArgs::from_json(
        unsafe { std::ffi::CStr::from_ptr(args) }.to_str().unwrap(),
    );
    args.plugin_agent = plugin_agent as u64;
    exec_program(ParseSessionRef::default(), &args)
        .map_err(|e| PanicInfo::from(e.to_string()).to_json_string())
        .map(|r| r.json_result)
}

/// KCL CLI main function CAPI.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn libkcl_main(
    argc: c_int,
    argv: *const *const c_char,
) -> *mut ExitCode {
    let prev_hook = std::panic::take_hook();

    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let libkcl_main_result = std::panic::catch_unwind(|| {
        let args: Vec<&str> = unsafe {
            std::slice::from_raw_parts(argv, argc as usize)
                .iter()
                .map(|ptr| CStr::from_ptr(*ptr).to_str().unwrap())
                .collect()
        };
        kcl_cmd::main(args.as_slice())
    });
    std::panic::set_hook(prev_hook);

    match libkcl_main_result {
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
            let err_str = kcl_error::err_to_str(err);
            if !err_str.is_empty() {
                eprintln!("{err_str}");
            }
            Box::into_raw(Box::new(ExitCode::FAILURE))
        }
    }
}

fn intern_run(filename: &str, src: &str) -> Result<kcl_api::ExecProgramResult, String> {
    let api = API::default();
    let args = &ExecProgramArgs {
        k_filename_list: vec![filename.to_string()],
        k_code_list: vec![src.to_string()],
        ..Default::default()
    };
    api.exec_program(args).map_err(|err| err.to_string())
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
