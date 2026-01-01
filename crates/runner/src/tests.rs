#![allow(clippy::arc_with_non_send_sync)]

use crate::exec_program;
use crate::{execute, runner::ExecProgramArgs};
use anyhow::Result;
use kcl_ast::ast::{Module, Program};
use kcl_config::settings::load_file;
use kcl_parser::ParseSession;
use kcl_parser::load_program;
use kcl_utils::path::PathPrefix;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;
use std::{
    collections::HashMap,
    fs::{self, File},
};
use uuid::Uuid;
use walkdir::WalkDir;

const TEST_CASES: &[&str; 5] = &[
    "init_check_order_0",
    "init_check_order_1",
    "normal_2",
    "type_annotation_not_full_2",
    "multi_vars_0",
];

fn exec_data_path() -> String {
    Path::new("src").join("exec_data").display().to_string()
}

fn exec_err_data_path() -> String {
    Path::new("src").join("exec_err_data").display().to_string()
}

fn custom_manifests_data_path() -> String {
    Path::new("src")
        .join("custom_manifests_data")
        .display()
        .to_string()
}

fn exec_prog_args_test_case() -> Vec<String> {
    vec![
        Path::new("exec_prog_args")
            .join("default.json")
            .display()
            .to_string(),
    ]
}

fn settings_file_test_case() -> Vec<(String, String)> {
    vec![(
        Path::new("settings_file")
            .join("settings.yaml")
            .display()
            .to_string(),
        Path::new("settings_file")
            .join("settings.json")
            .display()
            .to_string(),
    )]
}

const EXPECTED_JSON_FILE_NAME: &str = "stdout.golden.json";

fn test_case_path() -> String {
    Path::new("src").join("test_datas").display().to_string()
}

const KCL_FILE_NAME: &str = "main.k";
const MAIN_PKG_NAME: &str = "__main__";

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SimplePanicInfo {
    line: i32,
    col: i32,
    message: String,
}

/// Load test kcl file to ast.Program
fn load_test_program(filename: String) -> Program {
    let module = kcl_parser::parse_file_force_errors(&filename, None).unwrap();
    construct_program(module)
}

/// Construct ast.Program by ast.Module and default configuration.
/// Default configuration:
///     module.pkg = "__main__"
///     Program.root = "__main__"
fn construct_program(module: Module) -> Program {
    let mut pkgs_ast = HashMap::new();
    pkgs_ast.insert(MAIN_PKG_NAME.to_string(), vec![module.filename.clone()]);
    let mut modules = HashMap::new();
    modules.insert(module.filename.clone(), Arc::new(RwLock::new(module)));
    Program {
        root: MAIN_PKG_NAME.to_string(),
        pkgs: pkgs_ast,
        modules,
        pkgs_not_imported: HashMap::new(),
        modules_not_imported: HashMap::new(),
    }
}

/// Load the expect result from stdout.golden.json
fn load_expect_file(filename: String) -> String {
    let f = File::open(filename).unwrap();
    let v: serde_json::Value = serde_json::from_reader(f).unwrap();
    v.to_string()
}

/// Format str by json str
fn format_str_by_json(str: String) -> String {
    let v: serde_json::Value = serde_json::from_str(&str).unwrap();
    v.to_string()
}

fn execute_for_test(kcl_path: &String) -> String {
    let args = ExecProgramArgs::default();
    // Parse kcl file
    let program = load_test_program(kcl_path.to_string());
    // Generate libs, link libs and execute.
    execute(Arc::new(ParseSession::default()), program, &args)
        .unwrap()
        .json_result
}

fn test_kcl_runner_execute() {
    for case in TEST_CASES {
        let kcl_path = &Path::new(&test_case_path())
            .join(case)
            .join(KCL_FILE_NAME)
            .display()
            .to_string();
        let expected_path = &Path::new(&test_case_path())
            .join(case)
            .join(EXPECTED_JSON_FILE_NAME)
            .display()
            .to_string();
        let result = execute_for_test(kcl_path);
        let expected_result = load_expect_file(expected_path.to_string());
        assert_eq!(expected_result, format_str_by_json(result));
    }
}

#[test]
fn test_to_json_program_arg() {
    for case in exec_prog_args_test_case() {
        let test_case_json_file = &Path::new(&test_case_path())
            .join(case)
            .display()
            .to_string();
        let expected_json_str = fs::read_to_string(test_case_json_file).unwrap();
        let exec_prog_args = ExecProgramArgs::default();
        assert_eq!(expected_json_str.trim(), exec_prog_args.to_json().trim());
    }
}

