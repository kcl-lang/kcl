mod diagnostic;
mod emitter;

use anyhow::{Context, Result};
use diagnostic::diagnostic_message::TemplateLoader;
use emitter::{Emitter, TerminalEmitter};
use fluent::FluentArgs;
use std::sync::Arc;

pub use diagnostic::{style::DiagnosticStyle, Diagnostic};

#[cfg(test)]
mod tests;

/// `DiagnosticHandler` supports diagnostic messages to terminal stderr.
///
/// `DiagnosticHandler` will load template file directory when instantiating through the constructor `new()`.
/// 
pub struct DiagnosticHandler {
    template_loader: Arc<TemplateLoader>,
    emitter: Box<dyn Emitter<DiagnosticStyle>>,
    diagnostics: Vec<Diagnostic<DiagnosticStyle>>,
}

impl DiagnosticHandler {
    /// Load all (*.ftl) template files under directory `template_dir`.
    /// `DiagnosticHandler` will load all the files end with "*.ftl" under the directory recursively.
    /// If directory `template_dir` does not exist, this method will return an error.
    ///
    /// template_files
    ///      |
    ///      |---- template.ftl
    ///      |---- sub_template_files
    ///                  |
    ///                  |---- sub_template.ftl
    ///
    /// 'template.ftl' and 'sub_template.ftl' can both loaded by the `new_with_template_dir()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticHandler;
    /// let diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/");
    /// match diag_handler{
    ///     Ok(_) => {}
    ///     Err(_) => {panic!("`diag_handler` should be Ok(...)")}
    /// }
    ///
    /// // './src_invalid/diagnostic/locales/en-US/' does not exist.
    /// let diag_handler_invalid = DiagnosticHandler::new_with_template_dir("./src_invalid/diagnostic/locales/en-US/");
    /// match diag_handler_invalid{
    ///     Ok(_) => {panic!("`diag_handler_invalid` should be Err(...)")}
    ///     Err(_) => {}
    /// }
    /// ```
    pub fn new_with_template_dir(template_dir: &str) -> Result<Self> {
        let template_loader = TemplateLoader::new_with_template_dir(template_dir)
            .with_context(|| format!("Failed to init `TemplateLoader` from '{}'", template_dir))?;
        Ok(Self {
            template_loader: Arc::new(template_loader),
            emitter: Box::new(TerminalEmitter::default()),
            diagnostics: vec![],
        })
    }

    /// Add a diagnostic to `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    /// assert_eq!(diag_handler.diagnostics_count(), 0);
    ///
    /// diag_handler.add_diagnostic(diag_1);
    /// assert_eq!(diag_handler.diagnostics_count(), 1);
    /// ```
    pub fn add_diagnostic(&mut self, diag: Diagnostic<DiagnosticStyle>) {
        self.diagnostics.push(diag);
    }

