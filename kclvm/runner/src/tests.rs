use crate::assembler::KclvmAssembler;
use crate::KclvmLibAssembler;
use crate::LlvmLibAssembler;
use crate::{execute, runner::ExecProgramArgs};
use kclvm_ast::ast::{Module, Program};
use kclvm_parser::load_program;
use kclvm_sema::resolver::resolve_program;
use std::{
    collections::HashMap,
    fs::{self, File},
};

const TEST_CASES: &[&'static str; 5] = &[
    "init_check_order_0",
    "init_check_order_1",
    "normal_2",
    "type_annotation_not_full_2",
    "multi_vars_0",
];
const EXPECTED_FILE_NAME: &str = "stdout.golden.json";
const TEST_CASE_PATH: &str = "./src/test_datas";
const KCL_FILE_NAME: &str = "main.k";
const MAIN_PKG_NAME: &str = "__main__";

/// Load test kcl file to ast.Program
pub fn load_test_program(filename: String) -> Program {
    let module = load_module(filename);
    construct_program(module)
}

/// Load test kcl file to ast.Module
pub fn load_module(filename: String) -> Module {
    kclvm_parser::parse_file(&filename, None).unwrap()
}

/// Construct ast.Program by ast.Module and default configuration.
/// Default configuration:
///     module.pkg = "__main__"
///     Program.root = "__main__"
///     Program.main = "__main__"
///     Program.cmd_args = []
///     Program.cmd_overrides = []
pub fn construct_program(mut module: Module) -> Program {
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

/// Load the expect result from stdout.golden.json
pub fn load_expect_file(filename: String) -> String {
    let f = File::open(filename).unwrap();
    let v: serde_json::Value = serde_json::from_reader(f).unwrap();
    v.to_string()
}

/// Format str by json str
pub fn format_str_by_json(str: String) -> String {
    let v: serde_json::Value = serde_json::from_str(&str).unwrap();
    v.to_string()
}

pub fn execute_for_test(kcl_path: &String) -> String {
    let plugin_agent = 0;
    let args = ExecProgramArgs::default();
    // Parse kcl file
    let program = load_test_program(kcl_path.to_string());
    // Generate libs, link libs and execute.
    execute(program, plugin_agent, &args).unwrap()
}

// TODO: need to fix issue #79
#[test]
fn test_kclvm_runner_execute() {
    for case in TEST_CASES {
        let kcl_path = &format!("{}/{}/{}", TEST_CASE_PATH, case, KCL_FILE_NAME);
        let expected_path = &format!("{}/{}/{}", TEST_CASE_PATH, case, EXPECTED_FILE_NAME);
        let result = execute_for_test(kcl_path);
        let expected_result = load_expect_file(expected_path.to_string());
        assert_eq!(expected_result, format_str_by_json(result));
    }
}
