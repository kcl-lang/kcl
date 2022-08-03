use std::panic;
use std::rc::Rc;

use emitter::{Emitter, EmitterWriter};
use pendant::NoPendant;
use style::{styled_buffer::StyledBuffer, Shader};

pub mod emitter;
pub mod pendant;

/// Diagnostic is just diagnostc, only responsible for output.
pub struct Diagnostic {
    pub messages: Vec<Sentence>,
}

impl Diagnostic {
    /// New a diagnostic with error code.
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

pub struct Sentence {
    pendant: Box<dyn Pendant>,
    sentence: Message,
}

// TODO(zongz): The 'impl Pendant' can also be replaced by macros 'regiester_pendants'.
pub trait Pendant {
    fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Str(String),
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

    /// TODO(zongz): add fluent msg
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

pub struct ErrHandler {
    emitter: Box<dyn Emitter>,
}

impl ErrHandler {
    pub fn new() -> Self {
        Self {
            emitter: Box::new(EmitterWriter::default()),
        }
    }

    pub fn after_emit(&self) {
        panic::set_hook(Box::new(|_| {}));
        panic!()
    }

    pub fn emit_err(&mut self, err: impl DiagnosticBuilder) {
        self.emitter.emit_diagnostic(&err.into_diagnostic());
        self.after_emit();
    }
}
