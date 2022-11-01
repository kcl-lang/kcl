// Copyright 2021 The KCL Authors. All rights reserved.

extern crate serde_json;
extern crate serde_yaml;

use crate::*;

#[derive(Debug, Default)]
pub struct YamlEncodeOptions {
    pub sort_keys: bool,
    pub ignore_private: bool,
    pub ignore_none: bool,
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
        let x = self.yaml_clone_with_filter(opt);
        x.to_yaml_string()
    }

    fn yaml_clone_with_filter(&self, opt: &YamlEncodeOptions) -> Self {
        match &*self.rc.borrow() {
            Value::undefined => ValueRef::undefined(),
            Value::none => ValueRef::none(),

            Value::bool_value(ref v) => ValueRef::bool(*v),
            Value::int_value(ref v) => ValueRef::int(*v),
            Value::float_value(ref v) => ValueRef::float(*v),
            Value::str_value(ref v) => ValueRef::str(v.as_ref()),
            Value::unit_value(ref v, ref raw, ref unit) => ValueRef::unit(*v, *raw, unit),
            Value::list_value(ref v) => {
                let mut list = ValueRef::list(None);
                for x in v.values.iter() {
                    match *x.rc.borrow() {
                        Value::undefined => {
                            continue;
                        }
                        Value::none => {
                            if !opt.ignore_none {
                                list.list_append(&x.yaml_clone_with_filter(opt));
                            }
                        }
                        Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            list.list_append(&x.yaml_clone_with_filter(opt));
                        }
                    }
                }
                list
            }
            Value::dict_value(ref v) => {
                let mut dict = ValueRef::dict(None);
                for (key, val) in v.values.iter() {
                    if opt.ignore_private && (*key).starts_with(KCL_PRIVATE_VAR_PREFIX) {
                        continue;
                    }
                    match *val.rc.borrow() {
                        Value::undefined => {
                            continue;
                        }
                        Value::none => {
                            if !opt.ignore_none {
                                dict.dict_insert(
                                    key,
                                    &val.yaml_clone_with_filter(opt),
                                    Default::default(),
                                    0,
                                );
                            }
                        }
                        Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            dict.dict_insert(
                                key,
                                &val.yaml_clone_with_filter(opt),
                                Default::default(),
                                0,
                            );
                        }
                    }
                }
                dict
            }

            Value::schema_value(ref v) => {
                let mut dict = ValueRef::dict(None);
                for (key, val) in v.config.values.iter() {
                    if opt.ignore_private && (*key).starts_with(KCL_PRIVATE_VAR_PREFIX) {
                        continue;
                    }
                    match *val.rc.borrow() {
                        Value::undefined => {
                            continue;
                        }
                        Value::none => {
                            if !opt.ignore_none {
                                dict.dict_insert(
                                    key,
                                    &val.yaml_clone_with_filter(opt),
                                    Default::default(),
                                    0,
                                );
                            }
                        }
                        Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            dict.dict_insert(
                                key,
                                &val.yaml_clone_with_filter(opt),
                                Default::default(),
                                0,
                            );
                        }
                    }
                }
                dict
            }
            Value::func_value(_) => ValueRef::undefined(),
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
}
