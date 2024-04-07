//! Copyright The KCL Authors. All rights reserved.

use crate::*;
use std::cell::Ref;
impl DictValue {
    pub fn new(values: &[(&str, &ValueRef)]) -> DictValue {
        let mut dict = DictValue::default();
        for x in values {
            dict.values.insert(x.0.to_string(), x.1.clone());
        }
        dict
    }

    pub fn get(&self, key: &ValueRef) -> Option<ValueRef> {
        match &*key.rc.borrow() {
            Value::str_value(ref s) => self.values.get(s).cloned(),
            _ => None,
        }
    }

    pub fn insert(&mut self, key: &ValueRef, value: &ValueRef) {
        if let Value::str_value(ref s) = &*key.rc.borrow() {
            self.values.insert(s.to_string(), value.clone());
        }
    }

    pub fn insert_unpack(&mut self, v: &ValueRef) {
        if let Value::dict_value(ref b) = &*v.rc.borrow() {
            for (k, v) in b.values.iter() {
                self.values.insert(k.clone(), v.clone());
            }
        }
    }
}

impl ValueRef {
    fn dict_config(&self) -> Ref<DictValue> {
        Ref::map(self.rc.borrow(), |val| match val {
            Value::dict_value(ref dict) => dict.as_ref(),
            Value::schema_value(ref schema) => schema.config.as_ref(),
            _ => panic!("invalid dict config value type {}", self.type_str()),
        })
    }
    pub fn dict_int(values: &[(&str, i64)]) -> Self {
        let mut dict = DictValue::default();
        for x in values {
            dict.values.insert(x.0.to_string(), Self::int(x.1));
        }
        Self::from(Value::dict_value(Box::new(dict)))
    }

    pub fn dict_float(values: &[(&str, f64)]) -> Self {
        let mut dict = DictValue::default();
        for x in values {
            dict.values.insert(x.0.to_string(), Self::float(x.1));
        }
        Self::from(Value::dict_value(Box::new(dict)))
    }

    pub fn dict_bool(values: &[(&str, bool)]) -> Self {
        let mut dict = DictValue::default();
        for x in values {
            dict.values.insert(x.0.to_string(), Self::bool(x.1));
        }
        Self::from(Value::dict_value(Box::new(dict)))
    }

    pub fn dict_str(values: &[(&str, &str)]) -> Self {
        let mut dict = DictValue::default();
        for x in values {
            dict.values.insert(x.0.to_string(), Self::str(x.1));
        }
        Self::from(Value::dict_value(Box::new(dict)))
    }

    /// Dict clear
    pub fn dict_clear(&mut self) {
        let mut binding = self.rc.borrow_mut();
        let dict = match &mut *binding {
            Value::dict_value(dict) => dict.as_mut(),
            Value::schema_value(schema) => schema.config.as_mut(),
            _ => panic!("invalid config value in dict_clear"),
        };
        dict.values.clear()
    }

    /// Dict get keys.
    pub fn dict_keys(&self) -> ValueRef {
        let dict = self.dict_config();
        let keys: Vec<String> = dict.values.keys().cloned().collect();
        ValueRef::list_str(&keys)
    }

    /// Dict get values
    pub fn dict_values(&self) -> ValueRef {
        let dict = self.dict_config();
        let values: Vec<&ValueRef> = dict.values.values().collect();
        ValueRef::list(Some(&values))
    }

