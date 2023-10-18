//! This crate provides `DiagnosticHandler` supports diagnostic messages to terminal stderr.
//!
//! `DiagnosticHandler` mainly consists of 4 parts：
//! - Emitter: Emit the styled string to terminal stderr.
//! - Template Loader: Load template files locally and find messages from file contents.
//! - A set for Diagnostics: All the diagnostic messages.
//!
//! For more information about diagnostic, see doc in "compiler_base/error/diagnostic/mod.rs".
//! For more information about emitter, see doc in "compiler_base/error/src/emitter.rs".
//! For more information about template loader, see doc in "compiler_base/error/src/diagnostic/diagnostic_message.rs".

use crate::{
    diagnostic::diagnostic_message::TemplateLoader, emit_diagnostic_to_uncolored_text, Diagnostic,
    DiagnosticStyle, Emitter, EmitterWriter,
};
use anyhow::{bail, Context, Result};
use compiler_base_span::fatal_error::FatalError;
use fluent::FluentArgs;
use std::{
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Mutex},
};

// Default template resource file path.
const DEFAULT_TEMPLATE_RESOURCE: &str = "src/diagnostic/locales/en-US/";
const DIAGNOSTIC_MESSAGES_ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// `DiagnosticHandler` supports diagnostic messages to terminal stderr.
///
/// `DiagnosticHandler` will load template file(*ftl) directory when instantiating through the constructor `new_with_template_dir()`.
/// "*.ftl" file looks like, e.g. './src/diagnostic/locales/en-US/default.ftl' :
/// ``` ignore
/// invalid-syntax = Invalid syntax
///       .expected = Expected one of `{$expected_items}`
/// ```
/// There are two lines in './src/diagnostic/locales/en-US/default.ftl'.
/// - In line 1, `invalid-syntax` is a `index`, `Invalid syntax` is the `Message String` to this `index`.
/// - In line 2, `.expected` is another `index`, it is a `sub_index` of `invalid-syntax`.
/// - In line 2, `sub_index` must start with a point `.` and it is optional and can be more than one.
/// - In line 2, `Expected one of `{$expected_items}`` is the `Message String` to `.expected`. It is an interpolated string.
/// - In line 2, `{$expected_items}` is a `MessageArgs` of the `Expected one of `{$expected_items}``
/// and `MessageArgs` can be recognized as a Key-Value entry, it is optional.
///
/// The pattern of above '*.ftl' file looks like:
/// ``` ignore
///    <'index'> = <'message_string' with optional 'MessageArgs'>
///             <optional 'sub_index' start with point> = <'message_string' with optional 'MessageArgs'>*
/// ```
///
/// Note: `DiagnosticHandler` uses `Mutex` internally to ensure thread safety,
/// so you don't need to use references like `Arc` or `Mutex` to make `DiagnosticHandler` thread safe.
///
/// When your compiler needs to use `Compiler-Base-Error` to displaying diagnostics, you need to create a `DiagnosticHandler` at first.
/// For more information about how to create a `DiagnosticHandler`, see the doc above method `new_with_template_dir()`.
/// Since creating `DiagnosticHandler` needs to load the locally template (*.ftl) file, it may cause I/O performance loss,
/// so we recommend you create `DiagnosticHandler` eagerly and globally in the compiler and pass references to other modules that use `DiagnosticHandler`.
///
/// And since `DiagnosticHandler` provides methods that do not supports mutable references "&mut self", so passing immutable references (&) is enough.
///
/// For Example:
///
/// 1. You can put `DiagnosticHandler` on the same level as `Lexer`, `Parser` and `CodeGenerator` in your compiler.
/// ```ignore
/// struct Compiler {
///     diag_handler: DiagnosticHandler,
///     lang_lexer: Lexer,
///     lang_parser: Parser,
///     code_generator: CodeGenerator
/// }
/// ```
///
/// 2. And send the immutable references to `Lexer`, `Parser` and `CodeGenerator` to displaying the diagnostic during compiling.
/// ```ignore
/// impl Compiler {
///     fn compile(&self) {
///         self.lang_lexer.lex(&self.diag_handler);
///         self.lang_parser.parse(&self.diag_handler);
///         self.code_generator.gen(&self.diag_handler);
///     }
/// }
/// ```
///
/// ```ignore
/// impl Lexer {
///     fn lex(&self, diag_handler: &DiagnosticHandler){
///        handler.XXXX(); // do something to diaplay diagnostic.
///     }
/// }
/// ```
///
pub struct DiagnosticHandler {
    handler_inner: Mutex<DiagnosticHandlerInner>,
}

