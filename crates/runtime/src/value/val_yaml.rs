//! Copyright The KCL Authors. All rights reserved.

extern crate serde_json;
extern crate serde_yaml;

use crate::*;

use serde::{Deserialize, Serialize};

/// YAML encode options.
/// - sort_keys: Sort the encode result by keys (defaults to false).
/// - ignore_private: Whether to ignore the attribute whose name starts with
///   a character `_` (defaults to false).
/// - ignore_none: Whether to ignore the attribute whose value is `None` (defaults to false).
/// - sep: Which separator to use between YAML documents (defaults to "---").
///
/// TODO: We have not yet supported the following options because serde_yaml
/// does not support these capabilities yet.
/// Ref: https://github.com/dtolnay/serde-yaml/issues/337
/// - indent: Which kind of indentation to use when emitting (defaults to 2).
/// - width: The character width to use when folding text (defaults to 80).
/// - use_fold: Force folding of text when emitting (defaults to false).
/// - use_block: Force all text to be literal when emitting (defaults to false).
/// - use_version: Display the YAML version when emitting (defaults to false).
/// - use_header: Display the YAML header when emitting (defaults to false).
#[derive(Debug, Serialize, Deserialize)]
pub struct YamlEncodeOptions {
    pub sort_keys: bool,
    pub ignore_private: bool,
    pub ignore_none: bool,
    pub sep: String,
    /// When true, strings containing literal escape sequences such as `\n` or
    /// `\t` are emitted using YAML block scalar (`|`) style with real
    /// newlines/tabs, instead of leaving the escapes unquoted in the output.
    /// Defaults to false to preserve the historical escaping behaviour.
    pub multiline_string: bool,
}

impl Default for YamlEncodeOptions {
    fn default() -> Self {
        Self {
            sort_keys: false,
            ignore_private: false,
            ignore_none: false,
            sep: "---".to_string(),
            multiline_string: false,
        }
    }
}

impl ValueRef {
    /// Decode a yaml single document string to a ValueRef.
    /// Returns [serde_yaml::Error] when decoding fails.
    pub fn from_yaml(ctx: &mut Context, s: &str) -> Result<Self, serde_yaml::Error> {
        // We use JsonValue to implement the KCL universal serialization object.
        let json_value: JsonValue = serde_yaml::from_str(s)?;
        Ok(Self::from_json(ctx, serde_json::to_string(&json_value).unwrap().as_ref()).unwrap())
    }

    /// Decode yaml stream string that contains `---` to a ValueRef.
    /// Returns [serde_yaml::Error] when decoding fails.
    pub fn from_yaml_stream(ctx: &mut Context, s: &str) -> Result<Self, serde_yaml::Error> {
        let documents = serde_yaml::Deserializer::from_str(s);
        let mut result = ValueRef::list_value(None);
        for document in documents {
            let json_value: JsonValue = JsonValue::deserialize(document)?;
            result.list_append(&ValueRef::parse_json(ctx, &json_value))
        }
        if result.is_empty() {
            // Empty result returns a empty dict.
            Ok(ValueRef::dict(None))
        } else if result.len() == 1 {
            Ok(result.list_get(0).unwrap())
        } else {
            Ok(result)
        }
    }

    /// Decode yaml stream string that contains `---` to a ValueRef.
    /// Returns [serde_yaml::Error] when decoding fails.
    pub fn list_from_yaml_stream(ctx: &mut Context, s: &str) -> Result<Self, serde_yaml::Error> {
        let documents = serde_yaml::Deserializer::from_str(s);
        let mut result = ValueRef::list_value(None);
        for document in documents {
            let json_value: JsonValue = JsonValue::deserialize(document)?;
            result.list_append(&ValueRef::parse_json(ctx, &json_value))
        }
        Ok(result)
    }

    pub fn to_yaml(&self) -> Vec<u8> {
        let json = self.to_json_string();
        let yaml_value: serde_yaml::Value = serde_json::from_str(json.as_ref()).unwrap();
        match serde_yaml::to_string(&yaml_value) {
            Ok(s) => s.into_bytes(),
            _ => Vec::new(),
        }
    }

