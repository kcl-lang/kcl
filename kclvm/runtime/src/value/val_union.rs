// Copyright 2021 The KCL Authors. All rights reserved.

use crate::unification::value_subsume;
use crate::*;

impl ValueRef {
    fn do_union(
        &self,
        x: &Self,
        should_list_override: bool,
        should_idempotent_check: bool,
        should_config_resolve: bool,
    ) -> Self {
        let union_fn = |obj: &DictValue, delta: &DictValue| {
            let result_dict = get_ref_mut(obj);
            // Update attribute map
            for (k, v) in &delta.ops {
                result_dict.ops.insert(k.clone(), v.clone());
            }
            // Update index map
            for (k, v) in &delta.insert_indexs {
                result_dict.insert_indexs.insert(k.clone(), *v);
            }
            for (k, v) in &delta.values {
                let operation = if let Some(op) = delta.ops.get(k) {
                    op
                } else {
                    &ConfigEntryOperationKind::Union
                };
                let index = if let Some(idx) = delta.insert_indexs.get(k) {
                    *idx
                } else {
                    -1
                };
                if !result_dict.values.contains_key(k) {
                    result_dict.values.insert(k.clone(), v.clone());
                } else {
                    match operation {
                        ConfigEntryOperationKind::Union => {
                            if should_idempotent_check
                                && obj.values.contains_key(k)
                                && !value_subsume(v, obj.values.get(k).unwrap(), false)
                            {
                                panic!("conflicting values on the attribute '{}' between {:?} and {:?}", k, self.to_string(), x.to_string());
                            }
                            let value = obj.values.get(k).unwrap().union(
                                v,
                                false,
                                should_list_override,
                                should_idempotent_check,
                                should_config_resolve,
                            );
                            result_dict.values.insert(k.clone(), value);
                        }
                        ConfigEntryOperationKind::Override => {
                            if index < 0 {
                                result_dict.values.insert(k.clone(), v.clone());
                            } else {
                                let origin_value = result_dict.values.get_mut(k).unwrap();
                                if !origin_value.is_list() {
                                    panic!("only list attribute can be inserted value");
                                }
                                if v.is_none_or_undefined() {
                                    origin_value.list_remove_at(index as usize);
                                } else {
                                    origin_value.list_set(index as usize, v);
                                }
                            }
                        }
                        ConfigEntryOperationKind::Insert => {
                            let value = v.deep_copy();
                            let origin_value = result_dict.values.get_mut(k).unwrap();
                            if origin_value.is_none_or_undefined() {
                                let list = ValueRef::list(None);
                                result_dict.values.insert(k.to_string(), list);
                            }
                            let origin_value = result_dict.values.get_mut(k).unwrap();
                            match (&*origin_value.rc, &*value.rc) {
                                (Value::list_value(origin_value), Value::list_value(value)) => {
                                    // As RefMut
                                    let origin_value: &mut ListValue = unsafe {
                                        &mut *(origin_value as *const ListValue as *mut ListValue)
                                    };
                                    // As RefMut
                                    let value: &mut ListValue = unsafe {
                                        &mut *(value as *const ListValue as *mut ListValue)
                                    };
                                    if index == -1 {
                                        origin_value.values.append(&mut value.clone().values);
                                    } else if index >= 0 {
                                        let mut insert_index = index;
                                        for v in &value.values {
                                            origin_value
                                                .values
                                                .insert(insert_index as usize, v.clone());
                                            insert_index += 1;
                                        }
                                    }
                                }
                                _ => panic!("only list attribute can be inserted value"),
                            }
                        }
                    }
                }
            }
            self.clone()
        };
        match (&*self.rc, &*x.rc) {
            (Value::list_value(obj), Value::list_value(delta)) => {
                // Clone reference
                let mut result_list = self.clone();
                if should_list_override {
                    return result_list;
                }
                let length = if obj.values.len() > delta.values.len() {
                    obj.values.len()
                } else {
                    delta.values.len()
                };
                let obj_len = obj.values.len();
                let delta_len = delta.values.len();
                for idx in 0..length {
                    if idx >= obj_len {
                        result_list.list_append(&delta.values[idx]);
                    } else if idx < delta_len {
                        let value = obj.values[idx].union(
                            &delta.values[idx],
                            false,
                            should_list_override,
                            should_idempotent_check,
                            should_config_resolve,
                        );
                        result_list.list_set(idx, &value);
                    }
                }
                result_list
            }
            (Value::dict_value(obj), Value::dict_value(delta)) => union_fn(obj, delta),
            (Value::schema_value(obj), Value::dict_value(delta)) => {
                let name = &obj.name;
                let pkgpath = &obj.pkgpath;
                let obj_value = obj.config.as_ref();
                let result = union_fn(obj_value, delta);
                let mut common_keys = obj.config_keys.clone();
                let mut other_keys: Vec<String> = delta.values.keys().cloned().collect();
                common_keys.append(&mut other_keys);
                let schema = result.dict_to_schema(name.as_str(), pkgpath.as_str(), &common_keys);
                if should_config_resolve {
                    resolve_schema(&schema, &common_keys)
                } else {
                    schema
                }
            }
            (Value::schema_value(obj), Value::schema_value(delta)) => {
                let name = &obj.name;
                let pkgpath = &obj.pkgpath;
                let obj_value = obj.config.as_ref();
                let delta_value = delta.config.as_ref();
                let result = union_fn(obj_value, delta_value);
                let mut common_keys = obj.config_keys.clone();
                let mut other_keys = delta.config_keys.clone();
                common_keys.append(&mut other_keys);
                let schema = result.dict_to_schema(name.as_str(), pkgpath.as_str(), &common_keys);
                if should_config_resolve {
                    resolve_schema(&schema, &common_keys)
                } else {
                    schema
                }
            }
            (Value::dict_value(obj), Value::schema_value(delta)) => {
                let name = &delta.name;
                let pkgpath = &delta.pkgpath;
                let delta_value = delta.config.as_ref();
                let result = union_fn(obj, delta_value);
                let mut common_keys = delta.config_keys.clone();
                let mut other_keys: Vec<String> = obj.values.keys().cloned().collect();
                common_keys.append(&mut other_keys);
                let schema =
                    result.dict_to_schema(name.as_str(), pkgpath.as_str(), &delta.config_keys);
                if should_config_resolve {
                    resolve_schema(&schema, &common_keys)
                } else {
                    schema
                }
            }
            _ => {
                panic!(
                    "union failure, expect {:?}, got {:?}",
                    self.type_str(),
                    x.type_str()
                );
            }
        }
    }

