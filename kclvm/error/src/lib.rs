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
use compiler_base_error::errors::ComponentFormatError;
use compiler_base_error::StyledBuffer;
use compiler_base_error::{
    components::{CodeSnippet, Label},
    Component, Diagnostic as DiagnosticTrait, DiagnosticStyle,
};
use compiler_base_session::{Session, SessionDiagnostic};
use compiler_base_span::{span::new_byte_pos, Span};
use diagnostic::Range;
use indexmap::IndexSet;
use kclvm_runtime::PanicInfo;
use std::{any::Any, sync::Arc};

pub use diagnostic::{Diagnostic, DiagnosticId, Level, Message, Position, Style};
pub use error::*;

/// A handler deals with errors and other compiler output.
/// Certain errors (error, bug) may cause immediate exit,
/// others log errors for later reporting.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Handler {
    pub diagnostics: IndexSet<Diagnostic>,
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

    /// Emit diagnostic to string.
    pub fn emit_to_string(&mut self) -> Result<String> {
        let sess = Session::default();
        for diag in &self.diagnostics {
            sess.add_err(diag.clone())?;
        }
        let errors = sess.emit_all_diags_into_string()?;
        let mut error_strings = vec![];
        for error in errors {
            error_strings.push(error?);
        }
        Ok(error_strings.join("\n"))
    }

    /// Emit all diagnostics and abort if has any errors.
    pub fn abort_if_any_errors(&mut self) {
        match self.emit() {
            Ok(has_error) => {
                if has_error {
                    std::process::exit(1);
                }
            }
            Err(err) => self.bug(&format!("{err}")),
        }
    }

    /// Construct a parse error and put it into the handler diagnostic buffer
    pub fn add_syntex_error(&mut self, msg: &str, range: Range) -> &mut Self {
        let message = format!("Invalid syntax: {msg}");
        let diag = Diagnostic::new_with_code(
            Level::Error,
            &message,
            None,
            range,
            Some(DiagnosticId::Error(E1001.kind)),
            None,
        );
        self.add_diagnostic(diag);

        self
    }

    /// Construct a type error and put it into the handler diagnostic buffer
    pub fn add_type_error(&mut self, msg: &str, range: Range) -> &mut Self {
        let diag = Diagnostic::new_with_code(
            Level::Error,
            msg,
            None,
            range,
            Some(DiagnosticId::Error(E2G22.kind)),
            None,
        );
        self.add_diagnostic(diag);

        self
    }

    /// Construct a type error and put it into the handler diagnostic buffer
    pub fn add_compile_error(&mut self, msg: &str, range: Range) -> &mut Self {
        self.add_compile_error_with_suggestions(msg, range, None)
    }

    pub fn add_compile_error_with_suggestions(
        &mut self,
        msg: &str,
        range: Range,
        suggestions: Option<Vec<String>>,
    ) -> &mut Self {
        let diag = Diagnostic::new_with_code(
            Level::Error,
            msg,
            None,
            range,
            Some(DiagnosticId::Error(E2L23.kind)),
            suggestions,
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
    ///         range: (Position::dummy_pos(), Position::dummy_pos()),
    ///         style: Style::LineAndColumn,
    ///         message: "Invalid syntax: expected '+', got '-'".to_string(),
    ///         note: None,
    ///         suggested_replacement: None,
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

    pub fn add_suggestions(&mut self, msgs: Vec<String>) -> &mut Self {
        msgs.iter().for_each(|s| {
            self.add_diagnostic(Diagnostic {
                level: Level::Suggestions,
                messages: vec![Message {
                    range: Range::default(),
                    style: Style::Line,
                    message: s.to_string(),
                    note: None,
                    suggested_replacement: None,
                }],
                code: Some(DiagnosticId::Suggestions),
            });
        });

        self
    }

    /// Add an warning into the handler
    /// ```
    /// use kclvm_error::*;
    /// let mut handler = Handler::default();
    /// handler.add_warning(WarningKind::UnusedImportWarning, &[
    ///     Message {
    ///         range: (Position::dummy_pos(), Position::dummy_pos()),
    ///         style: Style::LineAndColumn,
    ///         message: "Module 'a' imported but unused.".to_string(),
    ///         note: None,
    ///         suggested_replacement: None,
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
            if diag.level == Level::Error || diag.level == Level::Suggestions {
                errs.insert(diag.clone());
            } else if diag.level == Level::Warning {
                warnings.insert(diag.clone());
            } else {
                continue;
            }
        }
        (errs, warnings)
    }

    /// Store a diagnostics into the handler.
    ///
    /// # Example
    ///
    /// ```
    /// use kclvm_error::*;
    /// let mut handler = Handler::default();
    /// handler.add_diagnostic(Diagnostic::new_with_code(Level::Error, "error message", None, (Position::dummy_pos(), Position::dummy_pos()), Some(DiagnosticId::Error(E1001.kind)), None));
    /// ```
    #[inline]
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.diagnostics.insert(diagnostic);

        self
    }
}

