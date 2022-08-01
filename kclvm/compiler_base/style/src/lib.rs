use std::rc::Rc;

use diagnostic::shader::DiagnosticShader;
use termcolor::{Color, ColorSpec};

pub mod diagnostic;
pub mod styled_buffer;

#[cfg(test)]
mod tests;

pub trait Shader {
    fn logo_style(&self) -> Style;
    fn need_fix_style(&self) -> Style;
    fn need_attention_style(&self) -> Style;
    fn helpful_style(&self) -> Style;
    fn important_style(&self) -> Style;
    fn normal_msg_style(&self) -> Style;
    fn url_style(&self) -> Style;
    fn no_style(&self) -> Style;
}

pub enum ShaderFactory {
    Diagnostic,
}

impl ShaderFactory {
    pub fn get_shader(&self) -> Rc<dyn Shader> {
        match self {
            ShaderFactory::Diagnostic => Rc::new(DiagnosticShader::new()),
        }
    }
}

/// FIXME(zongz): Once the 'Style' changed, all the shader are deprecated.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Style {
    Logo,
    NeedFix,
    NeedAttention,
    Helpful,
    Important,
    Normal,
    Url,
    NoStyle,
}

impl Style {
    fn render_style(&mut self) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match self {
            Style::Logo | Style::Normal | Style::NoStyle => todo!(),
            Style::NeedFix => {
                spec.set_fg(Some(Color::Red)).set_intense(true);
                spec.set_bold(true);
            }
            Style::NeedAttention => {
                spec.set_bold(true);
                spec.set_intense(true);
                if cfg!(windows) {
                    spec.set_fg(Some(Color::Cyan));
                } else {
                    spec.set_fg(Some(Color::Blue));
                }
            }
            Style::Helpful | Style::Important | Style::Url => {
                spec.set_fg(Some(Color::Red)).set_intense(true);
                spec.set_bold(true);
            }
        }
        spec
    }
}
