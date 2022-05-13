// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;
use json_minimal::*;

#[derive(Debug, Default)]
pub struct JsonEncodeOptions {
    pub sort_keys: bool,
    pub indent: i64,
    pub ignore_private: bool,
    pub ignore_none: bool,
}

impl ValueRef {
    pub fn from_json(s: &str) -> Option<Self> {
        match Json::parse_with_option(
            s.as_bytes(),
            &json_minimal::ParseOption { support_int: true },
        ) {
            Ok(json) => Some(Self::parse_json(&json)),
            _ => None,
        }
    }
    fn parse_json(json: &Json) -> Self {
        match json {
            Json::OBJECT { name, value } => {
                let _ = name;
                let _ = value;
                panic!("unreachable");
            }
            Json::JSON(values) => {
                let mut dict = Self::dict(None);
                for value in values {
                    match value {
                        Json::OBJECT { name, value } => dict.dict_insert(
                            name.as_ref(),
                            &Self::parse_json(value),
                            ConfigEntryOperationKind::Union,
                            0,
                        ),
                        _ => panic!("unreachable"),
                    }
                }
                dict
            }
            Json::ARRAY(values) => {
                let mut list = Self::list(None);
                for value in values {
                    list.list_append(&Self::parse_json(value));
                }
                list
            }
            Json::STRING(val) => Self::str((*val).as_ref()),
            Json::NUMBER(val) => Self::float(*val),
            Json::INT(val) => Self::int(*val),
            Json::FLOAT(val) => Self::float(*val),
            Json::BOOL(val) => Self::bool(*val),
            Json::NULL => Self::none(),
        }
    }

    pub fn to_json(&self) -> Vec<u8> {
        let json = self.build_json(&Default::default());

        let opt = json_minimal::PrintOption {
            sep_space: true,
            py_style_f64: true,
            ..Default::default()
        };

        json.print_with_option(&opt).into_bytes()
    }

    pub fn to_json_string(&self) -> String {
        let json = self.build_json(&Default::default());

        let opt = json_minimal::PrintOption {
            sep_space: true,
            py_style_f64: true,
            ..Default::default()
        };

        json.print_with_option(&opt)
    }

    pub fn to_json_string_with_option(&self, opt: &JsonEncodeOptions) -> String {
        let json = self.build_json(opt);

        let opt = json_minimal::PrintOption {
            sort_keys: opt.sort_keys,
            indent: opt.indent as i32,
            sep_space: true,
            py_style_f64: true,
            ..Default::default()
        };

        json.print_with_option(&opt)
    }

    pub fn to_json_string_with_null(&self) -> String {
        let json = self.build_json(&Default::default());

        let opt = json_minimal::PrintOption {
            sep_space: true,
            py_style_f64: true,
            append_null: true,
            ..Default::default()
        };

        json.print_with_option(&opt)
    }

    fn build_json(&self, opt: &JsonEncodeOptions) -> Json {
        match &*self.rc {
            Value::undefined => Json::NULL,
            Value::none => Json::NULL,

            Value::bool_value(ref v) => Json::BOOL(*v),
            Value::int_value(ref v) => Json::INT(*v),
            Value::float_value(ref v) => Json::FLOAT(*v),
            Value::unit_value(..) => Json::STRING(self.to_string()),
            Value::str_value(ref v) => Json::STRING(v.clone()),

            Value::list_value(ref v) => {
                let mut list = Json::ARRAY(Vec::new());
                for x in v.values.iter() {
                    match *x.rc {
                        Value::undefined => {
                            continue;
                        }
                        Value::none => {
                            if !opt.ignore_none {
                                list.add(x.build_json(opt));
                            }
                        }
                        Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            list.add(x.build_json(opt));
                        }
                    }
                }
                list
            }
            Value::dict_value(ref v) => {
                let mut json = Json::new();
                for (key, val) in v.values.iter() {
                    if opt.ignore_private && (*key).starts_with(KCL_PRIVATE_VAR_PREFIX) {
                        continue;
                    }
                    match *val.rc {
                        Value::undefined => {
                            continue;
                        }
                        Value::none => {
                            if !opt.ignore_none {
                                json.add(Json::OBJECT {
                                    name: key.clone(),
                                    value: Box::new(val.build_json(opt)),
                                });
                            }
                        }
                        Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            json.add(Json::OBJECT {
                                name: key.clone(),
                                value: Box::new(val.build_json(opt)),
                            });
                        }
                    }
                }
                json
            }

            Value::schema_value(ref v) => {
                let mut json = Json::new();
                for (key, val) in v.config.values.iter() {
                    if opt.ignore_private && (*key).starts_with(KCL_PRIVATE_VAR_PREFIX) {
                        continue;
                    }
                    match *val.rc {
                        Value::undefined => {
                            continue;
                        }
                        Value::none => {
                            if !opt.ignore_none {
                                json.add(Json::OBJECT {
                                    name: key.clone(),
                                    value: Box::new(val.build_json(opt)),
                                });
                            }
                        }
                        Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            json.add(Json::OBJECT {
                                name: key.clone(),
                                value: Box::new(val.build_json(opt)),
                            });
                        }
                    }
                }
                json
            }
            Value::func_value(ref v) => Json::NUMBER(v.fn_ptr as f64),
        }
    }
}

#[cfg(test)]
mod test_value_json {
    use crate::*;

    #[test]
    fn test_value_from_json() {
        let cases = [
            (
                "{\"a\": 1}\n",
                ValueRef::dict(Some(&[("a", &ValueRef::int(1))])),
            ),
            (
                "{\"a\": 1,\n\"b\": 2}\n",
                ValueRef::dict(Some(&[("a", &ValueRef::int(1)), ("b", &ValueRef::int(2))])),
            ),
            (
                "{\"a\": [1, 2, 3],\n\"b\": \"s\"}\n",
                ValueRef::dict(Some(&[
                    ("a", &ValueRef::list_int(&[1, 2, 3])),
                    ("b", &ValueRef::str("s")),
                ])),
            ),
        ];
        for (json_str, expected) in cases {
            let result = ValueRef::from_json(json_str);
            assert_eq!(result, Some(expected));
        }
    }

    #[test]
    fn test_value_to_json_string() {
        let cases = [
            (
                ValueRef::dict(Some(&[("a", &ValueRef::int(1))])),
                "{\"a\": 1}",
            ),
            (
                ValueRef::dict(Some(&[("a", &ValueRef::int(1)), ("b", &ValueRef::int(2))])),
                "{\"a\": 1, \"b\": 2}",
            ),
            (
                ValueRef::dict(Some(&[
                    ("a", &ValueRef::list_int(&[1, 2, 3])),
                    ("b", &ValueRef::str("s")),
                ])),
                "{\"a\": [1, 2, 3], \"b\": \"s\"}",
            ),
        ];
        for (value, expected) in cases {
            let result = ValueRef::to_json_string(&value);
            assert_eq!(result, expected);
        }
    }
}
