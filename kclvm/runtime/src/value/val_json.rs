//! Copyright The KCL Authors. All rights reserved.

use bstr::ByteSlice;
use indexmap::IndexMap;
use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Serialize,
};

use crate::{ConfigEntryOperationKind, Context, ValueRef, KCL_PRIVATE_VAR_PREFIX};

macro_rules! tri {
    ($e:expr $(,)?) => {
        match $e {
            core::result::Result::Ok(val) => val,
            core::result::Result::Err(err) => return core::result::Result::Err(err),
        }
    };
}

#[derive(Debug, Clone, Default)]
pub struct JsonEncodeOptions {
    pub sort_keys: bool,
    pub indent: i64,
    pub ignore_private: bool,
    pub ignore_none: bool,
}

struct JsonFormatter {
    current_indent: usize,
    has_value: bool,
    indent: String,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum JsonValue {
    Null,

    Bool(bool),

    Number(serde_json::Number),

    String(String),

    Array(Vec<JsonValue>),

    Object(IndexMap<String, JsonValue>),
}

struct MapKeyClass;
impl<'de> DeserializeSeed<'de> for MapKeyClass {
    type Value = String;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for MapKeyClass {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s.to_owned())
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s)
    }
}

impl<'de> Deserialize<'de> for JsonValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<JsonValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = JsonValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Self::Value::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Self::Value::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Self::Value::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(serde_json::Number::from_f64(value)
                    .map_or(Self::Value::Null, Self::Value::Number))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Self::Value::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Self::Value::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Self::Value::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = tri!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(Self::Value::Array(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                match visitor.next_key_seed(MapKeyClass)? {
                    Some(first_key) => {
                        let mut values = IndexMap::new();

                        values.insert(first_key, tri!(visitor.next_value()));
                        while let Some((key, value)) = tri!(visitor.next_entry()) {
                            values.insert(key, value);
                        }

                        Ok(Self::Value::Object(values))
                    }
                    None => Ok(Self::Value::Object(IndexMap::new())),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for JsonValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        match self {
            JsonValue::Null => serializer.serialize_unit(),
            JsonValue::Bool(b) => serializer.serialize_bool(*b),
            JsonValue::Number(n) => n.serialize(serializer),
            JsonValue::String(s) => serializer.serialize_str(s),
            JsonValue::Array(v) => v.serialize(serializer),
            JsonValue::Object(m) => {
                use serde::ser::SerializeMap;
                let mut map = tri!(serializer.serialize_map(Some(m.len())));
                for (k, v) in m {
                    tri!(map.serialize_entry(k, v));
                }
                map.end()
            }
        }
    }
}

impl JsonFormatter {
    /// Construct a pretty printer formatter that defaults to using two spaces for indentation.
    pub fn new() -> Self {
        JsonFormatter::with_indent(0)
    }

    /// Construct a pretty printer formatter that uses the `indent` string for indentation.
    pub fn with_indent(indent: i64) -> Self {
        let indent = if indent < 0 { 0 } else { indent as usize };
        JsonFormatter {
            current_indent: 0,
            has_value: false,
            indent: String::from_utf8(vec![b' '; indent]).unwrap(),
        }
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        JsonFormatter::new()
    }
}

impl serde_json::ser::Formatter for JsonFormatter {
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"[")
    }

    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        self.current_indent -= 1;

        if self.has_value && !self.indent.is_empty() {
            tri!(writer.write_all(b"\n"));
            tri!(indent(writer, self.current_indent, self.indent.as_bytes()));
        }

        writer.write_all(b"]")
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        if !first {
            tri!(writer.write_all(b","));
        }
        if !self.indent.is_empty() {
            tri!(writer.write_all(b"\n"));
        } else if !first {
            tri!(writer.write_all(b" "));
        }
        tri!(indent(writer, self.current_indent, self.indent.as_bytes()));
        Ok(())
    }

    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        self.has_value = true;
        Ok(())
    }

    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"{")
    }

    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        self.current_indent -= 1;

        if self.has_value && !self.indent.is_empty() {
            tri!(writer.write_all(b"\n"));
            tri!(indent(writer, self.current_indent, self.indent.as_bytes()));
        }

        writer.write_all(b"}")
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        if !first {
            tri!(writer.write_all(b","));
        }
        if !self.indent.is_empty() {
            tri!(writer.write_all(b"\n"));
        } else if !first {
            tri!(writer.write_all(b" "));
        }
        indent(writer, self.current_indent, self.indent.as_bytes())
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        writer.write_all(b": ")
    }

    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        self.has_value = true;
        Ok(())
    }
}

