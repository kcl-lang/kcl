//! Arcanist-style JSON diagnostic output.
//!
//! Each [`Diagnostic`] becomes a JSON object with these PascalCase keys
//! (matching the Phabricator Arcanist convention):
//!
//! | Field           | Type   | Meaning                                              |
//! |-----------------|--------|------------------------------------------------------|
//! | `Char`          | number | 1-based column of the primary message (1 if absent)  |
//! | `Code`          | string | `error[E2F04]` / `warning[W1001]`                    |
//! | `Description`   | string | Primary message, `note`, related messages            |
//! | `Line`          | number | 1-based line of the primary message                  |
//! | `Name`          | string | Rule name, e.g. `CannotFindModule`                   |
//! | `OriginalText`  | string | Source line at the diagnostic location (may be empty)|
//! | `Path`          | string | Filename of the primary message                      |
//!
//! The full result is serialized as a pretty-printed JSON array. Empty
//! diagnostics sets serialize to `[]`.

use super::{
    external_column, lookup_source_line, primary_message, primary_position, related_messages,
    rule_code, rule_name,
};
use crate::Diagnostic;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ArcanistEntry {
    char: u64,
    code: String,
    description: String,
    line: u64,
    name: String,
    original_text: String,
    path: String,
}

impl ArcanistEntry {
    fn from_diag(diag: &Diagnostic) -> Self {
        let pos = primary_position(diag);
        let path = pos.map(|p| p.filename.clone()).unwrap_or_default();
        let line = pos.map(|p| p.line).unwrap_or(1);
        let char = pos.map(external_column).unwrap_or(1);

        let mut description = String::new();
        let mut original_text = String::new();
        if let Some(msg) = primary_message(diag) {
            description.push_str(&msg.message);
            if let Some(note) = &msg.note {
                description.push_str(" | note: ");
                description.push_str(note);
            }
        }
        if let Some(p) = pos {
            if !p.filename.is_empty() {
                original_text = lookup_source_line(&p.filename, p.line).unwrap_or_default();
            }
        }
        for msg in related_messages(diag) {
            if !description.is_empty() {
                description.push_str(" | related: ");
            } else {
                description.push_str("related: ");
            }
            description.push_str(&msg.message);
        }

        Self {
            char,
            code: rule_code(diag),
            description,
            line,
            name: rule_name(diag),
            original_text,
            path,
        }
    }
}

/// Render a slice of diagnostics as a pretty-printed JSON array.
pub fn render(diagnostics: &[Diagnostic]) -> String {
    let entries: Vec<ArcanistEntry> = diagnostics.iter().map(ArcanistEntry::from_diag).collect();
    serde_json::to_string_pretty(&entries)
        .unwrap_or_else(|e| format!("[{{\"error\":\"failed to serialize arcanist output: {e}\"}}]"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Diagnostic, DiagnosticId, ErrorKind, Level, Message, Position, Style};
    use serde_json::Value;

    fn sample_diag() -> Diagnostic {
        let pos = Position {
            filename: "/tmp/file.k".to_string(),
            line: 7,
            column: Some(0),
        };
        Diagnostic {
            level: Level::Error,
            messages: vec![Message {
                range: (pos.clone(), pos),
                style: Style::LineAndColumn,
                message: "imported module not found".to_string(),
                note: None,
                suggested_replacement: None,
            }],
            code: Some(DiagnosticId::Error(ErrorKind::CannotFindModule)),
        }
    }

    #[test]
    fn render_produces_valid_json_array() {
        let out = render(&[sample_diag()]);
        let parsed: Value = serde_json::from_str(&out).expect("must be valid JSON");
        let arr = parsed.as_array().expect("must be an array");
        assert_eq!(arr.len(), 1);
        let entry = &arr[0];
        assert_eq!(entry["Path"], "/tmp/file.k");
        assert_eq!(entry["Line"], 7);
        assert_eq!(entry["Char"], 1);
        assert_eq!(entry["Code"], "error[E2F04]");
        assert_eq!(entry["Name"], "CannotFindModule");
        assert_eq!(entry["Description"], "imported module not found");
    }

    #[test]
    fn render_empty_diagnostics_returns_empty_array() {
        let out = render(&[]);
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert!(parsed.as_array().unwrap().is_empty());
    }

    #[test]
    fn render_uses_one_based_column() {
        let mut diag = sample_diag();
        if let Some(msg) = diag.messages.first_mut() {
            msg.range.0.column = Some(4);
            msg.range.1.column = Some(4);
        }
        let out = render(&[diag]);
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed[0]["Char"], 5);
    }

    #[test]
    fn render_falls_back_to_one_for_missing_column() {
        let mut diag = sample_diag();
        if let Some(msg) = diag.messages.first_mut() {
            msg.range.0.column = None;
            msg.range.1.column = None;
        }
        let out = render(&[diag]);
        let parsed: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed[0]["Char"], 1);
    }

    #[test]
    fn render_appends_related_messages() {
        let main_pos = Position {
            filename: "a.k".to_string(),
            line: 1,
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
                    message: "secondary".to_string(),
                    note: None,
                    suggested_replacement: None,
                },
            ],
            code: Some(DiagnosticId::Error(ErrorKind::CompileError)),
        };
        let out = render(&[diag]);
        let parsed: Value = serde_json::from_str(&out).unwrap();
        let desc = parsed[0]["Description"].as_str().unwrap();
        assert!(desc.contains("primary"));
        assert!(desc.contains("related: secondary"), "got: {desc}");
    }
}
