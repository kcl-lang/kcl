// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

pub const SETTINGS_OUTPUT_KEY: &str = "output_type";
pub const SETTINGS_SCHEMA_TYPE_KEY: &str = "__schema_type__";
pub const SETTINGS_OUTPUT_STANDALONE: &str = "STANDALONE";
pub const SETTINGS_OUTPUT_INLINE: &str = "INLINE";
pub const SETTINGS_OUTPUT_IGNORE: &str = "IGNORE";
pub const SCHEMA_SETTINGS_ATTR_NAME: &str = "__settings__";
pub const CONFIG_META_FILENAME: &str = "$filename";
pub const CONFIG_META_LINE: &str = "$lineno";
pub const CONFIG_META_COLUMN: &str = "$columnno";
pub const CONFIG_ITEM_META_FILENAME: &str = "filename";
pub const CONFIG_ITEM_META_LINE: &str = "lineno";
pub const CONFIG_ITEM_META_COLUMN: &str = "columnno";
pub const CONFIG_ITEM_META: &str = "$config_meta";
pub const MAIN_PKG_PATH: &str = "__main__";
pub const PKG_PATH_PREFIX: char = '@';
pub const CAL_MAP_RUNTIME_TYPE: &str = "cal_map_runtime_type";
pub const CAL_MAP_META_LINE: &str = "cal_map_meta_line";

/// Get the schema runtime type use the schema name and pkgpath
pub fn schema_runtime_type(name: &str, pkgpath: &str) -> String {
    format!("{pkgpath}.{name}")
}

/// Construct a schema config meta dict using filename, line and column
#[inline]
pub fn schema_config_meta(filename: &str, line: u64, column: u64) -> ValueRef {
    ValueRef::dict(Some(&[
        (CONFIG_META_FILENAME, &ValueRef::str(filename)),
        (CONFIG_META_LINE, &ValueRef::int(line as i64)),
        (CONFIG_META_COLUMN, &ValueRef::int(column as i64)),
    ]))
}

impl ValueRef {
    pub fn dict_to_schema(
        &self,
        name: &str,
        pkgpath: &str,
        config_keys: &[String],
        config_meta: &ValueRef,
        optional_mapping: &ValueRef,
    ) -> Self {
        if self.is_dict() {
            Self::from(Value::schema_value(Box::new(SchemaValue {
                name: name.to_string(),
                pkgpath: pkgpath.to_string(),
                config: Box::new(self.as_dict_ref().clone()),
                config_keys: config_keys.to_owned(),
                config_meta: config_meta.clone(),
                optional_mapping: optional_mapping.clone(),
            })))
        } else if self.is_schema() {
            self.clone()
        } else {
            panic!("invalid dict object to schema")
        }
    }

    pub fn schema_to_dict(&self) -> Self {
        match &*self.rc.borrow() {
            Value::schema_value(ref schema) => {
                Self::from(Value::dict_value(Box::new(schema.config.as_ref().clone())))
            }
            Value::dict_value(_) => self.clone(),
            _ => panic!("invalid schema object to dict"),
        }
    }

    /// Get the schema attribute optional mapping.
    #[inline]
    pub fn schema_name(&self) -> String {
        if let Value::schema_value(schema) = &*self.rc.borrow() {
            schema.name.clone()
        } else {
            "".to_string()
        }
    }

    /// Get the schema name
    #[inline]
    pub fn schema_optional_mapping(&self) -> ValueRef {
        if let Value::schema_value(schema) = &*self.rc.borrow() {
            schema.optional_mapping.clone()
        } else {
            ValueRef::dict(None)
        }
    }

    /// Get the schema config meta information including filename, line and column.
    #[inline]
    pub fn schema_config_meta(&self) -> ValueRef {
        if let Value::schema_value(schema) = &*self.rc.borrow() {
            schema.config_meta.clone()
        } else {
            ValueRef::dict(None)
        }
    }

