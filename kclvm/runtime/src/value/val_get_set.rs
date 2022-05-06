// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    pub fn get_by_key(&self, key: &str) -> Option<&Self> {
        match &*self.rc {
            Value::list_value(ref list) => match key.parse::<usize>() {
                Ok(i) => list.values.as_slice().get(i),
                Err(_) => None,
            },
            Value::dict_value(ref dict) => dict.values.get(key),
            Value::schema_value(ref schema) => schema.config.values.get(key),
            _ => None,
        }
    }

    pub fn get_mut_by_key(&mut self, key: &str) -> Option<&mut Self> {
        match &*self.rc {
            Value::list_value(ref list) => match key.parse::<usize>() {
                Ok(i) => {
                    let list: &mut ListValue = get_ref_mut(list);
                    return list.values.as_mut_slice().get_mut(i);
                }
                Err(_) => None,
            },
            Value::dict_value(ref dict) => {
                let dict: &mut DictValue = get_ref_mut(dict);
                dict.values.get_mut(key)
            }
            Value::schema_value(ref schema) => {
                let schema: &mut SchemaValue = get_ref_mut(schema);
                let dict: &mut DictValue = get_ref_mut(schema.config.as_ref());
                dict.values.get_mut(key)
            }
            _ => None,
        }
    }

    pub fn get_by_path(&self, path: &str) -> Option<&Self> {
        let mut val: &Self = self;
        for key in path.split('.') {
            match val.get_by_key(key) {
                Some(x) => {
                    val = x;
                }
                None => {
                    return None;
                }
            }
        }
        Some(val)
    }

    pub fn get_mut(&mut self, path: &str) -> Option<&Self> {
        let mut val: &mut Self = self;
        for key in path.split('.') {
            match val.get_mut_by_key(key) {
                Some(x) => {
                    val = x;
                }
                None => return None,
            }
        }
        Some(val)
    }
}

#[cfg(test)]
mod test_value_get {
    use crate::*;

    #[test]
    fn test_get() {
        let mut list_int = ValueRef::list_int(&vec![10 as i64, 20, 30]);

        let mut dict = ValueRef::dict(None);
        dict.dict_insert("a", &ValueRef::str("a-value"), Default::default(), 0);
        dict.dict_insert("b", &ValueRef::str("b-value"), Default::default(), 0);

        list_int.list_set(1, &dict);
        list_int.list_set(2, &ValueRef::list_int(&vec![100 as i64, 200, 300]));

        assert_eq!(list_int.get_by_path("1.a").unwrap().as_str(), "a-value");

        assert_eq!(list_int.get_by_path("2.2").unwrap().as_int(), 300);

        let dict = ValueRef::dict(Some(&vec![
            ("aaa", &ValueRef::int(111)),
            (
                "bbb",
                &ValueRef::list(Some(&vec![
                    &ValueRef::str("a"),
                    &ValueRef::str("b"),
                    &ValueRef::dict(Some(&vec![("key0", &ValueRef::int(12345))])),
                ])),
            ),
        ]));

        assert_eq!(dict.get_by_path("aaa").unwrap().as_int(), 111);
        assert_eq!(dict.get_by_path("bbb.1").unwrap().as_str(), "b");
        assert_eq!(dict.get_by_path("bbb.2.key0").unwrap().as_int(), 12345);
    }
}
