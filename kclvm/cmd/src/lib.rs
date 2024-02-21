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

use clap::{ArgAction, Command};

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
    match matches.subcommand() {
        Some(("run", sub_matches)) => run_command(sub_matches, &mut io::stdout()),
        Some(("lint", sub_matches)) => lint_command(sub_matches),
        Some(("fmt", sub_matches)) => fmt_command(sub_matches),
        Some(("vet", sub_matches)) => vet_command(sub_matches),
        Some(("server", _)) => kclvm_api::service::jsonrpc::start_stdio_server(),
        Some(("version", _)) => {
            println!("{}", kclvm_version::get_version_info());
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Get the KCLVM CLI application.
pub fn app() -> Command {
    Command::new("kclvm_cli")
        .version(kclvm_version::VERSION)
        .about("KCL main CLI.")
        .subcommand(
            Command::new("run")
            .about("run")
            .arg(arg!([input] ... "Specify the input files to run").num_args(0..))
            .arg(arg!(output: -o --output <output> "Specify the YAML output file path"))
            .arg(arg!(setting: -Y --setting <setting> ... "Specify the input setting file").num_args(1..))
            .arg(arg!(verbose: -v --verbose "Print test information verbosely").action(ArgAction::Count))
            .arg(arg!(disable_none: -n --disable_none "Disable dumping None values"))
            .arg(arg!(strict_range_check: -r --strict_range_check "Do perform strict numeric range checks"))
            .arg(arg!(debug: -d --debug "Run in debug mode (for developers only)"))
            .arg(arg!(sort_keys: -k --sort_keys "Sort result keys"))
            .arg(arg!(show_hidden: -H --show_hidden "Display hidden attributes"))
            .arg(arg!(arguments: -D --argument <arguments> ... "Specify the top-level argument").num_args(1..))
            .arg(arg!(path_selector: -S --path_selector <path_selector> ... "Specify the path selector").num_args(1..))
            .arg(arg!(overrides: -O --overrides <overrides> ... "Specify the configuration override path and value").num_args(1..))
            .arg(arg!(target: --target <target> "Specify the target type"))
            .arg(arg!(recursive: -R --recursive "Compile the files directory recursively"))
            .arg(arg!(package_map: -E --external <package_map> ... "Mapping of package name and path where the package is located").num_args(1..)),
        )
        .subcommand(
            Command::new("lint")
            .about("lint")
            .arg(arg!([input] ... "Sets the input file to use").num_args(0..))
            .arg(arg!(output: -o --output <output> "Specify the YAML output file path"))
            .arg(arg!(setting: -Y --setting <setting> ... "Sets the input file to use").num_args(1..))
            .arg(arg!(verbose: -v --verbose "Print test information verbosely").action(ArgAction::Count))
            .arg(arg!(emit_warning: --emit_warning "Emit warning message"))
            .arg(arg!(disable_none: -n --disable_none "Disable dumping None values"))
            .arg(arg!(strict_range_check: -r --strict_range_check "Do perform strict numeric range checks"))
            .arg(arg!(debug: -d --debug "Run in debug mode (for developers only)"))
            .arg(arg!(sort_keys: -k --sort_keys "Sort result keys"))
            .arg(arg!(show_hidden: -H --show_hidden "Display hidden attributes"))
            .arg(arg!(arguments: -D --argument <arguments> ... "Specify the top-level argument").num_args(1..))
            .arg(arg!(path_selector: -S --path_selector <path_selector> ... "Specify the path selector").num_args(1..))
            .arg(arg!(overrides: -O --overrides <overrides> ... "Specify the configuration override path and value").num_args(1..))
            .arg(arg!(target: --target <target> "Specify the target type"))
            .arg(arg!(recursive: -R --recursive "Compile the files directory recursively"))
            .arg(arg!(package_map: -E --external <package_map> ... "Mapping of package name and path where the package is located").num_args(1..))
            .arg(arg!(fix: -f --fix "Auto fix")),
        )
        .subcommand(
            Command::new("fmt")
                .about("Format KCL files")
                .arg(arg!(<input> "Input file or path name for formatting"))
                .arg(arg!(recursive: -R --recursive "Iterate through subdirectories recursively"))
                .arg(arg!(std_output: -w --std_output "Whether to output format to stdout")),
        )
        .subcommand(
            Command::new("vet")
                .about("Validate data files witch KCL files")
                .arg(arg!(<data_file> "Validation data file"))
                .arg(arg!(<kcl_file> "KCL file"))
                .arg(arg!(schema: -d --schema <schema> "Iterate through subdirectories recursively").num_args(1..))
                .arg(arg!(attribute_name: -n --attribute_name <attribute_name> "The attribute name for the data loading"))
                .arg(arg!(format: --format <format> "Validation data file format, support YAML and JSON, default is JSON")),
        )
    .subcommand(Command::new("server").about("Start a rpc server for APIs"))
    .subcommand(Command::new("version").about("Show the KCL version"))
}