    /// Get count of diagnostics in `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    pub fn diagnostics_count(&self) -> usize {
        self.diagnostics.len()
    }

    /// Emit the diagnostic messages to to terminal stderr.
    pub fn emit_diagnostic(&mut self, diag: Diagnostic<DiagnosticStyle>) {
        self.emitter.emit_diagnostic(&diag);
    }

    /// Emit all the diagnostics messages to to terminal stderr.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    pub fn emit_all_diagnostics(&mut self) {
        for diag in &self.diagnostics {
            self.emitter.emit_diagnostic(&diag)
        }
    }

    /// Get the message string from "*.ftl" file by `index`, `sub_index` and `MessageArgs`.
    /// "*.ftl" file looks like, e.g. './src/diagnostic/locales/en-US/default.ftl' :
    ///
    /// ``` ignore
    /// 1.   invalid-syntax = Invalid syntax
    /// 2.             .expected = Expected one of `{$expected_items}`
    /// ```
    ///
    /// - In line 1, `invalid-syntax` is a `index`, `Invalid syntax` is the `Message String` to this `index`.
    /// - In line 2, `.expected` is another `index`, it is a `sub_index` of `invalid-syntax`.
    /// - In line 2, `sub_index` must start with a point `.` and it is optional.
    /// - In line 2, `Expected one of `{$expected_items}`` is the `Message String` to `.expected`. It is an interpolated string.
    /// - In line 2, `{$expected_items}` is a `MessageArgs` of the `Expected one of `{$expected_items}``
    /// and `MessageArgs` can be recognized as a Key-Value entry, it is optional.  
    ///
    /// The pattern of above '*.ftl' file looks like:
    /// ``` ignore
    /// 1.   <'index'> = <'message_string' with optional 'MessageArgs'>
    /// 2.             <optional 'sub_index' start with point> = <'message_string' with optional 'MessageArgs'>
    /// ```
    /// And for the 'default.ftl' shown above, you can get messages as follow:
    ///
    /// 1. If you want the message 'Invalid syntax' in line 1.
    ///
    /// ``` rust
    /// # use compiler_base_error::MessageArgs;
    /// # use compiler_base_error::DiagnosticHandler;
    ///
    /// // 1. Prepare an empty `MessageArgs`, Message in line 1 is not an interpolated string.
    /// let no_args = MessageArgs::new();
    ///
    /// // 2. `index` is 'invalid-syntax' and has no `sub_index`.
    /// let index = "invalid-syntax";
    /// let sub_index = None;
    ///
    /// // 3. Create the `DiagnosticHandler` with template (*.ftl) files directory.
    /// let diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// // 4. Get the message.
    /// let msg_in_line_1 = diag_handler.get_diagnostic_msg(index, sub_index, &no_args).unwrap();
    ///
    /// assert_eq!(msg_in_line_1, "Invalid syntax");
    /// ```
    ///
    /// 2. If you want the message 'Expected one of `{$expected_items}`' in line 2.
    ///
    /// ``` rust
    /// # use compiler_base_error::MessageArgs;
    /// # use compiler_base_error::DiagnosticHandler;
    ///
    /// // 1. Prepare the `MessageArgs` for `{$expected_items}`.
    /// let mut args = MessageArgs::new();
    /// args.set("expected_items", "I am an expected item");
    ///
    /// // 2. `index` is 'invalid-syntax'.
    /// let index = "invalid-syntax";
    ///
    /// // 3. `sub_index` is 'expected'.
    /// let sub_index = "expected";
    ///
    /// // 4. Create the `DiagnosticHandler` with template (*.ftl) files directory.
    /// let diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// // 5. Get the message.
    /// let msg_in_line_2 = diag_handler.get_diagnostic_msg(index, Some(sub_index), &args).unwrap();
    ///
    /// assert_eq!(msg_in_line_2, "Expected one of `\u{2068}I am an expected item\u{2069}`");
    /// ```
    pub fn get_diagnostic_msg(
        &self,
        index: &str,
        sub_index: Option<&str>,
        args: &MessageArgs,
    ) -> Result<String> {
        self.template_loader.get_msg_to_str(index, sub_index, &args)
    }
}

/// `MessageArgs` is the arguments of the interpolated string.
///
/// `MessageArgs` is a Key-Value entry which only supports "set" and without "get".
/// You need getting nothing from `MessageArgs`. Only setting it and senting it to `DiagnosticHandler` is enough.
///
/// Note: Currently both `Key` and `Value` of `MessageArgs` types only support string (&str).
///
/// # Examples
///
/// ``` rust
/// # use compiler_base_error::DiagnosticHandler;
/// # use compiler_base_error::MessageArgs;
///
/// let index = "invalid-syntax";
/// let sub_index = Some("expected");
/// let mut msg_args = MessageArgs::new();
/// // You only need "set()".
/// msg_args.set("This is Key", "This is Value");
///
/// // Create the `DiagnosticHandler` with template (*.ftl) files directory.
/// let diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
///
/// // When you use it, just sent it to `DiagnosticHandler`.
/// let msg_in_line_1 = diag_handler.get_diagnostic_msg(index, sub_index, &msg_args);
/// ```
///
/// For more information about the `DiagnosticHandler` see the doc above struct `DiagnosticHandler`.
pub struct MessageArgs<'a>(FluentArgs<'a>);
impl<'a> MessageArgs<'a> {
    pub fn new() -> Self {
        Self(FluentArgs::new())
    }

    pub fn set(&mut self, k: &'a str, v: &'a str) {
        self.0.set(k, v);
    }
}
