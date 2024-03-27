use std::{mem::transmute_copy, os::raw::c_char};

use generational_arena::Index;

use crate::{
    kclvm_plugin_invoke, ptr_as_ref, schema_config_meta, BacktraceFrame, Context, SchemaTypeFunc,
    ValueRef,
};

impl ValueRef {
    /// Try get the proxy function index
    pub fn try_get_proxy(&self) -> Option<Index> {
        match &*self.rc.borrow() {
            crate::Value::func_value(func) => func.proxy.clone(),
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
    pkgpath: &str,
    is_in_schema: bool,
) -> ValueRef {
    if func.is_func() {
        let func = func.as_function();
        let fn_ptr = func.fn_ptr;
        let closure = &func.closure;
        let is_schema = !func.runtime_type.is_empty();
        if ctx.cfg.debug_mode {
            ctx.backtrace
                .push(BacktraceFrame::from_panic_info(&ctx.panic_info));
            ctx.panic_info.kcl_func = func.name.clone();
        }
        let now_meta_info = ctx.panic_info.clone();
        unsafe {
            let call_fn: SchemaTypeFunc = transmute_copy(&fn_ptr);
            // Call schema constructor twice
            let value = if is_schema {
                // Schema function closure
                let mut args_new = args.deep_copy();
                let mut closure_new = closure.deep_copy();
                let config_meta_index: isize = 1;
                let cal_map_index: isize = 5;
                let record_instance_index = closure.len() - 2;
                let instance_pkgpath_index = closure.len() - 1;
                args.list_append_unpack(closure);
                let args = args.clone().into_raw(ctx);
                call_fn(ctx, args, kwargs);
                let cal_map = closure.list_get(cal_map_index).unwrap();
                // is sub schema
                closure_new.list_set(0, &ValueRef::bool(true));
                // record instance
                closure_new.list_set(record_instance_index, &ValueRef::bool(true));
                // instance pkgpath
                closure_new.list_set(instance_pkgpath_index, &ValueRef::str(pkgpath));
                // cal map
                closure_new.list_set(cal_map_index as usize, &cal_map);
                // config meta
                let config_meta = schema_config_meta(
                    &ctx.panic_info.kcl_file,
                    ctx.panic_info.kcl_line as u64,
                    ctx.panic_info.kcl_col as u64,
                );
                closure_new.list_set(config_meta_index as usize, &config_meta);
                args_new.list_append_unpack(&closure_new);
                call_fn(ctx, args_new.into_raw(ctx), kwargs)
            // Normal kcl function, call directly
            } else if func.is_external {
                let name = format!("{}\0", func.name);
                kclvm_plugin_invoke(ctx, name.as_ptr() as *const c_char, args, kwargs)
            } else {
                args.list_append_unpack_first(closure);
                let args = args.clone().into_raw(ctx);
                call_fn(ctx, args, kwargs)
            };
            if is_schema && !is_in_schema {
                let schema_value = ptr_as_ref(value);
                schema_value.schema_check_attr_optional(ctx, true);
            }
            if ctx.cfg.debug_mode {
                ctx.backtrace.pop();
            }
            ctx.panic_info = now_meta_info;
            return ptr_as_ref(value).clone();
        };
    }
    ValueRef::none()
}
