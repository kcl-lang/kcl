//! The `kclvm` command-line interface.

#[macro_use]
extern crate clap;

use clap::ArgMatches;
use kclvm::PanicInfo;
use kclvm_config::settings::{load_file, merge_settings, SettingsFile};
use kclvm_error::Handler;
use kclvm_runner::{exec_program, ExecProgramArgs};
use kclvm_tools::lint::lint_files;

fn main() {
    let matches = clap_app!(kcl =>
        (@subcommand run =>
            (@arg INPUT: ... "Sets the input file to use")
            (@arg OUTPUT: -o --output +takes_value "Sets the LLVM IR/BC output file path")
            (@arg SETTING: ... -Y --setting +takes_value "Sets the input file to use")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg disable_none: -n --disable-none "Disable dumping None values")
            (@arg debug: -d --debug "Run in debug mode (for developers only)")
            (@arg sort_key: -k --sort "Sort result keys")
            (@arg ARGUMENT: ... -D --argument "Specify the top-level argument")
        )
        (@subcommand lint =>
            (@arg INPUT: ... "Sets the input file to use")
            (@arg OUTPUT: -o --output +takes_value "Sets the LLVM IR/BC output file path")
            (@arg SETTING: ... -Y --setting +takes_value "Sets the input file to use")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg disable_none: -n --disable-none "Disable dumping None values")
            (@arg debug: -d --debug "Run in debug mode (for developers only)")
            (@arg sort_key: -k --sort "Sort result keys")
            (@arg ARGUMENT: ... -D --argument "Specify the top-level argument")
            (@arg EMIT_WARNING: --emit_warning "Emit warning message")
        )
    )
    .arg_required_else_help(true)
    .get_matches();
    if let Some(matches) = matches.subcommand_matches("run") {
        let (files, setting) = (matches.values_of("INPUT"), matches.values_of("SETTING"));
        match (files, setting) {
            (None, None) => println!("Error: no KCL files"),
            (_, _) => {
                // Config settings build
                let settings = build_settings(matches);
                match exec_program(&settings.into(), 1) {
                    Ok(result) => {
                        println!("{}", result.yaml_result);
                    }
                    Err(msg) => {
                        let mut handler = Handler::default();
                        handler
                            .add_panic_info(&PanicInfo::from_json_string(&msg))
                            .abort_if_any_errors();
                    }
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("lint") {
        let (files, setting) = (matches.values_of("INPUT"), matches.values_of("SETTING"));
        match (files, setting) {
            (None, None) => println!("Error: no KCL files"),
            (_, _) => {
                let mut files: Vec<&str> = match matches.values_of("INPUT") {
                    Some(files) => files.into_iter().collect::<Vec<&str>>(),
                    None => vec![],
                };
                // Config settings build
                let settings = build_settings(matches);
                // Convert settings into execute arguments.
                let args: ExecProgramArgs = settings.into();
                files = if !files.is_empty() {
                    files
                } else {
                    args.get_files()
                };
                let (mut err_handler, mut warning_handler) =
                    (Handler::default(), Handler::default());
                (err_handler.diagnostics, warning_handler.diagnostics) =
                    lint_files(&files, Some(args.get_load_program_options()));
                err_handler.emit();
                if matches.occurrences_of("EMIT_WARNING") > 0 {
                    warning_handler.emit();
                }
            }
        }
    }
}

/// Build settings from arg matches.
fn build_settings(matches: &ArgMatches) -> SettingsFile {
    let files: Vec<&str> = match matches.values_of("INPUT") {
        Some(files) => files.into_iter().collect::<Vec<&str>>(),
        None => vec![],
    };
    let debug_mode = matches.occurrences_of("debug") > 0;
    let disable_none = matches.occurrences_of("disable_none") > 0;

    let mut settings = if let Some(files) = matches.values_of("SETTING") {
        let files: Vec<&str> = files.into_iter().collect::<Vec<&str>>();
        merge_settings(
            &files
                .iter()
                .map(|f| load_file(f))
                .collect::<Vec<SettingsFile>>(),
        )
    } else {
        SettingsFile::new()
    };
    if let Some(config) = &mut settings.kcl_cli_configs {
        if !files.is_empty() {
            config.files = Some(files.iter().map(|f| f.to_string()).collect());
        }
        config.debug = Some(debug_mode);
        config.disable_none = Some(disable_none);
    }
    settings
}
