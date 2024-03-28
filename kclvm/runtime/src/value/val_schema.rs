//! Copyright The KCL Authors. All rights reserved.

use indexmap::IndexSet;

use crate::*;

use self::walker::walk_value_mut;

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
pub const CAL_MAP_INDEX_SIGNATURE: &str = "$cal_map_index_signature";

/// Get the schema runtime type use the schema name and pkgpath.
#[inline]
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

pub fn schema_assert(ctx: &mut Context, value: &ValueRef, msg: &str, config_meta: &ValueRef) {
    if !value.is_truthy() {
        ctx.set_err_type(&RuntimeErrorType::SchemaCheckFailure);
        if let Some(config_meta_file) = config_meta.get_by_key(CONFIG_META_FILENAME) {
            let config_meta_line = config_meta.get_by_key(CONFIG_META_LINE).unwrap();
            let config_meta_column = config_meta.get_by_key(CONFIG_META_COLUMN).unwrap();
            ctx.set_kcl_config_meta_location_info(
                Some("Instance check failed"),
                Some(config_meta_file.as_str().as_str()),
                Some(config_meta_line.as_int() as i32),
                Some(config_meta_column.as_int() as i32),
            );
        }

        let arg_msg = format!(
            "Check failed on the condition{}",
            if msg.is_empty() {
                "".to_string()
            } else {
                format!(": {msg}")
            }
        );
        ctx.set_kcl_location_info(Some(arg_msg.as_str()), None, None, None);

        panic!("{}", msg);
    }
}

