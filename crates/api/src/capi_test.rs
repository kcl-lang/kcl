use crate::service::capi::*;
use crate::{call, gpyrpc::*};
use kcl_utils::path::PathPrefix;
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
        "KclService.ExecProgram",
        "exec-program.json",
        "exec-program.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_external_pkg() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclService.ExecProgram",
        "exec-program-with-external-pkg.json",
        "exec-program-with-external-pkg.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_include_schema_type_path() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclService.ExecProgram",
        "exec-program-with-include-schema-type-path.json",
        "exec-program-with-include-schema-type-path.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_path_selector() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclService.ExecProgram",
        "exec-program-with-path-selector.json",
        "exec-program-with-path-selector.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_exec_program_with_print() {
    test_c_api::<ExecProgramArgs, ExecProgramResult, _>(
        "KclService.ExecProgram",
        "exec-program-with-print.json",
        "exec-program-with-print.response.json",
        |_| {},
    );
}

#[test]
fn test_c_api_call_override_file() {
    let test_cases = [
        ("override-file.json", "override-file.response.json"),
        (
            "override-file-dict.json",
            "override-file-dict.response.json",
        ),
        (
            "override-file-dict_0.json",
            "override-file-dict_0.response.json",
        ),
        (
            "override-file-list.json",
            "override-file-list.response.json",
        ),
        (
            "override-file-bool.json",
            "override-file-bool.response.json",
        ),
    ];

    for (input, output) in &test_cases {
        test_c_api_without_wrapper::<OverrideFileArgs, OverrideFileResult>(
            "KclService.OverrideFile",
            input,
            output,
        );
    }
}

#[test]
fn test_c_api_get_schema_type_mapping() {
    test_c_api::<GetSchemaTypeMappingArgs, GetSchemaTypeMappingResult, _>(
        "KclService.GetSchemaTypeMapping",
        "get-schema-type-mapping.json",
        "get-schema-type-mapping.response.json",
        |r| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            for (_, s_ty) in &mut r.schema_type_mapping {
                let filename = {
                    let filename = s_ty.filename.adjust_canonicalization();
                    match filename.strip_prefix(root.to_str().unwrap()) {
                        Some(f) => f.to_string(),
                        None => s_ty.filename.clone(),
                    }
                };

                s_ty.filename = filename.replace('.', "").replace('/', "").replace('\\', "")
            }
        },
    );
}

#[test]
fn test_c_api_format_code() {
    test_c_api_without_wrapper::<FormatCodeArgs, FormatCodeResult>(
        "KclService.FormatCode",
        "format-code.json",
        "format-code.response.json",
    );
}

#[test]
fn test_c_api_format_path() {
    test_c_api_without_wrapper::<FormatPathArgs, FormatPathResult>(
        "KclService.FormatPath",
        "format-path.json",
        "format-path.response.json",
    );
}

#[test]
fn test_c_api_lint_path() {
    test_c_api_without_wrapper::<LintPathArgs, LintPathResult>(
        "KclService.LintPath",
        "lint-path.json",
        "lint-path.response.json",
    );
}

#[test]
fn test_c_api_call_exec_program_with_compile_only() {
    test_c_api_panic::<ExecProgramArgs>(
        "KclService.ExecProgram",
        "exec-program-with-compile-only.json",
        "exec-program-with-compile-only.response.panic",
    );
}

#[test]
fn test_c_api_validate_code_with_dep() {
    test_c_api_without_wrapper::<ValidateCodeArgs, ValidateCodeResult>(
        "KclService.ValidateCode",
        "validate-code-file-with-dep.json",
        "validate-code-file-with-dep.response.json",
    );
}

#[test]
fn test_c_api_validate_code() {
    test_c_api_without_wrapper::<ValidateCodeArgs, ValidateCodeResult>(
        "KclService.ValidateCode",
        "validate-code.json",
        "validate-code.response.json",
    );
}

#[test]
fn test_c_api_validate_code_file() {
    test_c_api_without_wrapper::<ValidateCodeArgs, ValidateCodeResult>(
        "KclService.ValidateCode",
        "validate-code-file.json",
        "validate-code-file.response.json",
    );
}

