//! Diagnostics creation and emission for `KCLVM`.
//! This module contains the code for creating and emitting diagnostics.
//!
//! We can use `Handler` to create and emit diagnostics.

pub mod diagnostic;
mod error;

use annotate_snippets::{
    display_list::DisplayList,
    display_list::FormatOptions,
    snippet::{AnnotationType, Slice, Snippet, SourceAnnotation},
};
use anyhow::Result;
use compiler_base_error::{
    components::{CodeSnippet, Label},
    Diagnostic as DiagnosticTrait, DiagnosticStyle,
};
use compiler_base_session::{Session, SessionDiagnostic};
use compiler_base_span::{span::new_byte_pos, Span};
use indexmap::IndexSet;
use kclvm_runtime::PanicInfo;
use std::{any::Any, sync::Arc};

pub use diagnostic::{Diagnostic, DiagnosticId, Level, Message, Position, Style};
pub use error::*;

/// A handler deals with errors and other compiler output.
/// Certain errors (error, bug) may cause immediate exit,
/// others log errors for later reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handler {
    pub diagnostics: IndexSet<Diagnostic>,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            diagnostics: Default::default(),
        }
    }
}

impl Handler {
    /// New a handler using a emitter
    pub fn new() -> Self {
        Self {
            diagnostics: Default::default(),
        }
    }

    /// Panic program and report a bug
    #[inline]
    pub fn bug(&self, msg: &str) -> ! {
        compiler_base_macros::bug!("{}", msg)
    }