impl Debug for DiagnosticHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.handler_inner.lock() {
            Ok(inner) => {
                write!(f, "{:?}", inner)
            }
            Err(_) => {
                write!(f, "")
            }
        }
    }
}

impl DiagnosticHandler {
    /// Create a `DiagnosticHandler` with no (*.ftl) template files.
    /// Use this method if the diagnostic message does not need to be loaded from the template file (*.ftl).
    pub fn default() -> Self {
        Self {
            handler_inner: Mutex::new(DiagnosticHandlerInner::default()),
        }
    }

    /// Load all (*.ftl) template files under default directory.
    ///
    /// Default directory "./src/diagnostic/locales/en-US/"
    /// Call the constructor 'new_with_template_dir()' to load the file.
    /// For more information about the constructor 'new_with_template_dir()', see the doc above 'new_with_template_dir()'.
    ///
    /// Note: This method has not been completed, and it may throw an error that the file path does not exist.
    pub fn new_with_default_template_dir() -> Result<Self> {
        let mut cargo_file_path = PathBuf::from(DIAGNOSTIC_MESSAGES_ROOT);
        cargo_file_path.push(DEFAULT_TEMPLATE_RESOURCE);
        let abs_path = cargo_file_path.to_str().with_context(|| {
            format!("No such file or directory '{}'", DEFAULT_TEMPLATE_RESOURCE)
        })?;

        DiagnosticHandler::new_with_template_dir(abs_path).with_context(|| {
            format!(
                "Failed to init `TemplateLoader` from '{}'",
                DEFAULT_TEMPLATE_RESOURCE
            )
        })
    }

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
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// let diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/");
    /// match diag_handler {
    ///     Ok(_) => {}
    ///     Err(_) => {panic!("`diag_handler` should be Ok(...)")}
    /// }
    ///
    /// // './src_invalid/diagnostic/locales/en-US/' does not exist.
    /// let diag_handler_invalid = DiagnosticHandler::new_with_template_dir("./src_invalid/diagnostic/locales/en-US/");
    /// match diag_handler_invalid {
    ///     Ok(_) => {panic!("`diag_handler_invalid` should be Err(...)")}
    ///     Err(_) => {}
    /// }
    /// ```
    pub fn new_with_template_dir(template_dir: &str) -> Result<Self> {
        let handler_inner = DiagnosticHandlerInner::new_with_template_dir(template_dir)
            .with_context(|| format!("Failed to init `TemplateLoader` from '{}'", template_dir))?;
        Ok(Self {
            handler_inner: Mutex::new(handler_inner),
        })
    }

