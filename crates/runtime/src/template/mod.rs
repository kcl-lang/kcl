use std::collections::HashMap;

use crate::*;
use handlebars::{Handlebars, html_escape};

/// Applies a parsed template to the specified data object and
/// returns the string output.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn kcl_template_execute(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(template) = get_call_arg_str(args, kwargs, 0, Some("template")) {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("template", template)
            .expect("register template failed");
        let data = get_call_arg(args, kwargs, 1, Some("data")).unwrap_or(ValueRef::dict(None));
        let data: HashMap<String, JsonValue> = HashMap::from_iter(
            data.as_dict_ref()
                .values
                .iter()
                .map(|(k, v)| (k.to_string(), v.build_json(&Default::default()))),
        );
        let result = handlebars
            .render("template", &data)
            .expect("render template failed");
        return ValueRef::str(&result).into_raw(ctx);
    }
    panic!("execute() takes exactly one argument (0 given)");
}

/// Replaces the characters `&"<>` with the equivalent html / xml entities.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn kcl_template_html_escape(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = ptr_as_ref(args);
    let kwargs = ptr_as_ref(kwargs);
    let ctx = mut_ptr_as_ref(ctx);

    if let Some(data) = get_call_arg_str(args, kwargs, 0, Some("data")) {
        return ValueRef::str(&html_escape(&data)).into_raw(ctx);
    }
    panic!("html_escape() takes exactly one argument (0 given)");
}
