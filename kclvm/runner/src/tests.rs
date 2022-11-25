use crate::assembler::clean_path;
use crate::assembler::KclvmAssembler;
use crate::assembler::KclvmLibAssembler;
use crate::assembler::LibAssembler;
use crate::exec_program;
use crate::temp_file;
use crate::{execute, runner::ExecProgramArgs};
use anyhow::Context;
use anyhow::Result;
use kclvm_ast::ast::{Module, Program};
use kclvm_compiler::codegen::llvm::OBJECT_FILE_SUFFIX;
use kclvm_config::settings::load_file;
use kclvm_parser::load_program;
use kclvm_sema::resolver::resolve_program;
use std::fs::create_dir_all;
use std::panic::catch_unwind;
use std::panic::set_hook;
use std::path::{Path, PathBuf};
use std::thread;
use std::{
    collections::HashMap,
    fs::{self, File},
};
use tempfile::tempdir;
use walkdir::WalkDir;

const EXEC_DATA_PATH: &str = "src/exec_data/";
const CUSTOM_MANIFESTS_DATA_PATH: &str = "src/custom_manifests_data/";
const TEST_CASES: &[&str; 5] = &[
    "init_check_order_0",
    "init_check_order_1",
    "normal_2",
    "type_annotation_not_full_2",
    "multi_vars_0",
];

const MULTI_FILE_TEST_CASES: &[&str; 7] = &[
    "multi_file_compilation/no_kcl_mod_file",
    "multi_file_compilation/relative_import",
    "multi_file_compilation/relative_import_as",
    "multi_file_compilation/import_abs_path/app-main",
    "multi_file_compilation/import_regular_module",
    "multi_file_compilation/import_regular_module_as",
    "../../../../test/konfig/base/examples/job-example/dev",
];

const EXEC_PROG_ARGS_TEST_CASE: &[&str; 1] = &["exec_prog_args/default.json"];

const SETTINGS_FILE_TEST_CASE: &[&(&str, &str); 1] =
    &[&("settings_file/settings.yaml", "settings_file/settings.json")];

const EXPECTED_JSON_FILE_NAME: &str = "stdout.golden.json";
const TEST_CASE_PATH: &str = "src/test_datas";
const KCL_FILE_NAME: &str = "main.k";
const MAIN_PKG_NAME: &str = "__main__";
const CARGO_PATH: &str = env!("CARGO_MANIFEST_DIR");

fn gen_full_path(rel_path: String) -> Result<String> {
    let mut cargo_file_path = PathBuf::from(CARGO_PATH);
    cargo_file_path.push(&rel_path);
    let full_path = cargo_file_path
        .to_str()
        .with_context(|| format!("No such file or directory '{}'", rel_path))?;
    Ok(full_path.to_string())
}

/// Load test kcl file to ast.Program
fn load_test_program(filename: String) -> Program {
    let module = kclvm_parser::parse_file(&filename, None).unwrap();
    construct_program(module)
}

fn parse_program(test_kcl_case_path: &str) -> Program {
    let args = ExecProgramArgs::default();
    let opts = args.get_load_program_options();
    load_program(&[test_kcl_case_path], Some(opts)).unwrap()
}

/// Construct ast.Program by ast.Module and default configuration.
/// Default configuration:
///     module.pkg = "__main__"
///     Program.root = "__main__"
///     Program.main = "__main__"
///     Program.cmd_args = []
///     Program.cmd_overrides = []
fn construct_program(mut module: Module) -> Program {
    module.pkg = MAIN_PKG_NAME.to_string();
    let mut pkgs_ast = HashMap::new();
    pkgs_ast.insert(MAIN_PKG_NAME.to_string(), vec![module]);
    Program {
        root: MAIN_PKG_NAME.to_string(),
        main: MAIN_PKG_NAME.to_string(),
        pkgs: pkgs_ast,
        cmd_args: vec![],
        cmd_overrides: vec![],
    }
}

fn construct_pkg_lib_path(
    prog: &Program,
    assembler: &KclvmAssembler,
    main_path: &str,
    suffix: String,
) -> Vec<PathBuf> {
    let cache_dir = assembler.construct_cache_dir(&prog.root);
    let mut result = vec![];
    for (pkgpath, _) in &prog.pkgs {
        if pkgpath == "__main__" {
            result.push(PathBuf::from(format!("{}{}", main_path, suffix)));
        } else {
            result.push(cache_dir.join(format!("{}{}", pkgpath.clone(), suffix)));
        }
    }
    result
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
    let plugin_agent = 0;
    let args = ExecProgramArgs::default();
    // Parse kcl file
    let program = load_test_program(kcl_path.to_string());
    // Generate libs, link libs and execute.
    execute(program, plugin_agent, &args).unwrap()
}

