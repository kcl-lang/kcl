// Copyright 2022 The KCL Authors. All rights reserved.

use crate::*;
use std::cell::RefCell;
use std::convert::{From, TryFrom};
use std::iter::FromIterator;
use std::rc::Rc;

impl ValueRef {
    pub fn into(&self) -> Self {
        self.clone()
    }
}

// basic type

macro_rules! define_value_from_trait {
    ($from_type: ty, $kcl_type :ident) => {
        impl From<$from_type> for ValueRef {
            fn from(v: $from_type) -> Self {
                ValueRef::$kcl_type(v)
            }
        }
    };
}

macro_rules! define_value_try_from_trait {
    ($for_type: ty, $kcl_type_value :ident, $for_type_name :expr) => {
        impl TryFrom<ValueRef> for $for_type {
            type Error = String;

            fn try_from(v: ValueRef) -> Result<Self, Self::Error> {
                match &*v.rc.borrow() {
                    Value::$kcl_type_value(v) => Ok(v.clone()),
                    _ => Err(format!("can't convert {} to {}", v, $for_type_name)),
                }
            }
        }
        impl TryFrom<&ValueRef> for $for_type {
            type Error = String;

            fn try_from(v: &ValueRef) -> Result<Self, Self::Error> {
                match &*v.rc.borrow() {
                    Value::$kcl_type_value(v) => Ok(v.clone()),
                    _ => Err(format!("can't convert {} to {}", v, $for_type_name)),
                }
            }
        }
    };
}

macro_rules! define_value_try_into_method {
    ($try_into_type: ident, $type: ty) => {
        impl ValueRef {
            pub fn $try_into_type(&self) -> Result<$type, String> {
                use std::convert::TryInto;
                self.try_into()
            }
        }
    };
}

define_value_from_trait!(bool, bool);
define_value_from_trait!(i64, int);
define_value_from_trait!(f64, float);
define_value_from_trait!(&str, str);

define_value_try_into_method!(try_into_bool, bool);
define_value_try_into_method!(try_into_int, i64);
define_value_try_into_method!(try_into_float, f64);
define_value_try_into_method!(try_into_str, String);

define_value_try_from_trait!(bool, bool_value, "bool");
define_value_try_from_trait!(i64, int_value, "i64");
define_value_try_from_trait!(f64, float_value, "f64");
define_value_try_from_trait!(String, str_value, "String");

// value

impl From<Value> for ValueRef {
    fn from(v: Value) -> Self {
        Self {
            rc: Rc::new(RefCell::new(v)),
        }
    }
}

// list

macro_rules! define_value_list_from_iter_trait {
    ($elem_type: ty) => {
        impl FromIterator<$elem_type> for ValueRef {
            fn from_iter<I: IntoIterator<Item = $elem_type>>(iter: I) -> Self {
                let mut list: ListValue = Default::default();
                for i in iter {
                    list.values.push(i.into());
                }
                Self::from(Value::list_value(Box::new(list)))
            }
        }
    };
    ($_ref_: ident, $elem_type: ty) => {
        impl<'a> FromIterator<&'a $elem_type> for ValueRef {
            fn from_iter<I: IntoIterator<Item = &'a $elem_type>>(iter: I) -> Self {
                let mut list: ListValue = Default::default();
                for i in iter {
                    list.values.push(i.into());
                }
                Self::from(Value::list_value(Box::new(list)))
            }
        }
    };
}

define_value_list_from_iter_trait!(bool);
define_value_list_from_iter_trait!(i64);
define_value_list_from_iter_trait!(f64);

define_value_list_from_iter_trait!(ref, str);
define_value_list_from_iter_trait!(ref, ValueRef);

define_value_try_from_trait!(Box<ListValue>, list_value, "ListValue");

define_value_try_into_method!(try_into_list, Box<ListValue>);

// dict

macro_rules! define_value_dict_from_iter_trait {
    ($elem_type: ty) => {
        impl<'a> FromIterator<(&'a str, $elem_type)> for ValueRef {
            fn from_iter<I: IntoIterator<Item = (&'a str, $elem_type)>>(iter: I) -> Self {
                let mut dict: DictValue = Default::default();
                for (k, v) in iter {
                    dict.values.insert(k.to_string(), v.into());
                }
                Self::from(Value::dict_value(Box::new(dict)))
            }
        }
    };
    ($_ref_: ident, $elem_type: ty) => {
        impl<'a> FromIterator<(&'a str, &'a $elem_type)> for ValueRef {
            fn from_iter<I: IntoIterator<Item = (&'a str, &'a $elem_type)>>(iter: I) -> Self {
                let mut dict: DictValue = Default::default();
                for (k, v) in iter {
                    dict.values.insert(k.to_string(), v.into());
                }
                Self::from(Value::dict_value(Box::new(dict)))
            }
        }
    };
}

