extern crate serde;

use kclvm_parser::load_program;
use kclvm_runner::execute;
use kclvm_runner::runner::*;
use kclvm_tools::query::apply_overrides;

#[no_mangle]
pub extern "C" fn kclvm_cli_run(args: *const i8, plugin_agent: *const i8) -> *const i8 {
    let prev_hook = std::panic::take_hook();

    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let kclvm_cli_run_unsafe_result =
        std::panic::catch_unwind(|| kclvm_cli_run_unsafe(args, plugin_agent));
    std::panic::set_hook(prev_hook);

    match kclvm_cli_run_unsafe_result {
        Ok(result) => match result {
            Ok(result) => {
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const i8
            }
            Err(result) => {
                let result = format!("ERROR:{}", result);
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const i8
            }
        },
        Err(panic_err) => {
            let err_message = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<&String>() {
                (*s).clone()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                (*s).clone()
            } else {
                "".to_string()
            };

            let result = format!("ERROR:{:}", err_message);
            let c_string = std::ffi::CString::new(result.as_str()).expect("CString::new failed");
            let ptr = c_string.into_raw();
            ptr as *const i8
        }
    }
}

pub fn kclvm_cli_run_unsafe(args: *const i8, plugin_agent: *const i8) -> Result<String, String> {
    let args = ExecProgramArgs::from_str(kclvm::c2str(args));
    let plugin_agent = plugin_agent as u64;

    let files = args.get_files();
    let opts = args.get_load_program_options();

    // Parse AST program.
    let mut program = load_program(&files, Some(opts))?;
    if let Err(msg) = apply_overrides(&mut program, &args.overrides, &[], args.print_override_ast) {
        return Err(msg.to_string());
    }

    // Resolve AST program, generate libs, link libs and execute.
    execute(program, plugin_agent, &args)
}
