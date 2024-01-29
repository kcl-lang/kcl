//! Copyright The KCL Authors. All rights reserved.

use crate::*;

/// Calculate the partial order relationship between `KCL value objects`
/// and judge whether the value1 object ∈ the value2 object.
///
/// Please note that The type and value of KCL are defined and used separately,
/// so the partial order relationship calculation is also divided into two types,
/// type and value, and there is no partial order relationship between type
/// objects and value objects.
pub fn value_subsume(value1: &ValueRef, value2: &ValueRef, should_recursive_check: bool) -> bool {
    if value1.is_none_or_undefined() || value2.is_none_or_undefined() {
        return true;
    }
    if value1 == value2 {
        return true;
    }
    if value1.is_int() && value2.is_int() {
        return value1.as_int() == value2.as_int();
    }
    if value1.is_float() && value2.is_float() {
        return value1.as_float() == value2.as_float();
    }
    if value1.is_bool() && value2.is_bool() {
        return value1.as_bool() == value2.as_bool();
    }
    if value1.is_str() && value2.is_str() {
        return value1.as_str() == value2.as_str();
    }
    match (&*value1.rc.borrow(), &*value2.rc.borrow()) {
        (Value::list_value(value1), Value::list_value(value2)) => {
            return value1.values.len() == value2.values.len()
                && value1
                    .values
                    .iter()
                    .zip(value2.values.iter())
                    .all(|(item1, item2)| value_subsume(item1, item2, should_recursive_check));
        }
        (
            Value::dict_value(_) | Value::schema_value(_),
            Value::dict_value(_) | Value::schema_value(_),
        ) => {
            let value1_dict = &value1.as_dict_ref().values;
            let value2_dict = &value2.as_dict_ref().values;
            if value1_dict.is_empty() {
                return true;
            }
            if value1_dict.keys().all(|key| !value2_dict.contains_key(key)) {
                return true;
            }
            if should_recursive_check {
                for (key1, value1) in value1_dict {
                    if !value2_dict.contains_key(key1) {
                        continue;
                    }
                    let value2 = value2_dict.get(key1).unwrap();
                    if !value_subsume(value1, value2, should_recursive_check) {
                        return false;
                    }
                }
            }
            true
        }
        _ => false,
    }
}