fn gen_assembler(entry_file: &str, test_kcl_case_path: &str) -> KclvmAssembler {
    let mut prog = parse_program(test_kcl_case_path);
    let scope = resolve_program(&mut prog);
    KclvmAssembler::new(
        prog.clone(),
        scope,
        entry_file.to_string(),
        KclvmLibAssembler::LLVM,
    )
}

fn gen_libs_for_test(entry_file: &str, test_kcl_case_path: &str) {
    let assembler = gen_assembler(entry_file, test_kcl_case_path);

    let expected_pkg_paths = construct_pkg_lib_path(
        &parse_program(test_kcl_case_path),
        &assembler,
        PathBuf::from(entry_file).to_str().unwrap(),
        OBJECT_FILE_SUFFIX.to_string(),
    );

    let lib_paths = assembler.gen_libs();

    assert_eq!(lib_paths.len(), expected_pkg_paths.len());

    for pkg_path in &expected_pkg_paths {
        assert_eq!(pkg_path.exists(), true);
    }

    let tmp_main_lib_path =
        fs::canonicalize(format!("{}{}", entry_file, OBJECT_FILE_SUFFIX)).unwrap();
    assert_eq!(tmp_main_lib_path.exists(), true);

    clean_path(tmp_main_lib_path.to_str().unwrap());
    assert_eq!(tmp_main_lib_path.exists(), false);
}

fn assemble_lib_for_test(
    entry_file: &str,
    test_kcl_case_path: &str,
    assembler: &KclvmLibAssembler,
) -> String {
    // default args and configuration
    let mut args = ExecProgramArgs::default();

    args.k_filename_list.push(test_kcl_case_path.to_string());
    let files = args.get_files();
    let opts = args.get_load_program_options();

    // parse and resolve kcl
    let mut program = load_program(&files, Some(opts)).unwrap();

    let scope = resolve_program(&mut program);

    // tmp file
    let temp_entry_file_path = &format!("{}{}", entry_file, OBJECT_FILE_SUFFIX);

    // Assemble object files
    assembler.assemble(
        &program,
        scope.import_names,
        entry_file,
        temp_entry_file_path,
    )
}

fn test_kclvm_runner_execute() {
    for case in TEST_CASES {
        let kcl_path = &format!("{}/{}/{}", TEST_CASE_PATH, case, KCL_FILE_NAME);
        let expected_path = &format!("{}/{}/{}", TEST_CASE_PATH, case, EXPECTED_JSON_FILE_NAME);
        let result = execute_for_test(kcl_path);
        let expected_result = load_expect_file(expected_path.to_string());
        assert_eq!(expected_result, format_str_by_json(result));
    }
}

fn test_kclvm_runner_execute_timeout() {
    set_hook(Box::new(|_| {}));
    let result_time_out = catch_unwind(|| {
        gen_libs_for_test(
            "test/no_exist_path/",
            "./src/test_datas/multi_file_compilation/import_abs_path/app-main/main.k",
        );
    });
    let timeout_panic_msg = "called `Result::unwrap()` on an `Err` value: Timeout";
    match result_time_out {
        Err(panic_err) => {
            if let Some(s) = panic_err.downcast_ref::<String>() {
                assert_eq!(s, timeout_panic_msg)
            }
        }
        _ => {
            unreachable!()
        }
    }
}

#[test]
fn test_assemble_lib_llvm() {
    for case in TEST_CASES {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();
        let temp_entry_file = temp_file(temp_dir_path);

        let kcl_path = &format!("{}/{}/{}", TEST_CASE_PATH, case, KCL_FILE_NAME);
        let assembler = &KclvmLibAssembler::LLVM;

        let lib_file = assemble_lib_for_test(
            &format!("{}{}", temp_entry_file, "4assemble_lib"),
            kcl_path,
            assembler,
        );

        let lib_path = std::path::Path::new(&lib_file);
        assert_eq!(lib_path.exists(), true);
        clean_path(&lib_file);
        assert_eq!(lib_path.exists(), false);
    }
}

#[test]
fn test_gen_libs() {
    for case in MULTI_FILE_TEST_CASES {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();
        let temp_entry_file = temp_file(temp_dir_path);

        let kcl_path =
            gen_full_path(format!("{}/{}/{}", TEST_CASE_PATH, case, KCL_FILE_NAME)).unwrap();
        gen_libs_for_test(&format!("{}{}", temp_entry_file, "4gen_libs"), &kcl_path);
    }
}

#[test]
fn test_gen_libs_parallel() {
    let gen_lib_1 = thread::spawn(|| {
        for _ in 0..9 {
            test_gen_libs();
        }
    });

    let gen_lib_2 = thread::spawn(|| {
        for _ in 0..9 {
            test_gen_libs();
        }
    });

    gen_lib_1.join().unwrap();
    gen_lib_2.join().unwrap();
}

