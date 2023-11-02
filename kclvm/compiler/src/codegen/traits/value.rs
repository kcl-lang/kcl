//! Copyright The KCL Authors. All rights reserved.

use std::collections::HashMap;

use super::BackendTypes;

/// ValueMethods defines all value APIs.
pub trait ValueMethods: BackendTypes {
    /// Construct a 64-bit int value using i64
    fn int_value(&self, v: i64) -> Self::Value;
    /// Construct a 64-bit float value using f64
    fn float_value(&self, v: f64) -> Self::Value;
    /// Construct a string value using &str
    fn string_value(&self, v: &str) -> Self::Value;
    /// Construct a bool value
    fn bool_value(&self, v: bool) -> Self::Value;
    /// Construct a None value
    fn none_value(&self) -> Self::Value;
    /// Construct a Undefined value
    fn undefined_value(&self) -> Self::Value;
    /// Construct a empty list value
    fn list_value(&self) -> Self::Value;
    /// Construct a list value with `n` elements
    fn list_values(&self, values: &[Self::Value]) -> Self::Value;
    /// Construct a empty dict value.
    fn dict_value(&self) -> Self::Value;
    /// Construct a unit value.
    fn unit_value(&self, v: f64, raw: i64, unit: &str) -> Self::Value;
    /// Construct a function value using a native function value.
    fn function_value(&self, function: Self::Function) -> Self::Value;
    /// Construct a closure function value with the closure variable.
    fn closure_value(&self, function: Self::Function, closure: Self::Value) -> Self::Value;
    /// Construct a structure function value using native functions.
    fn struct_function_value(
        &self,
        functions: &[Self::Function],
        attr_functions: &HashMap<String, Vec<Self::Function>>,
        runtime_type: &str,
    ) -> Self::Value;
    /// Construct a builtin function value using the function name.
    fn builtin_function_value(&self, function_name: &str) -> Self::Value;
    /// Get a global value pointer named `name`.
    fn global_value_ptr(&self, name: &str) -> Self::Value;
    /// Get current runtime context pointer.
    fn current_runtime_ctx_ptr(&self) -> Self::Value;
}

/// DerivedValueCalculationMethods defines all value base calculation APIs.
pub trait ValueCalculationMethods: BackendTypes {
    // calculations
    /// lhs + rhs
    fn add(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs - rhs
    fn sub(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs * rhs
    fn mul(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs / rhs
    fn div(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs // rhs
    fn floor_div(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs % rhs
    fn r#mod(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs ** rhs
    fn pow(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs << rhs
    fn bit_lshift(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs >> rhs
    fn bit_rshift(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs & rhs
    fn bit_and(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs | rhs
    fn bit_or(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs ^ rhs
    fn bit_xor(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs and rhs
    fn logic_and(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs or rhs
    fn logic_or(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs == rhs
    fn cmp_equal_to(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs != rhs
    fn cmp_not_equal_to(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs > rhs
    fn cmp_greater_than(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs >= rhs
    fn cmp_greater_than_or_equal(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs < rhs
    fn cmp_less_than(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs <= rhs
    fn cmp_less_than_or_equal(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs as rhs
    fn r#as(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs is rhs
    fn is(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs is not rhs
    fn is_not(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs in rhs
    fn r#in(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    /// lhs not in rhs
    fn not_in(&self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
}

/// DerivedValueCalculationMethods based ValueCalculationMethods defines all value derived
/// operation APIs such as deep copy, union and collection operations.
pub trait DerivedValueCalculationMethods: ValueMethods + ValueCalculationMethods {
    /// Value subscript a[b]
    fn value_subscript(&self, value: Self::Value, item: Self::Value) -> Self::Value;
    /// Value is truth function, return i1 value.
    fn value_is_truthy(&self, value: Self::Value) -> Self::Value;
    /// Value deep copy
    fn value_deep_copy(&self, value: Self::Value) -> Self::Value;
    /// value_union unions two collection elements.
    fn value_union(&self, lhs: Self::Value, rhs: Self::Value);
    // List get the item using the index.
    fn list_get(&self, list: Self::Value, index: Self::Value) -> Self::Value;
    // List set the item using the index.
    fn list_set(&self, list: Self::Value, index: Self::Value, value: Self::Value);
    // List slice.
    fn list_slice(
        &self,
        list: Self::Value,
        start: Self::Value,
        stop: Self::Value,
        step: Self::Value,
    ) -> Self::Value;
    /// Append a item into the list.
    fn list_append(&self, list: Self::Value, item: Self::Value);
    /// Append a list item and unpack it into the list.
    fn list_append_unpack(&self, list: Self::Value, item: Self::Value);
    /// List value pop
    fn list_pop(&self, list: Self::Value) -> Self::Value;
    /// List value pop first
    fn list_pop_first(&self, list: Self::Value) -> Self::Value;
    /// List clear value
    fn list_clear(&self, list: Self::Value);
    /// Return number of occurrences of the list value.
    fn list_count(&self, list: Self::Value, item: Self::Value) -> Self::Value;
    /// Return first index of the list value. Panic if the value is not present.
    fn list_find(&self, list: Self::Value, item: Self::Value) -> Self::Value;
    /// Insert object before index of the list value.
    fn list_insert(&self, list: Self::Value, index: Self::Value, value: Self::Value);
    /// List length.
    fn list_len(&self, list: Self::Value) -> Self::Value;
    /// Dict get the value of the key.
    fn dict_get(&self, dict: Self::Value, key: Self::Value) -> Self::Value;
    /// Dict set the value of the key.
    fn dict_set(&self, dict: Self::Value, key: Self::Value, value: Self::Value);
    /// Return all dict keys.
    fn dict_keys(&self, dict: Self::Value) -> Self::Value;
    /// Return all dict values.
    fn dict_values(&self, dict: Self::Value) -> Self::Value;
    /// Dict clear value.
    fn dict_clear(&self, dict: Self::Value);
    /// Dict pop the value of the key.
    fn dict_pop(&self, dict: Self::Value, key: Self::Value) -> Self::Value;
    /// Dict length.
    fn dict_len(&self, dict: Self::Value) -> Self::Value;
    /// Insert a dict entry including key, value, op and insert_index into the dict.
    /// and the type of key is `Self::Value`
    fn dict_insert_with_key_value(
        &self,
        dict: Self::Value,
        key: Self::Value,
        value: Self::Value,
        op: i32,
        insert_index: i32,
    );
    /// Insert a dict entry including key, value, op and insert_index into the dict,
    /// and the type of key is `&str`
    fn dict_insert(
        &self,
        dict: Self::Value,
        key: &str,
        value: Self::Value,
        op: i32,
        insert_index: i32,
    ) {
        self.dict_insert_with_key_value(dict, self.string_value(key), value, op, insert_index);
    }
    /// Insert a dict entry with the override = attribute operator including key, value into the dict.
    fn dict_insert_override_item(&self, dict: Self::Value, key: &str, value: Self::Value) {
        self.dict_insert(dict, key, value, 1, -1);
    }
    /// Dict contains key.
    fn dict_contains_key(&self, dict: Self::Value, key: &str) -> Self::Value {
        self.r#in(self.string_value(key), dict)
    }
}

/// ValueCodeGen defines all value APIs.
pub trait ValueCodeGen:
    ValueMethods + ValueCalculationMethods + DerivedValueCalculationMethods
{
}
