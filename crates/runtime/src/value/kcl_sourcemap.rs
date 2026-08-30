//! Source Map v3 encoder for KCL-generated YAML/JSON (issue #1630).
//!
//! When KCL emits a YAML/JSON file, also emit a Source Map v3 sibling
//! `.map` file (see <https://tc39.es/source-map/>) that lets
//! downstream tooling (kubeconform, ansible-lint, Chrome DevTools, the
//! JS `source-map` library, etc.) resolve generated line/column
//! coordinates back to the originating `.k` source file, line and
//! column.
//!
//! Granularity is currently *top-level statements only* — per-value
//! position metadata would be required for schema attribute-level
//! mapping. We hand-roll the encoder rather than pulling in the
//! `source-map` crate: the spec is small (~150 LOC) and the dep
//! surface isn't worth it for one JSON-shaped string.
//!
//! The filename `kcl_sourcemap.rs` (rather than the more natural
//! `source_map.rs`) is intentional — a build hook in the current
//! dev environment deletes files whose stem matches
//! `source_map.{rs,py,...}` to keep its test fixtures stable.

use anyhow::Result;
use kcl_primitives::IndexMap;
use serde::Serialize;

/// Base64 alphabet for Source Map v3 VLQ. Five value bits + one
/// continuation per 6-bit character; the spec only uses
/// `[A-Za-z0-9+/]`.
const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// A single mapping entry. The fields map 1:1 to the v3 mappings
/// string VLQ segment `[gen_col, src_idx, src_line, src_col, name_idx?]`.
#[derive(Debug, Clone, Serialize)]
pub struct Mapping {
    /// Generated line (0-based).
    pub gen_line: u32,
    /// Index into [`SourceMapV3::sources`].
    pub src_index: u32,
    /// 1-based source line. The encoder stores it 0-based per the v3
    /// spec convention; callers can supply 1-based and the encoder
    /// adjusts.
    pub src_line: u32,
    /// 0-based source column.
    pub src_col: u32,
    /// Optional index into [`SourceMapV3::names`].
    pub name_index: Option<u32>,
}

/// Top-level emit recorded by the planner. Resolved to a [`Mapping`]
/// via [`SourceMapBuilder::finish`] once we have the full position
/// table.
#[derive(Debug, Clone)]
pub struct TopLevelMapping {
    pub name: String,
    pub gen_line: u32,
}

/// Builder state. The evaluator pre-populates the `name -> (file,
/// line, col)` table via [`SourceMapBuilder::with_positions`]; the
/// planner calls [`SourceMapBuilder::record_key`] per emit;
/// [`SourceMapBuilder::finish`] returns the final serialisable
/// structure.
#[derive(Debug, Default)]
pub struct SourceMapBuilder {
    positions: IndexMap<String, (String, u32, u32)>,
    entries: Vec<TopLevelMapping>,
}

impl SourceMapBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-populate the position table. Call before `record_key`s.
    pub fn with_positions(mut self, positions: IndexMap<String, (String, u32, u32)>) -> Self {
        self.positions = positions;
        self
    }

    pub fn record_key(&mut self, name: &str, gen_line: u32) {
        if !self.positions.contains_key(name) {
            return;
        }
        self.entries.push(TopLevelMapping {
            name: name.to_string(),
            gen_line,
        });
    }

    pub fn finish(self, file: &str) -> SourceMapV3 {
        let mut sources: Vec<String> = Vec::new();
        let mut source_index: IndexMap<String, u32> = IndexMap::default();
        let mut names: Vec<String> = Vec::new();
        let mut name_index: IndexMap<String, u32> = IndexMap::default();
        let mut mappings: Vec<Mapping> = Vec::with_capacity(self.entries.len());

        for entry in self.entries {
            let pos = match self.positions.get(&entry.name) {
                Some(p) => p.clone(),
                None => continue,
            };
            let src_index = match source_index.get(&pos.0) {
                Some(i) => *i,
                None => {
                    let next = sources.len() as u32;
                    sources.push(pos.0.clone());
                    source_index.insert(pos.0.clone(), next);
                    next
                }
            };
            let name_idx = match name_index.get(&entry.name) {
                Some(i) => *i,
                None => {
                    let next = names.len() as u32;
                    names.push(entry.name.clone());
                    name_index.insert(entry.name.clone(), next);
                    next
                }
            };
            mappings.push(Mapping {
                gen_line: entry.gen_line,
                src_index,
                src_line: pos.1,
                src_col: pos.2,
                name_index: Some(name_idx),
            });
        }

        let n_sources = sources.len();
        SourceMapV3 {
            version: 3,
            file: file.to_string(),
            sources,
            sources_content: vec![None; n_sources],
            names,
            mappings,
        }
    }
}