#[test]
fn test_clean_path_for_genlibs() {
    let mut prog =
        parse_program("./src/test_datas/multi_file_compilation/import_abs_path/app-main/main.k");
    let scope = resolve_program(&mut prog);
    let assembler = KclvmAssembler::new(prog, scope, String::new(), KclvmLibAssembler::LLVM);

    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_str().unwrap();
    let tmp_file_path = &temp_file(temp_dir_path);

    create_dir_all(tmp_file_path).unwrap();

    let file_name = &format!("{}/{}", tmp_file_path, "test");
    let file_suffix = ".o";

    File::create(file_name).unwrap();
    let path = std::path::Path::new(file_name);
    assert_eq!(path.exists(), true);

    assembler.clean_path_for_genlibs(file_name, file_suffix);
    assert_eq!(path.exists(), false);

    let test1 = &format!("{}{}", file_name, ".test1.o");
    let test2 = &format!("{}{}", file_name, ".test2.o");
    File::create(test1).unwrap();
    File::create(test2).unwrap();
    let path1 = std::path::Path::new(test1);

    let path2 = std::path::Path::new(test2);
    assert_eq!(path1.exists(), true);
    assert_eq!(path2.exists(), true);

    assembler.clean_path_for_genlibs(file_name, file_suffix);
    assert_eq!(path1.exists(), false);
    assert_eq!(path2.exists(), false);
}

#[test]
fn test_to_json_program_arg() {
    for case in EXEC_PROG_ARGS_TEST_CASE {
        let test_case_json_file = &format!("{}/{}", TEST_CASE_PATH, case);
        let expected_json_str = fs::read_to_string(test_case_json_file).unwrap();
        let exec_prog_args = ExecProgramArgs::default();
        assert_eq!(expected_json_str.trim(), exec_prog_args.to_json().trim());
    }
}

#[test]
fn test_from_str_program_arg() {
    for case in EXEC_PROG_ARGS_TEST_CASE {
        let test_case_json_file = &format!("{}/{}", TEST_CASE_PATH, case);
        let expected_json_str = fs::read_to_string(test_case_json_file).unwrap();
        let exec_prog_args = ExecProgramArgs::from_str(&expected_json_str);
        assert_eq!(expected_json_str.trim(), exec_prog_args.to_json().trim());
    }
}

#[test]
fn test_from_setting_file_program_arg() {
    for (case_yaml, case_json) in SETTINGS_FILE_TEST_CASE {
        let test_case_yaml_file = &format!("{}/{}", TEST_CASE_PATH, case_yaml);
        let settings_file = load_file(test_case_yaml_file);

        let test_case_json_file = &format!("{}/{}", TEST_CASE_PATH, case_json);
        let expected_json_str = fs::read_to_string(test_case_json_file).unwrap();

        let exec_prog_args = ExecProgramArgs::from(settings_file);
        assert_eq!(expected_json_str.trim(), exec_prog_args.to_json().trim());
    }
}

fn test_exec_file() {
    let prev_hook = std::panic::take_hook();
    // disable print panic info
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| {
        for file in get_files(EXEC_DATA_PATH, false, true, ".k") {
            exec(&file).unwrap();
        }
    });
    assert!(result.is_ok());
    std::panic::set_hook(prev_hook);
}

fn test_custom_manifests_output() {
    exec_with_result_at(CUSTOM_MANIFESTS_DATA_PATH)
}

#[test]
fn test_exec() {
    test_exec_file();
    test_kclvm_runner_execute();
    test_kclvm_runner_execute_timeout();
    test_custom_manifests_output();
}

fn exec(file: &str) -> Result<String, String> {
    let mut args = ExecProgramArgs::default();
    args.k_filename_list.push(file.to_string());
    let plugin_agent = 0;
    let opts = args.get_load_program_options();
    // Load AST program
    let program = load_program(&[file], Some(opts)).unwrap();
    // Resolve ATS, generate libs, link libs and execute.
    execute(program, plugin_agent, &args)
}

/// Run all kcl files at path and compare the exec result with the expect output.
fn exec_with_result_at(path: &str) {
    let kcl_files = get_files(path, false, true, ".k");
    let output_files = get_files(path, false, true, ".stdout.golden");
    for (kcl_file, output_file) in kcl_files.iter().zip(&output_files) {
        let mut args = ExecProgramArgs::default();
        args.k_filename_list.push(kcl_file.to_string());
        let result = exec_program(&args, 0).unwrap();
        let expected = std::fs::read_to_string(output_file)
            .unwrap()
            .strip_suffix("\n")
            .unwrap()
            .to_string();
        assert_eq!(result.yaml_result, expected);
    }
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
