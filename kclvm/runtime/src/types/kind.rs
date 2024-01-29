//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl Type {
    pub fn kind(&self) -> Kind {
        match self {
            Type::any_type => Kind::Any,
            Type::bool_type => Kind::Bool,
            Type::bool_lit_type(..) => Kind::BoolLit,
            Type::int_type => Kind::Int,
            Type::int_lit_type(..) => Kind::IntLit,
            Type::float_type => Kind::Float,
            Type::float_lit_type(..) => Kind::FloatLit,
            Type::str_type => Kind::Str,
            Type::str_lit_type(..) => Kind::StrLit,
            Type::list_type(..) => Kind::List,
            Type::dict_type(..) => Kind::Dict,
            Type::union_type(..) => Kind::Union,
            Type::schema_type(..) => Kind::Schema,
            Type::func_type(..) => Kind::Func,
        }
    }
}
