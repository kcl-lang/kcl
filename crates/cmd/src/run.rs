#![allow(clippy::arc_with_non_send_sync)]

use anyhow::Result;
use clap::ArgMatches;
use kcl_error::format::DiagnosticFormat;
use kcl_error::{Diagnostic, Handler, Level, Message, StringError};
use kcl_parser::ParseSession;
use kcl_runner::exec_program;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;

use crate::settings::must_build_settings;

/// Resolve the diagnostic output format from CLI flag and `KCL_ERROR_FORMAT`
/// environment variable.
///
/// Precedence: CLI flag > `KCL_ERROR_FORMAT` > default `Pretty`. Invalid
/// values produce an error listing the valid options.
pub fn resolve_error_format(matches: &ArgMatches) -> Result<DiagnosticFormat> {
    if let Some(s) = matches.get_one::<String>("error_format") {
        return DiagnosticFormat::from_str(s).map_err(anyhow::Error::from);
    }
    if let Ok(s) = std::env::var("KCL_ERROR_FORMAT") {
        if !s.is_empty() {
            return DiagnosticFormat::from_str(&s).map_err(anyhow::Error::from);
        }
    }
    Ok(DiagnosticFormat::Pretty)
}

/// Build a fallback KCL [`Diagnostic`] from a plain error message string.
///
/// The runner reports compile/eval failures as plain text via
/// `ExecProgramResult.err_message`. For machine-readable formats we still
/// want to surface *something*, even without structured position info.
fn diag_from_err_message(message: &str) -> Diagnostic {
    Diagnostic {
        level: Level::Error,
        messages: vec![Message {
            range: (kcl_error::Position::dummy_pos(), kcl_error::Position::dummy_pos()),
            style: kcl_error::Style::LineAndColumn,
            message: message.to_string(),
            note: None,
            suggested_replacement: None,
        }],
        code: None,
    }
}

fn emit_machine_readable(handler: &mut Handler, message: &str, format: DiagnosticFormat) -> Result<()> {
    handler.add_diagnostic(diag_from_err_message(message));
    let _ = handler.emit_as(format)?;
    Ok(())
}

/// Run the KCL run command.
pub fn run_command<W: Write>(matches: &ArgMatches, writer: &mut W) -> Result<()> {
    // Config settings building
    let settings = must_build_settings(matches);
    let output = settings.output();
    let format_opt = matches.get_one::<String>("format").map(|s| s.as_str());
    let error_format = resolve_error_format(matches)?;
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into()?) {
        Ok(result) => {
            // Output log message
            if !result.log_message.is_empty() {
                write!(writer, "{}", result.log_message)?;
            }
            // Output execute error message
            if !result.err_message.is_empty() {
                if error_format == DiagnosticFormat::Pretty {
                    if !sess.0.diag_handler.has_errors()? {
                        sess.0.add_err(StringError(result.err_message))?;
                    }
                    sess.0.emit_stashed_diagnostics_and_abort()?;
                } else {
                    let mut handler = Handler::new();
                    emit_machine_readable(&mut handler, &result.err_message, error_format)?;
                    std::process::exit(1);
                }
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
            if error_format == DiagnosticFormat::Pretty {
                if !sess.0.diag_handler.has_errors()? {
                    sess.0.add_err(StringError(msg.to_string()))?;
                }
                sess.0.emit_stashed_diagnostics_and_abort()?;
            } else {
                let mut handler = Handler::new();
                emit_machine_readable(&mut handler, &msg.to_string(), error_format)?;
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
