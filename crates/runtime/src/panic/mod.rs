use std::{any::Any, mem::transmute_copy, os::raw::c_char};

use std::cell::UnsafeCell;
use std::panic::AssertUnwindSafe;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::panic::catch_unwind;

use crate::*;

/// Executes the provided function and catches any potential runtime errors.
/// Returns undefined if execution is successful, otherwise returns an error
/// message in case of a runtime panic.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn kcl_runtime_catch(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(func) = get_call_arg(args, kwargs, 0, Some("func")) {
        let func = func.as_function();
        if ctx.cfg.debug_mode {
            ctx.backtrace
                .push(BacktraceFrame::from_panic_info(&ctx.panic_info));
            ctx.panic_info.kcl_func = func.name.clone();
        }
        let now_meta_info = ctx.panic_info.clone();
        let fn_ptr = func.fn_ptr;
        let wrapper = UnsafeWrapper::new(|| {
            let args = ValueRef::list(None).into_raw(ctx);
            let kwargs = ValueRef::dict(None).into_raw(ctx);
            unsafe {
                let call_fn: SchemaTypeFunc = transmute_copy(&fn_ptr);
                // Call schema constructor twice
                if func.is_external {
                    let name = format!("{}\0", func.name);
                    kcl_plugin_invoke(ctx, name.as_ptr() as *const c_char, args, kwargs)
                } else {
                    call_fn(ctx, args, kwargs)
                };
            };
        });
        let result = catch_unwind(AssertUnwindSafe(|| unsafe {
            (wrapper.get())();
        }));
        if ctx.cfg.debug_mode {
            ctx.backtrace.pop();
        }
        ctx.panic_info = now_meta_info;
        return match result {
            Ok(_) => ValueRef::undefined(),
            Err(err) => ValueRef::str(&err_to_str(err)),
        }
        .into_raw(ctx);
    }
    panic!("catch() takes exactly one argument (0 given)");
}

#[inline]
pub fn is_runtime_catch_function(ptr: u64) -> bool {
    ptr == kcl_runtime_catch as *const () as u64
}

/// Convert an error to string.
pub fn err_to_str(err: Box<dyn Any + Send>) -> String {
    if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<&String>() {
        (*s).clone()
    } else if let Some(s) = err.downcast_ref::<String>() {
        (*s).clone()
    } else {
        "".to_string()
    }
}

/// A wrapper struct that holds a value of type T inside an UnsafeCell.
/// UnsafeCell is the core primitive for interior mutability in Rust.
pub struct UnsafeWrapper<T> {
    value: UnsafeCell<T>,
}

impl<T> UnsafeWrapper<T> {
    /// Creates a new instance of UnsafeWrapper<T> with the provided value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be wrapped inside an UnsafeCell.
    ///
    /// # Returns
    ///
    /// A new instance of UnsafeWrapper containing the provided value.
    pub fn new(value: T) -> Self {
        UnsafeWrapper {
            value: UnsafeCell::new(value),
        }
    }

    /// Provides a mutable reference to the inner value.
    ///
    /// # Safety
    ///
    /// This is an unsafe function because obtaining multiple mutable references
    /// can lead to undefined behavior. The caller must ensure that the returned
    /// reference does not violate Rust's borrowing rules.
    ///
    /// # Returns
    ///
    /// A mutable reference to the inner value of type T.
    pub unsafe fn get(&self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }
}

// Implementing the UnwindSafe and RefUnwindSafe traits for UnsafeWrapper<T>
// to ensure it can be safely used across panic boundaries.
impl<T> UnwindSafe for UnsafeWrapper<T> {}
impl<T> RefUnwindSafe for UnsafeWrapper<T> {}
