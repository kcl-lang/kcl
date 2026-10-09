#![allow(clippy::arc_with_non_send_sync)]

use anyhow::Result;
use clap::ArgMatches;
use kcl_error::format::DiagnosticFormat;
use kcl_error::{Diagnostic, Handler, Level, Message, RenderedError, StringError};
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
/// Used only when the run produced no structured diagnostics (e.g. an
/// argument error reported before compilation starts). The text has already
/// been rendered for a terminal, so its ANSI escapes are stripped before it
/// is embedded in a machine-readable record.
fn diag_from_err_message(message: &str) -> Diagnostic {
    Diagnostic {
        level: Level::Error,
        messages: vec![Message {
            range: (
                kcl_error::Position::dummy_pos(),
                kcl_error::Position::dummy_pos(),
            ),
            style: kcl_error::Style::LineAndColumn,
            message: kcl_error::format::strip_ansi(message).into_owned(),
            note: None,
            suggested_replacement: None,
        }],
        code: None,
    }
}

/// A [`Handler`] pre-loaded with the diagnostics the run produced.
fn handler_with_diagnostics(diagnostics: Vec<Diagnostic>) -> Handler {
    let mut handler = Handler::new();
    for diag in diagnostics {
        handler.add_diagnostic(diag);
    }
    handler
}

/// The structured diagnostics behind a failed run.
///
/// Compile failures arrive as a [`RenderedError`] carrying them; evaluation
/// failures carry them on the result. Either way they hold the real
/// file/line/column that the machine-readable formats report (issue #2216).
fn diagnostics_from_error(err: &anyhow::Error) -> Vec<Diagnostic> {
    err.downcast_ref::<RenderedError>()
        .map(|rendered| rendered.diagnostics.clone())
        .unwrap_or_default()
}

fn emit_machine_readable(
    handler: &mut Handler,
    message: &str,
    format: DiagnosticFormat,
) -> Result<()> {
    if handler.diagnostics.is_empty() {
        handler.add_diagnostic(diag_from_err_message(message));
    }
    let _ = handler.emit_as(format)?;
    Ok(())
}

/// Run the KCL run command.
pub fn run_command<W: Write>(matches: &ArgMatches, writer: &mut W) -> Result<()> {
    // Config settings building
    let settings = must_build_settings(matches);
    let output = settings.output();
    let sourcemap_output = settings.sourcemap_output();
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
                    let mut handler = handler_with_diagnostics(result.diagnostics);
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
            if let (Some(map), Some(json)) = (sourcemap_output.as_ref(), result.sourcemap.as_ref())
            {
                std::fs::write(map, json)?;
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
                let mut handler = handler_with_diagnostics(diagnostics_from_error(&msg));
                emit_machine_readable(&mut handler, &msg.to_string(), error_format)?;
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
