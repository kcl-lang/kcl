//! The `kclvm` command-line interface.

#[macro_use]
extern crate clap;

use kclvm_runner::{execute, ExecProgramArgs};

use clap::ArgMatches;
use kclvm_config::settings::{load_file, merge_settings, SettingsFile};
use kclvm_parser::load_program;

fn main() {
    let matches = clap_app!(kcl =>
        (@subcommand run =>
            (@arg INPUT: ... "Sets the input file to use")
            (@arg OUTPUT: -o --output +takes_value "Sets the LLVM IR/BC output file path")
            (@arg SETTING: ... -Y --setting +takes_value "Sets the input file to use")
            (@arg EMIT_TYPE: --emit +takes_value "Sets the emit type, expect (ast)")
            (@arg BC_PATH: --bc +takes_value "Sets the linked LLVM bitcode file path")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg disable_none: -n --disable-none "Disable dumping None values")
            (@arg debug: -d --debug "Run in debug mode (for developers only)")
            (@arg sort_key: -k --sort "Sort result keys")
            (@arg ARGUMENT: ... -D --argument "Specify the top-level argument")
        )
    )
    .get_matches();
    if let Some(matches) = matches.subcommand_matches("run") {
        match (matches.values_of("INPUT"), matches.values_of("SETTING")) {
            (None, None) => {
                println!("{}", matches.usage());
            }
            (_, _) => {
                let mut files: Vec<&str> = match matches.values_of("INPUT") {
                    Some(files) => files.into_iter().collect::<Vec<&str>>(),
                    None => vec![],
                };
                // Config settings build
                let settings = build_settings(&matches);
                // Convert settings into execute arguments.
                let args: ExecProgramArgs = settings.into();
                files = if !files.is_empty() {
                    files
                } else {
                    args.get_files()
                };
                // Parse AST program.
                let program = load_program(&files, Some(args.get_load_program_options())).unwrap();
                // Resolve AST program, generate libs, link libs and execute.
                // TODO: The argument "plugin_agent" need to be read from python3.
                execute(program, 0, &ExecProgramArgs::default()).unwrap();
            }
        }
    } else {
        println!("{}", matches.usage());
    }
}

/// Build settings from arg matches.
fn build_settings(matches: &ArgMatches) -> SettingsFile {
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
        config.debug = Some(debug_mode);
        config.disable_none = Some(disable_none);
    }
    settings
}
