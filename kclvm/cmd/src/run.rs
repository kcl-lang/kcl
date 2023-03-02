use clap::ArgMatches;
use kclvm_error::Handler;
use kclvm_runner::exec_program;
use kclvm_runtime::PanicInfo;

use crate::settings::must_build_settings;

/// Run the KCL main command.
pub fn run_command(matches: &ArgMatches) {
    // Config settings building
    let settings = must_build_settings(matches);
    let output = settings.output();
    match exec_program(&settings.into(), 1) {
        Ok(result) => match output {
            Some(o) => {
                std::fs::write(o, result.yaml_result).unwrap();
            }
            None => println!("{}", result.yaml_result),
        },
        Err(msg) => {
            Handler::default()
                .add_panic_info(&PanicInfo::from(msg))
                .abort_if_any_errors();
        }
    }
}