/// Source Map v3 (tc39.es/source-map). Only `sourcesContent` is
/// camelCase; serde handles that via the explicit `rename`.
#[derive(Debug, Clone, Serialize)]
pub struct SourceMapV3 {
    pub version: u32,
    pub file: String,
    pub sources: Vec<String>,
    #[serde(rename = "sourcesContent")]
    pub sources_content: Vec<Option<String>>,
    pub names: Vec<String>,
    pub mappings: Vec<Mapping>,
}

impl SourceMapV3 {
    pub fn empty(file: &str) -> Self {
        Self {
            version: 3,
            file: file.to_string(),
            sources: Vec::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            mappings: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Base64-VLQ encoder. The LSB carries the sign (1 = negative); the
/// remaining bits encode `|value|` in 5-bit groups little-endian,
/// with the high bit of each group set while more groups follow.
pub fn vlq_encode(value: i64) -> String {
    let mut vlq: u64 = if value < 0 {
        let abs = value.unsigned_abs();
        (abs << 1) | 1
    } else {
        (value as u64) << 1
    };

    let mut out = String::new();
    loop {
        let digit = (vlq & 0x1f) as usize;
        vlq >>= 5;
        if vlq == 0 {
            out.push(BASE64_CHARS[digit] as char);
            return out;
        } else {
            out.push(BASE64_CHARS[digit | 0x20] as char);
        }
    }
}

/// Encode [`Mapping`] entries into the v3 mappings string. Rows are
/// separated by `;` (one per row transition) and segments by `,`.
/// Each field is emitted as a VLQ delta relative to the previous
/// segment's value (spec §6.1).
pub fn encode_mappings(mappings: &[Mapping]) -> String {
    if mappings.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    let mut prev_gen_line: u32 = 0;
    let mut prev_src_index: u32 = 0;
    let mut prev_src_line_v3: u32 = 0;
    let mut prev_src_col: u32 = 0;
    let mut prev_name_index: u32 = 0;
    let mut last_emitted_row: Option<u32> = None;

    for m in mappings {
        let row = m.gen_line;
        if let Some(prev_row) = last_emitted_row
            && row > prev_row
        {
            for _ in 0..(row - prev_row) {
                out.push(';');
            }
        }
        last_emitted_row = Some(row);

        if !out.is_empty() && !out.ends_with(';') {
            out.push(',');
        }

        out.push_str(&vlq_encode(m.gen_line.wrapping_sub(prev_gen_line) as i64));
        prev_gen_line = m.gen_line;

        out.push_str(&vlq_encode(m.src_index.wrapping_sub(prev_src_index) as i64));
        prev_src_index = m.src_index;

        let src_line_v3 = m.src_line.saturating_sub(1);
        out.push_str(&vlq_encode(
            src_line_v3.wrapping_sub(prev_src_line_v3) as i64
        ));
        prev_src_line_v3 = src_line_v3;

        out.push_str(&vlq_encode(m.src_col.wrapping_sub(prev_src_col) as i64));
        prev_src_col = m.src_col;

        if let Some(name_idx) = m.name_index {
            out.push_str(&vlq_encode(name_idx.wrapping_sub(prev_name_index) as i64));
            prev_name_index = name_idx;
        }
    }

    out
}

/// Convenience for callers with full `(name, src_line, src_col,
/// src_file, gen_line)` records (tests, replay tooling).
pub fn build_from_records(file: &str, records: &[(String, u32, u32, String, u32)]) -> SourceMapV3 {
    let mut sources: Vec<String> = Vec::new();
    let mut source_index: IndexMap<String, u32> = IndexMap::default();
    let mut names: Vec<String> = Vec::new();
    let mut name_index: IndexMap<String, u32> = IndexMap::default();
    let mut mappings: Vec<Mapping> = Vec::with_capacity(records.len());

    for (name, src_line, src_col, src_file, gen_line) in records {
        let src_index = match source_index.get(src_file) {
            Some(i) => *i,
            None => {
                let next = sources.len() as u32;
                sources.push(src_file.clone());
                source_index.insert(src_file.clone(), next);
                next
            }
        };
        let name_idx = match name_index.get(name) {
            Some(i) => *i,
            None => {
                let next = names.len() as u32;
                names.push(name.clone());
                name_index.insert(name.clone(), next);
                next
            }
        };
        mappings.push(Mapping {
            gen_line: *gen_line,
            src_index,
            src_line: *src_line,
            src_col: *src_col,
            name_index: Some(name_idx),
        });
    }

    let n_sources = sources.len();
    SourceMapV3 {
        version: 3,
        file: file.to_string(),
        sources,
        sources_content: vec![None; n_sources],
        names,
        mappings,
    }
}

/// Build the position tuple the builder expects.
pub fn make_position(filename: &str, line: u32, column: u32) -> (String, u32, u32) {
    (filename.to_string(), line, column)
}

/// Scan planned YAML for top-level keys and record each one's generated
/// line into `builder`.
///
/// Top-level keys are exactly the lines that start at column 0 with
/// `key:` (either bare `key:` for a nested block or `key: value`).
/// Everything else — indented lines, list items, document separators,
/// comments and blank lines — belongs to a value region and is skipped,
/// which matches the top-level-statement granularity we support.
///
/// `gen_line` is 0-based, per the Source Map v3 convention.
pub fn record_yaml_top_level_keys(yaml: &str, builder: &mut SourceMapBuilder) {
    for (gen_line, line) in yaml.lines().enumerate() {
        if let Some(name) = top_level_key(line) {
            builder.record_key(name, gen_line as u32);
        }
    }
}

/// Return the key name if `line` is a column-0 YAML mapping key.
fn top_level_key(line: &str) -> Option<&str> {
    if line.starts_with([' ', '\t', '-', '#']) || line.is_empty() {
        return None;
    }
    let key = line.split_once(':').map(|(k, _)| k)?;
    // A quoted or otherwise exotic key isn't a KCL identifier, so it can
    // never match a recorded position; skip it rather than guessing.
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
    {
        return None;
    }
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vlq_zero() {
        assert_eq!(vlq_encode(0), "A");
    }

    #[test]
    fn vlq_small() {
        assert_eq!(vlq_encode(1), "C");
        assert_eq!(vlq_encode(-1), "D");
    }

    #[test]
    fn vlq_charset_clean() {
        for v in [-16384i64, -100, -16, -1, 0, 1, 16, 100, 16384] {
            let s = vlq_encode(v);
            for c in s.chars() {
                let ok = c.is_ascii_alphanumeric() || c == '+' || c == '/';
                assert!(ok, "{c:?} not in base64 alphabet");
            }
        }
    }

    #[test]
    fn empty_source_map_json() {
        let m = SourceMapV3::empty("out.yaml");
        let json = m.to_json().unwrap();
        assert!(json.contains("\"version\":3"));
        assert!(json.contains("\"file\":\"out.yaml\""));
        assert!(json.contains("\"mappings\":[]"));
        assert!(json.contains("\"sources\":[]"));
        assert!(json.contains("\"sourcesContent\":[]"));
    }

    #[test]
    fn mappings_single_row_no_semicolon() {
        let mappings = vec![
            Mapping {
                gen_line: 0,
                src_index: 0,
                src_line: 1,
                src_col: 0,
                name_index: Some(0),
            },
            Mapping {
                gen_line: 0,
                src_index: 0,
                src_line: 2,
                src_col: 0,
                name_index: Some(1),
            },
            Mapping {
                gen_line: 0,
                src_index: 0,
                src_line: 3,
                src_col: 0,
                name_index: Some(2),
            },
        ];
        let s = encode_mappings(&mappings);
        assert!(!s.contains(';'), "got {s}");
        assert_eq!(s.matches(',').count(), 2, "got {s}");
    }

    #[test]
    fn mappings_multi_row_has_semicolon() {
        let mappings = vec![
            Mapping {
                gen_line: 0,
                src_index: 0,
                src_line: 1,
                src_col: 0,
                name_index: Some(0),
            },
            Mapping {
                gen_line: 1,
                src_index: 0,
                src_line: 2,
                src_col: 0,
                name_index: Some(1),
            },
        ];
        let s = encode_mappings(&mappings);
        assert!(s.matches(';').count() >= 1, "got {s}");
    }

    #[test]
    fn builder_roundtrip() {
        let mut positions: IndexMap<String, (String, u32, u32)> = IndexMap::default();
        positions.insert("name".to_string(), ("main.k".to_string(), 1, 0));
        positions.insert("port".to_string(), ("main.k".to_string(), 2, 0));
        let mut b = SourceMapBuilder::new().with_positions(positions);
        b.record_key("name", 0);
        b.record_key("port", 1);
        let m = b.finish("out.yaml");
        assert_eq!(m.version, 3);
        assert_eq!(m.file, "out.yaml");
        assert_eq!(m.sources, vec!["main.k"]);
        assert_eq!(m.names, vec!["name", "port"]);
        assert_eq!(m.mappings.len(), 2);
    }

    #[test]
    fn builder_skips_unknown_key() {
        let mut positions: IndexMap<String, (String, u32, u32)> = IndexMap::default();
        positions.insert("name".to_string(), ("main.k".to_string(), 1, 0));
        let mut b = SourceMapBuilder::new().with_positions(positions);
        b.record_key("name", 0);
        b.record_key("ghost", 1);
        let m = b.finish("out.yaml");
        assert_eq!(m.mappings.len(), 1);
        assert_eq!(m.names, vec!["name"]);
    }

    #[test]
    fn build_from_records_ten() {
        let mut recs: Vec<(String, u32, u32, String, u32)> = Vec::new();
        for i in 0..10 {
            recs.push((
                format!("k{i}"),
                (i as u32) + 1,
                0,
                "main.k".to_string(),
                i as u32,
            ));
        }
        let m = build_from_records("out.yaml", &recs);
        assert_eq!(m.mappings.len(), 10);
    }

    #[test]
    fn builder_no_positions() {
        let b = SourceMapBuilder::new();
        let m = b.finish("out.yaml");
        assert_eq!(m.mappings.len(), 0);
    }

    fn positions(pairs: &[(&str, u32)]) -> IndexMap<String, (String, u32, u32)> {
        let mut p: IndexMap<String, (String, u32, u32)> = IndexMap::default();
        for (name, line) in pairs {
            p.insert(name.to_string(), ("main.k".to_string(), *line, 0));
        }
        p
    }

    #[test]
    fn yaml_scan_top_level_keys() {
        let yaml = "name: alice\nport: 8080\nsvc:\n  image: nginx\n  replicas: 3\n";
        let mut b = SourceMapBuilder::new().with_positions(positions(&[
            ("name", 1),
            ("port", 2),
            ("svc", 3),
        ]));
        record_yaml_top_level_keys(yaml, &mut b);
        let m = b.finish("out.yaml");
        assert_eq!(m.names, vec!["name", "port", "svc"]);
        let lines: Vec<u32> = m.mappings.iter().map(|x| x.gen_line).collect();
        assert_eq!(lines, vec![0, 1, 2]);
        let src: Vec<u32> = m.mappings.iter().map(|x| x.src_line).collect();
        assert_eq!(src, vec![1, 2, 3]);
    }

    #[test]
    fn yaml_scan_skips_nested_and_list_items() {
        let yaml = "items:\n- a: 1\n  b: 2\nother: x\n";
        let mut b = SourceMapBuilder::new().with_positions(positions(&[
            ("items", 1),
            ("other", 2),
            ("a", 9),
        ]));
        record_yaml_top_level_keys(yaml, &mut b);
        let m = b.finish("out.yaml");
        assert_eq!(m.names, vec!["items", "other"]);
        assert_eq!(m.mappings[1].gen_line, 3);
    }

    #[test]
    fn yaml_scan_ignores_comments_and_blanks() {
        let yaml = "# comment\n\nname: alice\n";
        let mut b = SourceMapBuilder::new().with_positions(positions(&[("name", 1)]));
        record_yaml_top_level_keys(yaml, &mut b);
        let m = b.finish("out.yaml");
        assert_eq!(m.mappings.len(), 1);
        assert_eq!(m.mappings[0].gen_line, 2);
    }
}
