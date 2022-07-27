use kclvm_error::Position;

use crate::{
    diagnostic::Diagnostic,
    shader::Level, pendant::{HeaderPendant, CodeCtxPendant, LabelPendant}, sentence::{Sentence, Message},
};

/// Demo Error Struct
/// #[error(msg="oh no! this is an error!", code = "E0124")]
pub struct ThisIsAnErr {
    // #[position(msg =“help....”)]
    pub pos: Position,
    // #[error(msg="oh no! this is an error!")]
    // #[nopendant(msg="For more...")]
}

pub trait DiagnosticBuilder {
    fn into_diagnostic(&self) -> Diagnostic;
}

impl DiagnosticBuilder for ThisIsAnErr {
    fn into_diagnostic(&self) -> Diagnostic {
        let mut diagnostic = Diagnostic::new();

        let title_pendant = HeaderPendant::new(Level::Error, "E3030".to_string());
        let codectx_pendant = CodeCtxPendant::new(self.pos.clone());
        let label_pendant = LabelPendant::new("error".to_string());
        
        let title_sentence = Sentence::new_sentence_str(Box::new(title_pendant), Message::Str("oh no! this is an error!".to_string()));
        let codectx_sentence = Sentence::new_sentence_str(Box::new(codectx_pendant), Message::Str("help: try using a qux here: `qux sad()`".to_string()));
        let label_sentence = Sentence::new_sentence_str(Box::new(label_pendant), Message::Str("oh no! this is an error!".to_string()));
        let nopendant_sentence = Sentence::new_nopendant_sentence(Message::Str("For more information about this error, try `rustc --explain E0999`.".to_string()));

        diagnostic.add_sentence(title_sentence);
        diagnostic.add_sentence(codectx_sentence);
        diagnostic.add_sentence(label_sentence);
        diagnostic.add_sentence(nopendant_sentence);
        
        diagnostic
    }
}
