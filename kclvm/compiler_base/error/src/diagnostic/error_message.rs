//! The crate provides `ErrorMessage` to define the message displayed in diagnostics,
//! and provides `TemplateLoader` to load text template file ("*.ftl").
//!
use std::fs;

use compiler_base_macros::bug;
use fluent::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::langid;

/// Enum `ErrorMessage` defines the message displayed in diagnostics.
///
/// The builtin `ErrorMessage` includes:
/// - StrMessage: the string messages.
/// - TemplateMessage: the string loaded from "*.ftl" file, depends on "fluent-0.16.0".
///
/// `StrMessage` is only a string.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
/// let str_msg = ErrorMessage::StrMessage("This is a str message".to_string());
/// ```
///
/// `TemplateMessage` is the message loaded from '*.ftl' file, depends on "fluent-0.16.0".
/// With the help of "fluent-0.16.0", you can get a message string by a message index.
///
/// "*.ftl" file looks like, e.g. './src/diagnostic/locales/en-US/default.ftl' :
///
/// ``` ignore
/// 1.   invalid-syntax = Invalid syntax
/// 2.             .expected = Expected one of `{$expected_items}`
/// ```
///
/// - In line 1, `invalid-syntax` is a `MessageIndex`, `Invalid syntax` is the `TemplateMessage` to this `MessageIndex`.
/// - In line 2, `.expected` is another `MessageIndex`, it is a sub-`MessageIndex` of `invalid-syntax`.
/// - In line 2, Sub-`MessageIndex` must start with a point `.` and it is optional.
/// - In line 2, `Expected one of `{$expected_items}`` is the `TemplateMessage` to `.expected`. It is an interpolated string.
/// - In line 2, `{$expected_items}` is a `MessageArgs` of the `Expected one of `{$expected_items}``
/// and `MessageArgs` can be recognized as a Key-Value entry, it is optional.  
///
/// The pattern of above '*.ftl' file looks like:
/// ``` ignore
/// 1.   <TemplateMessageIndex>.<MessageIndex> = <TemplateMessage, Optional<MessageArgs>>
/// 2.             .Optional<<TemplateMessageIndex>.<MessageIndex>> = <TemplateMessage, Optional<MessageArgs>>
/// ```
///
/// And with the help of `TemplateLoader`, you can load '*ftl' files from local.
/// For more information about the `TemplateLoader` see the doc above enum `TemplateLoader`.
///
/// And for the 'default.ftl', you can get messages as below:
///
/// 1. If you want the message 'Invalid syntax' in line 1.
///
/// ```rust
/// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
/// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
/// # use compiler_base_error::diagnostic::error_message::MessageArgs;
/// # use compiler_base_error::diagnostic::error_message::MessageIndex;
/// # use std::borrow::Borrow;
///
/// // 1. Prepare an empty `MessageArgs`, Message in line 1 is not an interpolated string.
/// let no_args = MessageArgs::new();
///
/// // 2. Prepare the `MessageIndex` whose value is 'invalid-syntax'.
/// let msg_index = MessageIndex::from("invalid-syntax");
///
/// // 3. Only need the message in line 1, so do not need the sub-`MessageIndex` which is optional.
/// let no_sub_msg_index = None;
///
/// // 4. Create a `TemplateMessage` by the `MessageIndex`, sub-`MessageIndex` and `MessageArgs`.
/// let template_msg = ErrorMessage::new_template_msg(msg_index, no_sub_msg_index, &no_args);
///
/// // 5. With the help of `TemplateLoader`, you can get the message in 'default.ftl'.
/// let template_loader = TemplateLoader::new_with_template_path("./src/diagnostic/locales/en-US/default.ftl".to_string());
/// let msg_in_line_1 = template_msg.trans_msg_to_str(Some(&template_loader));
///
/// assert_eq!(msg_in_line_1, "Invalid syntax");
/// ```
///
/// 2. If you want the message 'Expected one of `{$expected_items}`' in line 2.
///
/// ```rust
/// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
/// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
/// # use compiler_base_error::diagnostic::error_message::MessageArgs;
/// # use compiler_base_error::diagnostic::error_message::MessageIndex;
/// # use std::borrow::Borrow;
///
/// // 1. Prepare the `MessageArgs` for `{$expected_items}`.
/// let mut args = MessageArgs::new();
/// args.set("expected_items", "I am an expected item");
///
/// // 2. Prepare the `MessageIndex` whose value is 'invalid-syntax'.
/// let msg_index = MessageIndex::from("invalid-syntax");
///
/// // 3. The sub-`MessageIndex` is 'expected'.
/// let sub_msg_index = MessageIndex::from("expected");
///
/// // 4. Create a `TemplateMessage` by the `MessageIndex`, sub-`MessageIndex` and `MessageArgs`.
/// let template_msg = ErrorMessage::new_template_msg(msg_index, Some(sub_msg_index), &args);
///
/// // 5. With the help of `TemplateLoader`, you can get the message in 'default.ftl'.
/// let template_loader = TemplateLoader::new_with_template_path("./src/diagnostic/locales/en-US/default.ftl".to_string());
/// let msg_in_line_2 = template_msg.trans_msg_to_str(Some(&template_loader));
///
/// assert_eq!(msg_in_line_2, "Expected one of `\u{2068}I am an expected item\u{2069}`");
/// ```
pub enum ErrorMessage<'a> {
    StrMessage(String),
    TemplateMessage(TemplateMessageIndex, &'a MessageArgs<'a>),
}

