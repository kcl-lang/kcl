use kclvm_ast::ast::Program;
use kclvm_sema::resolver::resolve_program;
use runner::{ExecProgramArgs, KclvmRunner, KclvmRunnerOptions};

pub mod assembler;
pub mod command;
pub mod linker;
pub mod runner;

#[cfg(test)]
pub mod tests;

/// After the kcl program passed through kclvm-parser in the compiler frontend,
/// KCLVM needs to resolve ast, generate corresponding LLVM IR, dylibs or
/// executable file for kcl program in the compiler backend.
///
/// Method “execute” is the entry point for the compiler backend.
///
/// It returns the KCL program executing result as Result<a_json_string, an_err_string>,
/// and mainly takes "program" (ast.Program returned by kclvm-parser) as input.
///
/// "plugin_agent" is related to KCLVM plugin.
/// "args" is the items selected by the user in the KCLVM CLI.
///
/// This method will first resolve “program” (ast.Program) and save the result to the "scope" (ProgramScope).
///
/// Then, dylibs is generated by KclvmAssembler, and method "KclvmAssembler::gen_dylibs"
/// will return dylibs path in a "Vec<String>";
///
/// After linking all dylibs by KclvmLinker, method "KclvmLinker::link_all_dylibs" will return a path
/// for dylib.
///
/// At last, KclvmRunner will be constructed and call method "run" to execute the kcl program.
///
/// # Examples
///
/// ```
/// use kclvm_runner::{execute, runner::ExecProgramArgs};
/// use kclvm_parser::load_program;
/// use kclvm_ast::ast::Program;
///
/// fn main() {
///    // default plugin agent
///    let plugin_agent = 0;
///
///    // get default args
///    let args = ExecProgramArgs::default();
///    let opts = args.get_load_program_options();
///
///    // parse kcl file
///    let kcl_path = "./src/test_datas/init_check_order_0/main.k";
///    let prog = load_program(&[kcl_path], Some(opts)).unwrap();
///        
///    // resolve ast, generate dylibs, link dylibs and execute.
///    // result is the kcl in json format.
///    let result = execute(prog, plugin_agent, &args).unwrap();
/// }
/// ```
///
pub fn execute(
    mut program: Program,
    plugin_agent: u64,
    args: &ExecProgramArgs,
) -> Result<String, String> {
    // resolve ast
    let scope = resolve_program(&mut program);
    scope.check_scope_diagnostics();

    // generate dylibs
    let dylib_paths = assembler::KclvmAssembler::gen_dylibs(program, scope, plugin_agent);

    // link dylibsKclvmRunner
    let dylib_path = linker::KclvmLinker::link_all_dylibs(dylib_paths, plugin_agent);

    // run
    let runner = KclvmRunner::new(
        dylib_path.as_str(),
        Some(KclvmRunnerOptions {
            plugin_agent_ptr: plugin_agent,
        }),
    );
    runner.run(&args)
}
