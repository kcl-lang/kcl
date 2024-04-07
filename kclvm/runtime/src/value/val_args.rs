//! Copyright The KCL Authors. All rights reserved.

use crate::*;

/// Adjust ValueRef when the value is used with Rust option.
/// If the value is KCL None or Undefined, mapping it to Rust None,
/// else mapping it to Some(value).
#[inline]
pub fn adjust_parameter(value: Option<&ValueRef>) -> Option<&ValueRef> {
    value.and_then(|v| {
        if v.is_none_or_undefined() {
            None
        } else {
            Some(v)
        }
    })
}

impl ValueRef {
    pub fn arg_0(&self) -> Option<Self> {
        self.arg_i(0)
    }

    pub fn arg_last(&self) -> Option<Self> {
        match *self.rc.borrow() {
            Value::list_value(ref list) => Some(list.values[list.values.len() - 1].clone()),
            _ => None,
        }
    }

    pub fn pop_arg_last(&self) -> Option<Self> {
        match *self.rc.borrow_mut() {
            Value::list_value(ref mut list) => list.values.pop(),
            _ => None,
        }
    }

    pub fn pop_arg_first(&self) -> Option<Self> {
        match *self.rc.borrow_mut() {
            Value::list_value(ref mut list) => {
                if !list.values.is_empty() {
                    Some(list.values.remove(0))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn args_len(&self) -> usize {
        match *self.rc.borrow() {
            Value::list_value(ref list) => list.values.len(),
            _ => 1,
        }
    }

    pub fn arg_i(&self, i: usize) -> Option<Self> {
        match *self.rc.borrow() {
            Value::list_value(ref list) => {
                if i < list.values.len() {
                    return Some(list.values[i].clone());
                }
                None
            }
            _ => None,
        }
    }

    pub fn arg_i_bool(&self, i: usize, default: Option<bool>) -> Option<bool> {
        if let Some(x) = self.arg_i(i) {
            match *x.rc.borrow() {
                Value::bool_value(v) => return Some(v),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn arg_i_int(&self, i: usize, default: Option<i64>) -> Option<i64> {
        if let Some(x) = self.arg_i(i) {
            match *x.rc.borrow() {
                Value::int_value(v) => return Some(v),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn arg_i_int_or_bool(&self, i: usize, default: Option<i64>) -> Option<i64> {
        if let Some(x) = self.arg_i(i) {
            match *x.rc.borrow() {
                Value::bool_value(v) => return Some(v as i64),
                Value::int_value(v) => return Some(v),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn arg_i_float(&self, i: usize, default: Option<f64>) -> Option<f64> {
        if let Some(x) = self.arg_i(i) {
            match *x.rc.borrow() {
                Value::float_value(v) => return Some(v),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn arg_i_num(&self, i: usize, default: Option<f64>) -> Option<f64> {
        if let Some(x) = self.arg_i(i) {
            match *x.rc.borrow() {
                Value::float_value(v) => return Some(v),
                Value::int_value(v) => return Some(v as f64),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn arg_i_str(&self, i: usize, default: Option<String>) -> Option<String> {
        if let Some(x) = self.arg_i(i) {
            match &*x.rc.borrow() {
                Value::str_value(s) => return Some(s.to_string()),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn arg_i_list(&self, i: usize) -> Option<Self> {
        if let Some(x) = self.arg_i(i) {
            return if x.is_list() { Some(x) } else { None };
        }
        None
    }

    pub fn arg_i_dict(&self, i: usize) -> Option<Self> {
        if let Some(x) = self.arg_i(i) {
            return if x.is_dict() { Some(x) } else { None };
        }
        None
    }

    pub fn kwarg(&self, name: &str) -> Option<Self> {
        match *self.rc.borrow() {
            Value::dict_value(ref dict) => dict.values.get(&name.to_string()).cloned(),
            _ => None,
        }
    }

    pub fn kwarg_bool(&self, name: &str, default: Option<bool>) -> Option<bool> {
        if let Some(x) = self.kwarg(name) {
            match *x.rc.borrow() {
                Value::bool_value(v) => return Some(v),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn kwarg_int(&self, name: &str, default: Option<i64>) -> Option<i64> {
        if let Some(x) = self.kwarg(name) {
            match *x.rc.borrow() {
                Value::int_value(v) => return Some(v),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn kwarg_float(&self, name: &str, default: Option<f64>) -> Option<f64> {
        if let Some(x) = self.kwarg(name) {
            match *x.rc.borrow() {
                Value::float_value(v) => return Some(v),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn kwarg_str(&self, name: &str, default: Option<String>) -> Option<String> {
        if let Some(x) = self.kwarg(name) {
            match &*x.rc.borrow() {
                Value::str_value(s) => return Some(s.to_string()),
                Value::none => return default,
                _ => return None,
            }
        }
        default
    }

    pub fn kwarg_list(&self, name: &str) -> Option<Self> {
        if let Some(x) = self.kwarg(name) {
            return if x.is_list() { Some(x) } else { None };
        }
        None
    }

    pub fn kwarg_dict(&self, name: &str) -> Option<Self> {
        if let Some(x) = self.kwarg(name) {
            return if x.is_dict() { Some(x) } else { None };
        }
        None
    }
}

/// Get value from arguments and keyword arguments.
pub(crate) fn get_call_arg(
    args: &ValueRef,
    kwargs: &ValueRef,
    index: usize,
    key: Option<&str>,
) -> Option<ValueRef> {
    if let Some(key) = key {
        if let Some(val) = kwargs.get_by_key(key) {
            return Some(val);
        }
    }
    if index < args.len() {
        return Some(args.list_get(index as isize).unwrap());
    }
    None
}

#[inline]
pub(crate) fn get_call_arg_str(
    args: &ValueRef,
    kwargs: &ValueRef,
    index: usize,
    key: Option<&str>,
) -> Option<String> {
    get_call_arg(args, kwargs, index, key).map(|v| v.as_str())
}

#[inline]
pub(crate) fn get_call_arg_bool(
    args: &ValueRef,
    kwargs: &ValueRef,
    index: usize,
    key: Option<&str>,
) -> Option<bool> {
    get_call_arg(args, kwargs, index, key).map(|v| v.as_bool())
}

#[cfg(test)]
mod test_value_args {
    use crate::*;

    #[test]
    fn test_value_args() {
        let args = ValueRef::list(Some(&[
            &ValueRef::int(0),
            &ValueRef::float(1.0),
            &ValueRef::str("ss"),
        ]));
        let arg_first = args.pop_arg_first().unwrap();
        let arg_last = args.pop_arg_last().unwrap();
        assert_eq!(arg_first, ValueRef::int(0));
        assert_eq!(arg_last, ValueRef::str("ss"));
        assert_eq!(args.arg_0().unwrap().clone(), ValueRef::float(1.0));
        assert_eq!(args.arg_i_float(0, Some(2.0)).unwrap(), 1.0);
    }

    #[test]
    fn test_value_kwargs() {
        let mut kwargs = ValueRef::dict(None);
        kwargs.dict_update_key_value("key1", ValueRef::int(1));
        kwargs.dict_update_key_value("key2", ValueRef::str("2"));
        assert_eq!(kwargs.kwarg_int("key1", Some(2)).unwrap(), 1);
        assert_eq!(
            kwargs.kwarg_str("key2", Some("ss".to_string())).unwrap(),
            "2"
        );
    }
}