/// You can find the message in template file by `MessageIndex` which is part of `TemplateMessageIndex`.
/// `MessageIndex` only supports "String".
/// You need to use `MessageIndex` together with enum `ErrorMessage` and struct `TemplateMessageIndex`.
///
/// For more infomation, see the doc above enum `ErrorMessage` and struct `TemplateMessageIndex`.
pub type MessageIndex = String;

/// You can find the message in template file by `TemplateMessageIndex`.
/// `TemplateMessageIndex` includes one index and one sub-index.
/// Index and sub-index are both `MessageIndex`, and sub-index is optional.
/// You need to use `TemplateMessageIndex` together with enum `ErrorMessage` and type `MessageIndex`.
///
/// For more infomation, see the doc above enum `ErrorMessage` and struct `MessageIndex`.
pub struct TemplateMessageIndex(pub MessageIndex, pub Option<MessageIndex>);

impl<'a> ErrorMessage<'a> {
    /// Create a string error message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
    /// let str_msg = ErrorMessage::new_str_msg("This is a str message".to_string());
    /// ```
    pub fn new_str_msg(str_msg: String) -> Self {
        Self::StrMessage(str_msg)
    }

    /// Create an error message loaded from template file(*.ftl).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
    /// # use compiler_base_error::diagnostic::error_message::MessageArgs;
    /// # use compiler_base_error::diagnostic::error_message::MessageIndex;
    /// # use std::borrow::Borrow;
    ///
    /// // 1. Create the `MessageArgs` for the interpolated string 'Expected one of `{$expected_items}`'.
    /// let mut args = MessageArgs::new();
    /// args.set("expected_items", "I am an expected item");
    ///
    /// // 2. Create the `MessageIndex` whose value is 'invalid-syntax'.
    /// let msg_index = MessageIndex::from("invalid-syntax");
    ///
    /// // 3. Create sub-`MessageIndex` is 'expected'.
    /// let sub_msg_index = MessageIndex::from("expected");
    ///
    /// // 4. Create a `TemplateMessage` by the `MessageIndex`, sub-`MessageIndex` and `MessageArgs`.
    /// let template_msg = ErrorMessage::new_template_msg(msg_index, Some(sub_msg_index), &args);
    /// ```
    ///
    /// For more infomation about how to load message from '*.ftl' file,
    /// see the doc above enum `ErrorMessage` and struct `TemplateLoader`.
    pub fn new_template_msg(
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

    /// Get the content of the message in string.
    ///
    /// # Examples
    ///
    /// 1. ErrorMessage::StrMessage will directly return the "String" and do not need the `TemplateLoader`.
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
    /// let str_msg = ErrorMessage::new_str_msg("This is a str message".to_string());
    /// assert_eq!("This is a str message", str_msg.trans_msg_to_str(None))
    /// ```
    ///
    /// 2. ErrorMessage::TemplateMessage will load the message string with the help of `TemplateLoader`.
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
    /// # use compiler_base_error::diagnostic::error_message::MessageArgs;
    /// # use compiler_base_error::diagnostic::error_message::MessageIndex;
    /// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
    ///
    /// // 1. Create the `ErrorMessage`
    /// let mut args = MessageArgs::new();
    /// args.set("expected_items", "I am an expected item");
    /// let msg_index = MessageIndex::from("invalid-syntax");
    /// let sub_msg_index = MessageIndex::from("expected");
    /// let template_msg = ErrorMessage::new_template_msg(msg_index, Some(sub_msg_index), &args);
    ///
    /// // 2. Use the `ErrorMessage` and `TemplateLoader` to get the message.
    /// let template_loader = TemplateLoader::new_with_template_path("./src/diagnostic/locales/en-US/default.ftl".to_string());
    /// let msg_in_line_2 = template_msg.trans_msg_to_str(Some(&template_loader));
    /// ```
    ///
    /// # Panics
    ///
    /// Without the `TemplateLoader` for `ErrorMessage::TemplateMessage.trans_msg_to_str()`, it will `panic`.
    ///
    /// For more infomation about how to load message from '*.ftl' file,
    /// see the doc above enum `ErrorMessage` and struct `TemplateLoader`.
    pub fn trans_msg_to_str(&self, template_loader: Option<&'a TemplateLoader>) -> String {
        match self {
            ErrorMessage::StrMessage(s) => s.to_string(),
            ErrorMessage::TemplateMessage(index, msg_args) => match template_loader {
                Some(template_loader) => template_loader.load_message(index, msg_args),
                None => bug!("'TemplateLoader' is not found."),
            },
        }
    }
}

/// `MessageArgs` is the arguments of the interpolated string in `ErrorMessage`.
///
/// `MessageArgs` is a Key-Value entry which only supports "set" and without "get".
/// You need getting nothing from `MessageArgs`. Only setting it and senting it to `ErrorMessage` is enough.
///
/// Note: Currently both `Key` and `Value` of `MessageArgs` types only support string (&str).
///
/// # Examples
///
/// ```rust
/// # use compiler_base_error::diagnostic::error_message::MessageArgs;
/// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
/// # use compiler_base_error::diagnostic::error_message::MessageIndex;
/// # use std::borrow::Borrow;
///
/// # let mut args = MessageArgs::new();
/// # args.set("expected_items", "I am an expected item");
/// # let msg_index = MessageIndex::from("invalid-syntax");
/// # let sub_msg_index = MessageIndex::from("expected");
///
/// let mut msg_args = MessageArgs::new();
/// // You only need "set()".
/// msg_args.set("This is Key", "This is Value");
///
/// // When you use it, just sent it to `ErrorMessage`.
/// let template_msg = ErrorMessage::new_template_msg(msg_index, Some(sub_msg_index), &args);
/// ```
///
/// For more information about the `ErrorMessage` see the doc above enum `ErrorMessage`.
pub struct MessageArgs<'a>(FluentArgs<'a>);
impl<'a> MessageArgs<'a> {
    pub fn new() -> Self {
        Self(FluentArgs::new())
    }

