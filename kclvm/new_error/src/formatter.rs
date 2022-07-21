use crate::{shader::Shader, styled_buffer::StyledBuffer};

pub trait Formatter {
    fn format(&self, shader: Box<&dyn Shader>, styled_buffer: &mut StyledBuffer);
}
