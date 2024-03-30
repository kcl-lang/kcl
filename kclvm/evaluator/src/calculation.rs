/* Calculation methods */

use kclvm_ast::ast;
use kclvm_runtime::{type_pack_and_check, ConfigEntryOperationKind, UnionOptions, Value, ValueRef};

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
        lhs.bin_bit_or(&mut self.runtime_ctx.borrow_mut(), &rhs)
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
        type_pack_and_check(
            &mut self.runtime_ctx.borrow_mut(),
            &lhs,
            vec![&rhs.as_str()],
        )
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
    /// Value subscript a[b]
    #[inline]
    pub(crate) fn _value_subscript(&self, value: &ValueRef, item: &ValueRef) -> ValueRef {
        value.bin_subscr(item)
    }
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
        let ctx = &mut self.runtime_ctx.borrow_mut();
        if rhs.is_config() {
            let dict = rhs.as_dict_ref();
            for k in dict.values.keys() {
                let entry = rhs.dict_get_entry(k).unwrap();
                lhs.union_entry(ctx, &entry, true, &opts);
                // Has type annotation
                if let Some(ty) = attr_map.get(k) {
                    let value = lhs.dict_get_value(k).unwrap();
                    lhs.dict_update_key_value(k, type_pack_and_check(ctx, &value, vec![ty]));
                }
            }
            lhs.clone()
        } else {
            lhs.union_entry(ctx, rhs, true, &opts)
        }
    }
    // List get the item using the index.
    #[inline]
    pub(crate) fn _list_get(&self, list: &ValueRef, index: ValueRef) -> ValueRef {
        list.list_get(index.as_int() as isize).unwrap()
    }
    // List set the item using the index.
    #[inline]
    pub(crate) fn _list_set(&self, list: &mut ValueRef, index: ValueRef, value: &ValueRef) {
        list.list_set(index.as_int() as usize, value)
    }
    // List slice.
    #[inline]
    pub(crate) fn _list_slice(
        &self,
        list: &ValueRef,
        start: &ValueRef,
        stop: &ValueRef,
        step: &ValueRef,
    ) -> ValueRef {
        list.list_slice(start, stop, step)
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
    /// Runtime list value pop
    #[inline]
    pub(crate) fn _list_pop(&self, list: &mut ValueRef) -> Option<ValueRef> {
        list.list_pop()
    }
    /// Runtime list pop the first value
    #[inline]
    pub(crate) fn _list_pop_first(&self, list: &mut ValueRef) -> Option<ValueRef> {
        list.list_pop_first()
    }
    /// List clear value.
    #[inline]
    pub(crate) fn _list_clear(&self, list: &mut ValueRef) {
        list.list_clear()
    }
    /// Return number of occurrences of the list value.
    #[inline]
    pub(crate) fn _list_count(&self, list: &ValueRef, item: &ValueRef) -> ValueRef {
        ValueRef::int(list.list_count(item) as i64)
    }
    /// Return first index of the list value. Panic if the value is not present.
    #[inline]
    pub(crate) fn _list_find(&self, list: &ValueRef, item: &ValueRef) -> isize {
        list.list_find(item)
    }
    /// Insert object before index of the list value.
    #[inline]
    pub(crate) fn _list_insert(&self, list: &mut ValueRef, index: &ValueRef, value: &ValueRef) {
        list.list_insert_at(index.as_int() as usize, value)
    }
    /// List length.
    #[inline]
    pub(crate) fn _list_len(&self, list: &ValueRef) -> usize {
        list.len()
    }
    /// Dict get the value of the key.
    #[inline]
    pub(crate) fn _dict_get(&self, dict: &ValueRef, key: &ValueRef) -> ValueRef {
        dict.dict_get(key).unwrap()
    }
    #[inline]
    pub(crate) fn dict_get_value(&self, dict: &ValueRef, key: &str) -> ValueRef {
        dict.dict_get_value(key).unwrap()
    }
    /// Dict clear value.
    #[inline]
    pub(crate) fn _dict_clear(&self, dict: &mut ValueRef) {
        dict.dict_clear()
    }
    /// Dict length.
    #[inline]
    pub(crate) fn _dict_len(&self, dict: &ValueRef) -> usize {
        dict.len()
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
        dict.dict_insert(
            &mut self.runtime_ctx.borrow_mut(),
            key,
            value,
            op,
            insert_index,
        );
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
            let v = type_pack_and_check(
                &mut self.runtime_ctx.borrow_mut(),
                value,
                vec![attr_map.get(key).unwrap()],
            );
            dict.dict_merge(
                &mut self.runtime_ctx.borrow_mut(),
                key,
                &v,
                op,
                insert_index,
            );
        } else {
            dict.dict_merge(
                &mut self.runtime_ctx.borrow_mut(),
                key,
                value,
                op,
                insert_index,
            );
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
        dict.dict_insert(
            &mut self.runtime_ctx.borrow_mut(),
            key,
            value,
            ConfigEntryOperationKind::Union,
            -1,
        );
    }
}
