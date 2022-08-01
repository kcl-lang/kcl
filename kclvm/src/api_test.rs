use crate::*;
use kclvm_api::model::gpyrpc::*;
use kclvm_api::service::util::*;
use std::ffi::CStr;
use std::fs;
use std::path::Path;
const TEST_DATA_PATH: &str = "./src/testdata";

#[test]
fn test_c_api_call_exec_program() {
    let serv = kclvm_service_new();
    let input_path = Path::new(TEST_DATA_PATH).join("exec-program.json");
    let input = fs::read_to_string(&input_path)
        .expect(format!("Something went wrong reading {}", input_path.display()).as_str());
    let args = unsafe {
        CString::from_vec_unchecked(transform_json_to_protobuf::<ExecProgram_Args>(&input))
    };
    let call = CString::new("KclvmService.ExecProgram").unwrap();
    let result = unsafe { CStr::from_ptr(kclvm_service_call(serv, call.as_ptr(), args.as_ptr())) };

    let result = parse_message_from_protobuf::<ExecProgram_Result>(result.to_bytes());
    let except_result_path = Path::new(TEST_DATA_PATH).join("exec-program.response.json");
    let except_result_json = fs::read_to_string(&except_result_path).expect(
        format!(
            "Something went wrong reading {}",
            except_result_path.display()
        )
        .as_str(),
    );
    let except_result = parse_message_from_json::<ExecProgram_Result>(&except_result_json);
    assert_eq!(result.json_result, except_result.json_result);
    assert_eq!(result.yaml_result, except_result.yaml_result);

    kclvm_service_delete(serv);
}
