//! Copyright The KCL Authors. All rights reserved.

use crate::*;
use std::cell::Ref;
use std::cell::RefMut;
impl ValueRef {
    #[inline]
    pub fn as_bool(&self) -> bool {
        self.is_truthy()
    }

    #[inline]
    pub fn as_int(&self) -> i64 {
        match *self.rc.borrow() {
            Value::int_value(ref v) => *v,
            Value::float_value(ref v) => *v as i64,
            Value::unit_value(ref v, _, _) => *v as i64,
            _ => 0,
        }
    }

    #[inline]
    pub fn as_float(&self) -> f64 {
        match *self.rc.borrow() {
            Value::int_value(ref v) => *v as f64,
            Value::float_value(ref v) => *v,
            Value::unit_value(ref v, _, _) => *v,
            _ => 0.0,
        }
    }

    #[inline]
    pub fn as_str(&self) -> String {
        match *self.rc.borrow() {
            Value::str_value(ref v) => v.clone(),
            _ => "".to_string(),
        }
    }

    #[inline]
    pub fn as_list_ref(&self) -> Ref<ListValue> {
        Ref::map(self.rc.borrow(), |val| match val {
            Value::list_value(ref v) => v.as_ref(),
            _ => panic!("invalid list value"),
        })
    }

    #[inline]
    pub fn as_list_mut_ref(&mut self) -> RefMut<ListValue> {
        RefMut::map(self.rc.borrow_mut(), |val| match val {
            Value::list_value(ref mut v) => v.as_mut(),
            _ => panic!("invalid list value"),
        })
    }

    #[inline]
    pub fn as_dict_ref(&self) -> Ref<DictValue> {
        Ref::map(self.rc.borrow(), |val| match val {
            Value::dict_value(ref v) => v.as_ref(),
            Value::schema_value(ref v) => v.config.as_ref(),
            _ => panic!("invalid dict value"),
        })
    }

    #[inline]
    pub fn as_dict_mut_ref(&mut self) -> RefMut<DictValue> {
        RefMut::map(self.rc.borrow_mut(), |val| match val {
            Value::dict_value(ref mut v) => v.as_mut(),
            Value::schema_value(ref mut v) => v.config.as_mut(),
            _ => panic!("invalid dict value"),
        })
    }

    #[inline]
    pub fn as_schema(&self) -> Ref<SchemaValue> {
        Ref::map(self.rc.borrow(), |val| match val {
            Value::schema_value(ref v) => v.as_ref(),
            _ => panic!("invalid schema value"),
        })
    }

    #[inline]
    pub fn as_function(&self) -> Ref<FuncValue> {
        Ref::map(self.rc.borrow(), |val| match val {
            Value::func_value(ref v) => v.as_ref(),
            _ => panic!("invalid func value"),
        })
    }

    #[inline]
    pub fn as_unit(&self) -> (f64, i64, String) {
        match &*self.rc.borrow() {
            Value::unit_value(v, raw, unit) => (*v, *raw, unit.clone()),
            _ => panic!("invalid unit value"),
        }
    }
}

#[cfg(test)]
mod test_value_as {
    use crate::*;

    #[test]
    fn test_as_bool() {
        let cases = [
            (ValueRef::undefined(), false),
            (ValueRef::none(), false),
            (ValueRef::bool(false), false),
            (ValueRef::bool(true), true),
            (ValueRef::int(0), false),
            (ValueRef::int(1), true),
            (ValueRef::int(-1), true),
            (ValueRef::int(2), true),
            (ValueRef::int(123), true),
            (ValueRef::float(0.0), false),
            (ValueRef::float(0.1), true),
            (ValueRef::float(1234.5), true),
            (ValueRef::str(""), false),
            (ValueRef::str("false"), true),
            (ValueRef::str("1"), true),
            (ValueRef::list_int(&[0]), true),
            (ValueRef::list(None), false),
            (ValueRef::dict_int(&[("k", 0)]), true),
            (ValueRef::dict(None), false),
            (ValueRef::schema(), false),
        ];
        for (value, expected) in cases {
            assert_eq!(value.as_bool(), expected);
        }
    }

    #[test]
    fn test_as_int() {
        let cases = [
            (ValueRef::int(0), 0),
            (ValueRef::int(1), 1),
            (ValueRef::int(-1), -1),
            (ValueRef::int(256), 256),
            (ValueRef::float(0.0), 0),
            (ValueRef::float(0.1), 0),
            (ValueRef::float(1234.5), 1234),
            (ValueRef::unit(1024.0, 1, "Ki"), 1024),
        ];
        for (value, expected) in cases {
            assert_eq!(value.as_int(), expected);
        }
    }

    #[test]
    fn test_as_float() {
        let cases = [
            (ValueRef::int(0), 0.0),
            (ValueRef::float(256.0), 256.0),
            (ValueRef::unit(1024.0, 1, "Ki"), 1024.0),
        ];
        for (value, expected) in cases {
            assert_eq!(value.as_float(), expected);
        }
    }

    #[test]
    fn test_as_str() {
        let cases = [
            (ValueRef::int(0), ""),
            (ValueRef::float(1234.5), ""),
            (ValueRef::str("ss"), "ss"),
        ];
        for (value, expected) in cases {
            assert_eq!(value.as_str(), expected);
        }
    }
}
