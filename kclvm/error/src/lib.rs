//! Diagnostics creation and emission for `KCLVM`.
//! This module contains the code for creating and emitting diagnostics.
//!
//! We can use `Handler` to create and emit diagnostics.

use kclvm::PanicInfo;

#[macro_use]
pub mod bug;
mod diagnostic;
mod emitter;
mod error;
#[cfg(test)]
mod tests;

use std::{fmt, sync::Arc};

pub use diagnostic::{Diagnostic, DiagnosticId, Level, Message, Position, Style};
pub use emitter::{Emitter, EmitterWriter};
pub use error::*;
use indexmap::IndexSet;
use kclvm_span::SourceMap;

/// Default value of switch to hide panic messages.
const HIDE_PANIC_INFO: bool = false;

/// A handler deals with errors and other compiler output.
/// Certain errors (error, bug) may cause immediate exit,
/// others log errors for later reporting.
/// ```no_check
/// use kclvm_error::{Handler, Position, ParseError};
/// let mut handler = Handler::default();
/// handler.add_parse_error(
///     ParseError::unexpected_token(&["+", "-", "*", "/"], "//"),
///     Position::dummy_pos(),
/// );
/// handler.abort_if_errors();
/// ```
pub struct Handler {
    /// The number of errors that have been emitted, including duplicates.
    ///
    /// This is not necessarily the count that's reported to the user once
    /// compilation ends.
    emitter: Box<dyn Emitter>,
    /// A switch to hide panic messages.
    /// The default value is false, when an error occurs, the error message will be filled into 'PanicInfo' and panic out.
    /// If the 'hide_panic_info' is true, when an error occurs, the error message will not be raised by panic.
    hide_panic_info: bool,
    pub diagnostics: IndexSet<Diagnostic>,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            hide_panic_info: HIDE_PANIC_INFO,
            emitter: Box::new(EmitterWriter::default()),
            diagnostics: Default::default(),
        }
    }
}

impl Handler {
    /// New a handler using a emitter
    pub fn new(emitter: Box<dyn Emitter>) -> Self {
        Self {
            hide_panic_info: HIDE_PANIC_INFO,
            emitter,
            diagnostics: Default::default(),
        }
    }

    /// Return a flag show whether to hide the error panic message.
    #[inline]
    pub fn is_panic_info_hided(&self) -> bool {
        self.hide_panic_info
    }

    /// Set a flag show whether to hide the error panic message.
    #[inline]
    pub fn set_hide_panic_info(&mut self, is_hided: bool) -> &Self {
        self.hide_panic_info = is_hided;
        self
    }

    pub fn with_source_map(source_map: Arc<SourceMap>) -> Self {
        Self {
            hide_panic_info: HIDE_PANIC_INFO,
            emitter: Box::new(EmitterWriter::from_stderr(source_map)),
            diagnostics: Default::default(),
        }
    }

    /// Panic program and report a bug
    #[inline]
    pub fn bug(&self, msg: &str) -> ! {
        bug!("{}", msg)
    }