define_value_dict_from_iter_trait!(bool);
define_value_dict_from_iter_trait!(i64);
define_value_dict_from_iter_trait!(f64);
define_value_dict_from_iter_trait!(ref, str);
define_value_dict_from_iter_trait!(ref, ValueRef);

define_value_try_from_trait!(Box<DictValue>, dict_value, "DictValue");

define_value_try_into_method!(try_into_dict, Box<DictValue>);

// schema

define_value_try_from_trait!(Box<SchemaValue>, schema_value, "SchemaValue");

#[cfg(test)]
mod tests_from {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_sample() {
        use std::convert::From;

        assert_eq!(ValueRef::undefined(), ValueRef::from(UNDEFINED));
        assert_eq!(ValueRef::none(), ValueRef::from(NONE));
        assert_eq!(ValueRef::bool(true), ValueRef::from(TRUE));
        assert_eq!(ValueRef::bool(false), ValueRef::from(FALSE));

        assert!(ValueRef::from(true).try_into_bool().unwrap());
        assert_eq!(123, ValueRef::from(123).try_into_int().unwrap());
        assert_eq!(1.5, ValueRef::from(1.5).try_into_float().unwrap());
        assert_eq!("abc", ValueRef::from("abc").try_into_str().unwrap());

        assert!(bool::try_from(ValueRef::from(true)).unwrap());
        assert_eq!(123, i64::try_from(ValueRef::from(123)).unwrap());
        assert_eq!(1.5, f64::try_from(ValueRef::from(1.5)).unwrap());
        assert_eq!("abc", String::try_from(ValueRef::from("abc")).unwrap());
    }

    macro_rules! test_x_type {
        ($test_fn_name: ident, $kcl_type :ident, $tests: expr) => {
            #[test]
            fn $test_fn_name() {
                let tests = $tests;
                for v in tests {
                    let expect = ValueRef::$kcl_type(v);
                    let got: ValueRef = v.into();
                    assert_eq!(expect, got);
                }
            }
        };
    }

    test_x_type!(test_bool, bool, vec![true, false]);
    test_x_type!(test_int, int, vec![-1, 0, 1, 123, 0xFFFFFFFF + 1]);
    test_x_type!(
        test_float,
        float,
        vec![0.0, 1.5, 123.0, 0xFFFFFFFFi64 as f64 + 1.0]
    );

    test_x_type!(test_str, str, vec!["", "abc", "123"]);

    #[test]
    fn test_list() {
        let list = vec![true, false];

        let list_value: ValueRef = ValueRef::from_iter(list.clone());
        assert_eq!(list.len(), list_value.len());

        for (i, v) in list.iter().enumerate() {
            let x: bool = list_value.list_get(i as isize).unwrap().as_bool();
            assert_eq!(*v, x);
        }
    }

    #[test]
    fn test_list2() {
        let list = vec![1, 2, 4, 3];

        let list_value: ValueRef = ValueRef::from_iter(list.clone());
        let list_value: Box<ListValue> = list_value.try_into().unwrap();

        assert_eq!(
            list_value
                .values
                .iter()
                .map(|x| i64::try_from(x).unwrap())
                .collect::<Vec<i64>>(),
            list,
        );
    }

    macro_rules! test_try_into {
        ($test_fn_name: ident, $type: ty, $tests: expr) => {
            #[test]
            fn $test_fn_name() {
                for v in $tests {
                    let v0 = v;
                    let v1: ValueRef = v0.into();
                    let v2: $type = v1.try_into().unwrap();
                    assert_eq!(v0, v2);
                }
            }
        };
    }

    test_try_into!(test_try_into_bool, bool, [true, false, true, true]);
    test_try_into!(test_try_into_i64, i64, [1, 2, 3, -1]);
    test_try_into!(test_try_into_f64, f64, [1.5, 2.0]);
    test_try_into!(test_try_into_str, String, ["", "abc"]);
}
