use std::any::Any;
use termcolor::{ColorSpec, Color};

use crate::{option_box_style, Style};

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum DiagnosticStyle {
    Logo,
    NeedFix,
    NeedAttention,
    Helpful,
    Important,
    Normal,
    Url,
    NoStyle,
}

impl Style for DiagnosticStyle{
    fn as_any(&self) -> &dyn Any{
        self
    }

    fn box_clone(&self) -> Box<dyn Style> {
        Box::new((*self).clone())
    }

    fn render_style(&self) -> ColorSpec {
        self.render_style()
    }

    fn style_eq(&self, other: &Box<dyn Style>) -> bool {
        let other_style: &DiagnosticStyle = match other.as_any().downcast_ref::<DiagnosticStyle>() {
            Some(style) => style,
            // TODO(zongz): needs an error handler.
            None => panic!("&a isn't a style!"),
        };

        let self_style: &DiagnosticStyle = match self.as_any().downcast_ref::<DiagnosticStyle>() {
            Some(style) => style,
            None => panic!("&a isn't a style!"),
        };
        self_style == other_style
    }
}

impl DiagnosticStyle {

    pub fn render_style(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match self {
            DiagnosticStyle::Logo | DiagnosticStyle::Normal | DiagnosticStyle::NoStyle => {}
            DiagnosticStyle::NeedFix => {
                spec.set_fg(Some(Color::Red)).set_intense(true);
                spec.set_bold(true);
            }
            DiagnosticStyle::NeedAttention => {
                spec.set_fg(Some(Color::Yellow)).set_intense(true);
                spec.set_bold(true);
            }
            DiagnosticStyle::Helpful => {
                spec.set_fg(Some(Color::Green)).set_intense(true);
                spec.set_bold(true);
            }
            DiagnosticStyle::Important => {
                spec.set_fg(Some(Color::Cyan)).set_intense(true);
                spec.set_bold(true);
            }
            DiagnosticStyle::Url => {
                spec.set_fg(Some(Color::Blue)).set_intense(true);
                spec.set_bold(true);
            }
        }
        spec
    }

    pub fn check_is_expected_colorspec(&self, spec: &ColorSpec) {
        match self {
            DiagnosticStyle::Logo | DiagnosticStyle::Normal | DiagnosticStyle::NoStyle => assert!(true),
            DiagnosticStyle::NeedFix => {
                assert_eq!(spec.fg(), Some(&Color::Red));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            DiagnosticStyle::NeedAttention => {
                assert_eq!(spec.fg(), Some(&Color::Yellow));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            DiagnosticStyle::Helpful => {
                assert_eq!(spec.fg(), Some(&Color::Green));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            DiagnosticStyle::Important => {
                assert_eq!(spec.fg(), Some(&Color::Cyan));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
            DiagnosticStyle::Url => {
                assert_eq!(spec.fg(), Some(&Color::Blue));
                assert_eq!(spec.intense(), true);
                assert_eq!(spec.bold(), true);
            }
        }
    }
}

pub trait Shader {
    fn logo_style(&self) -> Option<Box<dyn Style>>;
    fn need_fix_style(&self) -> Option<Box<dyn Style>>;
    fn need_attention_style(&self) -> Option<Box<dyn Style>>;
    fn helpful_style(&self) -> Option<Box<dyn Style>>;
    fn important_style(&self) -> Option<Box<dyn Style>>;
    fn normal_msg_style(&self) -> Option<Box<dyn Style>>;
    fn url_style(&self) -> Option<Box<dyn Style>>;
    fn no_style(&self) -> Option<Box<dyn Style>>;
}

pub struct DiagnosticShader;
impl DiagnosticShader{
    pub fn new()-> Self{
        Self{}
    }
}

impl Shader for DiagnosticShader{
    fn logo_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::Logo)
    }

    fn need_fix_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::NeedFix)
    }

    fn need_attention_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::NeedAttention)
    }

    fn helpful_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::Helpful)
    }

    fn important_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::Important)
    }

    fn normal_msg_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::Normal)
    }

    fn url_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::Url)
    }

    fn no_style(&self) -> Option<Box<dyn Style>> {
        option_box_style!(DiagnosticStyle::NoStyle)
    }
}

#[cfg(test)]
mod tests {
    
mod test_style {
    use crate::diagnostic_style::DiagnosticStyle;

    #[test]
    fn test_render_style() {
        let color_spec = DiagnosticStyle::NeedFix.render_style();
        DiagnosticStyle::NeedFix.check_is_expected_colorspec(&color_spec);
        let color_spec = DiagnosticStyle::NeedAttention.render_style();
        DiagnosticStyle::NeedAttention.check_is_expected_colorspec(&color_spec);
        let color_spec = DiagnosticStyle::Helpful.render_style();
       DiagnosticStyle::Helpful.check_is_expected_colorspec(&color_spec);
        let color_spec =DiagnosticStyle::Important.render_style();
       DiagnosticStyle::Important.check_is_expected_colorspec(&color_spec);
        let color_spec =DiagnosticStyle::Logo.render_style();
       DiagnosticStyle::Logo.check_is_expected_colorspec(&color_spec);
        let color_spec =DiagnosticStyle::NoStyle.render_style();
       DiagnosticStyle::NoStyle.check_is_expected_colorspec(&color_spec);
        let color_spec =DiagnosticStyle::Normal.render_style();
       DiagnosticStyle::Normal.check_is_expected_colorspec(&color_spec);
        let color_spec =DiagnosticStyle::Url.render_style();
       DiagnosticStyle::Url.check_is_expected_colorspec(&color_spec);
    }
}

}