    /// Drop the trailing newline that `serde_yaml::to_string` always emits,
    /// but only when doing so does not change the meaning of the document.
    ///
    /// `serde_yaml::to_string` always appends exactly one `\n` to the end of
    /// the output. When the last emitted scalar is a normal `key: value`
    /// mapping that newline is a separator and can be removed safely. When
    /// the last emitted value is a block scalar (`|`, `|-`, `|+`, `>`, ...)
    /// however that newline is the block scalar's own terminator: stripping
    /// it changes the round-tripped value from `"line1\nline2\nline3\n"` to
    /// `"line1\nline2\nline3"` and breaks KCL's documented `"""\...\"\"`
    /// rendering (issue kcl-lang/kcl#1894).
    ///
    /// Heuristic:
    /// - Top-level last line: a `key: value` line at indent 0 is a regular
    ///   mapping - strip the trailing newline.
    /// - Indented last line: walk backwards until we find a line whose
    ///   indentation is strictly less than the last line's indentation.
    ///   That line is the parent mapping/block-scalar header. If the
    ///   parent header value is one of `|`, `|-`, `|+`, `>`, `>-`, `>+`
    ///   the trailing newline(s) belong to that block scalar and we keep
    ///   them; otherwise we strip a single trailing newline.
    pub(crate) fn strip_yaml_trailing_newline(s: &str) -> &str {
        if !s.ends_with('\n') {
            return s;
        }
        let trimmed = s.trim_end_matches('\n');
        if trimmed.is_empty() {
            return s;
        }
        let lines: Vec<&str> = trimmed.split('\n').collect();
        let last = lines.last().copied().unwrap_or("");
        let last_indent = indent_len(last);
        if last_indent == 0 {
            // Top-level last line. If it is itself a block-scalar header
            // (no body, e.g. `key: |`) the trailing newline has no semantic
            // content; strip it. If it is `key: value` (a normal scalar),
            // the trailing newline is a separator; strip it.
            return s.strip_suffix('\n').unwrap_or(s);
        }
        // Walk backwards until we hit a non-empty line whose indentation
        // is strictly less than `last_indent`. That line is the parent.
        for prev in lines.iter().rev().skip(1) {
            if prev.trim().is_empty() {
                continue;
            }
            let prev_indent = indent_len(prev);
            if prev_indent < last_indent {
                let header_value = prev
                    .find(':')
                    .map(|i| prev[i + 1..].trim_start())
                    .unwrap_or("");
                let is_block_scalar_header =
                    matches!(header_value, "|" | "|-" | "|+" | ">" | ">-" | ">+");
                return if is_block_scalar_header {
                    s
                } else {
                    s.strip_suffix('\n').unwrap_or(s)
                };
            }
        }
        // No less-indented parent found; preserve to be safe.
        s
    }

    pub fn to_yaml_string(&self) -> String {
        let json = self.to_json_string();
        let yaml_value: serde_yaml::Value = serde_json::from_str(json.as_ref()).unwrap();
        match serde_yaml::to_string(&yaml_value) {
            Ok(s) => {
                let s = s.strip_prefix("---\n").unwrap_or_else(|| s.as_ref());
                s.to_string()
            }
            Err(err) => panic!("{}", err),
        }
    }

    pub fn to_yaml_string_with_options(&self, opts: &YamlEncodeOptions) -> String {
        // convert Value to json in order to reuse
        // "crate::val_json::JsonValue" to customize the serialized results
        let json_opts = JsonEncodeOptions {
            sort_keys: opts.sort_keys,
            indent: 0,
            ignore_private: opts.ignore_private,
            ignore_none: opts.ignore_none,
        };
        let json = self.to_json_string_with_options(&json_opts);
        let yaml_value: serde_yaml::Value = if opts.multiline_string {
            // When the caller opts in, convert literal escape sequences inside
            // string values (e.g. the two characters `\n` that appear after the
            // JSON round-trip) into their real character form so that the YAML
            // emitter picks block-scalar style for multi-line content.
            let json_value: serde_json::Value = serde_json::from_str(json.as_ref()).unwrap();
            json_to_yaml_value_with_real_escapes(json_value)
        } else {
            serde_json::from_str(json.as_ref()).unwrap()
        };
        match serde_yaml::to_string(&yaml_value) {
            Ok(s) => {
                let s = s.strip_prefix("---\n").unwrap_or_else(|| s.as_ref());
                s.to_string()
            }
            Err(err) => panic!("{}", err),
        }
    }
}

