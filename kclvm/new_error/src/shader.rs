use crate::diagnostic::Level;

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Style {
    MainHeaderMsg,
    HeaderMsg,
    LineAndColumn,
    LineNumber,
    Quotation,
    UnderlinePrimary,
    UnderlineSecondary,
    LabelPrimary,
    LabelSecondary,
    NoStyle,
    Level(Level),
    Highlight,
    Addition,
    Removal,
    Empty,
    Line,
}

// Shader 是有必要的，因为把整个shader传进去，
// 想用什么Style就调用什么方法获取对应的Style，
// 如果只将style传递进去，那就只能用一种Style，
// 因为传递进去的东西是常量。
pub trait Shader {
    fn header_style(&self) -> Style;
    fn label_style(&self) -> Style;
    fn file_header_style(&self) -> Style;
    fn line_and_column_style(&self) -> Style;
    fn sentence_style(&self) -> Style;
}

pub struct ColorShader;

impl ColorShader {
    pub fn new() -> Self {
        Self {}
    }
}

impl Shader for ColorShader {
    fn header_style(&self) -> Style {
        Style::Line
    }

    fn label_style(&self) -> Style {
        Style::Line
    }

    fn file_header_style(&self) -> Style {
        Style::Line
    }

    fn line_and_column_style(&self) -> Style {
        Style::Line
    }

    fn sentence_style(&self) -> Style {
        Style::Line
    }
}
