use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, Criterion};
use kclvm_ast::ast::{Module, Program};
use kclvm_runner::{execute, runner::ExecProgramArgs};
use kclvm_sema::resolver::resolve_program;

const MAIN_PKG_NAME: &str = "__main__";
const TEST_CASE_PATH: &str = "/src/test_datas/init_check_order_0/main.k";

/// Load test kcl file to ast.Program
fn load_program(filename: String) -> Program {
    let module = load_module(filename);
    construct_program(module)
}

/// Load test kcl file to ast.Module
fn load_module(filename: String) -> Module {
    kclvm_parser::parse_file(&filename, None).unwrap()
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

pub fn execute_for_test(kcl_path: &String) -> String {
    let plugin_agent = 0;
    let args = ExecProgramArgs::default();
    // parse kcl file
    let mut program = load_program(kcl_path.to_string());
    // resolve ast
    let scope = resolve_program(&mut program);
    scope.check_scope_diagnostics();
    // generate dylibs, link dylibs and execute.
    execute(program, scope, plugin_agent, &args).unwrap()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let path = &format!(
        "{}{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        TEST_CASE_PATH
    );
    c.bench_function("kclvm-runner: ", |b| {
        b.iter(|| {
            execute_for_test(path);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
