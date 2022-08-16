//! Diagnostics creation and emission for `KCLVM`.
//! This module contains the code for creating and emitting diagnostics.
//!
//! We can use `Handler` to create and emit diagnostics.

use kclvm::{ErrType, PanicInfo};

#[macro_use]
pub mod bug;
mod diagnostic;
mod emitter;
mod error;
#[cfg(test)]
mod tests;

use std::sync::Arc;

pub use diagnostic::{Diagnostic, DiagnosticId, Level, Message, Position, Style};
pub use emitter::{Emitter, EmitterWriter};
pub use error::*;
use indexmap::IndexSet;
use kclvm_span::SourceMap;

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
    pub diagnostics: IndexSet<Diagnostic>,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            emitter: Box::new(EmitterWriter::default()),
            diagnostics: Default::default(),
        }
    }
}

impl Handler {
    /// New a handler using a emitter
    pub fn new(emitter: Box<dyn Emitter>) -> Self {
        Self {
            emitter,
            diagnostics: Default::default(),
        }
    }

    pub fn with_source_map(source_map: Arc<SourceMap>) -> Self {
        Self {
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
    pub fn abort_if_errors(&mut self) -> ! {
        if self.emit() {
            std::process::exit(1)
        } else {
            panic!("compiler internal error")
        }
    }

    /// Emit all diagnostics and abort if has any errors.
    pub fn abort_if_any_errors(&mut self) {
        if self.emit() {
            std::process::exit(1)
        }
    }

    /// Emit all diagnostics but do not abort.
    #[inline]
    pub fn alert_if_any_errors(&mut self) {
        if self.has_errors() {
            for diag in &self.diagnostics {
                let pos = diag.messages[0].pos.clone();
                let message = diag.messages[0].message.clone();

                let mut panic_info = PanicInfo::default();

                panic_info.__kcl_PanicInfo__ = true;
                panic_info.message = message;
                panic_info.err_type_code = ErrType::CompileError_TYPE as i32;

                panic_info.kcl_file = pos.filename.clone();
                panic_info.kcl_line = pos.line as i32;
                panic_info.kcl_col = pos.column.unwrap_or(0) as i32;

                panic!("{}", panic_info.to_json_string());
            }
        }
    }

    /// Construct a parse error and put it into the handler diagnostic buffer
    pub fn add_syntex_error(&mut self, msg: &str, pos: Position) -> &mut Self {
        let message = format!("Invalid syntax: {}", msg);
        let diag = Diagnostic::new_with_code(
            Level::Error,
            &message,
            pos,
            Some(DiagnosticId::Error(E1001.kind)),
        );
        self.add_diagnostic(diag);

        self
    }

    /// Construct a parse error and put it into the handler diagnostic buffer
    pub fn add_parse_error(&mut self, err: ParseError, pos: Position) -> &mut Self {
        match err {
            ParseError::UnexpectedToken { expected, got } => {
                let message = format!("expect {:?} got {}", expected, got);
                let diag = Diagnostic::new_with_code(
                    Level::Error,
                    &message,
                    pos,
                    Some(DiagnosticId::Error(E1001.kind)),
                );
                self.add_diagnostic(diag);
            }
        }
        self
    }

    /// Construct a type error and put it into the handler diagnostic buffer
    pub fn add_type_error(&mut self, msg: &str, pos: Position) -> &mut Self {
        let diag = Diagnostic::new_with_code(
            Level::Error,
            msg,
            pos,
            Some(DiagnosticId::Error(E2G22.kind)),
        );
        self.add_diagnostic(diag);

        self
    }

    /// Construct a type error and put it into the handler diagnostic buffer
    pub fn add_compile_error(&mut self, msg: &str, pos: Position) -> &mut Self {
        let diag = Diagnostic::new_with_code(
            Level::Error,
            msg,
            pos,
            Some(DiagnosticId::Error(E2L23.kind)),
        );
        self.add_diagnostic(diag);

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
