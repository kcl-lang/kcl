use std::collections::HashMap;

use crate::*;
use handlebars::{html_escape, Handlebars};

/// Applies a parsed template to the specified data object and
/// returns the string output.
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_template_execute(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(template) = get_call_arg_str(args, kwargs, 0, Some("template")) {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("template", template)
            .expect("register template failed");
        let data = get_call_arg(args, kwargs, 1, Some("data")).unwrap_or(ValueRef::dict(None));
        let data: HashMap<String, String> = HashMap::from_iter(
            data.as_dict_ref()
                .values
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        );
        let result = handlebars
            .render("template", &data)
            .expect("render template failed");
        return ValueRef::str(&result).into_raw(ctx);
    }
    panic!("execute() takes exactly one argument (0 given)");
}

/// Replaces the characters `&"<>` with the equivalent html / xml entities.
#[no_mangle]
#[runtime_fn]
pub extern "C" fn kclvm_template_html_escape(
    ctx: *mut kclvm_context_t,
    args: *const kclvm_value_ref_t,
    kwargs: *const kclvm_value_ref_t,
) -> *const kclvm_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(data) = get_call_arg_str(args, kwargs, 0, Some("data")) {
        return ValueRef::str(&html_escape(&data)).into_raw(ctx);
    }
    panic!("html_escape() takes exactly one argument (0 given)");
}
