use protobuf::Message;
use protobuf::MessageFull;

use crate::model::gpyrpc::*;
use crate::service::api::*;
use crate::service::util::*;
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

static TEST_MUTEX: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0i32));

const TEST_DATA_PATH: &str = "./src/testdata";

#[test]
fn test_c_api_call_exec_program() {
    let serv = kclvm_service_new(0);
    let input_path = Path::new(TEST_DATA_PATH).join("exec-program.json");
    let input = fs::read_to_string(&input_path)
        .unwrap_or_else(|_| panic!("Something went wrong reading {}", input_path.display()));
    let args = unsafe {
        CString::from_vec_unchecked(
            parse_message_from_json::<ExecProgram_Args>(&input)
                .unwrap()
                .write_to_bytes()
                .unwrap(),
        )
    };
    let call = CString::new("KclvmService.ExecProgram").unwrap();
    let result_ptr = kclvm_service_call(serv, call.as_ptr(), args.as_ptr()) as *mut i8;
    let result = unsafe { CStr::from_ptr(result_ptr) };

    let result = parse_message_from_protobuf::<ExecProgram_Result>(result.to_bytes()).unwrap();
    let except_result_path = Path::new(TEST_DATA_PATH).join("exec-program.response.json");
    let except_result_json = fs::read_to_string(&except_result_path).unwrap_or_else(|_| {
        panic!(
            "Something went wrong reading {}",
            except_result_path.display()
        )
    });
    let except_result = parse_message_from_json::<ExecProgram_Result>(&except_result_json).unwrap();
    assert_eq!(result.json_result, except_result.json_result);
    assert_eq!(result.yaml_result, except_result.yaml_result);

    kclvm_service_delete(serv);
    kclvm_service_free_string(result_ptr);
}

#[test]
fn test_c_api_call_override_file() {
    test_c_api::<OverrideFile_Args, OverrideFile_Result>(
        "KclvmService.OverrideFile",
        "override-file.json",
        "override-file.response.json",
    );
}

#[test]
fn test_c_api_get_schema_type_mapping() {
    test_c_api::<GetSchemaTypeMapping_Args, GetSchemaTypeMapping_Result>(
        "KclvmService.GetSchemaTypeMapping",
        "get-schema-type-mapping.json",
        "get-schema-type-mapping.response.json",
    );
}

#[test]
fn test_c_api_format_code() {
    test_c_api::<FormatCode_Args, FormatCode_Result>(
        "KclvmService.FormatCode",
        "format-code.json",
        "format-code.response.json",
    );
}

#[test]
fn test_c_api_format_path() {
    test_c_api::<FormatPath_Args, FormatPath_Result>(
        "KclvmService.FormatPath",
        "format-path.json",
        "format-path.response.json",
    );
}

#[test]
fn test_c_api_lint_path() {
    test_c_api::<LintPath_Args, LintPath_Result>(
        "KclvmService.LintPath",
        "lint-path.json",
        "lint-path.response.json",
    );
}

#[test]
fn test_c_api_validate_code() {
    test_c_api::<ValidateCode_Args, ValidateCode_Result>(
        "KclvmService.ValidateCode",
        "validate-code.json",
        "validate-code.response.json",
    );
}

#[test]
fn test_c_api_load_settings_files() {
    test_c_api::<LoadSettingsFiles_Args, LoadSettingsFiles_Result>(
        "KclvmService.LoadSettingsFiles",
        "load-settings-files.json",
        "load-settings-files.response.json",
    );
}

fn test_c_api<A, R>(svc_name: &str, input: &str, output: &str)
where
    A: MessageFull,
    R: MessageFull,
{
    let _test_lock = TEST_MUTEX.lock().unwrap();
    let serv = kclvm_service_new(0);
    let input_path = Path::new(TEST_DATA_PATH).join(input);
    let input = fs::read_to_string(&input_path)
        .unwrap_or_else(|_| panic!("Something went wrong reading {}", input_path.display()));
    let args = unsafe {
        CString::from_vec_unchecked(
            parse_message_from_json::<A>(&input)
                .unwrap()
                .write_to_bytes()
                .unwrap(),
        )
    };
    let call = CString::new(svc_name).unwrap();
    let result_ptr = kclvm_service_call(serv, call.as_ptr(), args.as_ptr()) as *mut i8;
    let result = unsafe { CStr::from_ptr(result_ptr) };

    let result = parse_message_from_protobuf::<R>(result.to_bytes()).unwrap();
    let except_result_path = Path::new(TEST_DATA_PATH).join(output);
    let except_result_json = fs::read_to_string(&except_result_path).unwrap_or_else(|_| {
        panic!(
            "Something went wrong reading {}",
            except_result_path.display()
        )
    });
    let except_result = parse_message_from_json::<R>(&except_result_json).unwrap();
    assert_eq!(result, except_result);
    kclvm_service_delete(serv);
    kclvm_service_free_string(result_ptr);
}
