use std::{mem::transmute_copy, os::raw::c_char};

use generational_arena::Index;

use crate::{kclvm_plugin_invoke, ptr_as_ref, Context, SchemaTypeFunc, ValueRef};

impl ValueRef {
    /// Try get the proxy function index
    pub fn try_get_proxy(&self) -> Option<Index> {
        match &*self.rc.borrow() {
            crate::Value::func_value(func) => func.proxy,
            _ => None,
        }
    }
}

/// Invoke functions with arguments and keyword arguments.
pub fn invoke_function(
    func: &ValueRef,
    args: &mut ValueRef,
    kwargs: &ValueRef,
    ctx: &mut Context,
) -> ValueRef {
    if func.is_func() {
        let func = func.as_function();
        let fn_ptr = func.fn_ptr;
        let closure = &func.closure;
        unsafe {
            let call_fn: SchemaTypeFunc = transmute_copy(&fn_ptr);
            // Call schema constructor twice
            let value = if func.is_external {
                let name = format!("{}\0", func.name);
                kclvm_plugin_invoke(ctx, name.as_ptr() as *const c_char, args, kwargs)
            } else {
                args.list_append_unpack_first(closure);
                let args = args.clone().into_raw(ctx);
                call_fn(ctx, args, kwargs)
            };
            let value = ptr_as_ref(value);
            value.clone()
        }
    } else {
        ValueRef::undefined()
    }
}