fn indent<W>(wr: &mut W, n: usize, s: &[u8]) -> std::io::Result<()>
where
    W: ?Sized + std::io::Write,
{
    for _ in 0..n {
        tri!(wr.write_all(s));
    }

    Ok(())
}

impl ValueRef {
    pub fn from_json(ctx: &mut Context, s: &str) -> Result<Self, serde_json::Error> {
        match serde_json::de::from_str::<JsonValue>(s) {
            Ok(json) => Ok(Self::parse_json(ctx, &json)),
            Err(err) => Err(err),
        }
    }
    pub(crate) fn parse_json(ctx: &mut Context, json: &JsonValue) -> Self {
        match json {
            JsonValue::Object(values) => {
                let mut dict = Self::dict(None);
                for (name, value) in values {
                    let v = Self::parse_json(ctx, value);
                    dict.dict_insert(ctx, name.as_ref(), &v, ConfigEntryOperationKind::Union, -1);
                }
                dict
            }
            JsonValue::Array(values) => {
                let mut list = Self::list(None);
                for value in values {
                    list.list_append(&Self::parse_json(ctx, value));
                }
                list
            }
            JsonValue::String(val) => Self::str((*val).as_ref()),
            JsonValue::Number(val) => {
                if val.is_i64() {
                    Self::int(val.as_i64().unwrap())
                } else if val.is_u64() {
                    let n = val.as_u64().unwrap();
                    if n <= i64::max_value() as u64 {
                        Self::int(n as i64)
                    } else {
                        Self::float(n as f64)
                    }
                } else {
                    Self::float(val.as_f64().unwrap())
                }
            }
            JsonValue::Bool(val) => Self::bool(*val),
            JsonValue::Null => Self::none(),
        }
    }

    pub fn to_json(&self) -> Vec<u8> {
        let json = self.build_json(&Default::default());

        let formatter = JsonFormatter::new();
        let mut writer = Vec::with_capacity(128);
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        json.serialize(&mut serializer).unwrap();
        writer
    }

    pub fn to_json_string(&self) -> String {
        let json = self.build_json(&Default::default());

        let formatter = JsonFormatter::new();
        let mut writer = Vec::with_capacity(128);
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        json.serialize(&mut serializer).unwrap();
        writer.to_str().unwrap().to_string()
    }

    pub fn to_json_string_with_options(&self, opt: &JsonEncodeOptions) -> String {
        let json = self.build_json(opt);
        let formatter = JsonFormatter::with_indent(opt.indent);
        let mut writer = Vec::with_capacity(128);
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        json.serialize(&mut serializer).unwrap();
        writer.to_str().unwrap().to_string()
    }

    pub fn to_json_string_with_null(&self) -> String {
        let json = self.build_json(&Default::default());
        let formatter = JsonFormatter::new();
        let mut writer = Vec::with_capacity(128);
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        json.serialize(&mut serializer).unwrap();
        writer.push(0);
        writer.to_str().unwrap().to_string()
    }

