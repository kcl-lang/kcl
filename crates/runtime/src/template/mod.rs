use std::collections::HashMap;

use crate::*;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderErrorReason,
    html_escape,
};

/// Custom helper that renders a value without HTML escaping.
/// Usage: {{raw var}}
fn raw_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h
        .param(0)
        .ok_or(RenderErrorReason::Other("Parameter not found".into()))?;
    let value = param.value();

    // Output the raw JSON value without HTML escaping
    if let Some(s) = value.as_str() {
        out.write(s)?;
    } else {
        out.write(&value.to_string())?;
    }
    Ok(())
}

/// Applies a parsed template to the specified data object and
/// returns the string output.
/// # Safety
/// The caller must ensure that `ctx`, `template_str`, and `data` are valid
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_template_execute(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    if let Some(template) = get_call_arg_str(args, kwargs, 0, Some("template")) {
        let mut handlebars = Handlebars::new();

        // Register helper for raw (unescaped) output
        handlebars.register_helper("raw", Box::new(raw_helper));

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
/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_template_html_escape(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    if let Some(data) = get_call_arg_str(args, kwargs, 0, Some("data")) {
        return ValueRef::str(&html_escape(&data)).into_raw(ctx);
    }
    panic!("html_escape() takes exactly one argument (0 given)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_helper_with_html() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("raw", Box::new(raw_helper));

        let template = r#"{{raw value}}"#.to_string();
        handlebars
            .register_template_string("test", template)
            .expect("register template failed");

        let mut data = serde_json::map::Map::new();
        data.insert(
            "value".to_string(),
            serde_json::Value::String("<div>test</div>".to_string()),
        );

        let result = handlebars.render("test", &data).expect("render failed");
        assert_eq!(result, "<div>test</div>");
    }

    #[test]
    fn test_raw_helper_with_quotes() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("raw", Box::new(raw_helper));

        let template = r#"{{raw value}}"#.to_string();
        handlebars
            .register_template_string("test", template)
            .expect("register template failed");

        let mut data = serde_json::map::Map::new();
        data.insert(
            "value".to_string(),
            serde_json::Value::String(r#"timeout: "5m""#.to_string()),
        );

        let result = handlebars.render("test", &data).expect("render failed");
        assert_eq!(result, r#"timeout: "5m""#);
    }

    #[test]
    fn test_raw_helper_with_yaml_like_content() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("raw", Box::new(raw_helper));

        let template = r#"{{raw value}}"#.to_string();
        handlebars
            .register_template_string("test", template)
            .expect("register template failed");

        let yaml_content = r#"config:
  timeout: "5m""#;
        let mut data = serde_json::map::Map::new();
        data.insert(
            "value".to_string(),
            serde_json::Value::String(yaml_content.to_string()),
        );

        let result = handlebars.render("test", &data).expect("render failed");
        assert_eq!(result, yaml_content);
    }

    #[test]
    fn test_normal_template_still_escapes() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("raw", Box::new(raw_helper));

        // Test that normal {{var}} still escapes
        let template = r#"{{value}}"#.to_string();
        handlebars
            .register_template_string("test", template)
            .expect("register template failed");

        let mut data = serde_json::map::Map::new();
        data.insert(
            "value".to_string(),
            serde_json::Value::String("<div>test</div>".to_string()),
        );

        let result = handlebars.render("test", &data).expect("render failed");
        // Normal syntax should escape
        assert_eq!(result, "&lt;div&gt;test&lt;/div&gt;");
    }

    #[test]
    fn test_triple_brace_syntax() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("raw", Box::new(raw_helper));

        // Test standard Handlebars triple brace syntax
        let template = r#"{{{value}}}"#.to_string();
        handlebars
            .register_template_string("test", template)
            .expect("register template failed");

        let mut data = serde_json::map::Map::new();
        data.insert(
            "value".to_string(),
            serde_json::Value::String("<div>test</div>".to_string()),
        );

        let result = handlebars.render("test", &data).expect("render failed");
        // Triple braces should not escape
        assert_eq!(result, "<div>test</div>");
    }

    #[test]
    fn test_raw_vs_normal_syntax() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("raw", Box::new(raw_helper));

        let template = r#"normal: {{value}}, raw: {{raw value}}, triple: {{{value}}}"#.to_string();
        handlebars
            .register_template_string("test", template)
            .expect("register template failed");

        let mut data = serde_json::map::Map::new();
        data.insert(
            "value".to_string(),
            serde_json::Value::String(r#""test""#.to_string()),
        );

        let result = handlebars.render("test", &data).expect("render failed");
        // normal should escape quotes to &quot;, raw and triple should not
        assert_eq!(
            result,
            r#"normal: &quot;test&quot;, raw: "test", triple: "test""#
        );
    }
}