impl From<PanicInfo> for Diagnostic {
    fn from(panic_info: PanicInfo) -> Self {
        let panic_msg = if panic_info.kcl_arg_msg.is_empty() {
            &panic_info.message
        } else {
            &panic_info.kcl_arg_msg
        };

        let mut diag = if panic_info.backtrace.is_empty() {
            let pos = Position {
                filename: panic_info.kcl_file.clone(),
                line: panic_info.kcl_line as u64,
                column: None,
            };
            Diagnostic::new_with_code(
                Level::Error,
                panic_msg,
                None,
                (pos.clone(), pos),
                None,
                None,
            )
        } else {
            let mut backtrace_msg = "backtrace:\n".to_string();
            let mut backtrace = panic_info.backtrace.clone();
            backtrace.reverse();
            for (index, frame) in backtrace.iter().enumerate() {
                backtrace_msg.push_str(&format!(
                    "\t{index}: {}\n\t\tat {}:{}",
                    frame.func, frame.file, frame.line
                ));
                if frame.col != 0 {
                    backtrace_msg.push_str(&format!(":{}", frame.col))
                }
                backtrace_msg.push('\n')
            }
            let pos = Position {
                filename: panic_info.kcl_file.clone(),
                line: panic_info.kcl_line as u64,
                column: None,
            };
            Diagnostic::new_with_code(
                Level::Error,
                panic_msg,
                Some(&backtrace_msg),
                (pos.clone(), pos),
                None,
                None,
            )
        };

        if panic_info.kcl_config_meta_file.is_empty() {
            return diag;
        }
        let pos = Position {
            filename: panic_info.kcl_config_meta_file.clone(),
            line: panic_info.kcl_config_meta_line as u64,
            column: Some(panic_info.kcl_config_meta_col as u64),
        };
        let mut config_meta_diag = Diagnostic::new_with_code(
            Level::Error,
            &panic_info.kcl_config_meta_arg_msg,
            None,
            (pos.clone(), pos),
            None,
            None,
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

/// A single string error.
pub struct StringError(pub String);

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

impl ParseError {
    /// Convert a parse error into a error diagnostic.
    pub fn into_diag(self, sess: &Session) -> Result<Diagnostic> {
        let span = match self {
            ParseError::UnexpectedToken { span, .. } => span,
            ParseError::Message { span, .. } => span,
        };
        let loc = sess.sm.lookup_char_pos(span.lo());
        let pos: Position = loc.into();
        Ok(Diagnostic::new_with_code(
            Level::Error,
            &self.to_string(),
            None,
            (pos.clone(), pos),
            Some(DiagnosticId::Error(ErrorKind::InvalidSyntax)),
            None,
        ))
    }
}

impl ToString for ParseError {
    fn to_string(&self) -> String {
        match self {
            ParseError::UnexpectedToken { expected, got, .. } => {
                format!("expected one of {expected:?} got {got}")
            }
            ParseError::Message { message, .. } => message.to_string(),
        }
    }
}

impl SessionDiagnostic for ParseError {
    fn into_diagnostic(self, sess: &Session) -> Result<DiagnosticTrait<DiagnosticStyle>> {
        let mut diag = DiagnosticTrait::<DiagnosticStyle>::new();
        diag.append_component(Box::new(Label::Error(E1001.code.to_string())));
        diag.append_component(Box::new(": invalid syntax\n".to_string()));
        match self {
            ParseError::UnexpectedToken { span, .. } => {
                let code_snippet = CodeSnippet::new(span, Arc::clone(&sess.sm));
                diag.append_component(Box::new(code_snippet));
                diag.append_component(Box::new(format!(" {}\n", self.to_string())));
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

#[derive(Default)]
pub struct SuggestionsLabel;

impl Component<DiagnosticStyle> for SuggestionsLabel {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, _: &mut Vec<ComponentFormatError>) {
        sb.appendl("suggestion: ", Some(DiagnosticStyle::NeedAttention));
    }
}

impl SessionDiagnostic for Diagnostic {
    fn into_diagnostic(self, _: &Session) -> Result<DiagnosticTrait<DiagnosticStyle>> {
        let mut diag = DiagnosticTrait::<DiagnosticStyle>::new();
        match self.code {
            Some(id) => match id {
                DiagnosticId::Error(error) => {
                    diag.append_component(Box::new(Label::Error(error.code())));
                    diag.append_component(Box::new(format!(": {}\n", error.name())));
                }
                DiagnosticId::Warning(warning) => {
                    diag.append_component(Box::new(Label::Warning(warning.code())));
                    diag.append_component(Box::new(format!(": {}\n", warning.name())));
                }
                DiagnosticId::Suggestions => {
                    diag.append_component(Box::new(SuggestionsLabel));
                }
            },
            None => match self.level {
                Level::Error => {
                    diag.append_component(Box::new(format!("{}\n", ErrorKind::EvaluationError)));
                }
                Level::Warning => {
                    diag.append_component(Box::new(format!("{}\n", WarningKind::CompilerWarning)));
                }
                Level::Note => {
                    diag.append_component(Box::new(Label::Note));
                }
                Level::Suggestions => {
                    diag.append_component(Box::new(SuggestionsLabel));
                }
            },
        }
        for msg in &self.messages {
            match Session::new_with_file_and_code(&msg.range.0.filename, None) {
                Ok(sess) => {
                    let source = sess.sm.lookup_source_file(new_byte_pos(0));
                    let line = source.get_line((msg.range.0.line - 1) as usize);
                    match line.as_ref() {
                        Some(content) => {
                            let length = content.chars().count();
                            let snippet = Snippet {
                                title: None,
                                footer: vec![],
                                slices: vec![Slice {
                                    source: content,
                                    line_start: msg.range.0.line as usize,
                                    origin: Some(&msg.range.0.filename),
                                    annotations: vec![SourceAnnotation {
                                        range: match msg.range.0.column {
                                            Some(column) if length >= 1 => {
                                                let column = column as usize;
                                                // If the position exceeds the length of the content,
                                                // put the annotation at the end of the line.
                                                if column >= length {
                                                    (length - 1, length)
                                                } else {
                                                    (column, column + 1)
                                                }
                                            }
                                            _ => (0, 0),
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
                diag.append_component(Box::new(format!(": {note}\n")));
            }
            // Append a new line.
            diag.append_component(Box::new(String::from("\n")));
        }
        Ok(diag)
    }
}

impl SessionDiagnostic for StringError {
    fn into_diagnostic(self, _: &Session) -> Result<DiagnosticTrait<DiagnosticStyle>> {
        let mut diag = DiagnosticTrait::<DiagnosticStyle>::new();
        diag.append_component(Box::new(Label::Error(E3M38.code.to_string())));
        diag.append_component(Box::new(format!(": {}\n", self.0)));
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
