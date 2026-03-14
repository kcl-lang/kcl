use std::os::raw::c_char;
use std::{
    mem::transmute_copy,
    panic::{AssertUnwindSafe, catch_unwind},
};

use kcl_runtime::{
    Context, SchemaTypeFunc, UnsafeWrapper, ValueRef, get_call_arg, kcl_builtin_reduce,
    kcl_plugin_invoke, kcl_runtime_catch, ptr_as_ref,
};

use crate::Evaluator;

/// Macro to define evaluator builtins.
/// Each entry maps: stub function pointer => handler function
macro_rules! evaluator_builtins {
    ($ptr:expr; $($stub:path => $handler:expr),* $(,)?) => {{
        $(
            if $ptr == $stub as *const () as u64 {
                return Some($handler);
            }
        )*
        None
    }};
}

/// Get the handler for an evaluator builtin, if the pointer matches one.
#[inline]
fn get_evaluator_builtin_handler(
    ptr: u64,
) -> Option<fn(&Evaluator, &ValueRef, &ValueRef) -> ValueRef> {
    // All evaluator builtins defined in one place:
    evaluator_builtins!(ptr;
        kcl_runtime_catch => runtime_catch,
        kcl_builtin_reduce => runtime_reduce,
    )
}

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
        if let Some(handler) = get_evaluator_builtin_handler(fn_ptr) {
            handler(s, args, kwargs)
        } else {
            let ctx: &mut Context = &mut s.runtime_ctx.borrow_mut();
            // Call schema constructor twice
            let value = if func.is_external {
                let name = format!("{}\0", func.name);
                unsafe { kcl_plugin_invoke(ctx, name.as_ptr() as *const c_char, args, kwargs) }
            } else {
                let call_fn: SchemaTypeFunc = unsafe { transmute_copy(&fn_ptr) };
                args.list_append_unpack_first(closure);
                let args = args.clone().into_raw(ctx);
                unsafe { call_fn(ctx, args, kwargs) }
            };
            let value = unsafe { ptr_as_ref(value) };
            value.clone()
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
            Err(err) => ValueRef::str(&kcl_error::err_to_str(err)),
        };
    }
    panic!("catch() takes exactly one argument (0 given)");
}

/// Apply a reducer function to an initial value. Returns the result of the function.
/// The list parameter is currently ignored.
pub fn runtime_reduce(s: &Evaluator, args: &ValueRef, kwargs: &ValueRef) -> ValueRef {
    let reducer = get_call_arg(args, kwargs, 0, Some("reducer"));
    let _list = get_call_arg(args, kwargs, 1, Some("list"));
    let initial = get_call_arg(args, kwargs, 2, Some("initial"));

    // Validate arguments
    let reducer =
        reducer.unwrap_or_else(|| panic!("reduce() missing required argument: 'reducer'"));
    let initial =
        initial.unwrap_or_else(|| panic!("reduce() missing required argument: 'initial'"));
    let proxy = reducer
        .try_get_proxy()
        .unwrap_or_else(|| panic!("reduce() argument 'reducer' must be a function"));

    // Call reducer(initial, initial)
    let call_args = ValueRef::list(Some(&[&initial, &initial]));
    let call_kwargs = ValueRef::dict(None);
    s.invoke_proxy_function(proxy, &call_args, &call_kwargs)
}