impl ValueRef {
    pub fn dict_to_schema(
        &self,
        name: &str,
        pkgpath: &str,
        config_keys: &[String],
        config_meta: &ValueRef,
        optional_mapping: &ValueRef,
        args: Option<ValueRef>,
        kwargs: Option<ValueRef>,
    ) -> Self {
        if self.is_dict() {
            Self::from(Value::schema_value(Box::new(SchemaValue {
                name: name.to_string(),
                pkgpath: pkgpath.to_string(),
                config: Box::new(self.as_dict_ref().clone()),
                config_keys: config_keys.to_owned(),
                config_meta: config_meta.clone(),
                optional_mapping: optional_mapping.clone(),
                args: args.unwrap_or(ValueRef::list(None)),
                kwargs: kwargs.unwrap_or(ValueRef::dict(None)),
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

    /// Set of keys not in the schema.
    pub fn keys_not_in_schema(&self, ty: &SchemaType, cal_order: &ValueRef) -> IndexSet<String> {
        let mut keys = IndexSet::new();
        if self.is_config() {
            let config = self.as_dict_ref();
            for (key, _) in &config.values {
                let no_such_attr = ty.attrs.get(key).is_none()
                    && cal_order.dict_get_value(key).is_none()
                    && !key.starts_with('_');
                let has_index_signature = ty.has_index_signature
                    || cal_order.dict_get_value(CAL_MAP_INDEX_SIGNATURE).is_some();
                if !has_index_signature && no_such_attr {
                    keys.insert(key.to_string());
                }
            }
        }
        keys
    }

    /// Check whether the config fits into the schema type.
    #[inline]
    pub fn is_fit_schema(&self, ty: &SchemaType, cal_order: &ValueRef) -> bool {
        self.keys_not_in_schema(ty, cal_order).is_empty()
    }

    /// Check schema optional attributes.
    pub fn schema_check_attr_optional(&self, ctx: &mut Context, recursive: bool) {
        let binding = self.rc.borrow();
        let attr_map = match &*binding {
            Value::schema_value(schema) => &schema.config.values,
            Value::dict_value(schema) => &schema.values,
            _ => panic!("invalid schema or dict value, got {}", self.type_str()),
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
                        // For composite type structures, we recursively check the schema within them.
                        walk_value_mut(value, &mut |value: &ValueRef| {
                            if value.is_schema() {
                                value.schema_check_attr_optional(ctx, true);
                            }
                        })
                    }
                }
            }
            _ => panic!(
                "Invalid optional mapping, got {}",
                optional_mapping.type_str()
            ),
        }
    }

    /// Set the schema instance value with arguments and keyword arguments.
    pub fn set_schema_args(&mut self, args: &ValueRef, kwargs: &ValueRef) {
        if let Value::schema_value(ref mut schema) = &mut *self.rc.borrow_mut() {
            schema.args = args.clone();
            schema.kwargs = kwargs.clone();
        }
    }

    pub fn get_potential_schema_type(&self) -> Option<String> {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => dict.potential_schema.clone(),
            Value::schema_value(ref schema) => schema.config.potential_schema.clone(),
            _ => None,
        }
    }

    pub fn set_potential_schema_type(&mut self, runtime_type: &str) {
        if !runtime_type.is_empty() {
            match &mut *self.rc.borrow_mut() {
                Value::dict_value(ref mut dict) => {
                    dict.potential_schema = Some(runtime_type.to_string())
                }
                Value::schema_value(ref mut schema) => {
                    schema.config.potential_schema = Some(runtime_type.to_string())
                }
                _ => {}
            }
        }
    }

    pub fn has_potential_schema_type(&self) -> bool {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => dict.potential_schema.is_some(),
            Value::schema_value(ref schema) => schema.config.potential_schema.is_some(),
            _ => false,
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
            // Reserve config keys for the schema update process. Issue: #785
            schema.config_keys = value.config_keys.clone();
            schema.config.potential_schema = value.config.potential_schema.clone();
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

    /// Schema additional value check
    pub fn schema_value_check(
        &mut self,
        ctx: &mut Context,
        schema_config: &ValueRef,
        schema_name: &str,
        index_sign_value: &ValueRef,
        index_key_name: &str,
        key_type: &str,
        value_type: &str,
    ) {
        let schema_value = self;
        let has_index_signature = !key_type.is_empty();
        let config = schema_config.as_dict_ref();
        for (key, value) in &config.values {
            let no_such_attr = schema_value.dict_get_value(key).is_none();
            if has_index_signature && no_such_attr {
                // Allow index signature value has different values
                // related to the index signature key name.
                let should_update =
                    if let Some(index_key_value) = schema_value.dict_get_value(index_key_name) {
                        index_key_value.is_str() && key == &index_key_value.as_str()
                    } else {
                        true
                    };
                if should_update {
                    let op = config
                        .ops
                        .get(key)
                        .unwrap_or(&ConfigEntryOperationKind::Union);
                    schema_value.dict_update_entry(
                        key.as_str(),
                        &index_sign_value.deep_copy(),
                        &ConfigEntryOperationKind::Override,
                        &-1,
                    );
                    schema_value.dict_insert(ctx, key.as_str(), value, op.clone(), -1);
                    let value = schema_value.dict_get_value(key).unwrap();
                    schema_value.dict_update_key_value(
                        key.as_str(),
                        type_pack_and_check(ctx, &value, vec![value_type]),
                    );
                }
            } else if !has_index_signature && no_such_attr {
                panic!("No attribute named '{key}' in the schema '{schema_name}'");
            }
        }
    }
}

#[cfg(test)]
mod test_value_schema {
    use crate::*;

    const TEST_SCHEMA_NAME: &str = "Data";

    fn get_test_schema_value() -> ValueRef {
        let mut schema = ValueRef::dict(None).dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
            None,
            None,
        );
        schema.set_potential_schema_type(&schema_runtime_type(TEST_SCHEMA_NAME, MAIN_PKG_PATH));
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
            None,
            None,
        );
        assert!(schema.is_schema());
        let schema = schema.dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &ValueRef::dict(None),
            &ValueRef::dict(None),
            None,
            None,
        );
        assert!(schema.is_schema());
        let dict = schema.schema_to_dict();
        assert!(dict.is_dict());
    }

    #[test]
    fn test_schema_check_attr_optional() {
        let mut ctx = Context::new();
        let dict = ValueRef::dict_str(&[("key", "value")]);
        let config_meta = ValueRef::dict(None);
        let optional_mapping = ValueRef::dict_bool(&[("key", true)]);
        let schema = dict.dict_to_schema(
            TEST_SCHEMA_NAME,
            MAIN_PKG_PATH,
            &[],
            &config_meta,
            &optional_mapping,
            None,
            None,
        );
        schema.schema_check_attr_optional(&mut ctx, true);
        schema.schema_check_attr_optional(&mut ctx, false);
    }

    #[test]
    fn test_schema_check_attr_optional_invalid() {
        let err = std::panic::catch_unwind(|| {
            let mut ctx = Context::new();
            let dict = ValueRef::dict_str(&[("key", "value")]);
            let config_meta = ValueRef::dict(None);
            let optional_mapping = ValueRef::dict_bool(&[("another_key", false)]);
            let schema = dict.dict_to_schema(
                TEST_SCHEMA_NAME,
                MAIN_PKG_PATH,
                &[],
                &config_meta,
                &optional_mapping,
                None,
                None,
            );
            schema.schema_check_attr_optional(&mut ctx, true);
        });
        assert!(err.is_err())
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
