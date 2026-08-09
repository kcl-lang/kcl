//! SARIF 2.1.0 diagnostic output.
//!
//! Emits a single-run SARIF log describing the diagnostics. The schema is
//! the official OASIS JSON: <https://json.schemastore.org/sarif-2.1.0.json>.
//!
//! - Tool: `kcl`, with the official site as `informationUri`.
//! - Each diagnostic with a known rule id produces one
//!   [`reportingDescriptor`](sarif::ReportingDescriptor) in `tool.driver.rules`
//!   (deduplicated by rule id).
//! - Each diagnostic becomes one [`Result`] in `runs[0].results`, with
//!   `locations` for the primary message and `relatedLocations` for any
//!   secondary messages.
//!
//! Columns are converted to 1-based per SARIF conventions.

use super::{external_column, primary_message, related_messages, rule_name};
use crate::{Diagnostic, Level, Message, Position};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

/// The official SARIF 2.1.0 schema URL.
const SARIF_SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
const KCL_INFORMATION_URI: &str = "https://kcl-lang.io/";

#[derive(Serialize)]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<Run>,
}

#[derive(Serialize)]
pub struct Run {
    tool: Tool,
    results: Vec<Result>,
}

#[derive(Serialize)]
pub struct Tool {
    driver: Driver,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Driver {
    name: &'static str,
    information_uri: &'static str,
    rules: Vec<ReportingDescriptor>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportingDescriptor {
    id: String,
    name: String,
    short_description: ShortDescription,
}

#[derive(Serialize)]
pub struct ShortDescription {
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    rule_id: String,
    level: String,
    message: MessageText,
    locations: Vec<Location>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related_locations: Vec<RelatedLocation>,
}

#[derive(Serialize)]
pub struct MessageText {
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    physical_location: PhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalLocation {
    artifact_location: ArtifactLocation,
    region: Region,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    start_line: u64,
    start_column: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_column: Option<u64>,
}

#[derive(Serialize)]
pub struct ArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedLocation {
    physical_location: PhysicalLocation,
    message: MessageText,
}

fn level_str(level: Level) -> &'static str {
    match level {
        Level::Error => "error",
        Level::Warning => "warning",
        Level::Note | Level::Suggestions => "note",
    }
}

fn region_for(pos: &Position, end: &Position) -> Region {
    let start_line = pos.line;
    let start_column = external_column(pos);
    let end_line = end.line;
    let end_column = external_column(end);

    let (end_line, end_column) = if end_line == start_line && end_column == start_column {
        // SARIF endColumn must be > startColumn; emit nothing when caller has
        // no meaningful end.
        (None, None)
    } else if end_line == start_line && end_column < start_column {
        (None, None)
    } else if end_line == start_line {
        (Some(end_line), Some(end_column))
    } else {
        (Some(end_line), Some(end_column))
    };

    Region {
        start_line,
        start_column,
        end_line,
        end_column,
    }
}

fn location_from_message(msg: &Message) -> Location {
    Location {
        physical_location: PhysicalLocation {
            artifact_location: ArtifactLocation {
                uri: msg.range.0.filename.clone(),
            },
            region: region_for(&msg.range.0, &msg.range.1),
        },
    }
}

fn build_description(diag: &Diagnostic) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(msg) = primary_message(diag) {
        parts.push(msg.message.clone());
        if let Some(note) = &msg.note {
            parts.push(format!("note: {note}"));
        }
    }
    for msg in related_messages(diag) {
        parts.push(format!("related: {}", msg.message));
    }
    if parts.is_empty() {
        parts.push("diagnostic".to_string());
    }
    parts.join("; ")
}

fn rule_id_for(diag: &Diagnostic) -> String {
    diag.code
        .as_ref()
        .map(|c| match c {
            crate::DiagnosticId::Error(err) => err.code(),
            crate::DiagnosticId::Warning(warn) => warn.code(),
            crate::DiagnosticId::Suggestions => "Suggestions".to_string(),
        })
        .unwrap_or_else(|| match diag.level {
            Level::Error => crate::ErrorKind::EvaluationError.code(),
            Level::Warning => crate::WarningKind::CompilerWarning.code(),
            Level::Note => "Note".to_string(),
            Level::Suggestions => "Suggestion".to_string(),
        })
}

/// Render a slice of diagnostics as a SARIF 2.1.0 log.
pub fn render(diagnostics: &[Diagnostic]) -> String {
    let mut seen_rules: BTreeSet<String> = BTreeSet::new();
    let mut rules: Vec<ReportingDescriptor> = Vec::new();
    let mut results: Vec<Result> = Vec::new();

    for diag in diagnostics {
        let rule_id = rule_id_for(diag);
        if seen_rules.insert(rule_id.clone()) {
            rules.push(ReportingDescriptor {
                id: rule_id.clone(),
                name: rule_name(diag),
                short_description: ShortDescription {
                    text: rule_name(diag),
                },
            });
        }

        let locations: Vec<Location> = primary_message(diag)
            .map(|m| vec![location_from_message(m)])
            .unwrap_or_default();

        let related_locations: Vec<RelatedLocation> = related_messages(diag)
            .iter()
            .map(|m| RelatedLocation {
                physical_location: PhysicalLocation {
                    artifact_location: ArtifactLocation {
                        uri: m.range.0.filename.clone(),
                    },
                    region: region_for(&m.range.0, &m.range.1),
                },
                message: MessageText {
                    text: m.message.clone(),
                },
            })
            .collect();

        results.push(Result {
            rule_id,
            level: level_str(diag.level).to_string(),
            message: MessageText {
                text: build_description(diag),
            },
            locations,
            related_locations,
        });
    }

    let log = SarifLog {
        schema: SARIF_SCHEMA,
        version: "2.1.0",
        runs: vec![Run {
            tool: Tool {
                driver: Driver {
                    name: "kcl",
                    information_uri: KCL_INFORMATION_URI,
                    rules,
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&log)
        .unwrap_or_else(|e| format!("{{\"error\":\"failed to serialize sarif output: {e}\"}}"))
}

/// Convenience helper used by tests: ensure the produced JSON is itself a
/// valid SARIF object (parses, has `$schema`, `version`, `runs`).
pub fn parse_log(text: &str) -> Option<Value> {
    let value: Value = serde_json::from_str(text).ok()?;
    if value.get("$schema")?.as_str()? != SARIF_SCHEMA {
        return None;
    }
    if value.get("version")?.as_str()? != "2.1.0" {
        return None;
    }
    if !value.get("runs")?.is_array() {
        return None;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Diagnostic, DiagnosticId, ErrorKind, Level, Style};

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
    fn render_produces_valid_sarif() {
        let out = render(&[sample_diag()]);
        let parsed = parse_log(&out).expect("output must be a valid SARIF log");
        assert_eq!(parsed["version"], "2.1.0");
        let runs = parsed["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);
        let driver = &runs[0]["tool"]["driver"];
        assert_eq!(driver["name"], "kcl");
        assert!(driver["informationUri"].as_str().unwrap().starts_with("https://"));
        let rules = driver["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["id"], "E2F04");
        assert_eq!(rules[0]["name"], "CannotFindModule");
        let results = runs[0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["ruleId"], "E2F04");
        assert_eq!(results[0]["level"], "error");
        assert_eq!(
            results[0]["message"]["text"],
            "imported module not found"
        );
        let locs = results[0]["locations"].as_array().unwrap();
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0]["physicalLocation"]["artifactLocation"]["uri"], "/tmp/file.k");
        assert_eq!(locs[0]["physicalLocation"]["region"]["startLine"], 7);
        assert_eq!(locs[0]["physicalLocation"]["region"]["startColumn"], 1);
    }

    #[test]
    fn render_warning_uses_warning_level() {
        let mut diag = sample_diag();
        diag.level = Level::Warning;
        diag.code = Some(DiagnosticId::Warning(
            crate::WarningKind::UnusedImportWarning,
        ));
        let out = render(&[diag]);
        let parsed = parse_log(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "warning");
        assert_eq!(parsed["runs"][0]["tool"]["driver"]["rules"][0]["id"], "W1001");
    }

    #[test]
    fn render_uses_one_based_column() {
        let mut diag = sample_diag();
        if let Some(msg) = diag.messages.first_mut() {
            msg.range.0.column = Some(6);
            msg.range.1.column = Some(8);
        }
        let out = render(&[diag]);
        let parsed = parse_log(&out).unwrap();
        let region = &parsed["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
        assert_eq!(region["startColumn"], 7);
        assert_eq!(region["endColumn"], 9);
    }

    #[test]
    fn render_omits_end_when_start_equals_end() {
        let mut diag = sample_diag();
        if let Some(msg) = diag.messages.first_mut() {
            // End position identical to start → no endLine/endColumn.
            msg.range.0.column = Some(0);
            msg.range.1.column = Some(0);
        }
        let out = render(&[diag]);
        let parsed = parse_log(&out).unwrap();
        let region = &parsed["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
        assert!(region.get("endLine").is_none() || region["endLine"].is_null());
        assert!(region.get("endColumn").is_none() || region["endColumn"].is_null());
    }

    #[test]
    fn render_related_locations_appear() {
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
                    message: "extra".to_string(),
                    note: None,
                    suggested_replacement: None,
                },
            ],
            code: Some(DiagnosticId::Error(ErrorKind::CompileError)),
        };
        let out = render(&[diag]);
        let parsed = parse_log(&out).unwrap();
        let result = &parsed["runs"][0]["results"][0];
        let related = result["relatedLocations"].as_array().unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(related[0]["message"]["text"], "extra");
        assert_eq!(
            related[0]["physicalLocation"]["artifactLocation"]["uri"],
            "b.k"
        );
    }

    #[test]
    fn rules_are_deduplicated() {
        // Two diagnostics with the same rule id should yield a single rule.
        let d1 = sample_diag();
        let mut d2 = sample_diag();
        d2.messages[0].message = "second occurrence".to_string();
        let out = render(&[d1, d2]);
        let parsed = parse_log(&out).unwrap();
        let rules = parsed["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules.len(), 1);
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);
    }
}