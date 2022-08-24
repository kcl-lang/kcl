use protobuf::Message;

use crate::model::gpyrpc::*;
use crate::service::api::*;
use crate::service::util::*;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::Path;
const TEST_DATA_PATH: &str = "./src/testdata";

#[test]
fn test_c_api_call_exec_program() {
    let serv = kclvm_service_new(0);
    let input_path = Path::new(TEST_DATA_PATH).join("exec-program.json");
    let input = fs::read_to_string(&input_path)
        .expect(format!("Something went wrong reading {}", input_path.display()).as_str());
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
    let except_result_json = fs::read_to_string(&except_result_path).expect(
        format!(
            "Something went wrong reading {}",
            except_result_path.display()
        )
        .as_str(),
    );
    let except_result = parse_message_from_json::<ExecProgram_Result>(&except_result_json).unwrap();
    assert_eq!(result.json_result, except_result.json_result);
    assert_eq!(result.yaml_result, except_result.yaml_result);

    kclvm_service_delete(serv);
    kclvm_service_free_string(result_ptr);
}

#[test]
fn test_c_api_call_override_file() {
    let serv = kclvm_service_new(0);
    let input_path = Path::new(TEST_DATA_PATH).join("override-file.json");
    let input = fs::read_to_string(&input_path)
        .expect(format!("Something went wrong reading {}", input_path.display()).as_str());
    let args = unsafe {
        CString::from_vec_unchecked(
            parse_message_from_json::<OverrideFile_Args>(&input)
                .unwrap()
                .write_to_bytes()
                .unwrap(),
        )
    };
    let call = CString::new("KclvmService.OverrideFile").unwrap();
    let result_ptr = kclvm_service_call(serv, call.as_ptr(), args.as_ptr()) as *mut i8;
    let result = unsafe { CStr::from_ptr(result_ptr) };

    let result = parse_message_from_protobuf::<OverrideFile_Result>(result.to_bytes()).unwrap();
    let except_result_path = Path::new(TEST_DATA_PATH).join("override-file.response.json");
    let except_result_json = fs::read_to_string(&except_result_path).expect(
        format!(
            "Something went wrong reading {}",
            except_result_path.display()
        )
        .as_str(),
    );
    let except_result =
        parse_message_from_json::<OverrideFile_Result>(&except_result_json).unwrap();
    assert_eq!(result.result, except_result.result);

    kclvm_service_delete(serv);
    kclvm_service_free_string(result_ptr);
}