    /// Add a diagnostic generated from error to `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    ///
    /// Note: `DiagnosticHandler` does not deduplicate diagnostics.
    /// If you add two same diagnostics, you will see two same messages in the terminal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 0);
    ///
    /// diag_handler.add_err_diagnostic(diag_1);
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 1);
    /// ```
    pub fn add_err_diagnostic(&self, diag: Diagnostic<DiagnosticStyle>) -> Result<&Self> {
        match self.handler_inner.lock() {
            Ok(mut inner) => {
                inner.add_err_diagnostic(diag);
                Ok(self)
            }
            Err(_) => bail!("Add Error Diagnostic Failed."),
        }
    }

    /// Add a diagnostic generated from warning to `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    ///
    /// Note: `DiagnosticHandler` does not deduplicate diagnostics.
    /// If you add two same diagnostics, you will see two same messages in the terminal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 0);
    ///
    /// diag_handler.add_warn_diagnostic(diag_1);
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 1);
    /// ```
    pub fn add_warn_diagnostic(&self, diag: Diagnostic<DiagnosticStyle>) -> Result<&Self> {
        match self.handler_inner.lock() {
            Ok(mut inner) => {
                inner.add_warn_diagnostic(diag);
                Ok(self)
            }
            Err(_) => bail!("Add Warn Diagnostic Failed."),
        }
    }

    /// Get count of diagnostics in `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 0);
    ///
    /// diag_handler.add_warn_diagnostic(diag_1);
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 1);
    /// ```
    pub fn diagnostics_count(&self) -> Result<usize> {
        match self.handler_inner.lock() {
            Ok(inner) => Ok(inner.diagnostics_count()),
            Err(_) => bail!("Diagnostics Counts Failed."),
        }
    }

    /// Emit all the diagnostics into strings and return.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compiler_base_error::DiagnosticStyle;
    /// use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// use compiler_base_error::Diagnostic;
    /// use compiler_base_error::components::Label;
    ///
    /// let mut diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// diag_1.append_component(Box::new(Label::Note));
    ///
    /// let mut diag_handler = DiagnosticHandler::default();
    ///
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 0);
    ///
    /// diag_handler.add_err_diagnostic(diag_1);
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 1);
    /// assert_eq!(diag_handler.emit_all_diags_into_string().unwrap().get(0).unwrap().as_ref().unwrap(), "note");
    /// ```
    pub fn emit_all_diags_into_string(&self) -> Result<Vec<Result<String>>> {
        match self.handler_inner.lock() {
            Ok(inner) => Ok(inner.emit_all_diags_into_string()),
            Err(_) => bail!("Emit Diagnostics Failed."),
        }
    }

    /// Emit the [`index`]th diagnostics into strings and return.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compiler_base_error::DiagnosticStyle;
    /// use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// use compiler_base_error::Diagnostic;
    /// use compiler_base_error::components::Label;
    ///
    /// let mut diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// diag_1.append_component(Box::new(Label::Note));
    ///
    /// let mut diag_handler = DiagnosticHandler::default();
    ///
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 0);
    ///
    /// diag_handler.add_err_diagnostic(diag_1);
    /// assert_eq!(diag_handler.diagnostics_count().unwrap(), 1);
    /// assert_eq!(diag_handler.emit_nth_diag_into_string(0).unwrap().unwrap().unwrap(), "note");
    /// ```
    pub fn emit_nth_diag_into_string(&self, index: usize) -> Result<Option<Result<String>>> {
        match self.handler_inner.lock() {
            Ok(inner) => Ok(inner.emit_nth_diag_into_string(index)),
            Err(_) => bail!("Emit Diagnostics Failed."),
        }
    }

    /// Emit the diagnostic messages generated from error to to terminal stderr.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// assert_eq!(diag_handler.has_errors().unwrap(), false);
    /// diag_handler.emit_error_diagnostic(diag_1);
    /// assert_eq!(diag_handler.has_errors().unwrap(), true);
    /// ```
    pub fn emit_error_diagnostic(&self, diag: Diagnostic<DiagnosticStyle>) -> Result<&Self> {
        match self.handler_inner.lock() {
            Ok(mut inner) => {
                inner
                    .emit_error_diagnostic(diag)
                    .with_context(|| ("Emit Error Diagnostics Failed."))?;
                Ok(self)
            }
            Err(_) => bail!("Emit Error Diagnostics Failed."),
        }
    }

    /// Emit the diagnostic messages generated from warning to to terminal stderr.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// assert_eq!(diag_handler.has_warns().unwrap(), false);
    /// diag_handler.emit_warn_diagnostic(diag_1);
    /// assert_eq!(diag_handler.has_warns().unwrap(), true);
    /// ```
    pub fn emit_warn_diagnostic(&self, diag: Diagnostic<DiagnosticStyle>) -> Result<&Self> {
        match self.handler_inner.lock() {
            Ok(mut inner) => {
                inner
                    .emit_warn_diagnostic(diag)
                    .with_context(|| ("Emit Warn Diagnostics Failed."))?;
                Ok(self)
            }
            Err(_) => bail!("Emit Warn Diagnostics Failed."),
        }
    }

    /// Emit all the diagnostics messages to to terminal stderr.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let diag_2 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// diag_handler.add_err_diagnostic(diag_1);
    /// diag_handler.add_err_diagnostic(diag_2);
    /// diag_handler.emit_stashed_diagnostics();
    /// ```
    pub fn emit_stashed_diagnostics(&self) -> Result<&Self> {
        match self.handler_inner.lock() {
            Ok(mut inner) => {
                inner
                    .emit_stashed_diagnostics()
                    .with_context(|| ("Emit Stashed Diagnostics Failed."))?;
                Ok(self)
            }
            Err(_) => bail!("Emit Stashed Diagnostics Failed."),
        }
    }

    /// If some diagnotsics generated by errors, `has_errors` returns `True`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// assert_eq!(diag_handler.has_errors().unwrap(), false);
    /// diag_handler.emit_error_diagnostic(diag_1);
    /// assert_eq!(diag_handler.has_errors().unwrap(), true);
    /// ```
    pub fn has_errors(&self) -> Result<bool> {
        match self.handler_inner.lock() {
            Ok(inner) => Ok(inner.has_errors()),
            Err(_) => bail!("Check Has Errors Failed."),
        }
    }

    /// If some diagnotsics generated by warnings, `has_errors` returns `True`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use compiler_base_error::Diagnostic;
    /// let diag_1 = Diagnostic::<DiagnosticStyle>::new();
    /// let mut diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// assert_eq!(diag_handler.has_warns().unwrap(), false);
    /// diag_handler.emit_warn_diagnostic(diag_1);
    /// assert_eq!(diag_handler.has_warns().unwrap(), true);
    /// ```
    pub fn has_warns(&self) -> Result<bool> {
        match self.handler_inner.lock() {
            Ok(inner) => Ok(inner.has_warns()),
            Err(_) => bail!("Check Has Warns Failed."),
        }
    }

    /// After emitting all the diagnostics, it will panic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use compiler_base_error::Diagnostic;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use std::panic;
    /// let diag_handler = DiagnosticHandler::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    ///
    /// diag_handler.abort_if_errors().unwrap();
    /// diag_handler.add_warn_diagnostic(Diagnostic::<DiagnosticStyle>::new()).unwrap();
    ///
    /// diag_handler.abort_if_errors().unwrap();
    /// diag_handler.add_err_diagnostic(Diagnostic::<DiagnosticStyle>::new()).unwrap();
    ///
    /// let result = panic::catch_unwind(|| {
    ///     diag_handler.abort_if_errors().unwrap();
    /// });
    /// assert!(result.is_err());
    /// ```
    pub fn abort_if_errors(&self) -> Result<&Self> {
        match self.handler_inner.lock() {
            Ok(mut inner) => {
                inner
                    .abort_if_errors()
                    .with_context(|| ("Abort If Errors Failed."))?;
                Ok(self)
            }
            Err(_) => bail!("Abort If Errors Failed."),
        }
    }

    /// Get the message string from "*.ftl" file by `index`, `sub_index` and `MessageArgs`.
    /// And for the 'default.ftl' shown above, you can get messages as follow:
    ///
    /// ```ignore
    /// invalid-syntax = Invalid syntax
    ///       .expected = Expected one of `{$expected_items}`
    /// ```
    ///
    /// 1. If you want the message 'Invalid syntax' in line 1.
    ///
    /// ``` rust
    /// # use compiler_base_error::diagnostic_handler::MessageArgs;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
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
    /// # use compiler_base_error::diagnostic_handler::MessageArgs;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
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
        match self.handler_inner.lock() {
            Ok(inner) => inner.get_diagnostic_msg(index, sub_index, args),
            Err(_) => bail!("Find Diagnostic Message Failed."),
        }
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
/// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
/// # use compiler_base_error::diagnostic_handler::MessageArgs;
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
#[derive(Default)]
pub struct MessageArgs<'a>(pub(crate) FluentArgs<'a>);
impl<'a> MessageArgs<'a> {
    pub fn new() -> Self {
        Self(FluentArgs::new())
    }

