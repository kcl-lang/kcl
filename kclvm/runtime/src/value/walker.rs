use crate::{Value, ValueRef};

/// Walk the value recursively and deal the type using the `walk_fn`
pub fn walk_value(val: &ValueRef, walk_fn: &impl Fn(&ValueRef)) {
    walk_fn(val);
    match &*val.rc.borrow() {
        Value::list_value(list_value) => {
            for v in &list_value.values {
                walk_value(v, walk_fn);
            }
        }
        Value::dict_value(dict_value) => {
            for (_, v) in &dict_value.values {
                walk_value(v, walk_fn);
            }
        }
        Value::schema_value(schema_value) => {
            for (_, v) in &schema_value.config.values {
                walk_value(v, walk_fn);
            }
        }
        _ => {}
    }
}

/// Walk the value recursively and mutably and deal the type using the `walk_fn`
pub fn walk_value_mut(val: &ValueRef, walk_fn: &mut impl FnMut(&ValueRef)) {
    walk_fn(val);
    match &*val.rc.borrow() {
        Value::list_value(list_value) => {
            for v in &list_value.values {
                walk_value_mut(v, walk_fn);
            }
        }
        Value::dict_value(dict_value) => {
            for (_, v) in &dict_value.values {
                walk_value_mut(v, walk_fn);
            }
        }
        Value::schema_value(schema_value) => {
            for (_, v) in &schema_value.config.values {
                walk_value_mut(v, walk_fn);
            }
        }
        _ => {}
    }
}
