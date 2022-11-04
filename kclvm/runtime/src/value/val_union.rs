// Copyright 2021 The KCL Authors. All rights reserved.

use crate::unification::value_subsume;
use crate::*;

impl ValueRef {
    fn do_union(
        &mut self,
        x: &Self,
        should_list_override: bool,
        should_idempotent_check: bool,
        should_config_resolve: bool,
    ) -> Self {
        let union_fn = |obj: &mut DictValue, delta: &DictValue| {
            // Update attribute map
            for (k, v) in &delta.ops {
                obj.ops.insert(k.clone(), v.clone());
            }
            // Update index map
            for (k, v) in &delta.insert_indexs {
                obj.insert_indexs.insert(k.clone(), *v);
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
                if !obj.values.contains_key(k) {
                    obj.values.insert(k.clone(), v.clone());
                } else {
                    match operation {
                        ConfigEntryOperationKind::Union => {
                            if should_idempotent_check
                                && obj.values.contains_key(k)
                                && !value_subsume(v, obj.values.get(k).unwrap(), false)
                            {
                                panic!("conflicting values on the attribute '{}' between {:?} and {:?}", k, self, x);
                            }
                            let value = obj.values.get_mut(k).unwrap().union(
                                v,
                                false,
                                should_list_override,
                                should_idempotent_check,
                                should_config_resolve,
                            );
                            obj.values.insert(k.clone(), value);
                        }
                        ConfigEntryOperationKind::Override => {
                            if index < 0 {
                                obj.values.insert(k.clone(), v.clone());
                            } else {
                                let origin_value = obj.values.get_mut(k).unwrap();
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
                            let origin_value = obj.values.get_mut(k).unwrap();
                            if origin_value.is_none_or_undefined() {
                                let list = ValueRef::list(None);
                                obj.values.insert(k.to_string(), list);
                            }
                            let origin_value = obj.values.get_mut(k).unwrap();
                            match (
                                &mut *origin_value.rc.borrow_mut(),
                                &mut *value.rc.borrow_mut(),
                            ) {
                                (Value::list_value(origin_value), Value::list_value(value)) => {
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
                            };
                        }
                    }
                }
            }
        };

        //union schema vars
        let mut union_schema = false;
        let mut pkgpath: String = "".to_string();
        let mut name: String = "".to_string();
        let mut common_keys: Vec<String> = vec![];
        let mut valid = true;

        match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::list_value(obj), Value::list_value(delta)) => {
                if !should_list_override {
                    let length = if obj.values.len() > delta.values.len() {
                        obj.values.len()
                    } else {
                        delta.values.len()
                    };
                    let obj_len = obj.values.len();
                    let delta_len = delta.values.len();
                    for idx in 0..length {
                        if idx >= obj_len {
                            obj.values.push(delta.values[idx].clone());
                        } else if idx < delta_len {
                            obj.values[idx].union(
                                &delta.values[idx],
                                false,
                                should_list_override,
                                should_idempotent_check,
                                should_config_resolve,
                            );
                        }
                    }
                }
            }
            (Value::dict_value(obj), Value::dict_value(delta)) => union_fn(obj, delta),
            (Value::schema_value(obj), Value::dict_value(delta)) => {
                name = obj.name.clone();
                pkgpath = obj.pkgpath.clone();
                let obj_value = obj.config.as_mut();
                union_fn(obj_value, delta);
                common_keys = obj.config_keys.clone();
                let mut other_keys: Vec<String> = delta.values.keys().cloned().collect();
                common_keys.append(&mut other_keys);
                union_schema = true;
            }
            (Value::schema_value(obj), Value::schema_value(delta)) => {
                name = obj.name.clone();
                pkgpath = obj.pkgpath.clone();
                let obj_value = obj.config.as_mut();
                let delta_value = delta.config.as_ref();
                union_fn(obj_value, delta_value);
                common_keys = obj.config_keys.clone();
                let mut other_keys: Vec<String> = delta.config_keys.clone();
                common_keys.append(&mut other_keys);
                union_schema = true;
            }
            (Value::dict_value(obj), Value::schema_value(delta)) => {
                name = delta.name.clone();
                pkgpath = delta.pkgpath.clone();
                let delta_value = delta.config.as_ref();
                union_fn(obj, delta_value);
                common_keys = delta.config_keys.clone();
                let mut other_keys: Vec<String> = obj.values.keys().cloned().collect();
                common_keys.append(&mut other_keys);
                union_schema = true;
            }
            _ => valid = false,
        }
        if !valid {
            panic!(
                "union failure, expect {:?}, got {:?}",
                self.type_str(),
                x.type_str()
            )
        }
        if union_schema {
            let result = self.clone();
            let schema = result.dict_to_schema(name.as_str(), pkgpath.as_str(), &common_keys);
            if should_config_resolve {
                *self = resolve_schema(&schema, &common_keys);
            } else {
                *self = schema;
            }
        }
        self.clone()
    }

    pub fn union(
        &mut self,
        x: &Self,
        or_mode: bool,
        should_list_override: bool,
        should_idempotent_check: bool,
        should_config_resolve: bool,
    ) -> Self {
        if self.is_none_or_undefined() {
            *self = x.clone();
            return self.clone();
        }
        if x.is_none_or_undefined() {
            return self.clone();
        }
        if self.is_list_or_config() && x.is_list_or_config() {
            self.do_union(
                x,
                should_list_override,
                should_idempotent_check,
                should_config_resolve,
            );
        } else if or_mode {
            match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
                (Value::int_value(a), Value::int_value(b)) => {
                    *a |= *b;
                    return self.clone();
                }
                _ => {}
            }
            panic!(
                "unsupported operand type(s) for |: '{:?}' and '{:?}'",
                self.type_str(),
                x.type_str()
            )
        } else {
            *self = x.clone();
        }
        self.clone()
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
