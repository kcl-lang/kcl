//! The `kclvm` command-line interface.

#[macro_use]
extern crate clap;

pub mod fmt;
pub mod lint;
pub mod run;
pub mod settings;
pub(crate) mod util;
pub mod vet;

#[cfg(test)]
mod tests;

use std::io;

use anyhow::Result;
use fmt::fmt_command;
use lint::lint_command;
use run::run_command;
use vet::vet_command;

/// Run the KCL main command.
pub fn main(args: &[&str]) -> Result<()> {
    let matches = app().arg_required_else_help(true).get_matches_from(args);
    // Sub commands
    if let Some(matches) = matches.subcommand_matches("run") {
        run_command(matches, &mut io::stdout())
    } else if let Some(matches) = matches.subcommand_matches("lint") {
        lint_command(matches)
    } else if let Some(matches) = matches.subcommand_matches("fmt") {
        fmt_command(matches)
    } else if let Some(matches) = matches.subcommand_matches("vet") {
        vet_command(matches)
    } else if matches.subcommand_matches("server").is_some() {
        kclvm_api::service::jsonrpc::start_stdio_server()
    } else if matches.subcommand_matches("version").is_some() {
        println!("{}", kclvm_version::get_version_info());
        Ok(())
    } else {
        Ok(())
    }
}

/// Get the KCLVM CLI application.
pub fn app() -> clap::App<'static> {
    clap_app!(kclvm_cli =>
        (version: kclvm_version::VERSION)
        (about: "KCL main CLI")
        (@subcommand run =>
            (about: "Run KCL files")
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
            (@arg package_map: ... -E --external +takes_value "Mapping of package name and path where the package is located")
        )
        (@subcommand lint =>
            (about: "Lint KCL files")
            (@arg input: ... "Sets the input file to use")
            (@arg setting: ... -Y --setting +takes_value "Sets the input file to use")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg emit_warning: --emit_warning "Emit warning message")
        )
        (@subcommand fmt =>
            (about: "Format KCL files")
            (@arg input: "Input file or path name for formatting")
            (@arg recursive: -R --recursive "Iterate through subdirectories recursively")
            (@arg std_output: -w --std_output "Whether to output format to stdout")
        )
        (@subcommand vet =>
            (about: "Validate data files with KCL files")
            (@arg data_file: "Validation data file")
            (@arg kcl_file: "KCL file")
            (@arg schema: -d --schema +takes_value "Iterate through subdirectories recursively")
            (@arg attribute_name: -n --attribute_name +takes_value "The attribute name for the data loading")
            (@arg format: --format +takes_value "Validation data file format, support YAML and JSON, default is JSON")
        )
        (@subcommand server =>
            (about: "Start a rpc server for APIs")
        )
        (@subcommand version =>
            (about: "Show the KCL version")
        )
    )
}
