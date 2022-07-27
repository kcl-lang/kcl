extern crate serde;

pub mod api_test;

use kclvm::api::utils::*;
use kclvm_api::service::api::_kclvm_get_service_fn_ptr_by_name;
use kclvm_api::service::service::KclvmService;
use kclvm_parser::load_program;
use kclvm_runner::execute;
use kclvm_runner::runner::*;
use kclvm_tools::query::apply_overrides;
use std::ffi::CString;
use std::os::raw::c_char;

#[allow(non_camel_case_types)]
type kclvm_service = KclvmService;

#[no_mangle]
pub extern "C" fn kclvm_service_new() -> *mut kclvm_service {
    new_mut_ptr(KclvmService::default())
}

#[no_mangle]
pub extern "C" fn kclvm_service_delete(serv: *mut kclvm_service) {
    free_mut_ptr(serv);
}

#[no_mangle]
pub extern "C" fn kclvm_service_free_result(res: *mut c_char) {
    if !res.is_null() {
        unsafe { CString::from_raw(res) };
    }
}

/// Call kclvm service by C API
///
/// # Parameters
///
/// `serv`: [*mut kclvm_service]
///     The pointer of &\[[KclvmService]]
///
/// `call`: [*const c_char]
///     The C str of the name of the called service,
///     with the format "KclvmService.{MethodName}"
///
/// `args`: [*const c_char]
///     Arguments of the call serialized as protobuf byte sequence,
///     refer to kclvm/api/src/gpyrpc.proto for the specific definitions of arguments
///
/// # Returns
///
/// result: [*const c_char]
///     Result of the call serialized as protobuf byte sequence
#[no_mangle]
pub extern "C" fn kclvm_service_call(
    serv: *mut kclvm_service,
    call: *const c_char,
    args: *const c_char,
) -> *const c_char {
    let serv = mut_ptr_as_ref(serv);
    let args = c2str(args).as_bytes();
    let call = c2str(call);
    let call = _kclvm_get_service_fn_ptr_by_name(call);
    if call == 0 {
        panic!("null fn ptr");
    }

    let call = (&call as *const u64) as *const ()
        as *const fn(serv: &mut KclvmService, args: &[u8]) -> *const c_char;
    unsafe { (*call)(serv, args) }
}

#[no_mangle]
pub extern "C" fn kclvm_cli_run(args: *const i8, plugin_agent: *const i8) -> *const i8 {
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
                let result = format!("ERROR:{}", result);
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const i8
            }
        },
        Err(panic_err) => {
            let err_message = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<&String>() {
                (*s).clone()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                (*s).clone()
            } else {
                "".to_string()
            };

            let result = format!("ERROR:{:}", err_message);
            let c_string = std::ffi::CString::new(result.as_str()).expect("CString::new failed");
            let ptr = c_string.into_raw();
            ptr as *const i8
        }
    }
}

pub fn kclvm_cli_run_unsafe(args: *const i8, plugin_agent: *const i8) -> Result<String, String> {
    let args = ExecProgramArgs::from_str(kclvm::c2str(args));
    let plugin_agent = plugin_agent as u64;

    let files = args.get_files();
    let opts = args.get_load_program_options();

    // Parse AST program.
    let mut program = load_program(&files, Some(opts))?;
    if let Err(msg) = apply_overrides(&mut program, &args.overrides, &[]) {
        return Err(msg.to_string());
    }

    // Resolve AST program, generate libs, link libs and execute.
    execute(program, plugin_agent, &args)
}
