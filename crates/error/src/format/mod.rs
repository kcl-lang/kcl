//! Diagnostic output formats.
//!
//! This module provides multiple rendering backends for [`Diagnostic`]s so that
//! KCL output can be consumed by both humans (default [`DiagnosticFormat::Pretty`]
//! using `annotate_snippets`) and downstream tooling ([`DiagnosticFormat::Short`],
//! [`DiagnosticFormat::Arcanist`] JSON, [`DiagnosticFormat::Sarif`]).
//!
//! All non-pretty formats share the same selection rule: a single logical
//! [`Diagnostic`] is mapped to a single output record. The "primary" message
//! is the first non-[`Style::Empty`] [`Message`] in the diagnostic; any
//! remaining messages are treated as related locations and surfaced via the
//! format-appropriate channel (SARIF `relatedLocations`, text suffixes in
//! Short/Arcanist).

pub mod arcanist;
pub mod sarif;
pub mod short;

use crate::{Diagnostic, DiagnosticId, Level, Message, Position, Style};
use compiler_base_session::Session;
use std::str::FromStr;
use thiserror::Error;

/// Selects the diagnostic output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticFormat {
    /// Default `annotate_snippets` style. Human-readable, ANSI-colored.
    Pretty,
    /// Single-line plain text. Designed for CI logs.
    Short,
    /// Arcanist-compatible JSON array. One object per diagnostic.
    Arcanist,
    /// SARIF 2.1.0 log. The OASIS industry standard for machine-readable
    /// static-analysis output.
    Sarif,
}

impl DiagnosticFormat {
    /// Lowercase identifier used in CLI flags and the `KCL_ERROR_FORMAT`
    /// environment variable.
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticFormat::Pretty => "pretty",
            DiagnosticFormat::Short => "short",
            DiagnosticFormat::Arcanist => "arcanist",
            DiagnosticFormat::Sarif => "sarif",
        }
    }

    /// Returns every supported value, for error messages and `--help` output.
    pub fn all() -> &'static [&'static str] {
        &["pretty", "short", "arcanist", "sarif"]
    }
}

impl FromStr for DiagnosticFormat {
    type Err = ParseDiagnosticFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pretty" => Ok(DiagnosticFormat::Pretty),
            "short" => Ok(DiagnosticFormat::Short),
            "arcanist" => Ok(DiagnosticFormat::Arcanist),
            "sarif" => Ok(DiagnosticFormat::Sarif),
            _ => Err(ParseDiagnosticFormatError(s.to_string())),
        }
    }
}

/// Returned by [`<DiagnosticFormat as FromStr>::from_str`] when the input is
/// not a recognized format name.
#[derive(Debug, Error)]
#[error("invalid diagnostic format `{0}` (expected one of: {valid})", valid = DiagnosticFormat::all().join(", "))]
pub struct ParseDiagnosticFormatError(pub String);

/// Pick the primary [`Message`] for an output record.
///
/// We define primary as the first message whose [`Style`] is not
/// [`Style::Empty`]. If every message is empty we fall back to the first
/// message in the vector so renderers always have *some* text to emit.
pub fn primary_message(diag: &Diagnostic) -> Option<&Message> {
    diag.messages
        .iter()
        .find(|m| m.style != Style::Empty)
        .or_else(|| diag.messages.first())
}

/// Convert the 0-based internal column to a 1-based external column.
///
/// Returns `1` when the position has no column (matching the Arcanist
/// convention where the absent/zero column is normalized to 1).
pub fn external_column(pos: &Position) -> u64 {
    pos.column.map(|c| c + 1).unwrap_or(1)
}

/// Look up the source line that contains `pos.line` from `filename`.
///
/// Returns `None` when the file cannot be opened or the line is out of
/// range. Renderers use this to populate Arcanist's `OriginalText` and
/// to make short/arcanist descriptions a little more useful; both are
/// tolerant of an empty source.
pub fn lookup_source_line(filename: &str, line: u64) -> Option<String> {
    if filename.is_empty() || line == 0 {
        return None;
    }
    let sess = Session::new_with_file_and_code(filename, None).ok()?;
    let source = sess
        .sm
        .lookup_source_file(compiler_base_span::span::new_byte_pos(0));
    source
        .get_line(line.saturating_sub(1) as usize)
        .as_ref()
        .map(|s| s.to_string())
}

