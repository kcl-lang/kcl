use crate::gpyrpc::*;
use crate::service::capi::*;
use once_cell::sync::Lazy;
use prost::Message;
use serde::de::DeserializeOwned;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
const TEST_DATA_PATH: &str = "./src/testdata";
static TEST_MUTEX: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0i32));

#[test]
fn test_c_api_call_exec_program() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclvmService.ExecProgram",
        "exec-program.json",
        "exec-program.response.json",
        |res| res.escaped_time = "0".to_owned(),
    );
}

#[test]
fn test_c_api_call_override_file() {
    test_c_api_without_wrapper::<OverrideFileArgs, OverrideFileResult>(
        "KclvmService.OverrideFile",
        "override-file.json",
        "override-file.response.json",
    );
}

#[test]
fn test_c_api_get_schema_type_mapping() {
    test_c_api_without_wrapper::<GetSchemaTypeMappingArgs, GetSchemaTypeMappingResult>(
        "KclvmService.GetSchemaTypeMapping",
        "get-schema-type-mapping.json",
        "get-schema-type-mapping.response.json",
    );
}

#[test]
fn test_c_api_format_code() {
    test_c_api_without_wrapper::<FormatCodeArgs, FormatCodeResult>(
        "KclvmService.FormatCode",
        "format-code.json",
        "format-code.response.json",
    );
}

#[test]
fn test_c_api_format_path() {
    test_c_api_without_wrapper::<FormatPathArgs, FormatPathResult>(
        "KclvmService.FormatPath",
        "format-path.json",
        "format-path.response.json",
    );
}

#[test]
fn test_c_api_lint_path() {
    test_c_api_without_wrapper::<LintPathArgs, LintPathResult>(
        "KclvmService.LintPath",
        "lint-path.json",
        "lint-path.response.json",
    );
}

#[test]
fn test_c_api_validate_code() {
    test_c_api_without_wrapper::<ValidateCodeArgs, ValidateCodeResult>(
        "KclvmService.ValidateCode",
        "validate-code.json",
        "validate-code.response.json",
    );
}

#[test]
fn test_c_api_load_settings_files() {
    test_c_api_without_wrapper::<LoadSettingsFilesArgs, LoadSettingsFilesResult>(
        "KclvmService.LoadSettingsFiles",
        "load-settings-files.json",
        "load-settings-files.response.json",
    );
}

fn test_c_api_without_wrapper<A, R>(svc_name: &str, input: &str, output: &str)
where
    A: Message + DeserializeOwned,
    R: Message + Default + PartialEq + DeserializeOwned + serde::Serialize,
{
    test_c_api::<A, R, _>(svc_name, input, output, |_| {})
}

fn test_c_api<A, R, F>(svc_name: &str, input: &str, output: &str, wrapper: F)
where
    A: Message + DeserializeOwned,
    R: Message + Default + PartialEq + DeserializeOwned + serde::Serialize,
    F: Fn(&mut R),
{
    let _test_lock = TEST_MUTEX.lock().unwrap();
    let serv = kclvm_service_new(0);

    let input_path = Path::new(TEST_DATA_PATH).join(input);
    let input = fs::read_to_string(&input_path)
        .unwrap_or_else(|_| panic!("Something went wrong reading {}", input_path.display()));
    let args = unsafe {
        CString::from_vec_unchecked(serde_json::from_str::<A>(&input).unwrap().encode_to_vec())
    };
    let call = CString::new(svc_name).unwrap();
    let result_ptr = kclvm_service_call(serv, call.as_ptr(), args.as_ptr()) as *mut i8;
    let result = unsafe { CStr::from_ptr(result_ptr) };

    let mut result = R::decode(result.to_bytes()).unwrap();
    let except_result_path = Path::new(TEST_DATA_PATH).join(output);
    let except_result_json = fs::read_to_string(&except_result_path).unwrap_or_else(|_| {
        panic!(
            "Something went wrong reading {}",
            except_result_path.display()
        )
    });
    let mut except_result = serde_json::from_str::<R>(&except_result_json).unwrap();
    wrapper(&mut result);
    wrapper(&mut except_result);
    assert_eq!(result, except_result);
    unsafe {
        kclvm_service_delete(serv);
        kclvm_service_free_string(result_ptr);
    }
}