    pub fn set(&mut self, k: &'a str, v: &'a str) {
        self.0.set(k, v);
    }
}

pub(crate) struct DiagnosticHandlerInner {
    emitter: Box<dyn Emitter<DiagnosticStyle>>,
    diagnostics: Vec<Diagnostic<DiagnosticStyle>>,
    err_count: usize,
    warn_count: usize,
    template_loader: Arc<TemplateLoader>,
}

impl Debug for DiagnosticHandlerInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut diag_fmt = String::new();
        for diag in &self.diagnostics {
            diag_fmt.push_str(&format!("{:?}", diag));
        }
        write!(f, "{}", diag_fmt)
    }
}

impl DiagnosticHandlerInner {
    pub(crate) fn default() -> Self {
        Self {
            err_count: 0,
            warn_count: 0,
            emitter: Box::new(EmitterWriter::default()),
            diagnostics: vec![],
            template_loader: Arc::new(TemplateLoader::default()),
        }
    }

    /// Load all (*.ftl) template files under directory `template_dir`.
    pub(crate) fn new_with_template_dir(template_dir: &str) -> Result<Self> {
        let template_loader = TemplateLoader::new_with_template_dir(template_dir)
            .with_context(|| format!("Failed to init `TemplateLoader` from '{}'", template_dir))?;

        Ok(Self {
            err_count: 0,
            warn_count: 0,
            emitter: Box::new(EmitterWriter::default()),
            diagnostics: vec![],
            template_loader: Arc::new(template_loader),
        })
    }

