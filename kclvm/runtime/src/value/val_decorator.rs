//! Copyright The KCL Authors. All rights reserved.

use crate::*;

pub const DEPRECATED_DECORATOR: &str = "deprecated";
pub const DEPRECATED_INFO: &str = "info";

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
            DEPRECATED_INFO => { /* Nothing to do on Info decorator */ }
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
        test_deprecated_decorator.run(&mut ctx, schema_name, true, &config_value, &config_meta);
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
            test_deprecated_decorator.run(&mut ctx, schema_name, true, &config_value, &config_meta);
        });
    }
}
