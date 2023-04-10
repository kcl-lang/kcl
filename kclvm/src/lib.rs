#![allow(clippy::missing_safety_doc)]

extern crate serde;

use kclvm_parser::ParseSession;
use kclvm_runner::exec_program;
use kclvm_runner::runner::*;
pub use kclvm_runtime::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::CStr;
use std::process::ExitCode;
use std::sync::Arc;

/// KCLVM CLI run function CAPI.
///
/// args is a ExecProgramArgs JSON string.
#[no_mangle]
pub unsafe extern "C" fn kclvm_cli_run(args: *const i8, plugin_agent: *const i8) -> *const i8 {
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
                ptr as *const i8
            }
            Err(result) => {
                let result = format!("ERROR:{result}");
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const i8
            }
        },
        Err(err) => {
            let err_message = kclvm_error::err_to_str(err);
            let result = format!("ERROR:{err_message:}");
            let c_string = std::ffi::CString::new(result.as_str()).expect("CString::new failed");
            let ptr = c_string.into_raw();
            ptr as *const i8
        }
    }
}

/// KCLVM CLI run function CAPI.
fn kclvm_cli_run_unsafe(args: *const i8, plugin_agent: *const i8) -> Result<String, String> {
    let mut args = ExecProgramArgs::from_str(kclvm_runtime::c2str(args));
    args.plugin_agent = plugin_agent as u64;
    exec_program(Arc::new(ParseSession::default()), &args)
        .map_err(|e| PanicInfo::from(e).to_json_string())
        .map(|r| r.json_result)
}

/// KCLVM CLI main function CAPI.
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
                if backtrace.is_empty() {
                    println!("Error: {}", err);
                } else {
                    println!("Error: {}\n\nStack backtrace:\n{}", err, backtrace);
                }
                Box::into_raw(Box::new(ExitCode::FAILURE))
            }
        },
        Err(err) => {
            let err_str = kclvm_error::err_to_str(err);
            if !err_str.is_empty() {
                println!("Error: {}", err_str);
            }
            Box::into_raw(Box::new(ExitCode::FAILURE))
        }
    }
}
