//! Copyright The KCL Authors. All rights reserved.

use crate::*;

// cmp
impl ValueRef {
    pub fn cmp_equal(&self, x: &Self) -> bool {
        match *self.rc.borrow() {
            Value::int_value(a) => match *x.rc.borrow() {
                Value::int_value(b) => a == b,
                Value::float_value(b) => a as f64 == b,
                _ => false,
            },
            Value::float_value(a) => match *x.rc.borrow() {
                Value::int_value(b) => a == b as f64,
                Value::float_value(b) => a == b,
                _ => false,
            },
            _ => match (&*self.rc.borrow(), &*x.rc.borrow()) {
                (Value::undefined, Value::undefined) => true,
                (Value::none, Value::none) => true,
                (Value::bool_value(a), Value::bool_value(b)) => *a == *b,
                (Value::str_value(a), Value::str_value(b)) => *a == *b,
                (Value::list_value(a), Value::list_value(b)) => {
                    if a.values.len() != b.values.len() {
                        return false;
                    }
                    for i in 0..a.values.len() {
                        if !a.values[i].cmp_equal(&b.values[i]) {
                            return false;
                        }
                    }
                    true
                }
                (Value::dict_value(a), Value::dict_value(b)) => {
                    if a.values.len() != b.values.len() {
                        return false;
                    }
                    for (k, v) in a.values.iter() {
                        if !b.values.contains_key(k) {
                            return false;
                        }
                        if !v.cmp_equal(&b.values[k]) {
                            return false;
                        }
                    }
                    true
                }
                (Value::schema_value(a), Value::schema_value(b)) => {
                    if a.config.values.len() != b.config.values.len() {
                        return false;
                    }
                    for (k, v) in a.config.values.iter() {
                        if !b.config.values.contains_key(k) {
                            return false;
                        }
                        if !v.cmp_equal(&b.config.values[k]) {
                            return false;
                        }
                    }
                    true
                }
                (Value::func_value(a), Value::func_value(b)) => a.fn_ptr == b.fn_ptr,
                _ => false,
            },
        }
    }

    pub fn cmp_not_equal(&self, x: &Self) -> bool {
        !self.cmp_equal(x)
    }

