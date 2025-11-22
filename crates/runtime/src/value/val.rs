//! Copyright The KCL Authors. All rights reserved.

use generational_arena::Index;

use crate::*;

impl ValueRef {
    pub fn undefined() -> Self {
        Self::from(UNDEFINED)
    }

    pub fn none() -> Self {
        Self::from(NONE)
    }

    pub fn bool(v: bool) -> Self {
        Self::from(if v { TRUE } else { FALSE })
    }

    pub fn int(v: i64) -> Self {
        Self::from(Value::int_value(v))
    }

    pub fn float(v: f64) -> Self {
        Self::from(Value::float_value(v))
    }

    pub fn unit(v: f64, raw: i64, unit: &str) -> Self {
        Self::from(Value::unit_value(v, raw, unit.to_string()))
    }

    pub fn str(v: &str) -> Self {
        Self::from(Value::str_value(v.to_string()))
    }

    pub fn list(values: Option<&[&Self]>) -> Self {
        let mut list: ListValue = Default::default();
        if let Some(values) = values {
            for x in values {
                list.values.push((**x).clone());
            }
        }
        Self::from(Value::list_value(Box::new(list)))
    }

    pub fn list_value(values: Option<&[Self]>) -> Self {
        let mut list: ListValue = Default::default();
        if let Some(values) = values {
            for x in values {
                list.values.push((*x).clone());
            }
        }
        Self::from(Value::list_value(Box::new(list)))
    }

    pub fn dict(values: Option<&[(&str, &Self)]>) -> Self {
        let mut dict: DictValue = Default::default();
        if let Some(values) = values {
            for x in values {
                dict.values.insert(x.0.to_string(), (*x.1).clone());
            }
        }
        Self::from(Value::dict_value(Box::new(dict)))
    }

    pub fn schema() -> Self {
        let s: SchemaValue = Default::default();
        Self::from(Value::schema_value(Box::new(s)))
    }

    pub fn func(
        fn_ptr: u64,
        check_fn_ptr: u64,
        closure: ValueRef,
        name: &str,
        runtime_type: &str,
        is_external: bool,
    ) -> Self {
        Self::from(Value::func_value(Box::new(FuncValue {
            fn_ptr,
            check_fn_ptr,
            closure,
            name: name.to_string(),
            runtime_type: runtime_type.to_string(),
            is_external,
            proxy: None,
        })))
    }

    /// New a proxy function with function index in the function list.
    pub fn proxy_func(proxy: Index) -> Self {
        Self::from(Value::func_value(Box::new(FuncValue {
            fn_ptr: 0,
            check_fn_ptr: 0,
            closure: ValueRef::undefined(),
            name: "".to_string(),
            runtime_type: "".to_string(),
            is_external: false,
            proxy: Some(proxy),
        })))
    }

    /// New a proxy function with function index and the runtime type in the function list.
    pub fn proxy_func_with_type(proxy: Index, runtime_type: &str) -> Self {
        Self::from(Value::func_value(Box::new(FuncValue {
            fn_ptr: 0,
            check_fn_ptr: 0,
            closure: ValueRef::undefined(),
            name: "".to_string(),
            runtime_type: runtime_type.to_string(),
            is_external: false,
            proxy: Some(proxy),
        })))
    }
}
