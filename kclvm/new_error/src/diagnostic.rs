use std::hash::Hash;
use kclvm_error::ErrorKind;
use crate::{sentence::Sentence, shader::Level};

/// Diagnostic structure.
pub struct Diagnostic {
    pub level: Level,
    pub code: Option<DiagnosticId>,
    pub messages: Vec<Sentence>,
}

impl Diagnostic {
    pub fn new(level: Level) -> Self {
        Diagnostic::new_with_code(level, None)
    }

    /// New a diagnostic with error code.
    pub fn new_with_code(level: Level, code: Option<DiagnosticId>) -> Self {
        Diagnostic {
            level,
            messages: vec![],
            code,
        }
    }

    pub fn add_message(&mut self, sentence: Sentence) {
        self.messages.push(sentence);
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self.level, Level::Error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticId {
    Error(ErrorKind),
    Warning(String),
}