    pub fn union(
        &self,
        x: &Self,
        or_mode: bool,
        should_list_override: bool,
        should_idempotent_check: bool,
        should_config_resolve: bool,
    ) -> Self {
        if self.is_none_or_undefined() {
            return x.clone();
        }
        if x.is_none_or_undefined() {
            return self.clone();
        }
        match (&*self.rc, &*x.rc) {
            (
                Value::list_value(_) | Value::dict_value(_) | Value::schema_value(_),
                Value::list_value(_) | Value::dict_value(_) | Value::schema_value(_),
            ) => self.do_union(
                x,
                should_list_override,
                should_idempotent_check,
                should_config_resolve,
            ),
            _ => {
                if or_mode {
                    match (&*self.rc, &*x.rc) {
                        (Value::int_value(a), Value::int_value(b)) => Self::int(*a | *b),
                        _ => {
                            panic!(
                                "unsupported operand type(s) for |: '{:?}' and '{:?}'",
                                self.type_str(),
                                x.type_str()
                            );
                        }
                    }
                } else if x.is_none_or_undefined() {
                    self.clone()
                } else {
                    x.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod test_value_union {

    use crate::*;

    #[test]
    fn test_list_union() {
        let cases = [
            ("[0]", "[1, 2]", "[1, 2]"),
            ("[1, 2]", "[2]", "[2, 2]"),
            ("[0, 0]", "[1, 2]", "[1, 2]"),
        ];
        for (left, right, expected) in cases {
            let left_value = ValueRef::from_json(left).unwrap();
            let right_value = ValueRef::from_json(right).unwrap();
            let value = left_value.bin_bit_or(&right_value);
            assert_eq!(value.to_json_string(), expected);
        }
    }

    #[test]
    fn test_dict_union() {
        let cases = [
            (
                vec![("key", "value", ConfigEntryOperationKind::Union, -1)],
                vec![("key", "value", ConfigEntryOperationKind::Union, -1)],
                vec![("key", "value", ConfigEntryOperationKind::Union, -1)],
            ),
            (
                vec![("key", "value", ConfigEntryOperationKind::Override, -1)],
                vec![("key", "value", ConfigEntryOperationKind::Override, -1)],
                vec![("key", "value", ConfigEntryOperationKind::Override, -1)],
            ),
            (
                vec![("key", "value1", ConfigEntryOperationKind::Union, -1)],
                vec![("key", "value2", ConfigEntryOperationKind::Override, -1)],
                vec![("key", "value2", ConfigEntryOperationKind::Override, -1)],
            ),
            (
                vec![
                    ("key1", "value1", ConfigEntryOperationKind::Union, -1),
                    ("key2", "value2", ConfigEntryOperationKind::Union, -1),
                ],
                vec![
                    (
                        "key1",
                        "override_value1",
                        ConfigEntryOperationKind::Override,
                        -1,
                    ),
                    (
                        "key2",
                        "override_value2",
                        ConfigEntryOperationKind::Override,
                        -1,
                    ),
                ],
                vec![
                    (
                        "key1",
                        "override_value1",
                        ConfigEntryOperationKind::Override,
                        -1,
                    ),
                    (
                        "key2",
                        "override_value2",
                        ConfigEntryOperationKind::Override,
                        -1,
                    ),
                ],
            ),
        ];
        for (left_entries, right_entries, expected) in cases {
            let mut left_value = ValueRef::dict(None);
            let mut right_value = ValueRef::dict(None);
            for (key, val, op, index) in left_entries {
                left_value.dict_update_entry(key, &ValueRef::str(val), &op, &index);
            }
            for (key, val, op, index) in right_entries {
                right_value.dict_update_entry(key, &ValueRef::str(val), &op, &index);
            }
            let result = left_value.bin_bit_or(&right_value);
            for (key, val, op, index) in expected {
                let result_dict = result.as_dict_ref();
                let result_val = result_dict.values.get(key).unwrap().as_str();
                let result_op = result_dict.ops.get(key).unwrap();
                let result_index = result_dict.insert_indexs.get(key).unwrap();
                assert_eq!(result_val, val);
                assert_eq!(*result_op, op);
                assert_eq!(*result_index, index);
            }
        }
    }
    #[test]
    fn test_dict_union_insert() {
        let cases = [
            (
                vec![("key", vec![0, 1], ConfigEntryOperationKind::Override, -1)],
                vec![("key", vec![2, 3], ConfigEntryOperationKind::Insert, -1)],
                vec![(
                    "key",
                    vec![0, 1, 2, 3],
                    ConfigEntryOperationKind::Insert,
                    -1,
                )],
            ),
            (
                vec![("key", vec![0, 1], ConfigEntryOperationKind::Override, -1)],
                vec![("key", vec![2, 3], ConfigEntryOperationKind::Insert, 0)],
                vec![("key", vec![2, 3, 0, 1], ConfigEntryOperationKind::Insert, 0)],
            ),
            (
                vec![("key", vec![0, 1], ConfigEntryOperationKind::Override, -1)],
                vec![("key", vec![2, 3], ConfigEntryOperationKind::Insert, 1)],
                vec![("key", vec![0, 2, 3, 1], ConfigEntryOperationKind::Insert, 1)],
            ),
        ];
        for (left_entries, right_entries, expected) in cases {
            let mut left_value = ValueRef::dict(None);
            let mut right_value = ValueRef::dict(None);
            for (key, val, op, index) in left_entries {
                left_value.dict_update_entry(key, &ValueRef::list_int(val.as_slice()), &op, &index);
            }
            for (key, val, op, index) in right_entries {
                right_value.dict_update_entry(
                    key,
                    &ValueRef::list_int(val.as_slice()),
                    &op,
                    &index,
                );
            }
            let result = left_value.bin_bit_or(&right_value);
            for (key, val, op, index) in expected {
                let result_dict = result.as_dict_ref();
                let result_val = result_dict.values.get(key).unwrap();
                let result_op = result_dict.ops.get(key).unwrap();
                let result_index = result_dict.insert_indexs.get(key).unwrap();
                assert_eq!(result_val.clone(), ValueRef::list_int(val.as_slice()));
                assert_eq!(*result_op, op);
                assert_eq!(*result_index, index);
            }
        }
    }
}
