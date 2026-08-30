//! Copyright The KCL Authors. All rights reserved.

use crate::*;

pub const DEPRECATED_DECORATOR: &str = "deprecated";
pub const DEPRECATED_INFO: &str = "info";
/// Recognised value of `@info(type=...)` that marks a schema attribute
/// as an XML attribute (rather than a child element) for downstream
/// XML emitters. See `PlanOptions::emit_attribute_metadata`.
pub const INFO_ROLE_XML_ATTR: &str = "attr";

impl DecoratorValue {
    pub fn new(name: &str, args: &ValueRef, kwargs: &ValueRef) -> DecoratorValue {
        DecoratorValue {
            name: name.to_string(),
            args: args.clone(),
            kwargs: kwargs.clone(),
        }
    }

    pub fn run(
        &self,
        ctx: &mut Context,
        attr_name: &str,
        schema_name: &str,
        is_schema_target: bool,
        config_value: &ValueRef,
        config_meta: &ValueRef,
    ) {
        let filename = config_meta.get_by_key(CONFIG_META_FILENAME);
        let line = config_meta.get_by_key(CONFIG_META_LINE);
        match self.name.as_str() {
            DEPRECATED_DECORATOR => {
                let version = self.kwargs.kwarg("version");
                let reason = self.kwargs.kwarg("reason");
                let strict = self.kwargs.kwarg("strict");
                let version = if let Some(v) = version {
                    v.as_str()
                } else {
                    "".to_string()
                };
                let reason = if let Some(v) = reason {
                    v.as_str()
                } else {
                    "".to_string()
                };
                let strict = if let Some(v) = strict {
                    v.as_bool()
                } else {
                    true
                };
                let mut msg = String::new();
                if !version.is_empty() {
                    let version = format!("since version {version}");
                    msg.push_str(&version);
                }
                if !reason.is_empty() {
                    let reason = format!(", {reason}");
                    msg.push_str(&reason);
                }
                if strict {
                    if is_schema_target || config_value.get_by_key(attr_name).is_some() {
                        let mut err_msg = format!("{attr_name} was deprecated ");
                        if !msg.is_empty() {
                            err_msg.push_str(&msg);
                        }
                        if let (Some(filename), Some(line)) = (filename, line) {
                            ctx.set_kcl_filename(&filename.as_str());
                            ctx.panic_info.kcl_line = line.as_int() as i32;
                        }
                        ctx.set_err_type(&RuntimeErrorType::Deprecated);

                        panic!("{}", err_msg)
                    }
                } else if is_schema_target || config_value.get_by_key(attr_name).is_some() {
                    let mut err_msg = format!("{attr_name} was deprecated ");
                    if !msg.is_empty() {
                        err_msg.push_str(&msg);
                    }
                    ctx.set_err_type(&RuntimeErrorType::DeprecatedWarning);
                    ctx.set_warning_message(err_msg.as_str());
                } else {
                    let err_msg = format!("{attr_name} was deprecated ");
                    ctx.set_err_type(&RuntimeErrorType::DeprecatedWarning);
                    ctx.set_warning_message(err_msg.as_str());
                }
            }
            DEPRECATED_INFO => {
                // `@info` is free-form metadata. Only the `type=attr` role
                // is recognised by the runtime, and it is recorded for
                // the planner to emit the XML-attribute side channel.
                // Any other (or absent) value is silently ignored so
                // existing usages of `@info` keep working unchanged.
                if let Some(v) = self.kwargs.kwarg("type") {
                    let role = v.as_str();
                    if role == INFO_ROLE_XML_ATTR
                        && !schema_name.is_empty()
                        && !attr_name.is_empty()
                    {
                        ctx.attr_decorator_meta
                            .entry(schema_name.to_string())
                            .or_default()
                            .insert(attr_name.to_string(), role.to_string());
                    }
                }
            }
            _ => {
                let msg = format!("Unknown decorator {}", self.name);
                panic!("{}", msg);
            }
        };
    }

    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}

#[cfg(test)]
mod test_value_decorator {
    use crate::*;

    fn assert_panic<F: FnOnce() + std::panic::UnwindSafe>(func: F) {
        let result = std::panic::catch_unwind(func);
        assert!(result.is_err())
    }

    #[test]
    fn test_decorator() {
        let mut ctx = Context::new();
        let args = ValueRef::list(None);
        let mut kwargs = ValueRef::dict(None);
        let test_deprecated_decorator = DecoratorValue::new(DEPRECATED_DECORATOR, &args, &kwargs);
        kwargs.dict_update_key_value("strict", ValueRef::bool(false));
        let schema_name = "Data";
        let config_meta = ValueRef::dict(None);
        let config_value = ValueRef::dict_str(&[("key1", "value1")]);
        test_deprecated_decorator.run(
            &mut ctx,
            "attr1",
            schema_name,
            true,
            &config_value,
            &config_meta,
        );
    }

