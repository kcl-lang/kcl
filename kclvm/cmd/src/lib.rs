//! The `kclvm` command-line interface.

#[macro_use]
extern crate clap;

pub mod lint;
pub mod run;
pub mod settings;

#[cfg(test)]
mod tests;

use anyhow::Result;
use lint::lint_command;
use run::run_command;

/// Run the KCL main command.
pub fn main(args: &[&str]) -> Result<()> {
    let matches = app().arg_required_else_help(true).get_matches_from(args);
    // Sub commands
    if let Some(matches) = matches.subcommand_matches("run") {
        run_command(matches)
    } else if let Some(matches) = matches.subcommand_matches("lint") {
        lint_command(matches)
    } else if let Some(_matches) = matches.subcommand_matches("server") {
        kclvm_api::service::jsonrpc::start_stdio_server()
    } else if matches.subcommand_matches("version").is_some() {
        println!("{}", kclvm_version::get_full_version());
        Ok(())
    } else {
        Ok(())
    }
}

/// Get the KCLVM CLI application.
pub fn app() -> clap::App<'static> {
    clap_app!(kclvm_cli =>
        (@subcommand run =>
            (@arg input: ... "Specify the input files to run")
            (@arg output: -o --output +takes_value "Specify the YAML output file path")
            (@arg setting: ... -Y --setting +takes_value "Specify the input setting file")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg disable_none: -n --disable_none "Disable dumping None values")
            (@arg strict_range_check: -r --strict_range_check "Do perform strict numeric range checks")
            (@arg debug: -d --debug "Run in debug mode (for developers only)")
            (@arg sort_key: -k --sort "Sort result keys")
            (@arg arguments: ... -D --argument +takes_value "Specify the top-level argument")
            (@arg path_selector: ... -S --path_selector "Specify the path selector")
            (@arg overrides: ... -O --overrides +takes_value "Specify the configuration override path and value")
            (@arg target: --target +takes_value "Specify the target type")
        )
        (@subcommand lint =>
            (@arg input: ... "Sets the input file to use")
            (@arg setting: ... -Y --setting +takes_value "Sets the input file to use")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg emit_warning: --emit_warning "Emit warning message")
        )
        (@subcommand server =>
        )
        (@subcommand version =>
        )
    )
}
