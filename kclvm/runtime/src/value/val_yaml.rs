//! Copyright The KCL Authors. All rights reserved.

extern crate serde_json;
extern crate serde_yaml;

use crate::*;

use serde::{Deserialize, Serialize};

/// YAML encode options.
/// - sort_keys: Sort the encode result by keys (defaults to false).
/// - ignore_private: Whether to ignore the attribute whose name starts with
///     a character `_` (defaults to false).
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
}

impl Default for YamlEncodeOptions {
    fn default() -> Self {
        Self {
            sort_keys: false,
            ignore_private: false,
            ignore_none: false,
            sep: "---".to_string(),
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

    pub fn to_yaml(&self) -> Vec<u8> {
        let json = self.to_json_string();
        let yaml_value: serde_yaml::Value = serde_json::from_str(json.as_ref()).unwrap();
        match serde_yaml::to_string(&yaml_value) {
            Ok(s) => s.into_bytes(),
            _ => Vec::new(),
        }
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

    pub fn to_yaml_string_with_options(&self, opt: &YamlEncodeOptions) -> String {
        // convert Value to json in order to reuse
        // "crate::val_json::JsonValue" to customize the serialized results
        let json_opt = JsonEncodeOptions {
            sort_keys: opt.sort_keys,
            indent: 0,
            ignore_private: opt.ignore_private,
            ignore_none: opt.ignore_none,
        };
        let json = self.to_json_string_with_options(&json_opt);
        let yaml_value: serde_yaml::Value = serde_json::from_str(json.as_ref()).unwrap();
        match serde_yaml::to_string(&yaml_value) {
            Ok(s) => {
                let s = s.strip_prefix("---\n").unwrap_or_else(|| s.as_ref());
                s.to_string()
            }
            Err(err) => panic!("{}", err),
        }
    }
}

#[cfg(test)]
mod test_value_yaml {
    use crate::*;

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
                },
            ),
        ];
        for (value, expected, opts) in cases {
            let result = ValueRef::to_yaml_string_with_options(&value, &opts);
            assert_eq!(result, expected);
        }
    }
}
