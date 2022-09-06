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
///     ^^^^ This is an underline under variable `test`
/// ```
pub struct UnderLine {
    start: usize,
    end: usize,
    label: String,
    style: Option<DiagnosticStyle>,
}

const DEFAULT_UNDERLINE_LABEL: &str = "^";
impl UnderLine {
    /// You can new an underline with a default label.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::Component;
    /// # use compiler_base_error::components::UnderLine;
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let mut errs = vec![];
    ///
    /// // rendering text: "^^^^^^^^^^"
    /// let ul = UnderLine::new_with_default_label(0, 10, None);
    /// ul.format(&mut sb, &mut errs);
    ///
    /// // rendering text: "^^^^^^^^^^" in `DiagnosticStyle::NeedFix`.
    /// let ul_need_fix = UnderLine::new_with_default_label(0, 10, Some(DiagnosticStyle::NeedFix));
    /// ul_need_fix.format(&mut sb, &mut errs);
    /// ```
    pub fn new_with_default_label(
        start: usize,
        end: usize,
        style: Option<DiagnosticStyle>,
    ) -> Self {
        Self {
            start,
            end,
            label: DEFAULT_UNDERLINE_LABEL.to_string(),
            style,
        }
    }

    /// You can new an underline with a custom label.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::Component;
    /// # use compiler_base_error::components::UnderLine;
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let mut errs = vec![];
    ///
    /// // rendering text: "__________"
    /// let ul = UnderLine::new(0, 10, "_".to_string(), None);
    /// ul.format(&mut sb, &mut errs);
    ///
    /// // rendering text: "~~" in `DiagnosticStyle::NeedFix`.
    /// let ul_need_fix = UnderLine::new(0, 2, "~".to_string(), Some(DiagnosticStyle::NeedFix));
    /// ul_need_fix.format(&mut sb, &mut errs);
    /// ```
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
/// An indent is a whitespace.
/// ```ignore
/// "|   " is three indent with prefix "|".
/// ```
pub struct IndentWithPrefix {
    prefix_label: String,
    prefix_indent: usize,
    style: Option<DiagnosticStyle>,
}

const DEFAULT_INDENT_PREFIX_LABEL: &str = "|";

impl IndentWithPrefix {
    /// You can new a `IndentWithPrefix` by default label with 0 indent.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::Component;
    /// # use compiler_base_error::components::IndentWithPrefix;
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let mut errs = vec![];
    ///
    /// // If you want to render default text: "|"
    /// let indent = IndentWithPrefix::default();
    /// indent.format(&mut sb, &mut errs);
    /// ```
    pub fn default() -> Self {
        Self {
            prefix_label: DEFAULT_INDENT_PREFIX_LABEL.to_string(),
            prefix_indent: 0,
            style: None,
        }
    }

    /// You can new a `IndentWithPrefix` by default label with custom indents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::Component;
    /// # use compiler_base_error::components::IndentWithPrefix;
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let mut errs = vec![];
    ///
    /// // If you want to add 3 indents and render text: "   |"
    /// let indent = IndentWithPrefix::new_with_default_label(3, None);
    /// indent.format(&mut sb, &mut errs);
    /// ```
    pub fn new_with_default_label(prefix_indent: usize, style: Option<DiagnosticStyle>) -> Self {
        Self {
            prefix_label: DEFAULT_INDENT_PREFIX_LABEL.to_string(),
            prefix_indent,
            style,
        }
    }

