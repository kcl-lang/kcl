//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    // +x
    pub fn unary_plus(&self) -> Self {
        match &*self.rc.borrow() {
            Value::int_value(ref a) => Self::int(*a),
            Value::float_value(ref a) => Self::float(*a),
            _ => panic!("bad operand type for unary +: '{}'", self.type_str()),
        }
    }

    // -x
    pub fn unary_minus(&self) -> Self {
        match &*self.rc.borrow() {
            Value::int_value(ref a) => Self::int(0 - *a),
            Value::float_value(ref a) => Self::float(0.0 - *a),
            _ => panic!("bad operand type for unary -: '{}'", self.type_str()),
        }
    }

    // ~ x
    pub fn unary_not(&self) -> Self {
        Self::int(!self.as_int())
    }

    // not x
    pub fn unary_l_not(&self) -> Self {
        Self::bool(!self.is_truthy())
    }
}

#[cfg(test)]
mod test_value_unary {
    use crate::*;

    #[test]
    fn test_unary_plus() {
        let cases = [(0, 0), (2, 2), (-2, -2)];
        for (value, expected) in cases {
            assert_eq!(ValueRef::int(value).unary_plus().as_int(), expected);
        }
    }

    #[test]
    fn test_unary_minus_not() {
        let cases = [(0, 0), (2, -2), (-2, 2)];
        for (value, expected) in cases {
            assert_eq!(ValueRef::int(value).unary_minus().as_int(), expected);
        }
    }

    #[test]
    fn test_unary_not() {
        let cases = [(0, -1), (-1, 0), (2, -3), (-3, 2), (0xFF, -256)];
        for (value, expected) in cases {
            assert_eq!(ValueRef::int(value).unary_not().as_int(), expected);
        }
    }

    #[test]
    fn test_unary_l_not() {
        let cases = [
            // true cases
            (ValueRef::undefined(), true),
            (ValueRef::none(), true),
            (ValueRef::int(0), true),
            (ValueRef::float(0.0f64), true),
            (ValueRef::bool(false), true),
            (ValueRef::str(""), true),
            (ValueRef::list(Some(&[])), true),
            (ValueRef::dict(Some(&[])), true),
            // false cases
            (ValueRef::int(1), false),
            (ValueRef::float(2.0f64), false),
            (ValueRef::bool(true), false),
            (ValueRef::str("s"), false),
            (ValueRef::list_int(&[0]), false),
            (ValueRef::dict_str(&[("key", "value")]), false),
        ];
        for (value, expected) in cases {
            let result = value.unary_l_not().as_bool();
            assert_eq!(result, expected);
        }
    }
}