    /// Dict get e.g., {k1: v1, k2, v2}.get(ValueRef::str(k1)) == v1
    pub fn dict_get(&self, key: &ValueRef) -> Option<ValueRef> {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => dict.get(key),
            Value::schema_value(ref schema) => schema.config.get(key),
            _ => panic!("invalid config value in dict_get"),
        }
    }

    /// Dict get value e.g., {k1: v1, k2, v2}.get_value(k1) == v1
    pub fn dict_get_value(&self, key: &str) -> Option<ValueRef> {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => dict.values.get(key).cloned(),
            Value::schema_value(ref schema) => schema.config.values.get(key).cloned(),
            _ => None,
        }
    }

    /// Dict get value e.g., {k1 = v1, k2 = v2}.get_attr_operator(k1) == Some(ConfigEntryOperationKind::Override)
    pub fn dict_get_attr_operator(&self, key: &str) -> Option<ConfigEntryOperationKind> {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => dict.ops.get(key).cloned(),
            Value::schema_value(ref schema) => schema.config.ops.get(key).cloned(),
            _ => None,
        }
    }

    /// Dict get value e.g., {k1 = v1, k2 = v2}.get_attr_operator(k1) == Some(ConfigEntryOperationKind::Override)
    pub fn dict_get_insert_index(&self, key: &str) -> Option<i32> {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => Some(*dict.insert_indexs.get(key).unwrap_or(&-1)),
            Value::schema_value(ref schema) => {
                Some(*schema.config.insert_indexs.get(key).unwrap_or(&-1))
            }
            _ => None,
        }
    }

    /// Dict get entry e.g., {k1: v1, k2, v2}.get_entry(k1) == {k1: v1}
    pub fn dict_get_entry(&self, key: &str) -> Option<ValueRef> {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => {
                if dict.values.contains_key(key) {
                    let mut d = ValueRef::dict(None);
                    let value = dict.values.get(key).unwrap();
                    let op = if let Some(op) = dict.ops.get(key) {
                        op
                    } else {
                        &ConfigEntryOperationKind::Union
                    };
                    let index = if let Some(idx) = dict.insert_indexs.get(key) {
                        *idx
                    } else {
                        -1
                    };
                    d.dict_update_entry(key, value, op, &index);
                    d.set_potential_schema_type(&dict.potential_schema.clone().unwrap_or_default());
                    Some(d)
                } else {
                    None
                }
            }
            Value::schema_value(ref schema) => {
                if schema.config.values.contains_key(key) {
                    let mut d = ValueRef::dict(None);
                    let value = schema.config.values.get(key).unwrap();
                    let op = if let Some(op) = schema.config.ops.get(key) {
                        op
                    } else {
                        &ConfigEntryOperationKind::Union
                    };
                    let index = if let Some(idx) = schema.config.insert_indexs.get(key) {
                        *idx
                    } else {
                        -1
                    };
                    d.dict_update_entry(key, value, op, &index);
                    d.set_potential_schema_type(
                        &schema.config.potential_schema.clone().unwrap_or_default(),
                    );
                    Some(d)
                } else {
                    None
                }
            }
            // Panic
            _ => panic!("invalid config value in dict_get_entry"),
        }
    }

    /// Dict get entries e.g., {k1: v1, k2, v2}.get_entries([k1, k2]) == {k1: v1, k1: v2}
    pub fn dict_get_entries(&self, keys: Vec<&str>) -> ValueRef {
        match &*self.rc.borrow() {
            Value::dict_value(ref dict) => {
                let mut d = ValueRef::dict(None);
                for key in keys {
                    if dict.values.contains_key(key) {
                        let value = dict.values.get(key).unwrap();
                        let op = dict
                            .ops
                            .get(key)
                            .unwrap_or(&ConfigEntryOperationKind::Union);
                        let index = dict.insert_indexs.get(key).unwrap_or(&-1);
                        d.dict_update_entry(key, value, op, index);
                    }
                }
                d.set_potential_schema_type(&dict.potential_schema.clone().unwrap_or_default());
                d
            }
            Value::schema_value(ref schema) => {
                let mut d = ValueRef::dict(None);
                for key in keys {
                    if schema.config.values.contains_key(key) {
                        let value = schema.config.values.get(key).unwrap();
                        let op = schema
                            .config
                            .ops
                            .get(key)
                            .unwrap_or(&ConfigEntryOperationKind::Union);
                        let index = schema.config.insert_indexs.get(key).unwrap_or(&-1);
                        d.dict_update_entry(key, value, op, index);
                    }
                }
                d.set_potential_schema_type(
                    &schema
                        .config
                        .potential_schema
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                );
                d
            }
            // Panic
            _ => panic!("invalid config value in dict_get_entries"),
        }
    }

    /// Update dict value without attribute operator check, only update
    pub fn dict_update(&mut self, v: &ValueRef) {
        let mut binding = self.rc.borrow_mut();
        let dict = match &mut *binding {
            Value::dict_value(v) => v.as_mut(),
            Value::schema_value(v) => v.config.as_mut(),
            _ => panic!("invalid dict update value: {}", self.type_str()),
        };
        if v.is_config() {
            let v = v.as_dict_ref();
            dict.potential_schema = v.potential_schema.clone();
            for (k, v) in v.values.iter() {
                dict.values.insert(k.clone(), v.clone());
            }
        }
    }

    /// Update key value pair without attribute operator check, only update
    pub fn dict_update_key_value(&mut self, key: &str, val: ValueRef) {
        match &mut *self.rc.borrow_mut() {
            Value::dict_value(dict) => {
                dict.values.insert(key.to_string(), val);
            }
            Value::schema_value(schema) => {
                schema.config.values.insert(key.to_string(), val);
            }
            _ => panic!(
                "failed to update the dict. An iterable of key-value pairs was expected, but got {}. Check if the syntax for updating the dictionary with the attribute '{}' is correct",
                self.type_str(),
                key
            ),
        }
    }

    /// Update entry without attribute operator check, only update
    pub fn dict_update_entry(
        &mut self,
        key: &str,
        val: &ValueRef,
        op: &ConfigEntryOperationKind,
        index: &i32,
    ) {
        let mut binding = self.rc.borrow_mut();
        let dict = match &mut *binding {
            Value::dict_value(v) => v.as_mut(),
            Value::schema_value(v) => v.config.as_mut(),
            _ => panic!("invalid dict update value: {}", self.type_str()),
        };
        dict.values.insert(key.to_string(), val.clone());
        dict.ops.insert(key.to_string(), op.clone());
        dict.insert_indexs.insert(key.to_string(), *index);
    }

    /// Insert key value pair with the idempotent check.
    #[inline]
    pub fn dict_insert(
        &mut self,
        ctx: &mut Context,
        key: &str,
        v: &ValueRef,
        op: ConfigEntryOperationKind,
        insert_index: i32,
    ) {
        self.dict_merge_key_value_pair(ctx, key, v, op, insert_index, true);
    }

    /// Merge key value pair without the idempotent check
    pub fn dict_merge(
        &mut self,
        ctx: &mut Context,
        key: &str,
        v: &ValueRef,
        op: ConfigEntryOperationKind,
        insert_index: i32,
    ) {
        self.dict_merge_key_value_pair(ctx, key, v, op, insert_index, false);
    }

    /// Private dict merge key value pair with the idempotent check option
    fn dict_merge_key_value_pair(
        &mut self,
        ctx: &mut Context,
        key: &str,
        v: &ValueRef,
        op: ConfigEntryOperationKind,
        insert_index: i32,
        idempotent_check: bool,
    ) {
        if ctx.cfg.debug_mode {
            if let Value::int_value(ref x) = *v.rc.borrow() {
                let strict_range_check_i32 = ctx.cfg.strict_range_check;
                let strict_range_check_i64 = ctx.cfg.debug_mode || !ctx.cfg.strict_range_check;
                let v_i128 = *x as i128;

                if strict_range_check_i32 {
                    if v_i128 != ((v_i128 as i32) as i128) {
                        ctx.set_err_type(&RuntimeErrorType::IntOverflow);

                        panic!("{v_i128}: A 32 bit integer overflow");
                    }
                } else if strict_range_check_i64 && v_i128 != ((v_i128 as i64) as i128) {
                    ctx.set_err_type(&RuntimeErrorType::IntOverflow);

                    panic!("{v_i128}: A 64 bit integer overflow");
                }
            }
        }

        if self.is_config() {
            let mut dict: DictValue = Default::default();
            dict.values.insert(key.to_string(), v.clone());
            dict.ops.insert(key.to_string(), op);
            dict.insert_indexs.insert(key.to_string(), insert_index);
            self.union_entry(
                ctx,
                &ValueRef::from(Value::dict_value(Box::new(dict))),
                true,
                &UnionOptions {
                    config_resolve: false,
                    idempotent_check,
                    ..Default::default()
                },
            );
        } else {
            panic!("invalid dict insert value: {}", self.type_str())
        }
    }

    /// Dict insert unpack value e.g., data = {**v}
    pub fn dict_insert_unpack(&mut self, ctx: &mut Context, v: &ValueRef) {
        let mut union = false;
        match (&*self.rc.borrow(), &*v.rc.borrow()) {
            (
                Value::dict_value(_) | Value::schema_value(_),
                Value::dict_value(_) | Value::schema_value(_),
            ) => {
                union = true;
            }
            (Value::dict_value(_) | Value::schema_value(_), Value::none) => { /*Do nothing on unpacking None/Undefined*/
            }
            (Value::dict_value(_) | Value::schema_value(_), Value::undefined) => { /*Do nothing on unpacking None/Undefined*/
            }
            _ => panic!("only list, dict and schema object can be used with unpack operators * and **, got {v}"),
        }
        if union {
            self.bin_aug_bit_or(ctx, &v.schema_to_dict().deep_copy());
        }
    }

    /// Dict remove the key-value pair equivalent to key
    pub fn dict_remove(&mut self, key: &str) {
        match &mut *self.rc.borrow_mut() {
            Value::dict_value(dict) => {
                dict.values.remove(key);
            }
            Value::schema_value(schema) => {
                schema.config.values.remove(key);
            }
            _ => panic!("invalid dict remove value: {}", self.type_str()),
        }
    }

    /// Set dict key with the value. When the dict is a schema and resolve schema validations.
    pub fn dict_set_value(&mut self, ctx: &mut Context, key: &str, val: &ValueRef) {
        let p = self;
        if p.is_config() {
            p.dict_update_key_value(key, val.clone());
            if p.is_schema() {
                let schema: ValueRef;
                {
                    let schema_value = p.as_schema();
                    let mut config_keys = schema_value.config_keys.clone();
                    config_keys.push(key.to_string());
                    schema = resolve_schema(ctx, p, &config_keys);
                }
                p.schema_update_with_schema(&schema);
            }
        } else {
            panic!(
                "failed to update the dict. An iterable of key-value pairs was expected, but got {}. Check if the syntax for updating the dictionary with the attribute '{}' is correct",
                p.type_str(),
                key
            );
        }
    }
}

