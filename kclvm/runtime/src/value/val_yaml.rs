// Copyright 2021 The KCL Authors. All rights reserved.

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
    pub fn from_yaml(s: &str) -> Option<Self> {
        let json_value: serde_json::Value = serde_yaml::from_str(s).unwrap();
        match serde_json::to_string(&json_value) {
            Ok(s) => Some(Self::from_json(s.as_ref()).unwrap()),
            _ => None,
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
        let json = self.to_json_string_with_option(&json_opt);
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
        ];
        for (yaml_str, expected) in cases {
            let result = ValueRef::from_yaml(yaml_str);
            assert_eq!(result, Some(expected));
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
                "a:\n  - 1\n  - 2\n  - 3\nb: s\n",
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
                "a: s\nb:\n  - 1\n  - 2\n  - 3\n",
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
