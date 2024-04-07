/* Calculation methods */

use kclvm_ast::ast;
use kclvm_runtime::{ConfigEntryOperationKind, DictValue, UnionOptions, Value, ValueRef};

use crate::ty::{resolve_schema, type_pack_and_check};
use crate::union::union_entry;
use crate::Evaluator;

impl<'ctx> Evaluator<'ctx> {
    /// lhs + rhs
    #[inline]
    pub(crate) fn add(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_add(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs - rhs
    #[inline]
    pub(crate) fn sub(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_sub(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs * rhs
    #[inline]
    pub(crate) fn mul(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_mul(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs / rhs
    #[inline]
    pub(crate) fn div(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_div(&rhs)
    }
    /// lhs // rhs
    #[inline]
    pub(crate) fn floor_div(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_floor_div(&rhs)
    }
    /// lhs % rhs
    #[inline]
    pub(crate) fn r#mod(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_mod(&rhs)
    }
    /// lhs ** rhs
    #[inline]
    pub(crate) fn pow(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_pow(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs << rhs
    #[inline]
    pub(crate) fn bit_lshift(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_lshift(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs >> rhs
    #[inline]
    pub(crate) fn bit_rshift(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_rshift(&mut self.runtime_ctx.borrow_mut(), &rhs)
    }
    /// lhs & rhs
    #[inline]
    pub(crate) fn bit_and(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_and(&rhs)
    }
    /// lhs | rhs
    #[inline]
    pub(crate) fn bit_or(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        if let (Value::int_value(a), Value::int_value(b)) = (&*lhs.rc.borrow(), &*rhs.rc.borrow()) {
            return ValueRef::int(*a | *b);
        };
        union_entry(
            self,
            &mut lhs.deep_copy(),
            &rhs,
            true,
            &UnionOptions::default(),
        )
    }
    /// lhs ^ rhs
    #[inline]
    pub(crate) fn bit_xor(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.bin_bit_xor(&rhs)
    }
    /// lhs and rhs
    #[inline]
    pub(crate) fn logic_and(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.logic_and(&rhs).into()
    }
    /// lhs or rhs
    #[inline]
    pub(crate) fn logic_or(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.logic_or(&rhs).into()
    }
    /// lhs == rhs
    #[inline]
    pub(crate) fn cmp_equal_to(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_equal(&rhs).into()
    }
    /// lhs != rhs
    #[inline]
    pub(crate) fn cmp_not_equal_to(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_not_equal(&rhs).into()
    }
    /// lhs > rhs
    #[inline]
    pub(crate) fn cmp_greater_than(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_greater_than(&rhs).into()
    }
    /// lhs >= rhs
    #[inline]
    pub(crate) fn cmp_greater_than_or_equal(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_greater_than_or_equal(&rhs).into()
    }
    /// lhs < rhs
    #[inline]
    pub(crate) fn cmp_less_than(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_less_than(&rhs).into()
    }
    /// lhs <= rhs
    #[inline]
    pub(crate) fn cmp_less_than_or_equal(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.cmp_less_than_or_equal(&rhs).into()
    }
    /// lhs as rhs
    #[inline]
    pub(crate) fn r#as(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        type_pack_and_check(self, &lhs, vec![&rhs.as_str()])
    }
    /// lhs is rhs
    #[inline]
    pub(crate) fn is(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        (lhs == rhs).into()
    }
    /// lhs is not rhs
    #[inline]
    pub(crate) fn is_not(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        (lhs != rhs).into()
    }
    /// lhs in rhs
    #[inline]
    pub(crate) fn r#in(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.r#in(&rhs).into()
    }
    /// lhs not in rhs
    #[inline]
    pub(crate) fn not_in(&self, lhs: ValueRef, rhs: ValueRef) -> ValueRef {
        lhs.not_in(&rhs).into()
    }
}

impl<'ctx> Evaluator<'ctx> {
    /// Value is truth function, return i1 value.
    #[inline]
    pub(crate) fn value_is_truthy(&self, value: &ValueRef) -> bool {
        value.is_truthy()
    }
    /// Value deep copy
    #[inline]
    pub(crate) fn value_deep_copy(&self, value: &ValueRef) -> ValueRef {
        value.deep_copy()
    }
    /// value_union unions two collection elements.
    pub(crate) fn value_union(&self, lhs: &mut ValueRef, rhs: &ValueRef) -> ValueRef {
        let attr_map = match &*lhs.rc.borrow() {
            Value::dict_value(dict) => dict.attr_map.clone(),
            Value::schema_value(schema) => schema.config.attr_map.clone(),
            _ => panic!("invalid object '{}' in attr_map", lhs.type_str()),
        };
        let opts = UnionOptions {
            list_override: false,
            idempotent_check: false,
            config_resolve: true,
        };
        if rhs.is_config() {
            let dict = rhs.as_dict_ref();
            for k in dict.values.keys() {
                let entry = rhs.dict_get_entry(k).unwrap();
                union_entry(self, lhs, &entry, true, &opts);
                // Has type annotation
                if let Some(ty) = attr_map.get(k) {
                    let value = lhs.dict_get_value(k).unwrap();
                    lhs.dict_update_key_value(k, type_pack_and_check(self, &value, vec![ty]));
                }
            }
            lhs.clone()
        } else {
            union_entry(self, lhs, rhs, true, &opts)
        }
    }
    /// Append a item into the list.
    #[inline]
    pub(crate) fn list_append(&self, list: &mut ValueRef, item: &ValueRef) {
        list.list_append(item)
    }
    /// Append a list item and unpack it into the list.
    #[inline]
    pub(crate) fn list_append_unpack(&self, list: &mut ValueRef, item: &ValueRef) {
        list.list_append_unpack(item)
    }
    #[inline]
    pub(crate) fn dict_get_value(&self, dict: &ValueRef, key: &str) -> ValueRef {
        dict.dict_get_value(key).unwrap_or(self.undefined_value())
    }
    /// Insert a dict entry including key, value, op and insert_index into the dict,
    /// and the type of key is `&str`
    #[inline]
    pub(crate) fn dict_insert(
        &self,
        dict: &mut ValueRef,
        key: &str,
        value: &ValueRef,
        op: &ast::ConfigEntryOperation,
        insert_index: i32,
    ) {
        let op = match op {
            ast::ConfigEntryOperation::Union => ConfigEntryOperationKind::Union,
            ast::ConfigEntryOperation::Override => ConfigEntryOperationKind::Override,
            ast::ConfigEntryOperation::Insert => ConfigEntryOperationKind::Insert,
        };
        self.dict_merge_key_value_pair(dict, key, value, op, insert_index, false);
    }

    /// Insert a dict entry including key, value, op and insert_index into the dict,
    /// and the type of key is `&str`
    #[inline]
    pub(crate) fn schema_dict_merge(
        &self,
        dict: &mut ValueRef,
        key: &str,
        value: &ValueRef,
        op: &ast::ConfigEntryOperation,
        insert_index: i32,
    ) {
        let op = match op {
            ast::ConfigEntryOperation::Union => ConfigEntryOperationKind::Union,
            ast::ConfigEntryOperation::Override => ConfigEntryOperationKind::Override,
            ast::ConfigEntryOperation::Insert => ConfigEntryOperationKind::Insert,
        };
        let attr_map = {
            match &*dict.rc.borrow() {
                Value::dict_value(dict) => dict.attr_map.clone(),
                Value::schema_value(schema) => schema.config.attr_map.clone(),
                _ => panic!("invalid object '{}' in attr_map", dict.type_str()),
            }
        };
        if attr_map.contains_key(key) {
            let v = type_pack_and_check(self, value, vec![attr_map.get(key).unwrap()]);
            self.dict_merge_key_value_pair(dict, key, &v, op, insert_index, false);
        } else {
            self.dict_merge_key_value_pair(dict, key, value, op, insert_index, false);
        }
    }

    /// Insert an entry including key and value into the dict.
    #[inline]
    pub(crate) fn dict_insert_value(&self, dict: &mut ValueRef, key: &str, value: &ValueRef) {
        dict.dict_update_key_value(key, value.clone())
    }

    /// Insert an entry including key and value into the dict, and merge the original entry.
    #[inline]
    pub(crate) fn dict_insert_merge_value(&self, dict: &mut ValueRef, key: &str, value: &ValueRef) {
        self.dict_merge_key_value_pair(
            dict,
            key,
            value,
            ConfigEntryOperationKind::Union,
            -1,
            false,
        );
    }

    /// Set dict key with the value. When the dict is a schema and resolve schema validations.
    pub(crate) fn dict_set_value(&self, p: &mut ValueRef, key: &str, val: &ValueRef) {
        if p.is_config() {
            p.dict_update_key_value(key, val.clone());
            if p.is_schema() {
                let schema: ValueRef;
                {
                    let schema_value = p.as_schema();
                    let mut config_keys = schema_value.config_keys.clone();
                    config_keys.push(key.to_string());
                    schema = resolve_schema(self, p, &config_keys);
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

    /// Private dict merge key value pair with the idempotent check option
    pub(crate) fn dict_merge_key_value_pair(
        &self,
        p: &mut ValueRef,
        key: &str,
        v: &ValueRef,
        op: ConfigEntryOperationKind,
        insert_index: i32,
        idempotent_check: bool,
    ) {
        if p.is_config() {
            let mut dict: DictValue = Default::default();
            dict.values.insert(key.to_string(), v.clone());
            dict.ops.insert(key.to_string(), op);
            dict.insert_indexs.insert(key.to_string(), insert_index);
            union_entry(
                self,
                p,
                &ValueRef::from(Value::dict_value(Box::new(dict))),
                true,
                &UnionOptions {
                    config_resolve: false,
                    idempotent_check,
                    ..Default::default()
                },
            );
        } else {
            panic!("invalid dict insert value: {}", p.type_str())
        }
    }
}