#[cfg(test)]
mod test_value_dict {

    use crate::*;

    #[test]
    fn test_dict_get() {
        let entries = [("key1", 1), ("key2", 2)];
        let test_dict = ValueRef::dict_int(&entries);
        for (key, val) in entries {
            assert_eq!(
                test_dict.dict_get(&ValueRef::str(key)).unwrap().clone(),
                ValueRef::int(val)
            );
            assert_eq!(test_dict.dict_get_value(key).unwrap().clone().as_int(), val);
            assert_eq!(
                test_dict.dict_get_entry(key).unwrap().clone(),
                ValueRef::dict_int(&[(key, val)])
            );
            assert_eq!(
                test_dict.dict_get_entries(vec![key]),
                ValueRef::dict_int(&[(key, val)])
            );
        }
    }

    #[test]
    fn test_dict_update() {
        let entries = [("key1", "value1"), ("key2", "value2")];
        let mut test_dict = ValueRef::dict_str(&entries);
        let update_entries = [("key1", "override_value1"), ("key2", "override_value2")];
        let update_dict = ValueRef::dict_str(&update_entries);
        test_dict.dict_update(&update_dict);
        for (key, val) in update_entries {
            assert_eq!(
                test_dict.dict_get(&ValueRef::str(key)).unwrap().clone(),
                ValueRef::str(val)
            );
        }
        let mut test_dict = ValueRef::dict_str(&entries);
        for (key, val) in update_entries {
            test_dict.dict_update_key_value(key, ValueRef::str(val));
        }
        for (key, val) in update_entries {
            assert_eq!(
                test_dict.dict_get(&ValueRef::str(key)).unwrap().clone(),
                ValueRef::str(val)
            );
        }
        let mut test_dict = ValueRef::dict_str(&entries);
        for (key, val) in update_entries {
            test_dict.dict_update_entry(
                key,
                &ValueRef::str(val),
                &ConfigEntryOperationKind::Union,
                &-1,
            );
        }
        for (key, val) in update_entries {
            assert_eq!(
                test_dict.dict_get(&ValueRef::str(key)).unwrap().clone(),
                ValueRef::str(val)
            );
        }
    }
}