#[test]
fn test_from_str_program_arg() {
    for case in exec_prog_args_test_case() {
        let test_case_json_file = &Path::new(&test_case_path())
            .join(case)
            .display()
            .to_string();
        let expected_json_str = fs::read_to_string(test_case_json_file).unwrap();
        let exec_prog_args = ExecProgramArgs::from_json(&expected_json_str);
        assert_eq!(expected_json_str.trim(), exec_prog_args.to_json().trim());
    }
}

#[test]
fn test_from_setting_file_program_arg() {
    for (case_yaml, case_json) in settings_file_test_case() {
        let test_case_yaml_file = &Path::new(&test_case_path())
            .join(case_yaml)
            .display()
            .to_string();
        let settings_file = load_file(test_case_yaml_file).unwrap();

        let test_case_json_file = &Path::new(&test_case_path())
            .join(case_json)
            .display()
            .to_string();
        let expected_json_str = fs::read_to_string(test_case_json_file).unwrap();

        let exec_prog_args = ExecProgramArgs::try_from(settings_file).unwrap();
        assert_eq!(expected_json_str.trim(), exec_prog_args.to_json().trim());
    }
}

fn test_exec_file() {
    let result = std::panic::catch_unwind(|| {
        for file in get_files(exec_data_path(), false, true, ".k") {
            exec(&file).unwrap();
            println!("{} - PASS", file);
        }
    });
    assert!(result.is_ok());
}

fn test_custom_manifests_output() {
    exec_with_result_at(&custom_manifests_data_path());
}

fn test_exec_with_err_result() {
    exec_with_err_result_at(&exec_err_data_path());
}

fn clean_dir(path: String) {
    if fs::remove_dir_all(path).is_ok() {}
}

#[test]
fn test_exec() {
    clean_dir(
        Path::new(".")
            .join("src")
            .join("exec_data")
            .join(".kcl")
            .display()
            .to_string(),
    );

    clean_dir(
        Path::new(".")
            .join("src")
            .join("exec_err_data")
            .join(".kcl")
            .display()
            .to_string(),
    );

    test_exec_file();
    println!("test_exec_file - PASS");

    test_kcl_runner_execute();
    println!("test_kcl_runner_execute - PASS");

    test_custom_manifests_output();
    println!("test_custom_manifests_output - PASS");

    test_exec_with_err_result();
    println!("test_exec_with_err_result - PASS");

    test_indent_error();
    println!("test_indent_error - PASS");

    test_compile_with_file_pattern();
    println!("test_compile_with_file_pattern - PASS");

    test_uuid();
    println!("test_uuid - PASS");
}

fn test_indent_error() {
    let test_path = PathBuf::from("./src/test_indent_error");
    let kcl_files = get_files(test_path.clone(), false, true, ".k");
    let output_files = get_files(test_path, false, true, ".stderr");

    for (kcl_file, err_file) in kcl_files.iter().zip(&output_files) {
        let mut args = ExecProgramArgs::default();
        args.k_filename_list.push(kcl_file.to_string());
        let res = exec_program(Arc::new(ParseSession::default()), &args);
        assert!(res.is_err());
        if let Err(err_msg) = res {
            let expect_err = fs::read_to_string(err_file).expect("Failed to read file");
            assert!(err_msg.to_string().contains(&expect_err));
        }
    }
}