/// Build the human-readable rule name for a diagnostic.
///
/// Falls back to `EvaluationError` / `CompilerWarning` / `Suggestion` /
/// `Unknown` for diagnostics that don't carry a [`DiagnosticId`].
pub fn rule_name(diag: &Diagnostic) -> String {
    match &diag.code {
        Some(DiagnosticId::Error(err)) => err.name(),
        Some(DiagnosticId::Warning(warn)) => warn.name(),
        Some(DiagnosticId::Suggestions) => "Suggestion".to_string(),
        None => match diag.level {
            Level::Error => crate::ErrorKind::EvaluationError.name(),
            Level::Warning => crate::WarningKind::CompilerWarning.name(),
            Level::Note => "Note".to_string(),
            Level::Suggestions => "Suggestion".to_string(),
        },
    }
}

/// Build the human-readable rule code for a diagnostic.
///
/// Returns strings such as `error[E2F04]` or `warning[W1001]`. Falls back
/// to `error[EvaluationError]`, `warning[CompilerWarning]`, etc.
pub fn rule_code(diag: &Diagnostic) -> String {
    let level = diag.level.to_str();
    match &diag.code {
        Some(DiagnosticId::Error(err)) => format!("{}[{}]", level, err.code()),
        Some(DiagnosticId::Warning(warn)) => format!("{}[{}]", level, warn.code()),
        Some(DiagnosticId::Suggestions) => format!("{}[Suggestions]", level),
        None => match diag.level {
            Level::Error => format!("{}[{}]", level, crate::ErrorKind::EvaluationError.code()),
            Level::Warning => format!("{}[{}]", level, crate::WarningKind::CompilerWarning.code()),
            Level::Note => format!("{}[Note]", level),
            Level::Suggestions => format!("{}[Suggestion]", level),
        },
    }
}

/// The position used as the "primary" location for a diagnostic.
pub fn primary_position(diag: &Diagnostic) -> Option<&Position> {
    primary_message(diag).map(|m| &m.range.0)
}

