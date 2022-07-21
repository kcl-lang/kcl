use crate::snippet::Style;

pub trait Shader {
    fn header_style(&self) -> Style;
    fn label_style(&self) -> Style;
    fn file_header_style(&self) -> Style;
    fn line_and_column_style(&self) -> Style;
}

