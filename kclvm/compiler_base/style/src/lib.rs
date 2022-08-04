use std::rc::Rc;

use shader::{DefaultShader, DiagnosticShader};
use termcolor::{Color, ColorSpec};

mod shader;
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
    Default,
    Diagnostic,
}

impl ShaderFactory {
    pub fn get_shader(&self) -> Rc<dyn Shader> {
        match self {
            ShaderFactory::Diagnostic => Rc::new(DiagnosticShader::new()),
            ShaderFactory::Default => Rc::new(DefaultShader::new()),
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
    pub fn render_style(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match self {
            Style::Logo | Style::Normal | Style::NoStyle => {}
            Style::NeedFix => {
                spec.set_fg(Some(Color::Red)).set_intense(true);
                spec.set_bold(true);
            }
            Style::NeedAttention => {
                spec.set_fg(Some(Color::Yellow)).set_intense(true);
                spec.set_bold(true);
            }
            Style::Helpful => {
                spec.set_fg(Some(Color::Green)).set_intense(true);
                spec.set_bold(true);
            }
            Style::Important => {
                spec.set_fg(Some(Color::Cyan)).set_intense(true);
                spec.set_bold(true);
            }
            Style::Url => {
                spec.set_fg(Some(Color::Blue)).set_intense(true);
                spec.set_bold(true);
            }
        }
        spec
    }

    pub fn check_is_expected_colorspec(&self, spec: &ColorSpec) {
        match self {
            Style::Logo | Style::Normal | Style::NoStyle => assert!(true),
            Style::NeedFix => {
                assert_eq!(spec.fg(), Some(&Color::Red));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            Style::NeedAttention => {
                assert_eq!(spec.fg(), Some(&Color::Yellow));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            Style::Helpful => {
                assert_eq!(spec.fg(), Some(&Color::Green));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            Style::Important => {
                assert_eq!(spec.fg(), Some(&Color::Cyan));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            Style::Url => {
                assert_eq!(spec.fg(), Some(&Color::Blue));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
        }
    }
}