    pub fn cmp_less_than(&self, x: &Self) -> bool {
        match &*self.rc.borrow() {
            Value::int_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a < *b,
                Value::float_value(b) => (*a as f64) < *b,
                Value::bool_value(b) => *a < (*b as i64),
                _ => panic!(
                    "'<' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::float_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a < *b as f64,
                Value::float_value(b) => *a < *b,
                Value::bool_value(b) => *a < ((*b as i64) as f64),
                _ => panic!(
                    "'<' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::bool_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => (*a as i64) < *b,
                Value::float_value(b) => ((*a as i64) as f64) < *b,
                Value::bool_value(b) => !(*a) & *b,
                _ => panic!(
                    "'<' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::str_value(a) => match &*x.rc.borrow() {
                Value::str_value(b) => *a < *b,
                _ => panic!(
                    "'<' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::list_value(a) => match &*x.rc.borrow() {
                Value::list_value(b) => {
                    let len_a = a.values.len();
                    let len_b = b.values.len();
                    let len_min = if len_a >= len_b { len_b } else { len_a };
                    for i in 0..len_min {
                        let value1 = &a.values[i];
                        let value2 = &b.values[i];
                        if !value1.cmp_equal(value2) {
                            return value1.cmp_less_than(value2);
                        }
                    }
                    len_a < len_b
                }
                _ => panic!(
                    "'<' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            _ => panic!(
                "'<' not supported between instances of '{}' and '{}'",
                self.type_str(),
                x.type_str()
            ),
        }
    }

    pub fn cmp_less_than_or_equal(&self, x: &Self) -> bool {
        match &*self.rc.borrow() {
            Value::int_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a <= *b,
                Value::float_value(b) => (*a as f64) <= *b,
                Value::bool_value(b) => *a <= (*b as i64),
                _ => panic!(
                    "'<=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::float_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a <= *b as f64,
                Value::float_value(b) => *a <= *b,
                Value::bool_value(b) => *a <= ((*b as i64) as f64),
                _ => panic!(
                    "'<=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::bool_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => (*a as i64) <= *b,
                Value::float_value(b) => ((*a as i64) as f64) <= *b,
                Value::bool_value(b) => *a <= *b,
                _ => panic!(
                    "'<=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::str_value(a) => match &*x.rc.borrow() {
                Value::str_value(b) => *a <= *b,
                _ => panic!(
                    "'<=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::list_value(a) => match &*x.rc.borrow() {
                Value::list_value(b) => {
                    let len_a = a.values.len();
                    let len_b = b.values.len();
                    let len_min = if len_a >= len_b { len_b } else { len_a };
                    for i in 0..len_min {
                        let value1 = &a.values[i];
                        let value2 = &b.values[i];
                        if !value1.cmp_equal(value2) {
                            return value1.cmp_less_than_or_equal(value2);
                        }
                    }
                    len_a <= len_b
                }
                _ => panic!(
                    "'<=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            _ => panic!(
                "'<=' not supported between instances of '{}' and '{}'",
                self.type_str(),
                x.type_str()
            ),
        }
    }

    pub fn cmp_greater_than(&self, x: &Self) -> bool {
        match &*self.rc.borrow() {
            Value::int_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a > *b,
                Value::float_value(b) => (*a as f64) > *b,
                Value::bool_value(b) => *a > (*b as i64),
                _ => panic!(
                    "'>' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::float_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a > *b as f64,
                Value::float_value(b) => *a > *b,
                Value::bool_value(b) => *a > ((*b as i64) as f64),
                _ => panic!(
                    "'>' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::bool_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => (*a as i64) > *b,
                Value::float_value(b) => ((*a as i64) as f64) > *b,
                Value::bool_value(b) => *a & !(*b),
                _ => panic!(
                    "'>' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::str_value(a) => match &*x.rc.borrow() {
                Value::str_value(b) => *a > *b,
                _ => panic!(
                    "'>' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::list_value(a) => match &*x.rc.borrow() {
                Value::list_value(b) => {
                    let len_a = a.values.len();
                    let len_b = b.values.len();
                    let len_min = if len_a >= len_b { len_b } else { len_a };
                    for i in 0..len_min {
                        let value1 = &a.values[i];
                        let value2 = &b.values[i];
                        if !value1.cmp_equal(value2) {
                            return value1.cmp_greater_than(value2);
                        }
                    }
                    len_a > len_b
                }
                _ => panic!(
                    "'>' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            _ => panic!(
                "'>' not supported between instances of '{}' and '{}'",
                self.type_str(),
                x.type_str()
            ),
        }
    }

    pub fn cmp_greater_than_or_equal(&self, x: &Self) -> bool {
        match &*self.rc.borrow() {
            Value::int_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a >= *b,
                Value::float_value(b) => (*a as f64) >= *b,
                Value::bool_value(b) => *a >= (*b as i64),
                _ => panic!(
                    "'>=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::float_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => *a >= *b as f64,
                Value::float_value(b) => *a >= *b,
                Value::bool_value(b) => *a >= ((*b as i64) as f64),
                _ => panic!(
                    "'>=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::bool_value(a) => match &*x.rc.borrow() {
                Value::int_value(b) => (*a as i64) >= *b,
                Value::float_value(b) => ((*a as i64) as f64) >= *b,
                Value::bool_value(b) => *a >= *b,
                _ => panic!(
                    "'>=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::str_value(a) => match &*x.rc.borrow() {
                Value::str_value(b) => *a >= *b,
                _ => panic!(
                    "'>=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            Value::list_value(a) => match &*x.rc.borrow() {
                Value::list_value(b) => {
                    let len_a = a.values.len();
                    let len_b = b.values.len();
                    let len_min = if len_a >= len_b { len_b } else { len_a };
                    for i in 0..len_min {
                        let value1 = &a.values[i];
                        let value2 = &b.values[i];
                        if !value1.cmp_equal(value2) {
                            return value1.cmp_greater_than_or_equal(value2);
                        }
                    }
                    len_a >= len_b
                }
                _ => panic!(
                    "'>=' not supported between instances of '{}' and '{}'",
                    self.type_str(),
                    x.type_str()
                ),
            },
            _ => panic!(
                "'>=' not supported between instances of '{}' and '{}'",
                self.type_str(),
                x.type_str()
            ),
        }
    }
}

#[cfg(test)]
mod test_value_cmp {
    use crate::*;

    #[test]
    fn test_eq() {
        let cases = [
            // true cases
            (ValueRef::int(123), ValueRef::int(123), true),
            (ValueRef::int(123), ValueRef::float(123.0), true),
            (ValueRef::str("abc"), ValueRef::str("abc"), true),
            (ValueRef::bool(true), ValueRef::bool(true), true),
            (
                ValueRef::list_int(&[1, 2, 3]),
                ValueRef::list_int(&[1, 2, 3]),
                true,
            ),
            (
                ValueRef::dict_int(&[("k1", 1), ("k2", 2)]),
                ValueRef::dict_int(&[("k1", 1), ("k2", 2)]),
                true,
            ),
            // false cases
            (ValueRef::int(123), ValueRef::int(1234), false),
            (ValueRef::int(123), ValueRef::float(1234.0), false),
            (ValueRef::str("abc"), ValueRef::str("abcd"), false),
            (ValueRef::bool(true), ValueRef::bool(false), false),
            (
                ValueRef::list_int(&[1, 2, 3]),
                ValueRef::list_int(&[2, 3, 4]),
                false,
            ),
            (
                ValueRef::dict_int(&[("k1", 1), ("k2", 2)]),
                ValueRef::dict_int(&[("1", 1), ("2", 2)]),
                false,
            ),
        ];
        for (left, right, expected) in cases {
            assert_eq!(left.cmp_equal(&right), expected);
        }
    }

    #[test]
    fn test_ne() {
        let cases = [
            // true cases
            (ValueRef::int(123), ValueRef::int(1234), true),
            (ValueRef::int(123), ValueRef::float(1234.0), true),
            (ValueRef::str("abc"), ValueRef::str("abcd"), true),
            // false cases
            (ValueRef::int(123), ValueRef::int(123), false),
            (ValueRef::int(123), ValueRef::float(123.0), false),
            (ValueRef::str("abc"), ValueRef::str("abc"), false),
        ];
        for (left, right, expected) in cases {
            assert_eq!(left.cmp_not_equal(&right), expected);
        }
    }

    #[test]
    fn test_cmp() {
        let cases = [
            // >
            (ValueRef::int(123), ValueRef::int(12), ">", true),
            (ValueRef::int(1234), ValueRef::float(123.0), ">", true),
            (ValueRef::str("abc"), ValueRef::str("ab"), ">", true),
            (ValueRef::bool(true), ValueRef::bool(false), ">", true),
            (
                ValueRef::list_int(&[1, 2, 3]),
                ValueRef::list_int(&[1, 1, 3]),
                ">",
                true,
            ),
            // >=
            (ValueRef::int(123), ValueRef::int(12), ">=", true),
            (ValueRef::int(1234), ValueRef::float(123.0), ">=", true),
            (ValueRef::str("abc"), ValueRef::str("ab"), ">=", true),
            (ValueRef::bool(true), ValueRef::bool(false), ">=", true),
            (
                ValueRef::list_int(&[1, 2]),
                ValueRef::list_int(&[1]),
                ">=",
                true,
            ),
            // <
            (ValueRef::int(123), ValueRef::int(12), "<", false),
            (ValueRef::int(1234), ValueRef::float(123.0), "<", false),
            (ValueRef::str("abc"), ValueRef::str("ab"), "<", false),
            (ValueRef::bool(true), ValueRef::bool(false), "<", false),
            (
                ValueRef::list_int(&[1, 2, 3]),
                ValueRef::list_int(&[1, 1, 3]),
                "<",
                false,
            ),
            // <=
            (ValueRef::int(123), ValueRef::int(12), "<=", false),
            (ValueRef::int(1234), ValueRef::float(123.0), "<=", false),
            (ValueRef::str("abc"), ValueRef::str("ab"), "<=", false),
            (ValueRef::bool(true), ValueRef::bool(false), "<=", false),
            (
                ValueRef::list_int(&[1, 2, 3]),
                ValueRef::list_int(&[1, 1, 3]),
                "<=",
                false,
            ),
        ];
        for (left, right, op, expected) in cases {
            match op {
                ">" => assert_eq!(left.cmp_greater_than(&right), expected),
                ">=" => assert_eq!(left.cmp_greater_than_or_equal(&right), expected),
                "<" => assert_eq!(left.cmp_less_than(&right), expected),
                "<=" => assert_eq!(left.cmp_less_than_or_equal(&right), expected),
                _ => panic!("invalid op {}", op),
            }
        }
    }
}
