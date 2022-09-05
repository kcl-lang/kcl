//! 'components.rs' defines all components with style `DiagnosticStyle` that builtin in compiler_base_error.
use std::sync::Arc;

use super::{style::DiagnosticStyle, Component};
use crate::errors::ComponentFormatError;
use compiler_base_span::{span_to_filename_string, SourceFile, SourceMap, Span};
use rustc_errors::styled_buffer::StyledBuffer;

/// `Label` can be considered as a component of diagnostic to display a short label message in `Diagnositc`.
/// `Label` provides "error", "warning", "note" and "Help" four kinds of labels.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_error::Component;
/// # use compiler_base_error::components::Label;
/// # use compiler_base_error::DiagnosticStyle;
/// # use rustc_errors::styled_buffer::StyledBuffer;
///
/// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
/// let mut errs = vec![];
///
/// // rendering text: "error[E3131]"
/// Label::Error("E3131".to_string()).format(&mut sb, &mut errs);
///
/// // rendering text: "warning[W3131]"
/// Label::Warning("W3131".to_string()).format(&mut sb, &mut errs);
///
/// // rendering text: "note"
/// Label::Note.format(&mut sb, &mut errs);
///
/// // rendering text: "help"
/// Label::Help.format(&mut sb, &mut errs);
/// ```
pub enum Label {
    Error(String),
    Warning(String),
    Note,
    Help,
}

impl Component<DiagnosticStyle> for Label {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, _: &mut Vec<ComponentFormatError>) {
        let (text, style, code) = match self {
            Label::Error(ecode) => ("error", DiagnosticStyle::NeedFix, Some(ecode)),
            Label::Warning(wcode) => ("warning", DiagnosticStyle::NeedAttention, Some(wcode)),
            Label::Note => ("note", DiagnosticStyle::Important, None),
            Label::Help => ("help", DiagnosticStyle::Helpful, None),
        };
        sb.appendl(text, Some(style));

        // e.g. "error[E1010]"
        if let Some(c) = code {
            sb.appendl("[", Some(DiagnosticStyle::Helpful));
            sb.appendl(c.as_str(), Some(DiagnosticStyle::Helpful));
            sb.appendl("]", Some(DiagnosticStyle::Helpful));
        }
    }
}

/// `UnderLine` is a component of diagnostic to display an underline.
///
/// ```ignore
/// int test = 0;
///     ^^^^ This is an underline
/// ```
pub struct UnderLine {
    start: usize,
    end: usize,
    label: String,
    style: Option<DiagnosticStyle>,
}

impl UnderLine {
    pub fn new_with_default_label(
        start: usize,
        end: usize,
        style: Option<DiagnosticStyle>,
    ) -> Self {
        Self {
            start,
            end,
            label: "^".to_string(),
            style,
        }
    }

    pub fn new(start: usize, end: usize, label: String, style: Option<DiagnosticStyle>) -> Self {
        Self {
            start,
            end,
            label,
            style,
        }
    }
}

impl Component<DiagnosticStyle> for UnderLine {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
        if self.start < self.end {
            let start_col = self.start;
            let end_col = self.end;
            let col_offset = end_col - start_col;

            sb.appendl(&format!("{:>start_col$}", self.label), self.style);
            sb.appendl(&format!("{:^>col_offset$} ", ""), self.style);
        } else if self.start > self.end {
            errs.push(ComponentFormatError::new(
                "UnderLine",
                "Failed To Format UnderLine",
            ))
        }
    }
}

/// `IndentWithPrefix` is a component of diagnostic to display an indent with prefix.
///
/// ```ignore
/// "|   " is three indent with prefix "|".
/// ```
pub struct IndentWithPrefix {
    prefix_label: String,
    prefix_indent: usize,
    style: Option<DiagnosticStyle>,
}

impl IndentWithPrefix {
    pub fn default() -> Self {
        Self {
            prefix_label: "|".to_string(),
            prefix_indent: 0,
            style: None,
        }
    }

    pub fn new_with_default_indent(prefix_label: String, style: Option<DiagnosticStyle>) -> Self {
        Self {
            prefix_label,
            prefix_indent: 0,
            style,
        }
    }

    pub fn new_with_default_label(prefix_indent: usize, style: Option<DiagnosticStyle>) -> Self {
        Self {
            prefix_label: "|".to_string(),
            prefix_indent,
            style,
        }
    }

    pub fn new(prefix_label: String, prefix_indent: usize, style: Option<DiagnosticStyle>) -> Self {
        Self {
            prefix_label,
            prefix_indent,
            style,
        }
    }
}

impl Component<DiagnosticStyle> for IndentWithPrefix {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, _: &mut Vec<ComponentFormatError>) {
        let indent = self.prefix_indent;
        sb.appendl(&format!("{:>indent$}", self.prefix_label), self.style);
    }
}

/// `OneLineCodeSnippet` is a component of diagnostic to display one line code snippet.
pub struct OneLineCodeSnippet {
    line_num: usize,
    sf: Arc<SourceFile>,
    style: Option<DiagnosticStyle>,
}

impl OneLineCodeSnippet {
    pub fn new(line_num: usize, sf: Arc<SourceFile>, style: Option<DiagnosticStyle>) -> Self {
        Self {
            line_num,
            sf,
            style,
        }
    }
}

impl Component<DiagnosticStyle> for OneLineCodeSnippet {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, _: &mut Vec<ComponentFormatError>) {
        if let Some(line) = self.sf.get_line(self.line_num) {
            sb.appendl(&line.to_string(), self.style);
        }
    }
}

pub struct CodeSpan {
    code_span: Span,
    source_map: Arc<SourceMap>,
}

impl CodeSpan {
    pub fn new_with_source_map(code_span: Span, source_map: Arc<SourceMap>) -> Self {
        Self {
            code_span,
            source_map,
        }
    }
}

impl Component<DiagnosticStyle> for CodeSpan {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
        sb.pushs("---> File: ", Some(DiagnosticStyle::Url));
        let file_info = self.source_map.span_to_diagnostic_string(self.code_span);
        sb.appendl(&file_info, Some(DiagnosticStyle::Url));
        sb.appendl("\n", None);
        match self.source_map.span_to_lines(self.code_span) {
            Ok(affected_lines) => {
                match self
                    .source_map
                    .source_file_by_filename(&span_to_filename_string(
                        &self.code_span,
                        &self.source_map,
                    )) {
                    Some(sf) => {
                        for line in affected_lines.lines {
                            let line_index = line.line_index.to_string();
                            let indent = line_index.len() + 1;
                            IndentWithPrefix::new(line_index, indent, Some(DiagnosticStyle::Url))
                                .format(sb, errs);
                            IndentWithPrefix::default().format(sb, errs);
                            OneLineCodeSnippet::new(line.line_index, Arc::clone(&sf), None)
                                .format(sb, errs);
                            sb.appendl("\n", None);
                            IndentWithPrefix::new_with_default_label(indent + 1, None)
                                .format(sb, errs);
                            UnderLine::new_with_default_label(
                                line.start_col.0,
                                line.end_col.0,
                                Some(DiagnosticStyle::NeedFix),
                            )
                            .format(sb, errs);
                            sb.appendl("\n", None);
                        }
                    }
                    None => errs.push(ComponentFormatError::new(
                        "CodeSpan",
                        "Failed To Load Source File",
                    )),
                };
            }
            Err(_) => errs.push(ComponentFormatError::new(
                "CodeSpan",
                "Failed To Get Code Snippet Lines",
            )),
        };
    }
}
