use kclvm_span::Loc;
use std::fmt;
use std::hash::Hash;

use crate::{ErrorKind, WarningKind};

/// Diagnostic structure.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    pub level: Level,
    pub messages: Vec<Message>,
    pub code: Option<DiagnosticId>,
}

/// Position describes an arbitrary source position including the filename,
/// line, and column location.
///
/// A Position is valid if the line number is > 0.
/// The line is 1-based and the column is 0-based.
#[derive(PartialEq, Clone, Eq, Hash, Debug, Default)]
pub struct Position {
    pub filename: String,
    pub line: u64,
    pub column: Option<u64>,
}

impl Position {
    #[inline]
    pub fn dummy_pos() -> Self {
        Position {
            filename: "".to_string(),
            line: 1,
            column: None,
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.line > 0
    }

    pub fn less(&self, other: &Position) -> bool {
        if !self.is_valid() || !other.is_valid() || self.filename != other.filename {
            false
        } else if self.line < other.line {
            true
        } else if self.line == other.line {
            match (self.column, other.column) {
                (Some(column), Some(other_column)) => column < other_column,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn less_equal(&self, other: &Position) -> bool {
        if !self.is_valid() || !other.is_valid() {
            false
        } else if self.less(other) {
            true
        } else {
            self == other
        }
    }

    pub fn info(&self) -> String {
        if !self.filename.is_empty() {
            let mut info = "---> File ".to_string();
            info += &self.filename;
            info += &format!(":{}", self.line);
            if let Some(column) = self.column {
                info += &format!(":{}", column + 1);
            }
            info
        } else {
            "".to_string()
        }
    }
}

impl From<Loc> for Position {
    fn from(loc: Loc) -> Self {
        Self {
            filename: format!("{}", loc.file.name.prefer_remapped()),
            line: loc.line as u64,
            column: if loc.col_display > 0 {
                // Loc col is the (0-based) column offset.
                Some(loc.col.0 as u64)
            } else {
                None
            },
        }
    }
}

impl Diagnostic {
    pub fn new(level: Level, message: &str, range: Range) -> Self {
        Diagnostic::new_with_code(level, message, None, range, None, None)
    }

    /// New a diagnostic with error code.
    pub fn new_with_code(
        level: Level,
        message: &str,
        note: Option<&str>,
        range: Range,
        code: Option<DiagnosticId>,
        suggestions: Option<Vec<String>>,
    ) -> Self {
        Diagnostic {
            level,
            messages: vec![Message {
                range,
                style: Style::LineAndColumn,
                message: message.to_string(),
                note: note.map(String::from),
                suggested_replacement: suggestions,
            }],
            code,
        }
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self.level, Level::Error)
    }
}

pub type Range = (Position, Position);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Message {
    pub range: Range,
    pub style: Style,
    pub message: String,
    pub note: Option<String>,
    pub suggested_replacement: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticId {
    Error(ErrorKind),
    Warning(WarningKind),
    Suggestions,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    Error,
    Warning,
    Note,
    Suggestions,
}

impl Level {
    pub fn to_str(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
            Level::Suggestions => "suggestions",
        }
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