/// Walk a `serde_json::Value` and convert every string entry so that the JSON
/// escape sequences (`\n`, `\t`, `\r`, `\\`, `\"`, `\uXXXX`) are emitted as their
/// real characters. This lets the YAML serializer downstream pick block-scalar
/// (`|`) style for strings that span multiple lines.
fn json_to_yaml_value_with_real_escapes(value: serde_json::Value) -> serde_yaml::Value {
    match value {
        serde_json::Value::Null => serde_yaml::Value::Null,
        serde_json::Value::Bool(b) => serde_yaml::Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_yaml::Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                serde_yaml::Value::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                serde_yaml::Value::Number(f.into())
            } else {
                serde_yaml::Value::Null
            }
        }
        serde_json::Value::String(s) => serde_yaml::Value::String(unescape_json_string(&s)),
        serde_json::Value::Array(items) => {
            let mapped = items
                .into_iter()
                .map(json_to_yaml_value_with_real_escapes)
                .collect();
            serde_yaml::Value::Sequence(mapped)
        }
        serde_json::Value::Object(map) => {
            let mut mapped = serde_yaml::Mapping::with_capacity(map.len());
            for (k, v) in map {
                mapped.insert(
                    serde_yaml::Value::String(k),
                    json_to_yaml_value_with_real_escapes(v),
                );
            }
            serde_yaml::Value::Mapping(mapped)
        }
    }
}

