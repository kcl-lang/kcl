//! 'Diagnostic' is used to show the error/warning .etc diagnostic message.

use std::rc::Rc;
use compiler_base_style::{diagnostic_style::Shader};
use pendant::NoPendant;
use rustc_errors::styled_buffer::StyledBuffer;

pub mod emitter;
pub mod pendant;

#[cfg(test)]
mod tests;

/// 'Diagnostic' consists of 'Sentence'.
/// 
/// e.g. an error diagnostic.
/// error[E0999]: oh no! this is an error!
///  --> mycode.k:3:5
///  |
///3 |     a: int
///  |     ^ error here!
/// error: aborting due to previous error.
/// For more information about this error.
/// 
/// It consists of 4 'Sentence'
/// 
/// Sentence 1: error[E0999]: oh no! this is an error!
/// 
/// Sentence 2: 
/// --> mycode.rs:3:5
///  |
///3 |     a:int
///  |     ^ error here!
/// 
/// Sentence 3: error: aborting due to previous error.
/// 
/// Sentence 4: For more information about this error.
pub struct Diagnostic {
    pub messages: Vec<Sentence>,
}

impl Diagnostic {
    pub fn new() -> Self {
        Diagnostic { messages: vec![] }
    }

    pub fn add_sentence(&mut self, sentence: Sentence) {
        self.messages.push(sentence);
    }
}

pub trait DiagnosticBuilder {
    fn into_diagnostic(self) -> Diagnostic;
}

/// 'Sentence' consists of 'Pendant' and sentence content.
/// 'Pendant' is optional.
/// 
/// e.g. 
/// Sentence 1: error[E0999]: oh no! this is an error!
///     Pendant: error[E0999]:
///     Sentence content: oh no! this is an error!
/// 
/// Sentence 2: 
/// --> mycode.rs:3:5
///  |
///3 |     a:int
///  |     ^ error here!
///     Pendant: 
///     --> mycode.rs:3:5
///       |
///     3 |     a:int
///       |     ^ 
///     Sentence content: error here!
/// 
/// Sentence 3: error: aborting due to previous error.
///     Pendant: error
///     Sentence content: aborting due to previous error.
/// 
/// Sentence 4: For more information about this error.
///     Pendant: no pendant
///     Sentence content: For more information about this error.
pub struct Sentence {
    pendant: Box<dyn Pendant>,
    sentence: Message,
}

pub trait Pendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    // 'Sentence' text message
    Str(String),
    // 'Sentence' text template id
    // TODO(zong-zhe): Some text messages are too long and require template.
    FluentId(String),
}

impl Sentence {
    pub fn new_sentence_str(pendant: Box<dyn Pendant>, sentence: Message) -> Self {
        Self { pendant, sentence }
    }

    pub fn new_nopendant_sentence(sentence: Message) -> Self {
        Self {
            pendant: Box::new(NoPendant::new()),
            sentence,
        }
    }

    pub fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        let sentence_style = shader.normal_msg_style();
        self.pendant.format(shader, sb);
        match &self.sentence {
            Message::Str(s) => sb.appendl(&s, sentence_style),
            Message::FluentId(s) => sb.appendl(&s, sentence_style.clone()),
        }
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