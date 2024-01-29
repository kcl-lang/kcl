//! Copyright The KCL Authors. All rights reserved.

use crate::*;

// common
impl ValueRef {
    pub fn kind(&self) -> Kind {
        match *self.rc.borrow() {
            Value::undefined => Kind::Undefined,
            Value::none => Kind::None,
            Value::bool_value(_) => Kind::Bool,
            Value::int_value(_) => Kind::Int,
            Value::float_value(_) => Kind::Float,
            Value::str_value(_) => Kind::Str,
            Value::list_value(_) => Kind::List,
            Value::dict_value(_) => Kind::Dict,
            Value::schema_value(_) => Kind::Schema,
            Value::func_value(_) => Kind::Func,
            Value::unit_value(..) => Kind::Unit,
        }
    }
}
