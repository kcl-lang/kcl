/* Value methods */

use generational_arena::Index;
use kclvm_runtime::ValueRef;

use crate::Evaluator;

impl<'ctx> Evaluator<'ctx> {
    /// Construct a 64-bit int value using i64
    #[inline]
    pub(crate) fn int_value(&self, v: i64) -> ValueRef {
        ValueRef::int(v)
    }

    /// Construct a 64-bit float value using f64
    #[inline]
    pub(crate) fn float_value(&self, v: f64) -> ValueRef {
        ValueRef::float(v)
    }

    /// Construct a string value using &str
    #[inline]
    pub(crate) fn string_value(&self, v: &str) -> ValueRef {
        ValueRef::str(v)
    }

    /// Construct a bool value
    #[inline]
    pub(crate) fn bool_value(&self, v: bool) -> ValueRef {
        ValueRef::bool(v)
    }

    /// Construct a None value
    #[inline]
    pub(crate) fn none_value(&self) -> ValueRef {
        ValueRef::none()
    }

    /// Construct a Undefined value
    #[inline]
    pub(crate) fn undefined_value(&self) -> ValueRef {
        ValueRef::undefined()
    }

    /// Construct a empty kcl list value
    #[inline]
    pub(crate) fn list_value(&self) -> ValueRef {
        ValueRef::list(None)
    }

    /// Construct a list value with `n` elements
    pub(crate) fn _list_values(&self, values: &[&ValueRef]) -> ValueRef {
        ValueRef::list(Some(values))
    }

    /// Construct a empty kcl dict value.
    #[inline]
    pub(crate) fn dict_value(&self) -> ValueRef {
        ValueRef::dict(None)
    }

    /// Construct a unit value
    #[inline]
    pub(crate) fn unit_value(&self, v: f64, raw: i64, unit: &str) -> ValueRef {
        ValueRef::unit(v, raw, unit)
    }

    /// Construct a function value using a native function.
    #[inline]
    pub(crate) fn proxy_function_value(&self, proxy: Index) -> ValueRef {
        ValueRef::proxy_func(proxy)
    }

    /// Construct a function value using a native function.
    #[inline]
    pub(crate) fn proxy_function_value_with_type(
        &self,
        proxy: Index,
        runtime_type: &str,
    ) -> ValueRef {
        ValueRef::proxy_func_with_type(proxy, runtime_type)
    }

    /// Construct a function value using a native function.
    #[inline]
    pub(crate) fn function_value_with_ptr(&self, function_ptr: u64) -> ValueRef {
        ValueRef::func(function_ptr, 0, self.list_value(), "", "", false)
    }
}