    /// Check schema optional attributes.
    pub fn schema_check_attr_optional(&self, recursive: bool) {
        let binding = self.rc.borrow();
        let attr_map = match &*binding {
            Value::schema_value(schema) => &schema.config.values,
            Value::dict_value(schema) => &schema.values,
            _ => panic!("Invalid schema/dict value, got {}", self.type_str()),
        };
        let optional_mapping = self.schema_optional_mapping();
        let optional_mapping_ref = optional_mapping.rc.borrow();
        let config_meta = self.schema_config_meta();
        match &*optional_mapping_ref {
            Value::dict_value(optional_mapping) => {
                for (attr, is_optional) in &optional_mapping.values {
                    let is_required = !is_optional.as_bool();
                    let undefined = ValueRef::undefined();
                    let value = attr_map.get(attr).unwrap_or(&undefined);
                    if is_required && value.is_none_or_undefined() {
                        let filename = config_meta.get_by_key(CONFIG_META_FILENAME);
                        let line = config_meta.get_by_key(CONFIG_META_LINE);
                        let ctx = Context::current_context_mut();
                        if let Some(filename) = filename {
                            ctx.set_kcl_filename(&filename.as_str());
                        }
                        if let Some(line) = line {
                            ctx.panic_info.kcl_line = line.as_int() as i32;
                        }
                        panic!(
                            "attribute '{}' of {} is required and can't be None or Undefined",
                            attr,
                            self.schema_name()
                        );
                    }
                }
                // Recursive check schema values for every attributes.
                if recursive {
                    for value in attr_map.values() {
                        if value.is_schema() {
                            value.schema_check_attr_optional(recursive);
                        }
                    }
                }
            }
            _ => panic!(
                "Invalid optional mapping, got {}",
                optional_mapping.type_str()
            ),
        }
    }

    pub fn schema_default_settings(&mut self, config: &ValueRef, runtime_type: &str) {
        let settings = self.dict_get_value(SCHEMA_SETTINGS_ATTR_NAME);
        if settings.is_none() || (settings.is_some() && !settings.as_ref().unwrap().is_config()) {
            let mut default_settings = ValueRef::dict(None);
            default_settings
                .dict_update_key_value(SETTINGS_OUTPUT_KEY, ValueRef::str(SETTINGS_OUTPUT_INLINE));
            default_settings
                .dict_update_key_value(SETTINGS_SCHEMA_TYPE_KEY, ValueRef::str(runtime_type));
            self.dict_update_key_value(SCHEMA_SETTINGS_ATTR_NAME, default_settings);
        } else {
            settings
                .unwrap()
                .dict_update_key_value(SETTINGS_SCHEMA_TYPE_KEY, ValueRef::str(runtime_type));
        }
        if let Some(v) = config.dict_get_value(SCHEMA_SETTINGS_ATTR_NAME) {
            self.dict_update_key_value(SCHEMA_SETTINGS_ATTR_NAME, v);
        }
    }

    pub fn attr_str(&self) -> String {
        match &*self.rc.borrow() {
            Value::int_value(v) => v.to_string(),
            Value::float_value(v) => v.to_string(),
            Value::str_value(v) => v.clone(),
            _ => panic!("invalid attribute {}", self.type_str()),
        }
    }

    pub fn update_attr_map(&mut self, name: &str, type_str: &str) {
        match &mut *self.rc.borrow_mut() {
            Value::dict_value(dict) => {
                dict.attr_map.insert(name.to_string(), type_str.to_string());
            }
            Value::schema_value(schema) => {
                schema
                    .config
                    .attr_map
                    .insert(name.to_string(), type_str.to_string());
            }
            _ => panic!("invalid object '{}' in update_attr_map", self.type_str()),
        }
    }

    pub fn attr_map_get(&mut self, name: &str) -> Option<String> {
        match &*self.rc.borrow() {
            Value::dict_value(dict) => dict.attr_map.get(name).cloned(),
            Value::schema_value(schema) => schema.config.attr_map.get(name).cloned(),
            _ => panic!("invalid object '{}' in attr_map_get", self.type_str()),
        }
    }

