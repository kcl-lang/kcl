//! The `kclvm` command-line interface.

#[macro_use]
extern crate clap;

pub mod lint;
pub mod run;
mod settings;

#[cfg(test)]
mod tests;

use anyhow::Result;
use lint::lint_command;
use run::run_command;

/// Run the KCL main command.
pub fn main() -> Result<()> {
    let matches = clap_app!(kcl =>
        (@subcommand run =>
            (@arg input: ... "Sets the input file to use")
            (@arg output: -o --output +takes_value "Sets the LLVM IR/BC output file path")
            (@arg setting: ... -Y --setting +takes_value "Sets the input file to use")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg disable_none: -n --disable-none "Disable dumping None values")
            (@arg debug: -d --debug "Run in debug mode (for developers only)")
            (@arg sort_key: -k --sort "Sort result keys")
            (@arg argument: ... -D --argument "Specify the top-level argument")
        )
        (@subcommand lint =>
            (@arg input: ... "Sets the input file to use")
            (@arg output: -o --output +takes_value "Sets the LLVM IR/BC output file path")
            (@arg setting: ... -Y --setting +takes_value "Sets the input file to use")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg disable_none: -n --disable-none "Disable dumping None values")
            (@arg debug: -d --debug "Run in debug mode (for developers only)")
            (@arg sort_key: -k --sort "Sort result keys")
            (@arg argument: ... -D --argument "Specify the top-level argument")
            (@arg emit_warning: --emit_warning "Emit warning message")
        )
    )
    .arg_required_else_help(true)
    .get_matches();
    if let Some(matches) = matches.subcommand_matches("run") {
        run_command(matches)
    } else if let Some(matches) = matches.subcommand_matches("lint") {
        lint_command(matches)
    } else {
        Ok(())
    }
}
