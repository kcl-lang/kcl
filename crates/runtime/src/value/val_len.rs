//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    pub fn len(&self) -> usize {
        match *self.rc.borrow() {
            Value::str_value(ref s) => s.len(),
            Value::list_value(ref v) => v.values.len(),
            Value::dict_value(ref v) => v.values.len(),
            Value::schema_value(ref v) => v.config.values.len(),
            _ => panic!("object of type '{}' has no len()", self.type_str()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0_usize
    }
}

#[cfg(test)]
mod test_value_len {
    use crate::*;

    fn assert_panic<F: FnOnce() + std::panic::UnwindSafe>(func: F) {
        let result = std::panic::catch_unwind(func);
        assert!(result.is_err())
    }

    #[test]
    fn test_len() {
        let mut ctx = Context::new();
        assert_eq!(ValueRef::str("abc").len(), 3);
        assert_eq!(
            ValueRef::str("abc")
                .bin_aug_mul(&mut ctx, &ValueRef::int(10))
                .len(),
            3 * 10
        );
        assert_eq!(ValueRef::list_n(10, &ValueRef::undefined()).len(), 10);
        assert_eq!(ValueRef::list_int(&[1_i64, 2, 3]).len(), 3);
    }

    #[test]
    fn test_len_invalid() {
        assert_panic(|| {
            ValueRef::undefined().len();
        });
        assert_panic(|| {
            ValueRef::none().len();
        });
        assert_panic(|| {
            ValueRef::bool(false).len();
        });
        assert_panic(|| {
            ValueRef::int(1).len();
        });
        assert_panic(|| {
            ValueRef::float(2.0).len();
        });
    }
}
