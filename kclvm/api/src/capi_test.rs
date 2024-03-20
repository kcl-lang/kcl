use crate::gpyrpc::*;
use crate::service::capi::*;
use once_cell::sync::Lazy;
use prost::Message;
use serde::de::DeserializeOwned;
use std::default::Default;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const TEST_DATA_PATH: &str = "./src/testdata";
static TEST_MUTEX: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0i32));

#[test]
fn test_c_api_call_exec_program() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclvmService.ExecProgram",
        "exec-program.json",
        "exec-program.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_external_pkg() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclvmService.ExecProgram",
        "exec-program-with-external-pkg.json",
        "exec-program-with-external-pkg.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_include_schema_type_path() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclvmService.ExecProgram",
        "exec-program-with-include-schema-type-path.json",
        "exec-program-with-include-schema-type-path.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_path_selector() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclvmService.ExecProgram",
        "exec-program-with-path-selector.json",
        "exec-program-with-path-selector.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_print() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclvmService.ExecProgram",
        "exec-program-with-print.json",
        "exec-program-with-print.response.json",
        |_| {},
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
fn test_c_api_get_full_schema_type() {
    test_c_api::<GetFullSchemaTypeArgs, GetSchemaTypeResult, _>(
        "KclvmService.GetFullSchemaType",
        "get-full-schema-type.json",
        "get-full-schema-type.response.json",
        |r| {
            for s_ty in &mut r.schema_type_list {
                s_ty.filename = s_ty.filename.replace('/', "").replace('\\', "")
            }
        },
    );
}

#[test]
fn test_c_api_get_all_full_schema_types() {
    test_c_api::<GetFullSchemaTypeArgs, GetSchemaTypeResult, _>(
        "KclvmService.GetFullSchemaType",
        "get-all-full-schema-types.json",
        "get-all-full-schema-types.response.json",
        |r| {
            for s_ty in &mut r.schema_type_list {
                s_ty.filename = s_ty.filename.replace('/', "").replace('\\', "")
            }
        },
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
fn test_c_api_call_exec_program_with_compile_only() {
    test_c_api_paniced::<ExecProgramArgs>(
        "KclvmService.ExecProgram",
        "exec-program-with-compile-only.json",
        "exec-program-with-compile-only.response.panic",
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
fn test_c_api_validate_code_file() {
    test_c_api_without_wrapper::<ValidateCodeArgs, ValidateCodeResult>(
        "KclvmService.ValidateCode",
        "validate-code-file.json",
        "validate-code-file.response.json",
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

#[test]
fn test_c_api_rename() {
    // before test, load template from .bak
    let path = Path::new(TEST_DATA_PATH).join("rename").join("main.k");
    let backup_path = path.with_extension("bak");
    let content = fs::read_to_string(backup_path.clone()).unwrap();
    fs::write(path.clone(), content).unwrap();

    test_c_api::<RenameArgs, RenameResult, _>(
        "KclvmService.Rename",
        "rename.json",
        "rename.response.json",
        |r| {
            r.changed_files = r
                .changed_files
                .iter()
                .map(|f| {
                    PathBuf::from(f)
                        .canonicalize()
                        .unwrap()
                        .display()
                        .to_string()
                })
                .collect();
        },
    );

    // after test, restore template from .bak
    fs::remove_file(path.clone()).unwrap();
}

#[test]
fn test_c_api_rename_code() {
    test_c_api_without_wrapper::<RenameCodeArgs, RenameCodeResult>(
        "KclvmService.RenameCode",
        "rename-code.json",
        "rename-code.response.json",
    );
}

#[test]
fn test_c_api_list_options() {
    test_c_api_without_wrapper::<ParseProgramArgs, ListOptionsResult>(
        "KclvmService.ListOptions",
        "list-options.json",
        "list-options.response.json",
    );
}

#[test]
fn test_c_api_parse_file() {
    test_c_api_without_wrapper::<ParseFileArgs, ParseFileResult>(
        "KclvmService.ParseFile",
        "parse-file.json",
        "parse-file.response.json",
    );
}

#[test]
fn test_c_api_testing() {
    test_c_api::<TestArgs, TestResult, _>(
        "KclvmService.Test",
        "test.json",
        "test.response.json",
        |r| {
            for i in &mut r.info {
                i.duration = 0;
            }
        },
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
    R: Message + Default + PartialEq + DeserializeOwned + serde::Serialize + ?Sized,
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
    let mut result_len: usize = 0;
    let src_ptr =
        kclvm_service_call_with_length(serv, call.as_ptr(), args.as_ptr(), &mut result_len);

    let mut dest_data: Vec<u8> = Vec::with_capacity(result_len);
    unsafe {
        let dest_ptr: *mut u8 = dest_data.as_mut_ptr();
        std::ptr::copy_nonoverlapping(src_ptr as *const u8, dest_ptr, result_len);
        dest_data.set_len(result_len);
    }

    let mut result = R::decode(dest_data.as_slice()).unwrap();
    let result_json = serde_json::to_string(&result).unwrap();

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
    assert_eq!(result, except_result, "\nresult json is {result_json}");
    unsafe {
        kclvm_service_delete(serv);
        kclvm_service_free_string(src_ptr as *mut c_char);
    }
}

fn test_c_api_paniced<A>(svc_name: &str, input: &str, output: &str)
where
    A: Message + DeserializeOwned,
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
    let prev_hook = std::panic::take_hook();
    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let result =
        std::panic::catch_unwind(|| kclvm_service_call(serv, call.as_ptr(), args.as_ptr()));
    std::panic::set_hook(prev_hook);
    match result {
        Ok(result_ptr) => {
            let result = unsafe { CStr::from_ptr(result_ptr) };
            let except_result_path = Path::new(TEST_DATA_PATH).join(output);
            let except_result_panic_msg =
                fs::read_to_string(&except_result_path).unwrap_or_else(|_| {
                    panic!(
                        "Something went wrong reading {}",
                        except_result_path.display()
                    )
                });
            assert!(result.to_string_lossy().contains(&except_result_panic_msg));
            unsafe {
                kclvm_service_delete(serv);
                kclvm_service_free_string(result_ptr as *mut c_char);
            }
        }
        Err(_) => {
            panic!("unreachable code")
        }
    }
}