    pub fn schema_update_with_schema(&mut self, value: &ValueRef) {
        if let (Value::schema_value(schema), Value::schema_value(value)) =
            (&mut *self.rc.borrow_mut(), &*value.rc.borrow())
        {
            let values = &mut schema.config.values;
            let ops = &mut schema.config.ops;
            let insert_indexs = &mut schema.config.insert_indexs;
            for (k, v) in &value.config.values {
                let op = value
                    .config
                    .ops
                    .get(k)
                    .unwrap_or(&ConfigEntryOperationKind::Union);
                let index = value.config.insert_indexs.get(k).unwrap_or(&-1);
                values.insert(k.clone(), v.clone());
                ops.insert(k.clone(), op.clone());
                insert_indexs.insert(k.clone(), *index);
            }
        }
    }
}

#[cfg(test)]
mod test_value_schema {
    use crate::*;

    const TEST_SCHEMA_NAME: &str = "Data";

    fn get_test_schema_value() -> ValueRef {
        let config = ValueRef::dict(None);
        let mut schema = ValueRef::dict(None).dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
        );
        schema.schema_default_settings(
            &config,
            &schema_runtime_type(TEST_SCHEMA_NAME, MAIN_PKG_PATH),
        );
        schema
    }

    #[test]
    fn test_dict_schema_convention() {
        let dict = ValueRef::dict(None);
        let dict = dict.schema_to_dict();
        assert!(dict.is_dict());
        let schema = dict.dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
        );
        assert!(schema.is_schema());
        let schema = schema.dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
        );
        assert!(schema.is_schema());
        let dict = schema.schema_to_dict();
        assert!(dict.is_dict());
    }

    #[test]
    fn test_schema_check_attr_optional() {
        let dict = ValueRef::dict_str(&[("key", "value")]);
        let config_meta = ValueRef::dict(None);
        let optional_mapping = ValueRef::dict_bool(&[("key", true)]);
        let schema = dict.dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &config_meta,
            &optional_mapping,
        );
        schema.schema_check_attr_optional(true);
        schema.schema_check_attr_optional(false);
    }

    #[test]
    fn test_schema_check_attr_optional_invalid() {
        let err = std::panic::catch_unwind(|| {
            let dict = ValueRef::dict_str(&[("key", "value")]);
            let config_meta = ValueRef::dict(None);
            let optional_mapping = ValueRef::dict_bool(&[("another_key", false)]);
            let schema = dict.dict_to_schema(
                TEST_SCHEMA_NAME,
                MAIN_PKG_PATH,
                &[],
                &config_meta,
                &optional_mapping,
            );
            schema.schema_check_attr_optional(true);
        });
        assert!(err.is_err())
    }

    #[test]
    fn test_schema_default_settings() {
        let schema = get_test_schema_value();
        let schema_settings = schema.get_by_key(SCHEMA_SETTINGS_ATTR_NAME).unwrap();
        let output_type = schema_settings
            .get_by_key(SETTINGS_OUTPUT_KEY)
            .unwrap()
            .as_str();
        assert_eq!(output_type, SETTINGS_OUTPUT_INLINE);
    }

    #[test]
    fn test_schema_attr_map() {
        let mut schema = get_test_schema_value();
        let entries = [("key1", "str"), ("key2", "int"), ("key3", "str|int")];
        for (attr, type_str) in entries {
            schema.update_attr_map(attr, type_str);
        }
        for (attr, type_str) in entries {
            let result = schema.attr_map_get(attr).unwrap().clone();
            assert_eq!(result, type_str);
        }
    }

    #[test]
    fn test_schema_update_with_schema() {
        let mut schema1 = get_test_schema_value();
        let mut schema2 = get_test_schema_value();
        let entries = [("key1", "value1"), ("key2", "value2")];
        for (key, val) in entries {
            schema2.dict_update_entry(
                key,
                &ValueRef::str(val),
                &ConfigEntryOperationKind::Union,
                &-1,
            );
        }
        assert_ne!(schema1, schema2);
        schema1.schema_update_with_schema(&schema2);
        assert_eq!(schema1, schema2);
    }
}
