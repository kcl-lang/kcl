use crate::sentence::Sentence;
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
