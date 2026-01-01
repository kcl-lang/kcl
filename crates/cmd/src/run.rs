#![allow(clippy::arc_with_non_send_sync)]

use anyhow::Result;
use clap::ArgMatches;
use kcl_error::StringError;
use kcl_parser::ParseSession;
use kcl_runner::exec_program;
use std::io::Write;
use std::sync::Arc;

use crate::settings::must_build_settings;

/// Run the KCL run command.
pub fn run_command<W: Write>(matches: &ArgMatches, writer: &mut W) -> Result<()> {
    // Config settings building
    let settings = must_build_settings(matches);
    let output = settings.output();
    let format_opt = matches.get_one::<String>("format").map(|s| s.as_str());
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into()?) {
        Ok(result) => {
            // Output log message
            if !result.log_message.is_empty() {
                write!(writer, "{}", result.log_message)?;
            }
            // Output execute error message
            if !result.err_message.is_empty() {
                if !sess.0.diag_handler.has_errors()? {
                    sess.0.add_err(StringError(result.err_message))?;
                }
                sess.0.emit_stashed_diagnostics_and_abort()?;
            }
            // Select output based on format option
            let output_str = match format_opt {
                Some("json") => &result.json_result,
                Some("yaml") | None => &result.yaml_result,
                Some(f) => {
                    return Err(anyhow::anyhow!(
                        "Invalid format '{}', expected 'yaml' or 'json'",
                        f
                    ));
                }
            };
            if !output_str.is_empty() {
                match output {
                    Some(o) => std::fs::write(o, output_str)?,
                    // [`println!`] is not a good way to output content to stdout,
                    // using [`writeln`] can be better to redirect the output.
                    None => writeln!(writer, "{}", output_str)?,
                }
            }
        }
        // Other error message
        Err(msg) => {
            if !sess.0.diag_handler.has_errors()? {
                sess.0.add_err(StringError(msg.to_string()))?;
            }
            sess.0.emit_stashed_diagnostics_and_abort()?;
        }
    }
    Ok(())
}
