use anyhow::Result;
use clap::ArgMatches;
use kclvm_error::Handler;
use kclvm_runner::ExecProgramArgs;
use kclvm_tools::lint::lint_files;

use crate::settings::must_build_settings;

/// Run the KCL lint command.
pub fn lint_command(matches: &ArgMatches) -> Result<()> {
    let mut files: Vec<&str> = match matches.values_of("input") {
        Some(files) => files.into_iter().collect::<Vec<&str>>(),
        None => vec![],
    };
    // Config settings building
    let settings = must_build_settings(matches);
    // Convert settings into execute arguments.
    let args: ExecProgramArgs = settings.try_into()?;
    files = if !files.is_empty() {
        files
    } else {
        args.get_files()
    };
    let (mut err_handler, mut warning_handler) = (Handler::default(), Handler::default());
    (err_handler.diagnostics, warning_handler.diagnostics) =
        lint_files(&files, Some(args.get_load_program_options()));
    if matches.occurrences_of("emit_warning") > 0 {
        warning_handler.emit()?;
    }
    err_handler.abort_if_any_errors();
    Ok(())
}