    pub fn set(&mut self, k: &'a str, v: &'a str) {
        self.0.set(k, v);
    }
}

/// `TemplateLoader` load template contents from "*.ftl" file.
pub struct TemplateLoader {
    template_inner: TemplateLoaderInner,
}

impl TemplateLoader {
    /// You can only use the constructor 'new_with_template_path' to construct `TemplateLoader`.
    /// In the constructor 'new_with_template_path', it will load the template contents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
    /// let template_loader = TemplateLoader::new_with_template_path("./src/diagnostic/locales/en-US/default.ftl".to_string());
    /// ```
    pub fn new_with_template_path(template_path: String) -> Self {
        Self {
            template_inner: TemplateLoaderInner::new_with_template_path(template_path),
        }
    }

    /// You can use this method to find message from template by `TemplateMessageIndex` and `MessageArgs`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::ErrorMessage;
    /// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
    /// # use compiler_base_error::diagnostic::error_message::MessageArgs;
    /// # use compiler_base_error::diagnostic::error_message::MessageIndex;
    /// # use compiler_base_error::diagnostic::error_message::TemplateMessageIndex;
    ///
    /// // 1. Create the `TemplateMessageIndex` and `MessageArgs`.
    ///
    /// let mut args = MessageArgs::new();
    /// args.set("expected_items", "I am an expected item");
    /// let msg_index = MessageIndex::from("invalid-syntax");
    /// let sub_msg_index = MessageIndex::from("expected");
    /// let template_index = TemplateMessageIndex(msg_index, Some(sub_msg_index));
    ///
    /// // 2. With the help of `TemplateLoader`, you can get the message in 'default.ftl'.
    ///
    /// let template_loader = TemplateLoader::new_with_template_path("./src/diagnostic/locales/en-US/default.ftl".to_string());
    /// let msg_in_line_2 = template_loader.load_message(&template_index, &args);
    /// ```
    ///
    /// For more information about the `MessageArgs`, `TemplateMessageIndex` see the doc above enum `ErrorMessage`.
    pub fn load_message(
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
