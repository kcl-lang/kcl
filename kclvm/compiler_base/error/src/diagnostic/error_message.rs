//! The crate provides `ErrorMessage` to define the message displayed in diagnostics,
//!
use std::fs;

use compiler_base_macros::bug;
use fluent::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::langid;

// Enum `ErrorMessage` defines the message displayed in diagnostics.
//
// The builtin `ErrorMessage` includes:
// - StrMessage: the string messages.
// - TemplateMessage: the string loaded from "*.ftl" file, depends on "fluent-0.16.0".
//
// `StrMessage` is only a string.
//
// `TemplateMessage` is the message loaded from '*.ftl' file, depends on "fluent-0.16.0".
// With the help of "fluent-0.16.0", you can get a message string by a message index.
//
// "*.ftl" file looks like, e.g. './src/diagnostic/locales/en-US/default.ftl' :
//
// ```
// 1.   invalid-syntax = Invalid syntax
// 2.             .expected = Expected one of `{$expected_items}`
// ```
//
// - In line 1, `invalid-syntax` is a `MessageIndex`, `Invalid syntax` is the `TemplateMessage` to this `MessageIndex`.
// - In line 2, `.expected` is another `MessageIndex`, it is a sub-`MessageIndex` of `invalid-syntax`.
// - In line 2, Sub-`MessageIndex` must start with a point `.` and it is optional.
// - In line 2, `Expected one of `{$expected_items}`` is the `TemplateMessage` to `.expected`. It is an interpolated string.
// - In line 2, `{$expected_items}` is a `MessageArgs` of the `Expected one of `{$expected_items}``
// and `MessageArgs` can be recognized as a Key-Value entry, it is optional.
//
// The pattern of above '*.ftl' file looks like:
// ```
// 1.   <TemplateMessageIndex>.<MessageIndex> = <TemplateMessage, Optional<MessageArgs>>
// 2.             .Optional<<TemplateMessageIndex>.<MessageIndex>> = <TemplateMessage, Optional<MessageArgs>>
// ```
pub(crate) enum ErrorMessage<'a> {
    StrMessage(String),
    TemplateMessage(TemplateMessageIndex, &'a MessageArgs<'a>),
}

// `MessageIndex` only supports "String".
pub(crate) type MessageIndex = String;

// `TemplateMessageIndex` includes one index and one sub-index.
// Index and sub-index are both `MessageIndex`, and sub-index is optional.
pub(crate) struct TemplateMessageIndex(pub(crate) MessageIndex, pub(crate) Option<MessageIndex>);

impl<'a> ErrorMessage<'a> {
    // Create a string error message.
    pub(crate) fn new_str_msg(str_msg: String) -> Self {
        Self::StrMessage(str_msg)
    }

    // Create an error message loaded from template file(*.ftl).
    pub(crate) fn new_template_msg(
        index: String,
        sub_index: Option<String>,
        args: &'a MessageArgs,
    ) -> Self {
        let sub_index = match sub_index {
            Some(sub_i) => Some(MessageIndex::from(sub_i)),
            None => None,
        };
        let template_index = TemplateMessageIndex(MessageIndex::from(index), sub_index);
        Self::TemplateMessage(template_index, args)
    }

    // Get the content of the message in string.
    pub(crate) fn trans_msg_to_str(&self, template_loader: Option<&'a TemplateLoader>) -> String {
        match self {
            ErrorMessage::StrMessage(s) => s.to_string(),
            ErrorMessage::TemplateMessage(index, msg_args) => match template_loader {
                Some(template_loader) => template_loader.load_message(index, msg_args),
                None => bug!("'TemplateLoader' is not found."),
            },
        }
    }
}

// `MessageArgs` is the arguments of the interpolated string in `ErrorMessage`.
// `MessageArgs` is a Key-Value entry which only supports "set" and without "get".
// Note: Currently both `Key` and `Value` of `MessageArgs` types only support string (&str).
pub(crate) struct MessageArgs<'a>(FluentArgs<'a>);
impl<'a> MessageArgs<'a> {
    pub(crate) fn new() -> Self {
        Self(FluentArgs::new())
    }

    pub(crate) fn set(&mut self, k: &'a str, v: &'a str) {
        self.0.set(k, v);
    }
}

// `TemplateLoader` load template contents from "*.ftl" file.
pub(crate) struct TemplateLoader {
    template_inner: TemplateLoaderInner,
}

impl TemplateLoader {
    pub(crate) fn new_with_template_path(template_path: String) -> Self {
        Self {
            template_inner: TemplateLoaderInner::new_with_template_path(template_path),
        }
    }

    pub(crate) fn load_message(
        &self,
        err_msg_index: &TemplateMessageIndex,
        msg_args: &MessageArgs,
    ) -> String {
        let MessageArgs(args) = msg_args;

        let TemplateMessageIndex(index, sub_index) = err_msg_index;

        let msg = self
            .template_inner
            .get_template_bunder()
            .get_message(index)
            .unwrap_or_else(|| bug!("Message doesn't exist."));

        let pattern = match sub_index {
            Some(s_id) => {
                let attr = msg.get_attribute(s_id).unwrap();
                attr.value()
            }
            None => msg.value().unwrap_or_else(|| bug!("Message has no value.")),
        };
        let value = self.template_inner.get_template_bunder().format_pattern(
            pattern,
            Some(&args),
            &mut vec![],
        );
        value.to_string()
    }
}

// `TemplateLoaderInner` is used to privatize the default constructor of `TemplateLoader`.
struct TemplateLoaderInner {
    template_bunder: FluentBundle<FluentResource>,
}

impl TemplateLoaderInner {
    fn new_with_template_path(template_path: String) -> Self {
        let mut template_bunder = FluentBundle::new(vec![langid!("en-US")]);
        let resource = fs::read_to_string(template_path).unwrap_or_else(|_err| {
            bug!("Failed to read '*ftl' file");
        });
        let source = FluentResource::try_new(resource).unwrap_or_else(|_err| {
            bug!("Failed to add FTL resources to the bundle.");
        });
        template_bunder.add_resource(source).unwrap_or_else(|_err| {
            bug!("Failed to parse an FTL string.");
        });

        Self { template_bunder }
    }

    fn get_template_bunder(&self) -> &FluentBundle<FluentResource> {
        &self.template_bunder
    }
}
