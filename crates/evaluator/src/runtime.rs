use std::os::raw::c_char;
use std::{
    mem::transmute_copy,
    panic::{AssertUnwindSafe, catch_unwind},
};

use kclvm_runtime::{
    Context, SchemaTypeFunc, UnsafeWrapper, ValueRef, get_call_arg, is_runtime_catch_function,
    kclvm_plugin_invoke, ptr_as_ref,
};

use crate::Evaluator;

/// Invoke functions with arguments and keyword arguments.
pub fn invoke_function(
    s: &Evaluator,
    func: &ValueRef,
    args: &mut ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    if func.is_func() {
        let func = func.as_function();
        let fn_ptr = func.fn_ptr;
        let closure = &func.closure;
        if is_runtime_catch_function(fn_ptr) {
            let value = runtime_catch(s, args, kwargs);
            return value;
        } else {
            let ctx: &mut Context = &mut s.runtime_ctx.borrow_mut();
            unsafe {
                // Call schema constructor twice
                let value = if func.is_external {
                    let name = format!("{}\0", func.name);
                    kclvm_plugin_invoke(ctx, name.as_ptr() as *const c_char, args, kwargs)
                } else {
                    let call_fn: SchemaTypeFunc = transmute_copy(&fn_ptr);
                    args.list_append_unpack_first(closure);
                    let args = args.clone().into_raw(ctx);
                    call_fn(ctx, args, kwargs)
                };
                let value = ptr_as_ref(value);
                value.clone()
            }
        }
    } else {
        ValueRef::undefined()
    }
}

/// Executes the provided function and catches any potential runtime errors.
/// Returns undefined if execution is successful, otherwise returns an error
/// message in case of a runtime panic.
pub fn runtime_catch(s: &Evaluator, args: &ValueRef, kwargs: &ValueRef) -> ValueRef {
    if let Some(func) = get_call_arg(args, kwargs, 0, Some("func")) {
        let wrapper = UnsafeWrapper::new(|| {
            if let Some(proxy) = func.try_get_proxy() {
                let args = ValueRef::list(None);
                let kwargs = ValueRef::dict(None);
                s.invoke_proxy_function(proxy, &args, &kwargs);
            }
        });
        let result = catch_unwind(AssertUnwindSafe(|| unsafe {
            (wrapper.get())();
        }));
        return match result {
            Ok(_) => ValueRef::undefined(),
            Err(err) => ValueRef::str(&kclvm_error::err_to_str(err)),
        };
    }
    panic!("catch() takes exactly one argument (0 given)");
}
