//! Copyright The KCL Authors. All rights reserved.

use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;

use crate::*;

impl ValueRef {
    pub fn deep_copy(&self) -> ValueRef {
        match &*self.rc.borrow() {
            Value::undefined => ValueRef {
                rc: Rc::new(RefCell::new(Value::undefined)),
            },
            Value::none => ValueRef {
                rc: Rc::new(RefCell::new(Value::none)),
            },
            Value::func_value(ref v) => ValueRef {
                rc: Rc::new(RefCell::new(Value::func_value(Box::new(FuncValue {
                    fn_ptr: v.fn_ptr,
                    check_fn_ptr: v.check_fn_ptr,
                    // In KCL, functions are all pure, so we only need a shallow
                    // copy of the closure of the function.
                    // In addition, this can avoid stack overflow issues caused
                    // by deep copies of references to schema `self` held by functions
                    // within the schema. Because schema also holds a reference to
                    // the function.
                    closure: v.closure.clone(),
                    name: v.name.clone(),
                    runtime_type: v.runtime_type.clone(),
                    is_external: v.is_external,
                    proxy: v.proxy.clone(),
                })))),
            },
            Value::bool_value(ref v) => ValueRef {
                rc: Rc::new(RefCell::new(Value::bool_value(*v))),
            },
            Value::int_value(ref v) => ValueRef {
                rc: Rc::new(RefCell::new(Value::int_value(*v))),
            },
            Value::float_value(ref v) => ValueRef {
                rc: Rc::new(RefCell::new(Value::float_value(*v))),
            },
            Value::unit_value(ref v, ref raw, ref unit) => ValueRef {
                rc: Rc::new(RefCell::new(Value::unit_value(*v, *raw, unit.clone()))),
            },
            Value::str_value(ref v) => ValueRef {
                rc: Rc::new(RefCell::new(Value::str_value(v.to_string()))),
            },
            Value::list_value(ref v) => ValueRef {
                rc: Rc::new(RefCell::new(Value::list_value(Box::new(ListValue {
                    values: v.values.iter().map(|x| x.deep_copy()).collect(),
                })))),
            },
            Value::dict_value(ref v) => {
                let mut dict = ValueRef::from(Value::dict_value(Box::new(DictValue::new(&[]))));
                for (key, val) in &v.values {
                    let op = v.ops.get(key).unwrap_or(&ConfigEntryOperationKind::Union);
                    let index = v.insert_indexs.get(key).unwrap_or(&-1);
                    dict.dict_update_entry(
                        key.as_str(),
                        &val.deep_copy(),
                        &op.clone(),
                        &index.clone(),
                    );
                }
                dict.set_potential_schema_type(&v.potential_schema.clone().unwrap_or_default());
                dict
            }
            Value::schema_value(ref v) => {
                let mut dict = ValueRef::from(Value::dict_value(Box::new(DictValue::new(&[]))));
                dict.set_potential_schema_type(
                    &v.config.potential_schema.clone().unwrap_or_default(),
                );
                for (key, val) in &v.config.values {
                    let op = v
                        .config
                        .ops
                        .get(key)
                        .unwrap_or(&ConfigEntryOperationKind::Union);
                    let index = v.config.insert_indexs.get(key).unwrap_or(&-1);
                    dict.dict_update_entry(
                        key.as_str(),
                        &val.deep_copy(),
                        &op.clone(),
                        &index.clone(),
                    );
                    if let Some(type_str) = v.config.attr_map.get(key) {
                        dict.update_attr_map(key, type_str);
                    }
                }
                return ValueRef {
                    rc: Rc::new(RefCell::new(Value::schema_value(Box::new(SchemaValue {
                        name: v.name.clone(),
                        pkgpath: v.pkgpath.clone(),
                        config: Box::new(dict.as_dict_ref().clone()),
                        config_keys: v.config_keys.clone(),
                        config_meta: v.config_meta.clone(),
                        optional_mapping: v.optional_mapping.clone(),
                        // For KCL, args and kwargs are both immutable within the schema scope,
                        // so here we only need to clone the references.
                        args: v.args.clone(),
                        kwargs: v.kwargs.clone(),
                    })))),
                };
            }
        }
    }
}

#[cfg(test)]
mod test_value_deep_copy {
    use crate::*;

    #[test]
    fn test_deep_copy() {
        let values = [
            ValueRef::int(123),
            ValueRef::float(123.0),
            ValueRef::str("abc"),
            ValueRef::bool(true),
            ValueRef::list_int(&[1, 2, 3]),
            ValueRef::dict_int(&[("k1", 1), ("k2", 2)]),
        ];
        for value in values {
            let value_deep_copy = value.deep_copy();
            assert_eq!(value_deep_copy, value);
        }
    }
}
