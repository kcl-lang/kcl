use std::path::Path;

use assembler::KclvmLibAssembler;
use command::Command;
use kclvm_ast::ast::Program;
use kclvm_sema::resolver::resolve_program;
pub use runner::ExecProgramArgs;
use runner::{KclvmRunner, KclvmRunnerOptions};
use tempfile::tempdir;

pub mod assembler;
pub mod command;
pub mod linker;
pub mod runner;

#[cfg(test)]
pub mod tests;

/// After the kcl program passed through kclvm-parser in the compiler frontend,
/// KCLVM needs to resolve ast, generate corresponding LLVM IR, dynamic link library or
/// executable file for kcl program in the compiler backend.
///
/// Method “execute” is the entry point for the compiler backend.
///
/// It returns the KCL program executing result as Result<a_json_string, an_err_string>,
/// and mainly takes "program" (ast.Program returned by kclvm-parser) as input.
///
/// "args" is the items selected by the user in the KCLVM CLI.
///
/// This method will first resolve “program” (ast.Program) and save the result to the "scope" (ProgramScope).
///
/// Then, dynamic link libraries is generated by KclvmAssembler, and method "KclvmAssembler::gen_libs"
/// will return dynamic link library paths in a "Vec<String>";
///
/// KclvmAssembler is mainly responsible for concurrent compilation of multiple files.
/// Single-file compilation in each thread in concurrent compilation is the responsibility of KclvmLibAssembler.
/// In the future, it may support the dynamic link library generation of multiple intermediate language.
/// KclvmLibAssembler currently only supports LLVM IR.
///
/// After linking all dynamic link libraries by KclvmLinker, method "KclvmLinker::link_all_libs" will return a path
/// for dynamic link library after linking.
///
/// At last, KclvmRunner will be constructed and call method "run" to execute the kcl program.
///
/// # Examples
///
/// ```
/// use kclvm_runner::{execute, runner::ExecProgramArgs};
/// use kclvm_parser::load_program;
/// use kclvm_ast::ast::Program;
/// // plugin_agent is the address of plugin.
/// let plugin_agent = 0;
/// // Get default args
/// let args = ExecProgramArgs::default();
/// let opts = args.get_load_program_options();
///
/// // Parse kcl file
/// let kcl_path = "./src/test_datas/init_check_order_0/main.k";
/// let prog = load_program(&[kcl_path], Some(opts)).unwrap();
///     
/// // Resolve ast, generate libs, link libs and execute.
/// // Result is the kcl in json format.
/// let result = execute(prog, plugin_agent, &args).unwrap();
/// ```
pub fn execute(
    mut program: Program,
    plugin_agent: u64,
    args: &ExecProgramArgs,
) -> Result<String, String> {
    // Resolve ast
    let scope = resolve_program(&mut program);
    scope.check_scope_diagnostics();

    // Create a temp entry file and the temp dir will be delete automatically
    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_str().unwrap();
    let temp_entry_file = temp_file(temp_dir_path);

    // Generate libs
    let lib_paths = assembler::KclvmAssembler::default().gen_libs(
        program,
        scope,
        &temp_entry_file,
        KclvmLibAssembler::LLVM,
    );

    // Link libs
    let lib_suffix = Command::get_lib_suffix();
    let temp_out_lib_file = format!("{}.out{}", temp_entry_file, lib_suffix);
    let lib_path = linker::KclvmLinker::link_all_libs(lib_paths, temp_out_lib_file);

    // Run
    let runner = KclvmRunner::new(
        lib_path.as_str(),
        Some(KclvmRunnerOptions {
            plugin_agent_ptr: plugin_agent,
        }),
    );
    let result = runner.run(args);

    // Clean temp files
    remove_file(&lib_path);
    clean_tmp_files(&temp_entry_file, &lib_suffix);
    result
}

/// Clean all the tmp files generated during lib generating and linking.
#[inline]
fn clean_tmp_files(temp_entry_file: &String, lib_suffix: &String) {
    let temp_entry_lib_file = format!("{}{}", temp_entry_file, lib_suffix);
    remove_file(&temp_entry_lib_file);
}

#[inline]
fn remove_file(file: &str) {
    if Path::new(&file).exists() {
        std::fs::remove_file(&file).unwrap();
    }
}

/// Returns a temporary file name consisting of timestamp and process id.
fn temp_file(dir: &str) -> String {
    let timestamp = chrono::Local::now().timestamp_nanos();
    let id = std::process::id();
    let file = format!("{}_{}", id, timestamp);
    std::fs::create_dir_all(dir).unwrap();
    Path::new(dir).join(file).to_str().unwrap().to_string()
}
