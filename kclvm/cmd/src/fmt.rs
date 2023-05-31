use crate::util::*;
use anyhow::Result;
use clap::ArgMatches;
use kclvm_tools::format::{format, FormatOptions};

/// Run the KCL fmt command.
pub fn fmt_command(matches: &ArgMatches) -> Result<()> {
    let input = matches.get_one::<String>("input").map(|f| f.as_str());
    match input {
        Some(input) => {
            format(
                input,
                &FormatOptions {
                    is_stdout: bool_from_matches(matches, "std_output").unwrap_or_default(),
                    recursively: bool_from_matches(matches, "recursive").unwrap_or_default(),
                },
            )?;
            Ok(())
        }
        None => Err(anyhow::anyhow!("No input file or path")),
    }
}
