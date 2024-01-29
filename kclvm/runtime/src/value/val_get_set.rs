//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl ValueRef {
    pub fn get_by_key(&self, key: &str) -> Option<Self> {
        match &*self.rc.borrow() {
            Value::list_value(ref list) => match key.parse::<usize>() {
                Ok(i) => list.values.as_slice().get(i).cloned(),
                Err(_) => None,
            },
            Value::dict_value(ref dict) => dict.values.get(key).cloned(),
            Value::schema_value(ref schema) => schema.config.values.get(key).cloned(),
            _ => None,
        }
    }

    pub fn get_by_path(&self, path: &str) -> Option<Self> {
        let mut val: Self = self.clone();
        for key in path.split('.') {
            match val.get_by_key(key) {
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
        let mut list_int = ValueRef::list_int(&[10_i64, 20, 30]);
        let mut ctx = Context::new();
        let mut dict = ValueRef::dict(None);
        dict.dict_insert(
            &mut ctx,
            "a",
            &ValueRef::str("a-value"),
            Default::default(),
            0,
        );
        dict.dict_insert(
            &mut ctx,
            "b",
            &ValueRef::str("b-value"),
            Default::default(),
            0,
        );

        list_int.list_set(1, &dict);
        list_int.list_set(2, &ValueRef::list_int(&[100_i64, 200, 300]));

        assert_eq!(list_int.get_by_path("1.a").unwrap().as_str(), "a-value");

        assert_eq!(list_int.get_by_path("2.2").unwrap().as_int(), 300);

        let dict = ValueRef::dict(Some(&[
            ("aaa", &ValueRef::int(111)),
            (
                "bbb",
                &ValueRef::list(Some(&[
                    &ValueRef::str("a"),
                    &ValueRef::str("b"),
                    &ValueRef::dict(Some(&[("key0", &ValueRef::int(12345))])),
                ])),
            ),
        ]));

        assert_eq!(dict.get_by_path("aaa").unwrap().as_int(), 111);
        assert_eq!(dict.get_by_path("bbb.1").unwrap().as_str(), "b");
        assert_eq!(dict.get_by_path("bbb.2.key0").unwrap().as_int(), 12345);
    }
}
