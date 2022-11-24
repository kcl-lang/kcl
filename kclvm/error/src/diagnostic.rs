use std::fmt;
use std::hash::Hash;

use kclvm::{ErrType, PanicInfo};
use kclvm_span::Loc;
use rustc_span::Pos;
use termcolor::{Color, ColorSpec};

use crate::{ErrorKind, WarningKind, E2L23};

/// Diagnostic structure.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    pub level: Level,
    pub messages: Vec<Message>,
    pub code: Option<DiagnosticId>,
}

/// Construct 'Diagnostic' from 'PanicInfo'.
impl From<PanicInfo> for Diagnostic {
    fn from(panic_info: PanicInfo) -> Self {
        Self::new_with_code(
            Level::Error,
            &panic_info.message,
            Position {
                filename: panic_info.kcl_file.clone(),
                line: panic_info.kcl_line as u64,
                column: Some(panic_info.kcl_col as u64),
            },
            Some(DiagnosticId::Error(E2L23.kind)),
        )
    }
}

/// Construct 'PanicInfo' from 'Diagnostic'.
impl Into<PanicInfo> for Diagnostic {
    fn into(self) -> PanicInfo {
        let pos = self.messages[0].pos.clone();
        let message = self.messages[0].message.clone();

        let mut panic_info = PanicInfo::default();

        panic_info.__kcl_PanicInfo__ = true;
        panic_info.message = message;
        panic_info.err_type_code = ErrType::CompileError_TYPE as i32;

        panic_info.kcl_file = pos.filename.clone();
        panic_info.kcl_line = pos.line as i32;
        panic_info.kcl_col = pos.column.unwrap_or(0) as i32;
        panic_info
    }
}

impl From<String> for Diagnostic {
    fn from(item: String) -> Self {
        Self::new_with_code(
            Level::Error,
            &format!("{}", item),
            Position::dummy_pos(),
            Some(DiagnosticId::Error(E2L23.kind)),
        )
    }
}

impl Into<String> for Diagnostic {
    fn into(self) -> String {
        let panic_info: PanicInfo = self.into();
        panic_info.to_json_string()
    }
}

/// Position describes an arbitrary source position including the filename,
/// line, and column location.
///
/// A Position is valid if the line number is > 0.
/// The line and column are both 1 based.
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
        let mut info = "---> File ".to_string();
        info += &self.filename;
        info += &format!(":{}", self.line);
        if let Some(column) = self.column {
            info += &format!(":{}", column + 1);
        }
        info
    }
}

impl From<Loc> for Position {
    fn from(loc: Loc) -> Self {
        Self {
            filename: format!("{}", loc.file.name.prefer_remapped()),
            line: loc.line as u64,
            column: if loc.col_display > 0 {
                // Loc col is the (0-based) column offset.
                Some(loc.col.to_usize() as u64 + 1)
            } else {
                None
            },
        }
    }
}

impl Diagnostic {
    pub fn new(level: Level, message: &str, pos: Position) -> Self {
        Diagnostic::new_with_code(level, message, pos, None)
    }

    /// New a diagnostic with error code.
    pub fn new_with_code(
        level: Level,
        message: &str,
        pos: Position,
        code: Option<DiagnosticId>,
    ) -> Self {
        Diagnostic {
            level,
            messages: vec![Message {
                pos,
                style: Style::LineAndColumn,
                message: message.to_string(),
                note: None,
            }],
            code,
        }
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self.level, Level::Error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Message {
    pub pos: Position,
    pub style: Style,
    pub message: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticId {
    Error(ErrorKind),
    Warning(WarningKind),
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
