//! Copyright The KCL Authors. All rights reserved.

use crate::unification::value_subsume;
use crate::*;

/// UnionContext records some information during the value merging process,
/// including the merging path and whether there are conflicts.
#[derive(Default, Debug)]
struct UnionContext {
    path_backtrace: Vec<String>,
    conflict: bool,
    obj_json: String,
    delta_json: String,
}

/// UnionOptions denotes the union options between runtime values.
#[derive(Debug, Clone)]
pub struct UnionOptions {
    /// Whether to override list values.
    pub list_override: bool,
    /// Whether to do the idempotent check.
    pub idempotent_check: bool,
    /// Whether to resolve config including optional attributes, etc.
    pub config_resolve: bool,
}

impl Default for UnionOptions {
    fn default() -> Self {
        Self {
            list_override: false,
            idempotent_check: true,
            config_resolve: true,
        }
    }
}

impl ValueRef {
    fn do_union(
        &mut self,
        ctx: &mut Context,
        x: &Self,
        opts: &UnionOptions,
        union_context: &mut UnionContext,
    ) -> Self {
        if self.is_same_ref(x) {
            return self.clone();
        }

        let mut union_fn = |obj: &mut DictValue, delta: &DictValue| {
            // Update potential schema type
            obj.potential_schema = delta.potential_schema.clone();
            // Update attribute map
            for (k, v) in &delta.ops {
                obj.ops.insert(k.clone(), v.clone());
            }
            // Update index map
            for (k, v) in &delta.insert_indexs {
                obj.insert_indexs.insert(k.clone(), *v);
            }
            // Update values
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
                            let obj_value = obj.values.get_mut(k).unwrap();
                            if opts.idempotent_check && !value_subsume(v, obj_value, false) {
                                union_context.conflict = true;
                                union_context.path_backtrace.push(k.clone());
                                union_context.obj_json = if obj_value.is_config() {
                                    "{...}".to_string()
                                } else if obj_value.is_list() {
                                    "[...]".to_string()
                                } else {
                                    obj_value.to_json_string()
                                };

                                union_context.delta_json = if v.is_config() {
                                    "{...}".to_string()
                                } else if v.is_list() {
                                    "[...]".to_string()
                                } else {
                                    v.to_json_string()
                                };
                                return;
                            }
                            obj_value.union(ctx, v, false, opts, union_context);
                            if union_context.conflict {
                                union_context.path_backtrace.push(k.clone());
                                return;
                            }
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
                            let origin_value = obj.values.get_mut(k).unwrap();
                            if origin_value.is_none_or_undefined() {
                                let list = ValueRef::list(None);
                                obj.values.insert(k.to_string(), list);
                            }
                            let origin_value = obj.values.get_mut(k).unwrap();
                            if origin_value.is_same_ref(v) {
                                continue;
                            }
                            match (&mut *origin_value.rc.borrow_mut(), &*v.rc.borrow()) {
                                (Value::list_value(origin_value), Value::list_value(value)) => {
                                    if index == -1 {
                                        for elem in value.values.iter() {
                                            origin_value.values.push(elem.clone());
                                        }
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
        let mut args = None;
        let mut kwargs = None;
        let mut valid = true;
        match (&mut *self.rc.borrow_mut(), &*x.rc.borrow()) {
            (Value::list_value(obj), Value::list_value(delta)) => {
                if !opts.list_override {
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
                                ctx,
                                &delta.values[idx],
                                false,
                                opts,
                                union_context,
                            );
                            if union_context.conflict {
                                union_context.path_backtrace.push(format!("list[{idx}]"));
                            }
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
                args = Some(obj.args.clone());
                kwargs = Some(obj.kwargs.clone());
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
                args = Some(delta.args.clone());
                kwargs = Some(delta.kwargs.clone());
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
                args = Some(delta.args.clone());
                kwargs = Some(delta.kwargs.clone());
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
        if union_context.conflict {
            return self.clone();
        }
        if union_schema {
            // Override schema arguments and keyword arguments.
            let mut result = self.clone();
            if let (Some(args), Some(kwargs)) = (&args, &kwargs) {
                result.set_schema_args(args, kwargs);
            }
            let optional_mapping = if self.is_schema() {
                self.schema_optional_mapping()
            } else {
                x.schema_optional_mapping()
            };
            let schema = result.dict_to_schema(
                name.as_str(),
                pkgpath.as_str(),
                &common_keys,
                &x.schema_config_meta(),
                &optional_mapping,
                args,
                kwargs,
            );
            if opts.config_resolve {
                *self = resolve_schema(ctx, &schema, &common_keys);
            } else {
                *self = schema;
            }
        }
        self.clone()
    }
    fn union(
        &mut self,
        ctx: &mut Context,
        x: &Self,
        or_mode: bool,
        opts: &UnionOptions,
        union_context: &mut UnionContext,
    ) -> Self {
        if self.is_none_or_undefined() {
            *self = x.clone();
            return self.clone();
        }
        if x.is_none_or_undefined() {
            return self.clone();
        }
        if self.is_list_or_config() && x.is_list_or_config() {
            self.do_union(ctx, x, opts, union_context);
        } else if or_mode {
            if let (Value::int_value(a), Value::int_value(b)) =
                (&mut *self.rc.borrow_mut(), &*x.rc.borrow())
            {
                *a |= *b;
                return self.clone();
            };
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

    pub fn union_entry(
        &mut self,
        ctx: &mut Context,
        x: &Self,
        or_mode: bool,
        opts: &UnionOptions,
    ) -> Self {
        let mut union_context = UnionContext::default();
        let ret = self.union(ctx, x, or_mode, opts, &mut union_context);
        if union_context.conflict {
            union_context.path_backtrace.reverse();
            let conflict_key = union_context.path_backtrace.last().unwrap();
            let path_string = union_context.path_backtrace.join(".");

            // build note
            // it will be like:
            // {...} | {
            //         ...
            //         b = {...}
            //         ...
            // }

            let note = format!(
                "    {{...}} | {{\n            ...\n            {} = {}\n            ...\n    }}",
                conflict_key, union_context.delta_json
            );
            if conflict_key.is_empty() {
                panic!(
                    "conflicting values between {} and {}",
                    union_context.delta_json, union_context.obj_json
                );
            } else {
                panic!(
                    "conflicting values on the attribute '{}' between :\n    {}\nand\n    {}\nwith union path :\n    {}\ntry operator '=' to override the attribute, like:\n{}",
                    conflict_key,
                    union_context.obj_json,
                    union_context.delta_json,
                    path_string,
                    note,
                );
            }
        }
        ret
    }
}

#[cfg(test)]
mod test_value_union {

    use crate::*;

    #[test]
    fn test_list_union() {
        let mut ctx = Context::new();
        let cases = [
            ("[0]", "[1, 2]", "[1, 2]"),
            ("[1, 2]", "[2]", "[2, 2]"),
            ("[0, 0]", "[1, 2]", "[1, 2]"),
        ];
        for (left, right, expected) in cases {
            let left_value = ValueRef::from_json(&mut ctx, left).unwrap();
            let right_value = ValueRef::from_json(&mut ctx, right).unwrap();
            let value = left_value.bin_bit_or(&mut ctx, &right_value);
            assert_eq!(value.to_json_string(), expected);
        }
    }

    #[test]
    fn test_dict_union() {
        let mut ctx = Context::new();
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
            let result = left_value.bin_bit_or(&mut ctx, &right_value);
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
        let mut ctx = Context::new();
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
            let result = left_value.bin_bit_or(&mut ctx, &right_value);
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

    #[test]
    fn test_dict_union_same_ref() {
        let mut ctx = Context::new();
        let cases = [
            (
                vec![("key1", "value", ConfigEntryOperationKind::Union, -1)],
                vec![("key1", "value", ConfigEntryOperationKind::Union, -1)],
                vec![("key2", "value", ConfigEntryOperationKind::Union, -1)],
                vec![
                    ("key1", "value", ConfigEntryOperationKind::Union, -1),
                    ("key2", "value", ConfigEntryOperationKind::Union, -1),
                ],
            ),
            (
                vec![("key1", "value1", ConfigEntryOperationKind::Override, -1)],
                vec![("key1", "value2", ConfigEntryOperationKind::Override, -1)],
                vec![("key2", "value", ConfigEntryOperationKind::Override, -1)],
                vec![
                    ("key1", "value2", ConfigEntryOperationKind::Override, -1),
                    ("key2", "value", ConfigEntryOperationKind::Override, -1),
                ],
            ),
            (
                vec![("key1", "value1", ConfigEntryOperationKind::Union, -1)],
                vec![("key1", "value2", ConfigEntryOperationKind::Override, -1)],
                vec![("key2", "value", ConfigEntryOperationKind::Override, -1)],
                vec![
                    ("key1", "value2", ConfigEntryOperationKind::Override, -1),
                    ("key2", "value", ConfigEntryOperationKind::Override, -1),
                ],
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
                vec![("key3", "value", ConfigEntryOperationKind::Union, -1)],
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
                    ("key3", "value", ConfigEntryOperationKind::Union, -1),
                ],
            ),
        ];
        for (left_entries, right_entries, both_entries, expected) in cases {
            let mut left_value = ValueRef::dict(None);
            let mut right_value = ValueRef::dict(None);
            for (key, val, op, index) in left_entries {
                left_value.dict_update_entry(key, &ValueRef::str(val), &op, &index);
            }
            for (key, val, op, index) in right_entries {
                right_value.dict_update_entry(key, &ValueRef::str(val), &op, &index);
            }
            for (key, val, op, index) in both_entries {
                let both_val = ValueRef::str(val);
                left_value.dict_update_entry(key, &both_val, &op, &index);
                left_value.dict_update_entry(key, &both_val, &op, &index);
            }
            let result = left_value.bin_bit_or(&mut ctx, &right_value);
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
    fn test_dict_union_conflict_attr() {
        let pre_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let cases = [
            (
                r#"{"key" : "value"}"#,
                r#"{"key" : "value1"}"#,
                r#"conflicting values on the attribute 'key' between :
    "value"
and
    "value1"
with union path :
    key
try operator '=' to override the attribute, like:
    {...} | {
            ...
            key = "value1"
            ...
    }"#,
            ),
            (
                r#"[{"key" : "value"}]"#,
                r#"[{"key" : "value1"}]"#,
                r#"conflicting values on the attribute 'key' between :
    "value"
and
    "value1"
with union path :
    list[0].key
try operator '=' to override the attribute, like:
    {...} | {
            ...
            key = "value1"
            ...
    }"#,
            ),
            (
                r#"{"key1" : { "key2" : 3 }}"#,
                r#"{"key1" : { "key2" : 4 }}"#,
                r#"conflicting values on the attribute 'key2' between :
    3
and
    4
with union path :
    key1.key2
try operator '=' to override the attribute, like:
    {...} | {
            ...
            key2 = 4
            ...
    }"#,
            ),
            (
                r#"{"key1" : { "key2" : 3 }}"#,
                r#"{"key1" : [1,2,3]}"#,
                r#"conflicting values on the attribute 'key1' between :
    {...}
and
    [...]
with union path :
    key1
try operator '=' to override the attribute, like:
    {...} | {
            ...
            key1 = [...]
            ...
    }"#,
            ),
            (
                r#"{"key1" : [1,2,3]}"#,
                r#"{"key1" : { "key2" : 3 }}"#,
                r#"conflicting values on the attribute 'key1' between :
    [...]
and
    {...}
with union path :
    key1
try operator '=' to override the attribute, like:
    {...} | {
            ...
            key1 = {...}
            ...
    }"#,
            ),
        ];
        for (left, right, expected) in cases {
            assert_panic(expected, || {
                let mut ctx = Context::new();
                let left_value = ValueRef::from_json(&mut ctx, left).unwrap();
                let right_value = ValueRef::from_json(&mut ctx, right).unwrap();
                left_value.bin_bit_or(&mut ctx, &right_value);
            });
        }
        std::panic::set_hook(pre_hook);
    }
}
