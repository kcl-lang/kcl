use prost::Message;

use crate::gpyrpc::*;
use crate::service::service_impl::KclvmServiceImpl;
use kclvm_runtime::utils::*;
use std::ffi::CString;
use std::os::raw::c_char;

#[allow(non_camel_case_types)]
type kclvm_service = KclvmServiceImpl;

/// Create an instance of kclvm_service and return its pointer
#[no_mangle]
pub extern "C" fn kclvm_service_new(plugin_agent: u64) -> *mut kclvm_service {
    let serv = kclvm_service { plugin_agent };
    Box::into_raw(Box::new(serv))
}
/// # Safety
///
/// This function should not be called twice on the same ptr.
/// Delete KclvmService
#[no_mangle]
pub unsafe extern "C" fn kclvm_service_delete(serv: *mut kclvm_service) {
    free_mut_ptr(serv);
}

/// # Safety
///
/// This function should not be called twice on the same ptr.
/// Free memory for string returned to the outside
#[no_mangle]
pub unsafe extern "C" fn kclvm_service_free_string(res: *mut c_char) {
    if !res.is_null() {
        unsafe { CString::from_raw(res) };
    }
}

macro_rules! call {
    ($serv:expr, $args:expr, $arg_name:ident, $serv_name:ident) => {{
        let serv_ref = unsafe { mut_ptr_as_ref($serv) };
        let args = unsafe { std::ffi::CStr::from_ptr($args) }.to_bytes();
        let args = $arg_name::decode(args).unwrap();
        let res = serv_ref.$serv_name(&args);
        let result_byte = match res {
            Ok(res) => res.encode_to_vec(),
            Err(err) => panic!("{}", err),
        };
        CString::new(result_byte).unwrap().into_raw()
    }};
}

/// Call kclvm service by C API. **Note that it is not thread safe.**
///
/// # Parameters
///
/// `serv`: [*mut kclvm_service]
///     The pointer of &\[[KclvmServiceImpl]]
///
/// `call`: [*const c_char]
///     The C str of the name of the called service,
///     with the format "KclvmService.{MethodName}"
///
/// `args`: [*const c_char]
///     Arguments of the call serialized as protobuf byte sequence,
///     refer to internal/spec/gpyrpc/gpyrpc.proto for the specific definitions of arguments
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
    let result = std::panic::catch_unwind(|| {
        let call = c2str(call);
        let call = kclvm_get_service_fn_ptr_by_name(call);
        if call == 0 {
            panic!("null fn ptr");
        }
        let call = (&call as *const u64) as *const ()
            as *const fn(serv: *mut kclvm_service, args: *const c_char) -> *const c_char;
        unsafe { (*call)(serv, args) }
    });
    match result {
        //todo uniform error handling
        Ok(result) => result,
        Err(panic_err) => {
            let err_message = kclvm_error::err_to_str(panic_err);

            let c_string = std::ffi::CString::new(format!("ERROR:{}", err_message.as_str()))
                .expect("CString::new failed");
            let ptr = c_string.into_raw();
            ptr as *const i8
        }
    }
}

pub(crate) fn kclvm_get_service_fn_ptr_by_name(name: &str) -> u64 {
    match name {
        "KclvmService.Ping" => ping as *const () as u64,
        "KclvmService.ExecProgram" => exec_program as *const () as u64,
        "KclvmService.OverrideFile" => override_file as *const () as u64,
        "KclvmService.GetSchemaType" => get_schema_type as *const () as u64,
        "KclvmService.GetSchemaTypeMapping" => get_schema_type_mapping as *const () as u64,
        "KclvmService.FormatCode" => format_code as *const () as u64,
        "KclvmService.FormatPath" => format_path as *const () as u64,
        "KclvmService.LintPath" => lint_path as *const () as u64,
        "KclvmService.ValidateCode" => validate_code as *const () as u64,
        "KclvmService.LoadSettingsFiles" => load_settings_files as *const () as u64,
        _ => panic!("unknown method name : {name}"),
    }
}

/// ping is used to test whether kclvm service is successfully imported
/// arguments and return results should be consistent
pub(crate) fn ping(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, PingArgs, ping)
}

/// exec_program provides users with the ability to execute KCL code
///
/// # Parameters
///
/// `serv`: [*mut kclvm_service]
///     The pointer of &\[[KclvmServiceImpl]]
///
///
/// `args`: [*const c_char]
///     the items and compile parameters selected by the user in the KCLVM CLI
///     serialized as protobuf byte sequence
///
/// # Returns
///
/// result: [*const c_char]
///     Result of the call serialized as protobuf byte sequence
pub(crate) fn exec_program(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, ExecProgramArgs, exec_program)
}

/// override_file enable users override existing KCL file with specific KCl code
///
/// # Parameters
///
/// `serv`: [*mut kclvm_service]
///     The pointer of &\[[KclvmServiceImpl]]
///
///
/// `args`: [*const c_char]
///     kcl file , override specs and import paths selected by the user in the KCLVM CLI
///     serialized as protobuf byte sequence
///
/// # Returns
///
/// result: [*const c_char]
///     Result of the call serialized as protobuf byte sequence
pub(crate) fn override_file(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, OverrideFileArgs, override_file)
}

/// Get schema types from a kcl file or code.
///
/// # Parameters
/// file: [&str]. The kcl filename.
///
/// code: [Option<&str>]. The kcl code string
///
/// schema_name: [Option<&str>]. The schema name, when the schema name is empty, all schemas are returned.
pub(crate) fn get_schema_type(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, GetSchemaTypeArgs, get_schema_type)
}

/// Get schema types from a kcl file or code.
///
/// # Parameters
/// file: [&str]. The kcl filename.
///
/// code: [Option<&str>]. The kcl code string
///
/// schema_name: [Option<&str>]. The schema name, when the schema name is empty, all schemas are returned.
pub(crate) fn get_schema_type_mapping(
    serv: *mut kclvm_service,
    args: *const c_char,
) -> *const c_char {
    call!(
        serv,
        args,
        GetSchemaTypeMappingArgs,
        get_schema_type_mapping
    )
}

/// Service for formatting a code source and returns the formatted source and
/// whether the source is changed.
pub(crate) fn format_code(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, FormatCodeArgs, format_code)
}

/// Service for formatting kcl file or directory path contains kcl files and
/// returns the changed file paths.
pub(crate) fn format_path(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, FormatPathArgs, format_path)
}

/// Service for KCL Lint API, check a set of files, skips execute,
/// returns error message including errors and warnings.
pub(crate) fn lint_path(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, LintPathArgs, lint_path)
}

/// Service for validating the data string using the schema code string, when the parameter
/// `schema` is omitted, use the first schema appeared in the kcl code.
pub(crate) fn validate_code(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, ValidateCodeArgs, validate_code)
}

/// Service for building setting file config from args.
pub(crate) fn load_settings_files(serv: *mut kclvm_service, args: *const c_char) -> *const c_char {
    call!(serv, args, LoadSettingsFilesArgs, load_settings_files)
}