fn exec(file: &str) -> Result<String, String> {
    let mut args = ExecProgramArgs::default();
    args.k_filename_list.push(file.to_string());
    let opts = args.get_load_program_options();
    let sess = Arc::new(ParseSession::default());
    // Load AST program
    let program = load_program(sess.clone(), &[file], Some(opts), None)
        .unwrap()
        .program;
    // Resolve ATS, generate libs, link libs and execute.
    match execute(sess, program, &args) {
        Ok(result) => {
            if result.err_message.is_empty() {
                Ok(result.json_result)
            } else {
                Err(result.err_message)
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

/// Run all kcl files at path and compare the exec result with the expect output.
fn exec_with_result_at(path: &str) {
    let kcl_files = get_files(path, false, true, ".k");
    let output_files = get_files(path, false, true, ".stdout.golden");
    for (kcl_file, output_file) in kcl_files.iter().zip(&output_files) {
        let mut args = ExecProgramArgs::default();
        args.k_filename_list.push(kcl_file.to_string());
        let result = exec_program(Arc::new(ParseSession::default()), &args).unwrap();

        #[cfg(not(target_os = "windows"))]
        let newline = "\n";
        #[cfg(target_os = "windows")]
        let newline = "\r\n";

        let expected_str = std::fs::read_to_string(output_file).unwrap();
        let expected = expected_str
            .strip_suffix(newline)
            .unwrap_or(&expected_str)
            .to_string();

        #[cfg(target_os = "windows")]
        let expected = expected.replace("\r\n", "\n");

        assert_eq!(
            result.yaml_result, expected,
            "test case {} {} failed",
            path, kcl_file
        );
    }
}

/// Run all kcl files at path and compare the exec error result with the expect error output.
fn exec_with_err_result_at(path: &str) {
    let kcl_files = get_files(path, false, true, ".k");
    let output_files = get_files(path, false, true, ".stderr.json");

    let prev_hook = std::panic::take_hook();
    // disable print panic info
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| {
        for (kcl_file, _) in kcl_files.iter().zip(&output_files) {
            let mut args = ExecProgramArgs::default();
            args.k_filename_list.push(kcl_file.to_string());
            let result = exec_program(Arc::new(ParseSession::default()), &args);
            if let Ok(result) = result {
                assert!(!result.err_message.is_empty(), "{}", result.err_message);
            } else {
                assert!(result.is_err());
            }
        }
    });
    assert!(result.is_ok());
    std::panic::set_hook(prev_hook);
}

/// Get kcl files from path.
fn get_files<P: AsRef<Path>>(
    path: P,
    recursively: bool,
    sorted: bool,
    suffix: &str,
) -> Vec<String> {
    let mut files = vec![];
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let file = path.to_str().unwrap();
            if file.ends_with(suffix) && (recursively || entry.depth() == 1) {
                files.push(file.to_string())
            }
        }
    }
    if sorted {
        files.sort();
    }
    files
}

fn test_compile_with_file_pattern() {
    let test_path = PathBuf::from("./src/test_file_pattern/**/main.k");
    let mut args = ExecProgramArgs::default();
    args.k_filename_list.push(test_path.display().to_string());
    let res = exec_program(Arc::new(ParseSession::default()), &args);
    assert!(res.is_ok());
    assert_eq!(
        res.as_ref().unwrap().yaml_result,
        "k3: Hello World!\nk1: Hello World!\nk2: Hello World!"
    );
    assert_eq!(
        res.as_ref().unwrap().json_result,
        "{\"k3\": \"Hello World!\", \"k1\": \"Hello World!\", \"k2\": \"Hello World!\"}"
    );
}

fn test_uuid() {
    let res = exec(
        &PathBuf::from(".")
            .join("src")
            .join("test_uuid")
            .join("main.k")
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
    );

    let v: Value = serde_json::from_str(res.clone().unwrap().as_str()).unwrap();
    assert!(v["a"].as_str().is_some());
    if let Some(uuid_str) = v["a"].as_str() {
        assert!(Uuid::parse_str(uuid_str).is_ok());
    }
}

#[test]
fn test_compile_with_symbolic_link() {
    let main_test_path = PathBuf::from("./src/test_symbolic_link/test_pkg/bbb/main.k");
    let mut args = ExecProgramArgs::default();
    args.k_filename_list
        .push(main_test_path.display().to_string());
    let res = exec_program(Arc::new(ParseSession::default()), &args);
    assert!(res.is_ok());
    assert_eq!(
        res.as_ref().unwrap().yaml_result,
        "The_first_kcl_program: Hello World!\nb: 1"
    );
    assert_eq!(
        res.as_ref().unwrap().json_result,
        "{\"The_first_kcl_program\": \"Hello World!\", \"b\": 1}"
    );
}

#[test]
fn test_kcl_issue_1799() {
    let main_test_path = PathBuf::from("./src/test_issues/github.com/kcl-lang/kcl/1799/main.k");
    let mut args = ExecProgramArgs::default();
    args.k_filename_list
        .push(main_test_path.display().to_string());
    args.work_dir = Some(".".to_string());
    let res = exec_program(Arc::new(ParseSession::default()), &args);
    assert!(res.is_ok());
    assert_eq!(
        res.as_ref().unwrap().yaml_result,
        format!(
            "a: {}",
            main_test_path
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap()
                .adjust_canonicalization()
        )
    );
}
