//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl Type {
    #[inline]
    pub fn any() -> Self {
        Type::Any
    }

    #[inline]
    pub fn func() -> Self {
        Type::Func(Default::default())
    }

    #[inline]
    pub fn bool() -> Self {
        Type::Bool
    }

    #[inline]
    pub fn bool_lit(v: bool) -> Self {
        Type::BoolLit(v)
    }

    #[inline]
    pub fn int() -> Self {
        Type::Int
    }

    #[inline]
    pub fn int_lit(v: i64) -> Self {
        Type::IntLit(v)
    }

    #[inline]
    pub fn float() -> Self {
        Type::Float
    }

    #[inline]
    pub fn float_lit(v: f64) -> Self {
        Type::FloatLit(v)
    }

    #[inline]
    pub fn str() -> Self {
        Type::Str
    }

    #[inline]
    pub fn str_lit(s: &str) -> Self {
        Type::StrLit(s.to_string())
    }

    #[inline]
    pub fn list(elem_type: &Self) -> Self {
        Type::List(ListType {
            elem_type: Box::new(elem_type.clone()),
        })
    }

    #[inline]
    pub fn dict(key_type: &Self, elem_type: &Self) -> Self {
        Type::Dict(DictType {
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
        Type::Union(ut)
    }

    #[inline]
    pub fn schema(
        name: String,
        attrs: IndexMap<String, Type>,
        has_index_signature: bool,
        func: ValueRef,
    ) -> Self {
        Type::Schema(SchemaType {
            name,
            attrs,
            has_index_signature,
            func,
        })
    }
}
