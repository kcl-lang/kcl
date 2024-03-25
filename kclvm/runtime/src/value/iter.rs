//! Copyright The KCL Authors. All rights reserved.

use crate::*;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ValueIterator {
    pub len: usize,
    pub cur_key: ValueRef,
    pub cur_val: ValueRef,
    pub end_val: *const ValueRef,
    pub keys: Vec<String>,
    pub pos: i32,
}

impl Default for ValueIterator {
    fn default() -> Self {
        Self {
            len: 0,
            cur_key: Default::default(),
            cur_val: Default::default(),
            end_val: std::ptr::null(),
            keys: Vec::new(),
            pos: 0,
        }
    }
}

impl ValueIterator {
    pub fn from_value(p: &ValueRef) -> Self {
        if !p.is_str() && !p.is_list() && !p.is_config() {
            panic!("'{}' object is not iterable", p.type_str());
        }
        if p.is_empty() {
            return Default::default();
        }
        match *p.rc.borrow() {
            Value::str_value(ref s) => {
                ValueIterator {
                    len: s.len(),
                    cur_key: Default::default(),
                    cur_val: Default::default(),
                    end_val: 1 as *const ValueRef, // just as bool flag
                    keys: Vec::new(),
                    pos: 0,
                }
            }
            Value::list_value(ref list) => {
                ValueIterator {
                    len: list.values.len(),
                    cur_key: Default::default(),
                    cur_val: Default::default(),
                    end_val: 1 as *const ValueRef, // just as bool flag
                    keys: Vec::new(),
                    pos: 0,
                }
            }
            Value::dict_value(ref dict) => {
                let keys: Vec<String> = dict.values.keys().map(|s| (*s).clone()).collect();
                ValueIterator {
                    len: dict.values.len(),
                    cur_key: Default::default(),
                    cur_val: Default::default(),
                    end_val: 1 as *const ValueRef, // just as bool flag
                    keys,
                    pos: 0,
                }
            }

            Value::schema_value(ref schema) => {
                let keys: Vec<String> = schema.config.values.keys().map(|s| (*s).clone()).collect();
                ValueIterator {
                    len: schema.config.values.len(),
                    cur_key: Default::default(),
                    cur_val: Default::default(),
                    end_val: 1 as *const ValueRef, // just as bool flag
                    keys,
                    pos: 0,
                }
            }

            _ => Default::default(),
        }
    }

    pub fn is_end(&self) -> bool {
        self.pos >= self.len as i32
    }

    pub fn key(&self) -> Option<&ValueRef> {
        if self.pos == 0 || self.pos > self.len as i32 {
            return Option::None;
        }
        if !self.end_val.is_null() {
            Some(&self.cur_key)
        } else {
            Option::None
        }
    }

    pub fn value(&mut self) -> Option<&ValueRef> {
        if self.pos == 0 {
            return Option::None;
        }
        if !self.end_val.is_null() {
            Some(&self.cur_val)
        } else {
            Option::None
        }
    }

    /// Get the next value, iterate key and value of the iterator.
    pub fn next_with_key_value<'a>(
        &'a mut self,
        host: &'a ValueRef,
    ) -> Option<(ValueRef, ValueRef, ValueRef)> {
        let next_value = self.next(host);
        match next_value {
            Some(v) => Some((v.clone(), self.cur_key.clone(), self.cur_val.clone())),
            None => None,
        }
    }

    /// Get the next value reference of the iterator.
    pub fn next<'a>(&'a mut self, host: &'a ValueRef) -> Option<&'a ValueRef> {
        if host.is_empty() {
            return None;
        }
        if self.pos >= host.len() as i32 {
            self.end_val = std::ptr::null();
            return None;
        }
        match *host.rc.borrow() {
            Value::str_value(ref s) => {
                let ch = s.chars().nth(self.pos as usize).unwrap();
                self.cur_key = ValueRef::int(self.pos as i64);
                self.cur_val = ValueRef::str(&ch.to_string());
                self.end_val = &self.cur_val;
                self.pos += 1;
                Some(&self.cur_val)
            }
            Value::list_value(ref list) => {
                self.cur_key = ValueRef::int(self.pos as i64);
                self.cur_val = list.values[self.pos as usize].clone();
                self.end_val = &self.cur_val;
                self.pos += 1;
                Some(&self.cur_val)
            }
            Value::dict_value(ref dict) => {
                let key = &self.keys[self.pos as usize];
                self.cur_key = ValueRef::str(key);
                self.cur_val = dict.values[key].clone();
                self.end_val = &self.cur_val;
                self.pos += 1;
                Some(&self.cur_key)
            }
            Value::schema_value(ref schema) => {
                let key = &self.keys[self.pos as usize];
                self.cur_key = ValueRef::str(key);
                self.cur_val = schema.config.values[key].clone();
                self.end_val = &self.cur_val;
                self.pos += 1;
                Some(&self.cur_key)
            }
            _ => panic!("{} object is not iterable", host.type_str()),
        }
    }
}

impl ValueRef {
    #[inline]
    pub fn iter(&self) -> ValueIterator {
        ValueIterator::from_value(self)
    }
}

#[cfg(test)]
mod test_value_iter {
    use crate::*;

    #[test]
    fn test_str_iter() {
        let s = ValueRef::str("abc");

        let mut it = s.iter();
        assert!(!it.is_end());

        let _ = it.next(&s);
        assert_eq!(it.key().unwrap().as_int(), 0);
        assert_eq!(it.value().unwrap().as_str(), "a");

        let _ = it.next(&s);
        assert!(!it.is_end());
        assert_eq!(it.key().unwrap().as_int(), 1);
        assert_eq!(it.value().unwrap().as_str(), "b");

        let v = it.next(&s);
        assert_eq!(v.unwrap().as_str(), "c");
        assert_eq!(it.key().unwrap().as_int(), 2);
        assert_eq!(it.value().unwrap().as_str(), "c");
        assert!(it.is_end());

        let _ = it.next(&s);
        assert!(it.is_end());
    }

    #[test]
    fn test_list_iter() {
        let value = ValueRef::list_int(&[1, 2, 3]);

        let mut it = value.iter();
        assert!(!it.is_end());

        let _ = it.next(&value);
        assert_eq!(it.key().unwrap().as_int(), 0);
        assert_eq!(it.value().unwrap().as_int(), 1);

        let _ = it.next(&value);
        assert!(!it.is_end());
        assert_eq!(it.key().unwrap().as_int(), 1);
        assert_eq!(it.value().unwrap().as_int(), 2);

        let _ = it.next(&value);
        assert_eq!(it.key().unwrap().as_int(), 2);
        assert_eq!(it.value().unwrap().as_int(), 3);
        assert!(it.is_end());

        let _ = it.next(&value);
        assert!(it.is_end());
    }

    #[test]
    fn test_dict_iter() {
        let value = ValueRef::dict_int(&[("a", 1), ("b", 2), ("c", 3)]);

        let mut it = value.iter();
        assert!(!it.is_end());

        let _ = it.next(&value);
        assert_eq!(it.key().unwrap().as_str(), "a");
        assert_eq!(it.value().unwrap().as_int(), 1);

        let _ = it.next(&value);
        assert!(!it.is_end());
        assert_eq!(it.key().unwrap().as_str(), "b");
        assert_eq!(it.value().unwrap().as_int(), 2);

        let _ = it.next(&value);
        assert_eq!(it.key().unwrap().as_str(), "c");
        assert_eq!(it.value().unwrap().as_int(), 3);
        assert!(it.is_end());

        let _ = it.next(&value);
        assert!(it.is_end());
    }
}