/// All secondary messages (everything except the primary).
pub fn related_messages(diag: &Diagnostic) -> Vec<&Message> {
    match primary_message(diag) {
        Some(primary) => diag
            .messages
            .iter()
            .filter(|m| !std::ptr::eq(*m, primary))
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiagnosticId, ErrorKind, Handler};

    fn dummy_message(message: &str) -> Message {
        Message {
            range: (
                Position {
                    filename: "foo.k".to_string(),
                    line: 3,
                    column: Some(2),
                },
                Position {
                    filename: "foo.k".to_string(),
                    line: 3,
                    column: Some(2),
                },
            ),
            style: Style::LineAndColumn,
            message: message.to_string(),
            note: None,
            suggested_replacement: None,
        }
    }

    fn diag_with_messages(msgs: Vec<Message>, code: Option<DiagnosticId>) -> Diagnostic {
        Diagnostic {
            level: Level::Error,
            messages: msgs,
            code,
        }
    }

    #[test]
    fn parse_recognises_known_formats() {
        assert_eq!(
            "pretty".parse::<DiagnosticFormat>().unwrap(),
            DiagnosticFormat::Pretty
        );
        assert_eq!(
            "short".parse::<DiagnosticFormat>().unwrap(),
            DiagnosticFormat::Short
        );
        assert_eq!(
            "arcanist".parse::<DiagnosticFormat>().unwrap(),
            DiagnosticFormat::Arcanist
        );
        assert_eq!(
            "sarif".parse::<DiagnosticFormat>().unwrap(),
            DiagnosticFormat::Sarif
        );
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(
            "SHORT".parse::<DiagnosticFormat>().unwrap(),
            DiagnosticFormat::Short
        );
        assert_eq!(
            "  Arcanist ".parse::<DiagnosticFormat>().unwrap(),
            DiagnosticFormat::Arcanist
        );
    }

    #[test]
    fn parse_rejects_unknown_values() {
        let err = "json".parse::<DiagnosticFormat>().unwrap_err();
        assert_eq!(err.0, "json");
        assert!(err.to_string().contains("pretty"));
        assert!(err.to_string().contains("sarif"));
    }

    #[test]
    fn primary_message_skips_empty_style() {
        let msgs = vec![
            Message {
                range: (Position::dummy_pos(), Position::dummy_pos()),
                style: Style::Empty,
                message: "".to_string(),
                note: None,
                suggested_replacement: None,
            },
            dummy_message("real message"),
        ];
        let diag = diag_with_messages(msgs, None);
        assert_eq!(primary_message(&diag).unwrap().message, "real message");
    }

    #[test]
    fn primary_message_falls_back_when_all_empty() {
        let msgs = vec![Message {
            range: (Position::dummy_pos(), Position::dummy_pos()),
            style: Style::Empty,
            message: "anything".to_string(),
            note: None,
            suggested_replacement: None,
        }];
        let diag = diag_with_messages(msgs, None);
        // Falls back to first message rather than None.
        assert_eq!(primary_message(&diag).unwrap().message, "anything");
    }

    #[test]
    fn external_column_handles_missing_column() {
        let pos = Position {
            filename: "x".to_string(),
            line: 1,
            column: None,
        };
        assert_eq!(external_column(&pos), 1);
        let pos2 = Position {
            filename: "x".to_string(),
            line: 1,
            column: Some(0),
        };
        assert_eq!(external_column(&pos2), 1);
        let pos3 = Position {
            filename: "x".to_string(),
            line: 1,
            column: Some(6),
        };
        assert_eq!(external_column(&pos3), 7);
    }

    #[test]
    fn rule_code_uses_diagnostic_id() {
        let diag = diag_with_messages(vec![dummy_message("x")], None);
        // No DiagnosticId: still produces a valid bracketed code.
        let code = rule_code(&diag);
        assert!(code.starts_with("error["));
        assert!(code.ends_with(']'));
    }

    #[test]
    fn rule_code_with_known_error_kind() {
        let diag = diag_with_messages(
            vec![dummy_message("x")],
            Some(DiagnosticId::Error(ErrorKind::CannotFindModule)),
        );
        assert_eq!(rule_code(&diag), "error[E2F04]");
    }

    #[test]
    fn rule_name_for_warning_kind() {
        let diag = diag_with_messages(
            vec![dummy_message("x")],
            Some(DiagnosticId::Warning(
                crate::WarningKind::UnusedImportWarning,
            )),
        );
        assert_eq!(rule_name(&diag), "UnusedImportWarning");
    }

    #[test]
    fn related_messages_excludes_primary() {
        let m1 = dummy_message("a");
        let m2 = dummy_message("b");
        let m3 = dummy_message("c");
        let diag = diag_with_messages(vec![m1.clone(), m2.clone(), m3.clone()], None);
        let related = related_messages(&diag);
        assert_eq!(related.len(), 2);
        assert_eq!(related[0].message, "b");
        assert_eq!(related[1].message, "c");
    }

    #[test]
    fn handler_render_round_trips_through_each_format() {
        // Sanity check that Handler::emit_to_string_as produces non-empty
        // output for every non-pretty format and round-trips through JSON
        // for the structured ones.
        let pos = Position {
            filename: "does-not-exist.k".to_string(),
            line: 1,
            column: Some(0),
        };
        let diag = Diagnostic {
            level: Level::Error,
            messages: vec![Message {
                range: (pos.clone(), pos),
                style: Style::LineAndColumn,
                message: "sample".to_string(),
                note: None,
                suggested_replacement: None,
            }],
            code: None,
        };

        for fmt in [
            DiagnosticFormat::Short,
            DiagnosticFormat::Arcanist,
            DiagnosticFormat::Sarif,
        ] {
            let mut handler = Handler::default();
            handler.add_diagnostic(diag.clone());
            let rendered = handler.emit_to_string_as(fmt).expect("must render");
            assert!(!rendered.is_empty(), "format {fmt:?} produced empty output");
            // Structured formats must be valid JSON.
            if matches!(fmt, DiagnosticFormat::Arcanist | DiagnosticFormat::Sarif) {
                let _: serde_json::Value =
                    serde_json::from_str(&rendered).expect("must be valid JSON");
            }
        }
    }
}
