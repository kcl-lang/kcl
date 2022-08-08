//! 'Style' is responsible for providing 'Shader' for text color rendering.
use std::{rc::Rc, any::Any};

use diagnostic_style::{Shader, DiagnosticShader};
use termcolor::ColorSpec;

pub mod diagnostic_style;

#[macro_export]
macro_rules! option_box_style {
    ($node: expr) => {
        Some(Box::new($node))
    };
}

pub trait Style{
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn Style>;
    fn style_eq(&self, other: &Box<dyn Style>) -> bool;
    fn render_style(&self) -> ColorSpec;
}

impl PartialEq for Box<dyn Style> {
    fn eq(&self, other: &Self) -> bool {
        self.style_eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Clone for Box<dyn Style>
{
    fn clone(&self) -> Box<dyn Style> {
        self.box_clone()
    }
}

pub enum ShaderFactory {
    Default,
    Diagnostic,
}

impl ShaderFactory {
    pub fn get_shader(&self) -> Rc<dyn Shader> {
        match self {
            ShaderFactory::Diagnostic => Rc::new(DiagnosticShader::new()),
            _ => todo!()
            // ShaderFactory::Default => Rc::new(DefaultShader::new()),
        }
    }
}