//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl Type {
    #[inline]
    pub fn any() -> Self {
        Type::any_type
    }

    #[inline]
    pub fn func() -> Self {
        Type::func_type(Default::default())
    }

    #[inline]
    pub fn bool() -> Self {
        Type::bool_type
    }

    #[inline]
    pub fn bool_lit(v: bool) -> Self {
        Type::bool_lit_type(v)
    }

    #[inline]
    pub fn int() -> Self {
        Type::int_type
    }

    #[inline]
    pub fn int_lit(v: i64) -> Self {
        Type::int_lit_type(v)
    }

    #[inline]
    pub fn float() -> Self {
        Type::float_type
    }

    #[inline]
    pub fn float_lit(v: f64) -> Self {
        Type::float_lit_type(v)
    }

    #[inline]
    pub fn str() -> Self {
        Type::str_type
    }

    #[inline]
    pub fn str_lit(s: &str) -> Self {
        Type::str_lit_type(s.to_string())
    }

    #[inline]
    pub fn list(elem_type: &Self) -> Self {
        Type::list_type(ListType {
            elem_type: Box::new(elem_type.clone()),
        })
    }

    #[inline]
    pub fn dict(key_type: &Self, elem_type: &Self) -> Self {
        Type::dict_type(DictType {
            key_type: Box::new(key_type.clone()),
            elem_type: Box::new(elem_type.clone()),
        })
    }

    #[inline]
    pub fn union(elem_types: &[&Self]) -> Self {
        let mut ut: UnionType = Default::default();
        for typ in elem_types {
            ut.elem_types.push((*typ).clone());
        }
        Type::union_type(ut)
    }

    #[inline]
    pub fn schema(
        name: String,
        attrs: IndexMap<String, Type>,
        has_index_signature: bool,
        func: ValueRef,
    ) -> Self {
        Type::schema_type(SchemaType {
            name,
            attrs,
            has_index_signature,
            func,
        })
    }
}