    #[inline]
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diag| diag.level == Level::Error)
    }

    /// Emit all diagnostics and return whether has errors.
    pub fn emit(&mut self) -> bool {
        for diag in &self.diagnostics {
            self.emitter.emit_diagnostic(diag);
        }
        self.has_errors()
    }

    /// Format and return all diagnostics msg.
    pub fn format_diagnostic(&mut self) -> Vec<String> {
        let mut dia_msgs = Vec::new();
        for diag in &self.diagnostics {
            dia_msgs.append(&mut self.emitter.format_diagnostic(diag));
        }
        dia_msgs
    }

    /// Emit all diagnostics and abort if has any errors.
    /// If there is no diagnostic, this method will do nothing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kclvm_error::Handler;
    /// use kclvm_error::Position;
    /// let mut handler = Handler::default();
    /// // If there is no error in 'Handler', then calling the 'abort_if_errors' method will do nothing.
    /// handler.abort_if_errors();
    /// // Otherwise the program will panic.
    /// handler.add_compile_error("error message", Position::dummy_pos());
    /// // Panic here.
    /// handler.abort_if_errors();
    ///
    /// ```
    pub fn abort_if_errors(&mut self) {
        if self.emit_now() {
            let panic_info: PanicInfo = match self.diagnostics.first() {
                Some(diag) => diag.clone().into(),
                None => {
                    bug!("Internal Bugs: Please connect us to fix: There is no error diagnostic")
                }
            };
            self.panic_now(panic_info.to_json_string())
        }
    }

    /// Emit the diagnostic of an error and abort.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kclvm_error::E1001;
    /// use kclvm_error::Handler;
    /// use kclvm_error::Position;
    /// let mut handler = Handler::default();
    /// // The program will emit an error diagnostic and panic here.
    /// handler.abort_if_any_error(E1001.kind, "error message", Position::dummy_pos());
    /// ```
    pub fn abort_if_any_error(&mut self, err_kind: ErrorKind, msg: &str, pos: Position) -> ! {
        let panic_info: PanicInfo = self.contract_diagnostic(err_kind, msg, pos).into();
        self.emit_now();
        self.panic_now(panic_info.to_json_string())
    }

    /// Display diagnostic according to the flag 'hide_panic_info'.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_error::E1001;
    /// use kclvm_error::Handler;
    /// use kclvm_error::Position;
    /// let mut handler = Handler::default();
    /// // Set flag 'hide_panic_info' true.
    /// handler.set_hide_panic_info(true);
    /// handler.add_compile_error("error message", Position::dummy_pos());
    /// // Nothing will diplay.
    /// handler.emit_now();
    /// // Set flag 'hide_panic_info' false.
    /// handler.set_hide_panic_info(false);
    /// // 'Diagnostic' will display.
    /// handler.emit_now();
    /// ```
    pub fn emit_now(&mut self) -> bool {
        if self.is_panic_info_hided() {
            return self.emit();
        }
        self.has_errors()
    }

    /// Panic with 'PanicInfo' according to the flag 'hide_panic_info'.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use kclvm_error::E1001;
    /// use kclvm_error::Handler;
    /// use kclvm_error::Position;
    /// use kclvm::PanicInfo;
    ///
    /// let mut handler = Handler::default();
    /// // Set flag 'hide_panic_info' true.
    /// handler.set_hide_panic_info(true);
    /// let panic_info = PanicInfo::default();
    /// // It will panic with 'PanicInfo'.
    /// handler.panic_now(panic_info.to_json_string());
    /// // Set flag 'hide_panic_info' false.
    /// handler.set_hide_panic_info(false);
    /// // It will panic with an empty json string "{{}}".
    /// handler.panic_now(panic_info.to_json_string());
    /// ```
    pub fn panic_now(&self, panic_info: String) -> ! {
        if self.is_panic_info_hided() {
            std::panic::set_hook(Box::new(|_info| {}));
            panic!("{{}}")
        }
        panic!("{}", panic_info)
    }

    /// New a diagnostic and add it into 'Handler'.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kclvm_error::E1001;
    /// use kclvm_error::Handler;
    /// use kclvm_error::Position;
    /// use kclvm::PanicInfo;
    ///
    /// let mut handler = Handler::default();
    /// assert!(!handler.has_errors());
    /// // New a diagnostic and add it into 'Handler'.
    /// handler.contract_diagnostic(E1001.kind, "error msg", Position::default());
    /// assert!(handler.has_errors());
    /// ```
    pub fn contract_diagnostic(
        &mut self,
        err_kind: ErrorKind,
        msg: &str,
        pos: Position,
    ) -> Diagnostic {
        let diag =
            Diagnostic::new_with_code(Level::Error, &msg, pos, Some(DiagnosticId::Error(err_kind)));
        self.add_diagnostic(diag.clone());

        diag
    }

    // Construct 'PanicInfo' from 'Diagnositc' and panic it.
    pub fn panic_diagnostic(&self, diag: Diagnostic) {
        let panic_info: PanicInfo = diag.into();
        self.panic_now(panic_info.to_json_string());
    }

    /// Construct a parse error and put it into the handler diagnostic buffer
    pub fn add_syntex_error(&mut self, msg: &str, pos: Position) -> &mut Self {
        self.contract_diagnostic(E1001.kind, &format!("Invalid syntax: {}", msg), pos);
        self
    }

    /// Construct a parse error and put it into the handler diagnostic buffer
    pub fn add_parse_error(&mut self, err: ParseError, pos: Position) -> &mut Self {
        match err {
            ParseError::UnexpectedToken { expected, got } => {
                let message = format!("expect {:?} got {}", expected, got);
                self.contract_diagnostic(E1001.kind, &message, pos);
                self
            }
        }
    }

    /// Construct a type error and put it into the handler diagnostic buffer
    pub fn add_type_error(&mut self, msg: &str, pos: Position) -> &mut Self {
        self.contract_diagnostic(E2G22.kind, msg, pos);
        self
    }

    /// Construct a type error and put it into the handler diagnostic buffer
    pub fn add_compile_error(&mut self, msg: &str, pos: Position) -> &mut Self {
        self.contract_diagnostic(E2L23.kind, msg, pos);
        self
    }

    /// Put a runtime panic info the handler diagnostic buffer.
    pub fn add_panic_info(&mut self, panic_info: &PanicInfo) -> &mut Self {
        self.add_diagnostic(Diagnostic::from(panic_info.clone()));
        self
    }

    /// Add an error into the handler
    /// ```
    /// use kclvm_error::*;
    /// let mut handler = Handler::default();
    /// handler.add_error(ErrorKind::InvalidSyntax, &[
    ///     Message {
    ///         pos: Position::dummy_pos(),
    ///         style: Style::LineAndColumn,
    ///         message: "Invalid syntax: expected '+', got '-'".to_string(),
    ///         note: None,
    ///     }
    /// ]);
    /// ```
    pub fn add_error(&mut self, err: ErrorKind, msgs: &[Message]) -> &mut Self {
        let diag = Diagnostic {
            level: Level::Error,
            messages: msgs.to_owned(),
            code: Some(DiagnosticId::Error(err)),
        };
        self.add_diagnostic(diag);

        self
    }

    /// Add an warning into the handler
    /// ```
    /// use kclvm_error::*;
    /// let mut handler = Handler::default();
    /// handler.add_warning(WarningKind::UnusedImportWarning, &[
    ///     Message {
    ///         pos: Position::dummy_pos(),
    ///         style: Style::LineAndColumn,
    ///         message: "Module 'a' imported but unused.".to_string(),
    ///         note: None,
    ///     }],
    /// );
    /// ```
    pub fn add_warning(&mut self, warning: WarningKind, msgs: &[Message]) -> &mut Self {
        let diag = Diagnostic {
            level: Level::Warning,
            messages: msgs.to_owned(),
            code: Some(DiagnosticId::Warning(warning)),
        };
        self.add_diagnostic(diag);

        self
    }

    /// Store a diagnostics
    #[inline]
    fn add_diagnostic(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.diagnostics.insert(diagnostic);

        self
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { expected: Vec<String>, got: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, got } => {
                return write!(
                    f,
                    "Unexpected token, expected '{:?}', got '{}'",
                    expected, got
                );
            }
        };
    }
}

impl ParseError {
    pub fn unexpected_token(expected: &[&str], got: &str) -> Self {
        ParseError::UnexpectedToken {
            expected: expected
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            got: got.to_string(),
        }
    }
}

/// Used as a return value to signify a fatal error occurred. (It is also
/// used as the argument to panic at the moment, but that will eventually
/// not be true.)
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct FatalError;

pub struct FatalErrorMarker;

impl FatalError {
    pub fn raise(self) -> ! {
        std::panic::panic_any(Box::new(FatalErrorMarker))
    }
}

impl std::fmt::Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fatal error")
    }
}

impl std::error::Error for FatalError {}
