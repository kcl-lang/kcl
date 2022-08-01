use crate::{Shader, Style};

pub struct DiagnosticShader;

impl DiagnosticShader {
    pub fn new() -> Self {
        Self {}
    }
}

impl Shader for DiagnosticShader {
    fn logo_style(&self) -> Style {
        Style::Logo
    }

    fn need_fix_style(&self) -> Style {
        Style::NeedFix
    }

    fn need_attention_style(&self) -> Style {
        Style::NeedAttention
    }

    fn helpful_style(&self) -> Style {
        Style::Helpful
    }

    fn important_style(&self) -> Style {
        Style::Important
    }

    fn normal_msg_style(&self) -> Style {
        Style::Normal
    }

    fn url_style(&self) -> Style {
        Style::Url
    }

    fn no_style(&self) -> Style {
        Style::NoStyle
    }
}
