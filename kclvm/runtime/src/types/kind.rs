//! Copyright The KCL Authors. All rights reserved.

use crate::*;

impl Type {
    pub fn kind(&self) -> Kind {
        match self {
            Type::Any => Kind::Any,
            Type::Bool => Kind::Bool,
            Type::BoolLit(..) => Kind::BoolLit,
            Type::Int => Kind::Int,
            Type::IntLit(..) => Kind::IntLit,
            Type::Float => Kind::Float,
            Type::FloatLit(..) => Kind::FloatLit,
            Type::Str => Kind::Str,
            Type::StrLit(..) => Kind::StrLit,
            Type::List(..) => Kind::List,
            Type::Dict(..) => Kind::Dict,
            Type::Union(..) => Kind::Union,
            Type::Schema(..) => Kind::Schema,
            Type::Func(..) => Kind::Func,
        }
    }
}
