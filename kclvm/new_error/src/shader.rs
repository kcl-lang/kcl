use core::fmt;

use termcolor::{Color, ColorSpec};

pub trait Shader {
    // logo - "KCL"
    fn logo_style(&self) -> Style;
    // error - "error[E0101]"
    fn err_style(&self) -> Style;
    // warning - "warning[W5523]"
    fn warning_style(&self) -> Style;
    // suggestion
    // error: this is an error.
    // warning: this is an warning.
    // help: this is a help tip.
    // note: this is a note.
    fn msg_style(&self) -> Style;
    // line and column - "21:3"
    fn line_and_column_style(&self) -> Style;
    // file path - "User/xxx/xxx/xxx.k"
    fn file_path_style(&self) -> Style;
    // label = "~ \ ^"
    fn label_style(&self) -> Style;
    fn no_style(&self) -> Style;
}

pub struct ColorShader;

impl ColorShader {
    pub fn new() -> Self {
        Self {}
    }
}

impl Shader for ColorShader {
    fn logo_style(&self) -> Style {
        Style::Logo
    }

    fn err_style(&self) -> Style {
        Style::Level(Level::Error)
    }

    fn warning_style(&self) -> Style {
        Style::Level(Level::Warning)
    }

    fn msg_style(&self) -> Style {
        Style::NoStyle
    }

    fn line_and_column_style(&self) -> Style {
        Style::LineAndColumn
    }

    fn file_path_style(&self) -> Style {
        Style::NoStyle
    }

    fn label_style(&self) -> Style {
        Style::Label
    }

    fn no_style(&self) -> Style {
        Style::Empty
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Style {
    Logo,
    Level(Level),
    NoStyle,
    LineAndColumn,
    LineNumber,
    Line,
    Label,
    Quotation,
    Empty,
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