#[test]
fn test_c_api_load_settings_files() {
    test_c_api_without_wrapper::<LoadSettingsFilesArgs, LoadSettingsFilesResult>(
        "KclService.LoadSettingsFiles",
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
        "KclService.Rename",
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
        "KclService.RenameCode",
        "rename-code.json",
        "rename-code.response.json",
    );
}

#[test]
fn test_c_api_list_options() {
    test_c_api_without_wrapper::<ParseProgramArgs, ListOptionsResult>(
        "KclService.ListOptions",
        "list-options.json",
        "list-options.response.json",
    );
}

#[test]
fn test_c_api_list_variables() {
    test_c_api_without_wrapper::<ListVariablesArgs, ListVariablesResult>(
        "KclService.ListVariables",
        "list-variables.json",
        "list-variables.response.json",
    );
}

#[test]
fn test_c_api_parse_file() {
    test_c_api_without_wrapper::<ParseFileArgs, ParseFileResult>(
        "KclService.ParseFile",
        "parse-file.json",
        "parse-file.response.json",
    );
}

#[test]
fn test_c_api_testing() {
    test_c_api::<TestArgs, TestResult, _>(
        "KclService.Test",
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
    R: Message + Default + std::fmt::Debug + PartialEq + DeserializeOwned + serde::Serialize,
{
    test_c_api::<A, R, _>(svc_name, input, output, |_| {})
}

fn test_c_api<A, R, F>(svc_name: &str, input: &str, output: &str, wrapper: F)
where
    A: Message + DeserializeOwned,
    R: Message
        + Default
        + std::fmt::Debug
        + PartialEq
        + DeserializeOwned
        + serde::Serialize
        + ?Sized,
    F: Fn(&mut R),
{
    let _test_lock = TEST_MUTEX.lock().unwrap();
    let serv = unsafe { kcl_service_new(0) };

    let input_path = Path::new(TEST_DATA_PATH).join(input);
    let input = fs::read_to_string(&input_path)
        .unwrap_or_else(|_| panic!("Something went wrong reading {}", input_path.display()));
    let args_vec = serde_json::from_str::<A>(&input).unwrap().encode_to_vec();
    let args = unsafe { CString::from_vec_unchecked(args_vec.clone()) };
    let call = CString::new(svc_name).unwrap();
    let mut result_len: usize = 0;
    let src_ptr = unsafe {
        kcl_service_call_with_length(
            serv,
            call.as_ptr(),
            args.as_ptr(),
            args_vec.len(),
            &mut result_len,
        )
    };

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
        kcl_service_delete(serv);
        kcl_service_free_string(src_ptr as *mut c_char);
    }
}

fn test_c_api_panic<A>(svc_name: &str, input: &str, output: &str)
where
    A: Message + DeserializeOwned,
{
    let _test_lock = TEST_MUTEX.lock().unwrap();
    let serv = unsafe { kcl_service_new(0) };
    let prev_hook = std::panic::take_hook();
    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let result = std::panic::catch_unwind(|| {
        let input_path = Path::new(TEST_DATA_PATH).join(input);
        let input = fs::read_to_string(&input_path)
            .unwrap_or_else(|_| panic!("Something went wrong reading {}", input_path.display()));
        let args_vec = serde_json::from_str::<A>(&input).unwrap().encode_to_vec();
        let args = unsafe { CString::from_vec_unchecked(args_vec.clone()) };
        let call = CString::new(svc_name).unwrap();
        unsafe { kcl_service_call(serv, call.as_ptr(), args.as_ptr(), args_vec.len()) }
    });
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
            assert!(
                result.to_string_lossy().contains(&except_result_panic_msg),
                "{}",
                result.to_string_lossy()
            );
            unsafe {
                kcl_service_delete(serv);
                kcl_service_free_string(result_ptr as *mut c_char);
            }
        }
        Err(_) => {
            panic!("unreachable code")
        }
    }
}

#[test]
fn test_call_exec_program() {
    let name = b"KclService.ExecProgram";
    let args = b"\x12\x1a./src/testdata/test_call.k";
    let result = call(name, args).unwrap();
    assert!(
        !result.starts_with(b"ERROR"),
        "{}",
        String::from_utf8(result).unwrap()
    );
}

#[test]
fn test_call_get_version() {
    let name = b"KclService.GetVersion";
    let args = b"";
    let result = call(name, args).unwrap();
    assert!(!result.starts_with(b"ERROR"))
}
