//! One-line plain-text diagnostic output.
//!
//! Layout (one line per diagnostic):
//!
//! ```text
//! <path>:<line>:<col> - <level>[<code>]: <name> - <description>[; note: ...][; related: ...]
//! ```
//!
//! When the diagnostic has no associated source file, the leading
//! `path:line:col -` segment is omitted. There is no ANSI coloring.

use super::{
    external_column, primary_message, primary_position, related_messages, rule_code, rule_name,
};
use crate::Diagnostic;

/// Render a single diagnostic in the short (one-line) format.
pub fn render(diag: &Diagnostic) -> String {
    let mut out = String::new();

    if let Some(pos) = primary_position(diag) {
        if !pos.filename.is_empty() {
            out.push_str(&pos.filename);
            out.push(':');
            out.push_str(&pos.line.to_string());
            out.push(':');
            out.push_str(&external_column(pos).to_string());
            out.push_str(" - ");
        }
    }

    out.push_str(&rule_code(diag));
    out.push_str(": ");
    out.push_str(&rule_name(diag));
    out.push_str(" - ");

    let mut description = String::new();
    if let Some(msg) = primary_message(diag) {
        description.push_str(&msg.message);
        if let Some(note) = &msg.note {
            description.push_str("; note: ");
            description.push_str(note);
        }
        if let Some(replacements) = &msg.suggested_replacement {
            let non_empty: Vec<&str> = replacements
                .iter()
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .collect();
            if !non_empty.is_empty() {
                description.push_str("; replacement: ");
                description.push_str(&non_empty.join(" | "));
            }
        }
    }

    // Append related messages (the secondary messages of the diagnostic).
    for msg in related_messages(diag) {
        if !description.is_empty() {
            description.push_str("; related: ");
        } else {
            description.push_str("related: ");
        }
        description.push_str(&msg.message);
    }

    out.push_str(&description);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Diagnostic, DiagnosticId, ErrorKind, Level, Message, Position, Style, WarningKind,
    };

    fn diag_with(
        level: Level,
        message: &str,
        filename: &str,
        line: u64,
        column: Option<u64>,
        note: Option<&str>,
        code: Option<DiagnosticId>,
    ) -> Diagnostic {
        let pos = Position {
            filename: filename.to_string(),
            line,
            column,
        };
        let msg = Message {
            range: (pos.clone(), pos),
            style: Style::LineAndColumn,
            message: message.to_string(),
            note: note.map(String::from),
            suggested_replacement: None,
        };
        Diagnostic {
            level,
            messages: vec![msg],
            code,
        }
    }

    #[test]
    fn basic_error_with_code_and_location() {
        let diag = diag_with(
            Level::Error,
            "imported module not found",
            "/tmp/file.k",
            7,
            Some(0),
            None,
            Some(DiagnosticId::Error(ErrorKind::CannotFindModule)),
        );
        let out = render(&diag);
        assert_eq!(
            out,
            "/tmp/file.k:7:1 - error[E2F04]: CannotFindModule - imported module not found"
        );
    }

    #[test]
    fn column_one_based_external() {
        let diag = diag_with(
            Level::Error,
            "boom",
            "x.k",
            3,
            Some(6),
            None,
            Some(DiagnosticId::Error(ErrorKind::TypeError)),
        );
        let out = render(&diag);
        assert!(out.starts_with("x.k:3:7 - "), "got: {out}");
    }

    #[test]
    fn missing_column_falls_back_to_one() {
        let diag = diag_with(
            Level::Error,
            "boom",
            "x.k",
            1,
            None,
            None,
            Some(DiagnosticId::Error(ErrorKind::EvaluationError)),
        );
        let out = render(&diag);
        assert!(out.starts_with("x.k:1:1 - "), "got: {out}");
    }

    #[test]
    fn warning_renders_with_w_level() {
        let diag = diag_with(
            Level::Warning,
            "unused",
            "x.k",
            1,
            Some(0),
            None,
            Some(DiagnosticId::Warning(WarningKind::UnusedImportWarning)),
        );
        let out = render(&diag);
        assert!(out.contains("warning["), "got: {out}");
        assert!(out.contains("UnusedImportWarning"), "got: {out}");
    }

    #[test]
    fn note_is_appended() {
        let diag = diag_with(
            Level::Error,
            "primary",
            "x.k",
            1,
            Some(0),
            Some("see also foo.k:5"),
            Some(DiagnosticId::Error(ErrorKind::CompileError)),
        );
        let out = render(&diag);
        assert!(out.contains("; note: see also foo.k:5"), "got: {out}");
    }

    #[test]
    fn empty_filename_omits_location() {
        let diag = diag_with(
            Level::Error,
            "no location here",
            "",
            1,
            None,
            None,
            Some(DiagnosticId::Error(ErrorKind::EvaluationError)),
        );
        let out = render(&diag);
        assert!(!out.starts_with(':'), "got: {out}");
        assert!(out.contains("error["), "got: {out}");
        assert!(out.contains("no location here"), "got: {out}");
    }

    #[test]
    fn multi_message_appends_related() {
        let main_pos = Position {
            filename: "a.k".to_string(),
            line: 2,
            column: Some(0),
        };
        let related_pos = Position {
            filename: "b.k".to_string(),
            line: 5,
            column: Some(0),
        };
        let diag = Diagnostic {
            level: Level::Error,
            messages: vec![
                Message {
                    range: (main_pos.clone(), main_pos),
                    style: Style::LineAndColumn,
                    message: "primary".to_string(),
                    note: None,
                    suggested_replacement: None,
                },
                Message {
                    range: (related_pos.clone(), related_pos),
                    style: Style::LineAndColumn,
                    message: "extra info".to_string(),
                    note: None,
                    suggested_replacement: None,
                },
            ],
            code: Some(DiagnosticId::Error(ErrorKind::CompileError)),
        };
        let out = render(&diag);
        assert!(out.contains("primary"), "got: {out}");
        assert!(out.contains("; related: extra info"), "got: {out}");
    }
}
