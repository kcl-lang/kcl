use std::rc::Rc;

use crate::{pendant::Pendant, shader::Shader, styled_buffer::StyledBuffer};

pub struct Sentence {
    pendant: Box<dyn Pendant>,
    sentence: Message,
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

    pub fn format(&self, shader: Rc<dyn Shader>, sb: &mut StyledBuffer) {
        let sentence_style = shader.sentence_style();
        self.pendant.format(shader, sb);
        match &self.sentence {
            Message::Str(s) => sb.putl(&s, sentence_style),
            Message::FluentId(s) => sb.putl(&s, sentence_style.clone()),
        }
    }
}