    #[test]
    fn test_decorator_invalid() {
        assert_panic(|| {
            let mut ctx = Context::new();
            let args = ValueRef::list(None);
            let kwargs = ValueRef::dict(None);
            let test_deprecated_decorator =
                DecoratorValue::new(DEPRECATED_DECORATOR, &args, &kwargs);
            let schema_name = "Data";
            let config_meta = ValueRef::dict(None);
            let config_value = ValueRef::dict_str(&[("key1", "value1")]);
            test_deprecated_decorator.run(
                &mut ctx,
                "attr1",
                schema_name,
                true,
                &config_value,
                &config_meta,
            );
        });
    }

    #[test]
    fn test_info_attr_populates_registry() {
        let mut ctx = Context::new();
        let args = ValueRef::list(None);
        let mut kwargs = ValueRef::dict(None);
        kwargs.dict_update_key_value("type", ValueRef::str(INFO_ROLE_XML_ATTR));
        let decorator = DecoratorValue::new(DEPRECATED_INFO, &args, &kwargs);
        let config_meta = ValueRef::dict(None);
        let config_value = ValueRef::dict(None);
        decorator.run(
            &mut ctx,
            "android:id",
            "TextView",
            false,
            &config_value,
            &config_meta,
        );
        let attrs = ctx
            .attr_decorator_meta
            .get("TextView")
            .expect("registry must contain TextView");
        assert_eq!(
            attrs.get("android:id").map(|s| s.as_str()),
            Some(INFO_ROLE_XML_ATTR)
        );
    }

    #[test]
    fn test_info_two_schemas_same_attr_are_separate() {
        let mut ctx = Context::new();
        let args = ValueRef::list(None);
        let mut kwargs = ValueRef::dict(None);
        kwargs.dict_update_key_value("type", ValueRef::str(INFO_ROLE_XML_ATTR));
        let decorator = DecoratorValue::new(DEPRECATED_INFO, &args, &kwargs);
        let config_meta = ValueRef::dict(None);
        let config_value = ValueRef::dict(None);

        decorator.run(
            &mut ctx,
            "id",
            "SchemaA",
            false,
            &config_value,
            &config_meta,
        );
        decorator.run(
            &mut ctx,
            "id",
            "SchemaB",
            false,
            &config_value,
            &config_meta,
        );

        // Both schemas must be present with their own inner map.
        let a = ctx
            .attr_decorator_meta
            .get("SchemaA")
            .expect("registry must contain SchemaA");
        let b = ctx
            .attr_decorator_meta
            .get("SchemaB")
            .expect("registry must contain SchemaB");
        assert_eq!(a.get("id").map(|s| s.as_str()), Some(INFO_ROLE_XML_ATTR));
        assert_eq!(b.get("id").map(|s| s.as_str()), Some(INFO_ROLE_XML_ATTR));
        // Pointer identity: IndexMap::get returns a reference into the
        // outer map's storage slot; the slots must be distinct, so two
        // different schemas pointing at the same attribute name do not
        // share storage.
        assert!(
            !std::ptr::eq(a, b),
            "per-schema maps must be distinct storage slots"
        );

        // And a write to one must not leak into the other.
        ctx.attr_decorator_meta
            .get_mut("SchemaA")
            .unwrap()
            .insert("id".to_string(), "attr-element".to_string());
        assert_eq!(
            ctx.attr_decorator_meta
                .get("SchemaA")
                .unwrap()
                .get("id")
                .map(|s| s.as_str()),
            Some("attr-element")
        );
        assert_eq!(
            ctx.attr_decorator_meta
                .get("SchemaB")
                .unwrap()
                .get("id")
                .map(|s| s.as_str()),
            Some(INFO_ROLE_XML_ATTR)
        );
    }

    #[test]
    fn test_info_unknown_role_is_silently_ignored() {
        let mut ctx = Context::new();
        let args = ValueRef::list(None);
        let mut kwargs = ValueRef::dict(None);
        kwargs.dict_update_key_value("type", ValueRef::str("some-other-role"));
        let decorator = DecoratorValue::new(DEPRECATED_INFO, &args, &kwargs);
        let config_meta = ValueRef::dict(None);
        let config_value = ValueRef::dict(None);
        decorator.run(
            &mut ctx,
            "attr",
            "SchemaX",
            false,
            &config_value,
            &config_meta,
        );
        assert!(ctx.attr_decorator_meta.is_empty());
    }

    #[test]
    fn test_info_no_kwargs_is_silently_ignored() {
        let mut ctx = Context::new();
        let args = ValueRef::list(None);
        let kwargs = ValueRef::dict(None);
        let decorator = DecoratorValue::new(DEPRECATED_INFO, &args, &kwargs);
        let config_meta = ValueRef::dict(None);
        let config_value = ValueRef::dict(None);
        decorator.run(
            &mut ctx,
            "attr",
            "SchemaY",
            false,
            &config_value,
            &config_meta,
        );
        assert!(ctx.attr_decorator_meta.is_empty());
    }
}
