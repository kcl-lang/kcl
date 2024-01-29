//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match *self.rc.borrow() {
            Value::undefined => false,
            Value::none => false,
            Value::bool_value(ref v) => *v,
            Value::int_value(ref v) => *v != 0,
            Value::float_value(ref v) => *v != 0.0,
            Value::str_value(ref v) => !v.is_empty(),
            Value::list_value(ref v) => !v.values.is_empty(),
            Value::dict_value(ref v) => !v.values.is_empty(),
            Value::schema_value(ref v) => !v.config.values.is_empty(),
            Value::func_value(_) => true,
            Value::unit_value(ref v, _, _) => *v != 0.0,
        }
    }

    #[inline]
    pub fn logic_and(&self, x: &ValueRef) -> bool {
        self.is_truthy() && x.is_truthy()
    }

    #[inline]
    pub fn logic_or(&self, x: &ValueRef) -> bool {
        self.is_truthy() || x.is_truthy()
    }
}

#[cfg(test)]
mod test_value_logic {
    use crate::*;

    #[test]
    fn test_is_truthy() {
        let cases = [
            // false cases
            (ValueRef::int(1), true),
            (ValueRef::float(2.0f64), true),
            (ValueRef::bool(true), true),
            (ValueRef::str("s"), true),
            (ValueRef::list_int(&[0]), true),
            (ValueRef::dict_str(&[("key", "value")]), true),
            // true cases
            (ValueRef::undefined(), false),
            (ValueRef::none(), false),
            (ValueRef::int(0), false),
            (ValueRef::float(0.0f64), false),
            (ValueRef::bool(false), false),
            (ValueRef::str(""), false),
            (ValueRef::list(Some(&[])), false),
            (ValueRef::dict(Some(&[])), false),
        ];
        for (value, expected) in cases {
            let result = value.is_truthy();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_logic_and() {
        let cases = [
            // true cases
            (ValueRef::int(1), ValueRef::int(1), true),
            // false cases
            (ValueRef::int(0), ValueRef::int(0), false),
            (ValueRef::int(0), ValueRef::int(1), false),
            (ValueRef::int(1), ValueRef::int(0), false),
        ];
        for (left, right, expected) in cases {
            let result = left.logic_and(&right);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_logic_or() {
        let cases = [
            // true cases
            (ValueRef::int(1), ValueRef::int(1), true),
            (ValueRef::int(0), ValueRef::int(1), true),
            (ValueRef::int(1), ValueRef::int(0), true),
            // false cases
            (ValueRef::int(0), ValueRef::int(0), false),
        ];
        for (left, right, expected) in cases {
            let result = left.logic_or(&right);
            assert_eq!(result, expected);
        }
    }
}