    /// You can new a `IndentWithPrefix` by custom label with custom indents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::Component;
    /// # use compiler_base_error::components::IndentWithPrefix;
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let mut errs = vec![];
    /// // If you want to add 3 indents and rendering text: "   ^"
    /// let indent = IndentWithPrefix::new("^".to_string(), 3, None);
    /// indent.format(&mut sb, &mut errs);
    /// ```
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
    /// # Examples
    ///
    /// If you want to get one line code snippet from 'compiler_base/error/src/diagnostic/test_datas/code_snippet' file
    /// ```ignore
    /// Line 1 Code Snippet.
    /// Line 2 Code Snippet.
    /// ```
    ///
    /// ```rust
    /// # use compiler_base_error::{
    /// #     Component,
    /// #     components::OneLineCodeSnippet,
    /// #     DiagnosticStyle,
    /// # };
    /// # use compiler_base_span::{
    /// #     SourceMap,
    /// #     FilePathMapping,
    /// #     span_to_filename_string
    /// # };
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    /// # use compiler_base_span::{span::new_byte_pos, SpanData};
    /// # use std::{path::PathBuf, sync::Arc, fs};
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let mut errs = vec![];
    ///
    /// // 1. You shouled load the file and create the `SourceFile`
    /// let filename = fs::canonicalize(&PathBuf::from("./src/diagnostic/test_datas/code_snippet"))
    ///     .unwrap()
    ///     .display()
    ///     .to_string();
    ///
    /// let src = std::fs::read_to_string(filename.clone()).unwrap();
    /// let sm = SourceMap::new(FilePathMapping::empty());
    /// sm.new_source_file(PathBuf::from(filename.clone()).into(), src.to_string());
    ///
    /// // 2. You should create a code span for the code snippet.
    /// let code_span = SpanData {
    ///     lo: new_byte_pos(22),
    ///     hi: new_byte_pos(42),
    /// }.span();
    ///
    /// // 3. You should get the `SourceFile` by the code span.
    /// let sf = sm.source_file_by_filename(&span_to_filename_string(
    ///     &code_span,
    ///     &sm,
    /// )).unwrap();
    ///
    /// // 4. You can create the `OneLineCodeSnippet` by the `SourceFile`,
    /// // and render text "Line 2 Code Snippet.".
    /// let code_snippet = OneLineCodeSnippet::new(2, sf, None);
    /// code_snippet.format(&mut sb, &mut errs);
    /// ```
    pub fn new(line_num: usize, sf: Arc<SourceFile>, style: Option<DiagnosticStyle>) -> Self {
        Self {
            line_num,
            sf,
            style,
        }
    }
}

impl Component<DiagnosticStyle> for OneLineCodeSnippet {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
        if let Some(line) = self.sf.get_line(self.line_num) {
            sb.appendl(&line.to_string(), self.style);
        } else {
            errs.push(ComponentFormatError::new(
                "OneLineCodeSnippet",
                "Failed To Format OneLineCodeSnippet",
            ))
        }
    }
}

/// `CodeSnippet` is a component of diagnostic to display code snippets.
pub struct CodeSnippet {
    code_span: Span,
    source_map: Arc<SourceMap>,
}

impl CodeSnippet {
    /// # Examples
    ///
    /// If you want to get one line code snippet from 'compiler_base/error/src/diagnostic/test_datas/code_snippet' file
    /// ```ignore
    /// Line 1 Code Snippet.
    /// Line 2 Code Snippet.
    /// ```
    ///
    /// ```rust
    /// # use compiler_base_error::{
    /// #     Component,
    /// #     components::OneLineCodeSnippet,
    /// #     DiagnosticStyle,
    /// # };
    /// # use compiler_base_span::{
    /// #     SourceMap,
    /// #     FilePathMapping,
    /// #     span_to_filename_string
    /// # };
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    /// # use compiler_base_span::{span::new_byte_pos, SpanData};
    /// # use compiler_base_error::components::CodeSnippet;
    /// # use std::{path::PathBuf, sync::Arc, fs};
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let mut errs = vec![];
    ///
    /// // 1. You shouled load the file and create the `SourceFile`
    /// let filename = fs::canonicalize(&PathBuf::from("./src/diagnostic/test_datas/code_snippet"))
    ///     .unwrap()
    ///     .display()
    ///     .to_string();
    ///
    /// let src = std::fs::read_to_string(filename.clone()).unwrap();
    /// let sm = SourceMap::new(FilePathMapping::empty());
    /// sm.new_source_file(PathBuf::from(filename.clone()).into(), src.to_string());
    ///
    /// // 2. You should create a code span for the code snippet.
    /// let code_span = SpanData {
    ///     lo: new_byte_pos(22),
    ///     hi: new_byte_pos(42),
    /// }.span();
    ///
    /// // 3. You can create the `CodeSnippet` by the `SourceFile`,
    /// // and render text "Line 2 Code Snippet.".
    /// let code_snippet = CodeSnippet::new_with_source_map(code_span, Arc::new(sm));
    /// code_snippet.format(&mut sb, &mut errs);
    /// ```
    pub fn new_with_source_map(code_span: Span, source_map: Arc<SourceMap>) -> Self {
        Self {
            code_span,
            source_map,
        }
    }
}

const DEFAULT_FILE_PATH_PREFIX: &str = "---> File: ";

impl Component<DiagnosticStyle> for CodeSnippet {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
        sb.pushs(DEFAULT_FILE_PATH_PREFIX, Some(DiagnosticStyle::Url));
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
                        "CodeSnippet",
                        "Failed To Load Source File",
                    )),
                };
            }
            Err(_) => errs.push(ComponentFormatError::new(
                "CodeSnippet",
                "Failed To Get Code Snippet Lines",
            )),
        };
    }
}
