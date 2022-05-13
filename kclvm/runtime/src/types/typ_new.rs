// Copyright 2021 The KCL Authors. All rights reserved.

use crate::*;

impl Type {
    pub fn any() -> Self {
        Type::any_type
    }

    pub fn func() -> Self {
        Type::func_type(Default::default())
    }

    pub fn bool() -> Self {
        Type::bool_type
    }

    pub fn bool_lit(v: bool) -> Self {
        Type::bool_lit_type(v)
    }

    pub fn int() -> Self {
        Type::int_type
    }

    pub fn int_lit(v: i64) -> Self {
        Type::int_lit_type(v)
    }

    pub fn float() -> Self {
        Type::float_type
    }

    pub fn float_lit(v: f64) -> Self {
        Type::float_lit_type(v)
    }

    pub fn str() -> Self {
        Type::str_type
    }

    pub fn str_lit(s: &str) -> Self {
        Type::str_lit_type(s.to_string())
    }

    pub fn list(elem_type: &Self) -> Self {
        Type::list_type(ListType {
            elem_type: Box::new(elem_type.clone()),
        })
    }

    pub fn dict(key_type: &Self, elem_type: &Self) -> Self {
        Type::dict_type(DictType {
            key_type: Box::new(key_type.clone()),
            elem_type: Box::new(elem_type.clone()),
        })
    }

    pub fn union(elem_types: &[&Self]) -> Self {
        let mut ut: UnionType = Default::default();
        for typ in elem_types {
            ut.elem_types.push((*typ).clone());
        }
        Type::union_type(ut)
    }

    pub fn schema(
        name: &str,
        parent_name: &str,
        field_names: &[&str],
        field_types: &[&Self],
    ) -> Self {
        let mut st = SchemaType {
            name: name.to_string(),
            parent_name: parent_name.to_string(),
            ..Default::default()
        };

        for name in field_names {
            st.field_names.push((*name).to_string());
        }
        for typ in field_types {
            st.field_types.push((*typ).clone());
        }

        Type::schema_type(st)
    }
}