    fn build_json(&self, opt: &JsonEncodeOptions) -> JsonValue {
        match &*self.rc.borrow() {
            crate::Value::undefined => JsonValue::Null,
            crate::Value::none => JsonValue::Null,

            crate::Value::bool_value(ref v) => JsonValue::Bool(*v),
            crate::Value::int_value(ref v) => JsonValue::Number(serde_json::Number::from(*v)),
            crate::Value::float_value(ref v) => match serde_json::Number::from_f64(*v) {
                Some(n) => JsonValue::Number(n),
                None => JsonValue::Null,
            },
            // The number_multiplier is still a number, if we want to get the string form, we can
            // use the `str` function e.g. `str(1Mi)`
            crate::Value::unit_value(ref v, ..) => match serde_json::Number::from_f64(*v) {
                Some(n) => JsonValue::Number(n),
                None => JsonValue::Null,
            },
            crate::Value::str_value(ref v) => JsonValue::String(v.clone()),

            crate::Value::list_value(ref v) => {
                let mut val_array = Vec::new();
                for x in v.values.iter() {
                    match *x.rc.borrow() {
                        crate::Value::undefined => {
                            continue;
                        }
                        crate::Value::none => {
                            if !opt.ignore_none {
                                val_array.push(x.build_json(opt));
                            }
                        }
                        crate::Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            val_array.push(x.build_json(opt));
                        }
                    }
                }
                JsonValue::Array(val_array)
            }
            crate::Value::dict_value(ref v) => {
                let mut val_map = IndexMap::new();
                let mut vals = v.values.clone();
                if opt.sort_keys {
                    vals.sort_keys();
                }
                for (key, val) in vals.iter() {
                    if opt.ignore_private && (*key).starts_with(KCL_PRIVATE_VAR_PREFIX) {
                        continue;
                    }
                    match *val.rc.borrow() {
                        crate::Value::undefined => {
                            continue;
                        }
                        crate::Value::none => {
                            if !opt.ignore_none {
                                val_map.insert(key.clone(), val.build_json(opt));
                            }
                        }
                        crate::Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            val_map.insert(key.clone(), val.build_json(opt));
                        }
                    }
                }
                JsonValue::Object(val_map)
            }

            crate::Value::schema_value(ref v) => {
                let mut val_map = IndexMap::new();
                let mut vals = v.config.values.clone();
                if opt.sort_keys {
                    vals.sort_keys();
                }
                for (key, val) in vals.iter() {
                    if opt.ignore_private && (*key).starts_with(KCL_PRIVATE_VAR_PREFIX) {
                        continue;
                    }
                    match *val.rc.borrow() {
                        crate::Value::undefined => {
                            continue;
                        }
                        crate::Value::none => {
                            if !opt.ignore_none {
                                val_map.insert(key.clone(), val.build_json(opt));
                            }
                        }
                        crate::Value::func_value(_) => {
                            // ignore func
                        }
                        _ => {
                            val_map.insert(key.clone(), val.build_json(opt));
                        }
                    }
                }
                JsonValue::Object(val_map)
            }
            crate::Value::func_value(ref v) => {
                JsonValue::Number(serde_json::Number::from(v.fn_ptr))
            }
        }
    }
}

#[cfg(test)]
mod test_value_json {
    use crate::*;

    #[test]
    fn test_value_from_correct_json() {
        let mut ctx = Context::new();
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
            ("\n{}", ValueRef::dict(Some(&[]))),
        ];
        for (json_str, expected) in cases {
            let result = ValueRef::from_json(&mut ctx, json_str).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_value_from_err_json() {
        let mut ctx = Context::new();
        let cases = [
            ("{", "EOF while parsing an object at line 1 column 1"),
            ("{\"a\": 1,}", "trailing comma at line 1 column 9"),
            ("{\"a\": ]}", "expected value at line 1 column 7"),
            ("[}", "expected value at line 1 column 2"),
        ];
        for (json_str, expected) in cases {
            let result = ValueRef::from_json(&mut ctx, json_str);
            assert_eq!(result.err().unwrap().to_string(), expected);
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