    #[inline]
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diag| diag.level == Level::Error)
    }

    /// Emit all diagnostics and return whether has errors.
    pub fn emit(&mut self) -> Result<bool> {
        let sess = Session::default();
        for diag in &self.diagnostics {
            sess.add_err(diag.clone())?;
        }
        sess.emit_stashed_diagnostics()?;
        Ok(self.has_errors())
    }

    /// Emit all diagnostics but do not abort and return the error json string format.
    #[inline]
    pub fn alert_if_any_errors(&self) -> Result<(), String> {
        if self.has_errors() {
            for diag in &self.diagnostics {
                if !diag.messages.is_empty() {
                    let pos = diag.messages[0].pos.clone();

                    let mut panic_info = PanicInfo::from(diag.messages[0].message.clone());
                    panic_info.kcl_file = pos.filename.clone();
                    panic_info.kcl_line = pos.line as i32;
                    panic_info.kcl_col = pos.column.unwrap_or(0) as i32;

                    if diag.messages.len() >= 2 {
                        let pos = diag.messages[1].pos.clone();
                        panic_info.kcl_config_meta_file = pos.filename.clone();
                        panic_info.kcl_config_meta_line = pos.line as i32;
                        panic_info.kcl_config_meta_col = pos.column.unwrap_or(0) as i32;
                    }

                    return Err(panic_info.to_json_string());
                }
            }
        }
        Ok(())
    }

    /// Emit all diagnostics and abort if has any errors.
    pub fn abort_if_any_errors(&mut self) {
        match self.emit() {
            Ok(has_error) => {
                if has_error {
                    std::process::exit(1);
                }
            }
            Err(err) => self.bug(&format!("{}", err.to_string())),
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

    /// Put a runtime panic info the handler diagnostic buffer.
    pub fn add_panic_info(&mut self, panic_info: &PanicInfo) -> &mut Self {
        self.add_diagnostic(panic_info.clone().into());

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

    /// Classify diagnostics into errors and warnings.
    pub fn classification(&self) -> (IndexSet<Diagnostic>, IndexSet<Diagnostic>) {
        let (mut errs, mut warnings) = (IndexSet::new(), IndexSet::new());
        for diag in &self.diagnostics {
            if diag.level == Level::Error {
                errs.insert(diag.clone());
            } else if diag.level == Level::Warning {
                warnings.insert(diag.clone());
            } else {
                continue;
            }
        }
        (errs, warnings)
    }

    /// Store a diagnostics
    #[inline]
    fn add_diagnostic(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.diagnostics.insert(diagnostic);

        self
    }
}

impl From<PanicInfo> for Diagnostic {
    fn from(panic_info: PanicInfo) -> Self {
        let mut diag = Diagnostic::new_with_code(
            Level::Error,
            if panic_info.kcl_arg_msg.is_empty() {
                &panic_info.message
            } else {
                &panic_info.kcl_arg_msg
            },
            Position {
                filename: panic_info.kcl_file.clone(),
                line: panic_info.kcl_line as u64,
                column: None,
            },
            Some(DiagnosticId::Error(E3M38.kind)),
        );
        if panic_info.kcl_config_meta_file.is_empty() {
            return diag;
        }
        let mut config_meta_diag = Diagnostic::new_with_code(
            Level::Error,
            &panic_info.kcl_config_meta_arg_msg,
            Position {
                filename: panic_info.kcl_config_meta_file.clone(),
                line: panic_info.kcl_config_meta_line as u64,
                column: Some(panic_info.kcl_config_meta_col as u64),
            },
            Some(DiagnosticId::Error(E3M38.kind)),
        );
        config_meta_diag.messages.append(&mut diag.messages);
        config_meta_diag
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: Vec<String>,
        got: String,
        span: Span,
    },
    Message {
        message: String,
        span: Span,
    },
}

impl ParseError {
    /// New a unexpected token parse error with span and token information.
    pub fn unexpected_token(expected: &[&str], got: &str, span: Span) -> Self {
        ParseError::UnexpectedToken {
            expected: expected
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            got: got.to_string(),
            span,
        }
    }

    /// New a message parse error with span.
    pub fn message(message: String, span: Span) -> Self {
        ParseError::Message { message, span }
    }
}

impl ToString for ParseError {
    fn to_string(&self) -> String {
        match self {
            ParseError::UnexpectedToken { expected, got, .. } => {
                format!("unexpected one of {expected:?} got {got}")
            }
            ParseError::Message { message, .. } => message.to_string(),
        }
    }
}

impl SessionDiagnostic for ParseError {
    fn into_diagnostic(self, sess: &Session) -> Result<DiagnosticTrait<DiagnosticStyle>> {
        let mut diag = DiagnosticTrait::<DiagnosticStyle>::new();
        diag.append_component(Box::new(Label::Error(E1001.code.to_string())));
        diag.append_component(Box::new(": invalid syntax".to_string()));
        match self {
            ParseError::UnexpectedToken {
                expected,
                got,
                span,
            } => {
                let code_snippet = CodeSnippet::new(span, Arc::clone(&sess.sm));
                diag.append_component(Box::new(code_snippet));
                diag.append_component(Box::new(format!(
                    " expected one of {expected:?} got {got}\n"
                )));
                Ok(diag)
            }
            ParseError::Message { message, span } => {
                let code_snippet = CodeSnippet::new(span, Arc::clone(&sess.sm));
                diag.append_component(Box::new(code_snippet));
                diag.append_component(Box::new(format!(" {message}\n")));
                Ok(diag)
            }
        }
    }
}

impl SessionDiagnostic for Diagnostic {
    fn into_diagnostic(self, _: &Session) -> Result<DiagnosticTrait<DiagnosticStyle>> {
        let mut diag = DiagnosticTrait::<DiagnosticStyle>::new();
        match self.code {
            Some(id) => match id {
                DiagnosticId::Error(error) => {
                    diag.append_component(Box::new(Label::Error(E2L23.code.to_string())));
                    diag.append_component(Box::new(format!(": {}", error.name())));
                }
                DiagnosticId::Warning(warning) => {
                    diag.append_component(Box::new(Label::Warning(W1001.code.to_string())));
                    diag.append_component(Box::new(format!(": {}", warning.name())));
                }
            },
            None => match self.level {
                Level::Error => {
                    diag.append_component(Box::new(Label::Error(E2L23.code.to_string())));
                }
                Level::Warning => {
                    diag.append_component(Box::new(Label::Warning(W1001.code.to_string())));
                }
                Level::Note => {
                    diag.append_component(Box::new(Label::Note));
                }
            },
        }
        // Append a new line.
        diag.append_component(Box::new(String::from("\n")));
        for msg in &self.messages {
            match Session::new_with_file_and_code(&msg.pos.filename, None) {
                Ok(sess) => {
                    let source = sess.sm.lookup_source_file(new_byte_pos(0));
                    let line = source.get_line((msg.pos.line - 1) as usize);
                    match line.as_ref() {
                        Some(content) => {
                            let snippet = Snippet {
                                title: None,
                                footer: vec![],
                                slices: vec![Slice {
                                    source: content,
                                    line_start: msg.pos.line as usize,
                                    origin: Some(&msg.pos.filename),
                                    annotations: vec![SourceAnnotation {
                                        range: match msg.pos.column {
                                            Some(column) => {
                                                (column as usize, (column + 1) as usize)
                                            }
                                            None => (0, 0),
                                        },
                                        label: &msg.message,
                                        annotation_type: AnnotationType::Error,
                                    }],
                                    fold: true,
                                }],
                                opt: FormatOptions {
                                    color: true,
                                    anonymized_line_numbers: false,
                                    margin: None,
                                },
                            };
                            let dl = DisplayList::from(snippet);
                            diag.append_component(Box::new(format!("{dl}\n")));
                        }
                        None => {
                            diag.append_component(Box::new(format!("{}\n", msg.message)));
                        }
                    };
                }
                Err(_) => diag.append_component(Box::new(format!("{}\n", msg.message))),
            };
            if let Some(note) = &msg.note {
                diag.append_component(Box::new(Label::Note));
                diag.append_component(Box::new(format!(": {}\n", note)));
            }
            // Append a new line.
            diag.append_component(Box::new(String::from("\n")));
        }
        Ok(diag)
    }
}

/// Convert an error to string.
///
/// ```
/// use kclvm_error::err_to_str;
///
/// assert_eq!(err_to_str(Box::new("error_string".to_string())), "error_string");
/// ```
pub fn err_to_str(err: Box<dyn Any + Send>) -> String {
    if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<&String>() {
        (*s).clone()
    } else if let Some(s) = err.downcast_ref::<String>() {
        (*s).clone()
    } else {
        "".to_string()
    }
}
