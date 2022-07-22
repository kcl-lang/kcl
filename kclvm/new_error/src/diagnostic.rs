use std::fmt;
use std::hash::Hash;

use kclvm_error::ErrorKind;
use termcolor::{Color, ColorSpec};

use crate::sentence::Sentence;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    Error,
    Warning,
    Note,
}

impl Level {
    pub fn to_str(self) -> &'static str {
        match self {
            Level::Error => "Error",
            Level::Warning => "Warning",
            Level::Note => "Note",
        }
    }

    pub fn color(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match self {
            Level::Error => {
                spec.set_fg(Some(Color::Red)).set_intense(true);
            }
            Level::Warning => {
                spec.set_fg(Some(Color::Yellow)).set_intense(cfg!(windows));
            }
            Level::Note => {
                spec.set_fg(Some(Color::Green)).set_intense(true);
            }
        }
        spec
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

/// Style indicates the style of error message:
/// - `LineAndColumn` is <filename>:<line>:<column>
/// - `Line` is <filename>:<line>
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    Empty,
    LineAndColumn,
    Line,
}