    /// Add a diagnostic generated from error to `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    pub(crate) fn add_err_diagnostic(&mut self, diag: Diagnostic<DiagnosticStyle>) {
        self.diagnostics.push(diag);
        self.err_count += 1;
    }

    /// Add a diagnostic generated from warning to `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    pub(crate) fn add_warn_diagnostic(&mut self, diag: Diagnostic<DiagnosticStyle>) {
        self.diagnostics.push(diag);
        self.warn_count += 1;
    }

    /// Get count of diagnostics in `DiagnosticHandler`.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    #[inline]
    pub(crate) fn diagnostics_count(&self) -> usize {
        self.diagnostics.len()
    }

    /// Emit all the diagnostics into strings and return.
    pub(crate) fn emit_all_diags_into_string(&self) -> Vec<Result<String>> {
        self.diagnostics
            .iter()
            .map(|d| Ok(emit_diagnostic_to_uncolored_text(d)?))
            .collect()
    }

    /// Emit the [`index`]th diagnostic into string and return.
    pub(crate) fn emit_nth_diag_into_string(&self, index: usize) -> Option<Result<String>> {
        self.diagnostics
            .get(index)
            .map(|d| emit_diagnostic_to_uncolored_text(d))
    }

    /// Emit the diagnostic messages generated from error to to terminal stderr.
    pub(crate) fn emit_error_diagnostic(
        &mut self,
        diag: Diagnostic<DiagnosticStyle>,
    ) -> Result<()> {
        self.emitter.emit_diagnostic(&diag)?;
        self.err_count += 1;
        Ok(())
    }

    /// Emit the diagnostic messages generated from warning to to terminal stderr.
    pub(crate) fn emit_warn_diagnostic(&mut self, diag: Diagnostic<DiagnosticStyle>) -> Result<()> {
        self.emitter.emit_diagnostic(&diag)?;
        self.warn_count += 1;
        Ok(())
    }

    /// Emit all the diagnostics messages to to terminal stderr.
    /// `DiagnosticHandler` contains a set of `Diagnostic<DiagnosticStyle>`
    pub(crate) fn emit_stashed_diagnostics(&mut self) -> Result<()> {
        for diag in &self.diagnostics {
            self.emitter.emit_diagnostic(diag)?
        }
        Ok(())
    }

    /// If some diagnotsics generated by errors, `has_errors` returns `True`.
    #[inline]
    pub(crate) fn has_errors(&self) -> bool {
        self.err_count > 0
    }

    /// If some diagnotsics generated by warnings, `has_errors` returns `True`.
    #[inline]
    pub(crate) fn has_warns(&self) -> bool {
        self.warn_count > 0
    }

    /// After emitting all the diagnostics, it will panic.
    pub(crate) fn abort_if_errors(&mut self) -> Result<()> {
        self.emit_stashed_diagnostics()?;

        if self.has_errors() {
            FatalError.raise();
        }

        Ok(())
    }

    /// Get the message string from "*.ftl" file by `index`, `sub_index` and `MessageArgs`.
    /// "*.ftl" file looks like, e.g. './src/diagnostic/locales/en-US/default.ftl' :
    pub(crate) fn get_diagnostic_msg(
        &self,
        index: &str,
        sub_index: Option<&str>,
        args: &MessageArgs,
    ) -> Result<String> {
        self.template_loader.get_msg_to_str(index, sub_index, args)
    }
}