/// Replace the common JSON escape sequences in `s` with their real character
/// equivalents. Any backslash followed by a character that is not a recognised
/// escape is left untouched (the trailing `\` is dropped, matching the lenient
/// behaviour of `serde_json` when decoding strings).
fn unescape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('b') => out.push('\u{0008}'),
                Some('f') => out.push('\u{000C}'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some('u') => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        match chars.next() {
                            Some(c) => hex.push(c),
                            None => break,
                        }
                    }
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
                Some(other) => {
                    // Unknown escape: keep the character as-is and drop the
                    // backslash so that, for example, `\x` becomes `x`.
                    out.push(other);
                }
                None => {
                    // Trailing backslash with nothing after it: drop it.
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn indent_len(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

#[cfg(test)]
mod test_value_yaml {
    use crate::*;

    #[test]
    fn test_serde_yaml_1_1_str() {
        let on_str = serde_yaml::to_string("on").unwrap();
        assert_eq!(on_str, "'on'\n");
        let yes_str = serde_yaml::to_string("yes").unwrap();
        assert_eq!(yes_str, "'yes'\n");
    }

    #[test]
    fn test_value_from_yaml() {
        let mut ctx = Context::new();
        let cases = [
            ("a: 1\n", ValueRef::dict(Some(&[("a", &ValueRef::int(1))]))),
            (
                "a: 1\nb: 2\n",
                ValueRef::dict(Some(&[("a", &ValueRef::int(1)), ("b", &ValueRef::int(2))])),
            ),
            (
                "a: [1, 2, 3]\nb: \"s\"\n",
                ValueRef::dict(Some(&[
                    ("a", &ValueRef::list_int(&[1, 2, 3])),
                    ("b", &ValueRef::str("s")),
                ])),
            ),
            // This case is to test that the `from_yaml` function does not change
            // the order of dictionary keys.
            (
                "b: [1, 2, 3]\na: \"s\"\n",
                ValueRef::dict(Some(&[
                    ("b", &ValueRef::list_int(&[1, 2, 3])),
                    ("a", &ValueRef::str("s")),
                ])),
            ),
        ];
        for (yaml_str, expected) in cases {
            let result = ValueRef::from_yaml(&mut ctx, yaml_str);
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn test_value_from_yaml_fail() {
        let mut ctx = Context::new();
        let cases = [
            (
                "a: 1\n  b: 2\nc: 3",
                "mapping values are not allowed in this context at line 2 column 4",
            ),
            (
                "a:\n- 1\n  -2\n-3",
                "could not find expected ':' at line 5 column 1, while scanning a simple key at line 4 column 1",
            ),
        ];
        for (yaml_str, expected) in cases {
            let result = ValueRef::from_yaml(&mut ctx, yaml_str);
            assert_eq!(result.err().unwrap().to_string(), expected);
        }
    }

    #[test]
    fn test_value_from_yaml_stream() {
        let mut ctx = Context::new();
        let cases = [
            ("a: 1\n", ValueRef::dict(Some(&[("a", &ValueRef::int(1))]))),
            (
                "a: 1\nb: 2\n---\nb: 1\na: 2\n",
                ValueRef::list_value(Some(&[
                    ValueRef::dict(Some(&[("a", &ValueRef::int(1)), ("b", &ValueRef::int(2))])),
                    ValueRef::dict(Some(&[("b", &ValueRef::int(1)), ("a", &ValueRef::int(2))])),
                ])),
            ),
        ];
        for (yaml_str, expected) in cases {
            let result = ValueRef::from_yaml_stream(&mut ctx, yaml_str);
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn test_value_from_yaml_stream_fail() {
        let mut ctx = Context::new();
        let cases = [
            (
                "a: 1\n---\na: 1\n  b: 2\nc: 3",
                "mapping values are not allowed in this context at line 4 column 4",
            ),
            (
                "b:3\n---\na:\n- 1\n  -2\n-3",
                "could not find expected ':' at line 7 column 1, while scanning a simple key at line 6 column 1",
            ),
        ];
        for (yaml_str, expected) in cases {
            let result = ValueRef::from_yaml_stream(&mut ctx, yaml_str);
            assert_eq!(result.err().unwrap().to_string(), expected);
        }
    }

    #[test]
    fn test_value_to_yaml_string() {
        let cases = [
            (ValueRef::dict(Some(&[("a", &ValueRef::int(1))])), "a: 1\n"),
            (
                ValueRef::dict(Some(&[("a", &ValueRef::int(1)), ("b", &ValueRef::int(2))])),
                "a: 1\nb: 2\n",
            ),
            (
                ValueRef::dict(Some(&[
                    ("a", &ValueRef::list_int(&[1, 2, 3])),
                    ("b", &ValueRef::str("s")),
                ])),
                "a:\n- 1\n- 2\n- 3\nb: s\n",
            ),
        ];
        for (value, expected) in cases {
            let result = ValueRef::to_yaml_string(&value);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_value_to_yaml_string_with_opts() {
        let cases = [
            (
                ValueRef::dict(Some(&[("b", &ValueRef::int(2)), ("a", &ValueRef::int(1))])),
                "a: 1\nb: 2\n",
                YamlEncodeOptions {
                    sort_keys: true,
                    ignore_private: false,
                    ignore_none: false,
                    sep: "---".to_string(),
                    multiline_string: false,
                },
            ),
            (
                ValueRef::dict(Some(&[("b", &ValueRef::int(2)), ("a", &ValueRef::int(1))])),
                "b: 2\na: 1\n",
                YamlEncodeOptions {
                    sort_keys: false,
                    ignore_private: false,
                    ignore_none: false,
                    sep: "---".to_string(),
                    multiline_string: false,
                },
            ),
            (
                ValueRef::dict(Some(&[("_b", &ValueRef::int(2)), ("a", &ValueRef::int(1))])),
                "a: 1\n",
                YamlEncodeOptions {
                    sort_keys: false,
                    ignore_private: true,
                    ignore_none: false,
                    sep: "---".to_string(),
                    multiline_string: false,
                },
            ),
            (
                ValueRef::dict(Some(&[("b", &ValueRef::none()), ("a", &ValueRef::int(1))])),
                "a: 1\n",
                YamlEncodeOptions {
                    sort_keys: false,
                    ignore_private: true,
                    ignore_none: true,
                    sep: "---".to_string(),
                    multiline_string: false,
                },
            ),
            (
                ValueRef::dict(Some(&[
                    ("b", &ValueRef::list_int(&[1, 2, 3])),
                    ("a", &ValueRef::str("s")),
                ])),
                "a: s\nb:\n- 1\n- 2\n- 3\n",
                YamlEncodeOptions {
                    sort_keys: true,
                    ignore_private: false,
                    ignore_none: false,
                    sep: "---".to_string(),
                    multiline_string: false,
                },
            ),
        ];
        for (value, expected, opts) in cases {
            let result = ValueRef::to_yaml_string_with_options(&value, &opts);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_value_to_yaml_string_with_multiline_string_option() {
        // Without the option, literal `\n` sequences in strings remain escaped
        // (the historical behaviour).
        let value = ValueRef::dict(Some(&[("field", &ValueRef::str("a\\nb\\nc"))]));
        let result = ValueRef::to_yaml_string_with_options(
            &value,
            &YamlEncodeOptions {
                multiline_string: false,
                ..Default::default()
            },
        );
        assert_eq!(result, "field: a\\nb\\nc\n");

        // With the option enabled, the embedded `\n` sequences become real
        // newlines and the YAML emitter picks block-scalar style.
        let result = ValueRef::to_yaml_string_with_options(
            &value,
            &YamlEncodeOptions {
                multiline_string: true,
                ..Default::default()
            },
        );
        assert_eq!(result, "field: |-\n  a\n  b\n  c\n");

        // Nested values get the same treatment.
        let value = ValueRef::dict(Some(&[(
            "obj",
            &ValueRef::dict(Some(&[("field", &ValueRef::str("a\\nb"))])),
        )]));
        let result = ValueRef::to_yaml_string_with_options(
            &value,
            &YamlEncodeOptions {
                multiline_string: true,
                ..Default::default()
            },
        );
        assert_eq!(result, "obj:\n  field: |-\n    a\n    b\n");

        // Strings without escape sequences are unaffected by the option.
        let value = ValueRef::dict(Some(&[("field", &ValueRef::str("hello world"))]));
        let result = ValueRef::to_yaml_string_with_options(
            &value,
            &YamlEncodeOptions {
                multiline_string: true,
                ..Default::default()
            },
        );
        assert_eq!(result, "field: hello world\n");
    }

    #[test]
    fn test_strip_yaml_trailing_newline() {
        // Plain `key: value` mapping at the end: the trailing newline is
        // just a separator and may be stripped.
        assert_eq!(
            ValueRef::strip_yaml_trailing_newline("a: 1\nb: 2\n"),
            "a: 1\nb: 2"
        );
        // Single key/value: trailing newline is a separator.
        assert_eq!(ValueRef::strip_yaml_trailing_newline("a: 1\n"), "a: 1");
        // Block scalar (`|`) is the last value. The trailing newline belongs
        // to the block scalar's content and must be preserved so the
        // round-tripped value still ends with `\n` (issue kcl-lang/kcl#1894).
        assert_eq!(
            ValueRef::strip_yaml_trailing_newline("a: |\n  line1\n  line2\n"),
            "a: |\n  line1\n  line2\n"
        );
        // Nested block scalar (last field of a mapping).
        assert_eq!(
            ValueRef::strip_yaml_trailing_newline("outer:\n  inner: |\n    line1\n    line2\n"),
            "outer:\n  inner: |\n    line1\n    line2\n"
        );
        // Block scalar with strip-chomp indicator `|-`.
        assert_eq!(
            ValueRef::strip_yaml_trailing_newline("a: |-\n  line1\n  line2\n"),
            "a: |-\n  line1\n  line2\n"
        );
        // Folded block scalar `>`.
        assert_eq!(
            ValueRef::strip_yaml_trailing_newline("a: >\n  line1\n  line2\n"),
            "a: >\n  line1\n  line2\n"
        );
        // Block scalar followed by a normal key: the trailing newline is a
        // separator and may be stripped.
        assert_eq!(
            ValueRef::strip_yaml_trailing_newline("block: |\n  line1\nkey: value\n"),
            "block: |\n  line1\nkey: value"
        );
        // Nested mapping (no block scalar): the indented last line is a
        // mapping value, so the trailing newline is a separator and may
        // be stripped.
        assert_eq!(
            ValueRef::strip_yaml_trailing_newline("data:\n  _type: Data\n"),
            "data:\n  _type: Data"
        );
        // Already without trailing newline: returned untouched.
        assert_eq!(ValueRef::strip_yaml_trailing_newline("a: 1"), "a: 1");
    }
}
