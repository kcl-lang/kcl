//! 'components.rs' defines all components with style `DiagnosticStyle` that builtin in compiler_base_error.
use std::{cmp::Ordering, sync::Arc};

use super::{style::DiagnosticStyle, Component};
use crate::errors::ComponentFormatError;
use compiler_base_span::{span_to_filename_string, SourceMap, Span};
use rustc_errors::styled_buffer::{StyledBuffer, StyledString};

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

// Make `StyledString` into a component of diagnostic to display a string with style.
// For more information about `StyledString`, see doc in `/compiler_base/3rdparty/rustc_errors/src/styled_buffer.rs`.
impl Component<DiagnosticStyle> for StyledString<DiagnosticStyle> {
    #[inline]
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, _: &mut Vec<ComponentFormatError>) {
        sb.appendl(&self.text, self.style);
    }
}

/// `IndentWithPrefix` is a component of diagnostic to display an indent with prefix.
/// An indent is a whitespace.
/// ```ignore
/// "|   " is three indent with prefix "|".
/// ```
pub struct IndentWithPrefix {
    indent: usize,
    prefix: StyledString<DiagnosticStyle>,
}

const DEFAULT_INDENT_PREFIX_LABEL: &str = "|";

impl IndentWithPrefix {
    /// Constructs a new `IndentWithPrefix` by default label with 0 indent.
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
    #[inline]
    pub fn default() -> Self {
        Self {
            indent: 0,
            prefix: StyledString::<DiagnosticStyle> {
                text: DEFAULT_INDENT_PREFIX_LABEL.to_string(),
                style: None,
            },
        }
    }

    /// Constructs a new `IndentWithPrefix` by default label with custom indents.
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
    #[inline]
    pub fn new_with_default_label(indent: usize, style: Option<DiagnosticStyle>) -> Self {
        Self {
            indent,
            prefix: StyledString::<DiagnosticStyle>::new(
                DEFAULT_INDENT_PREFIX_LABEL.to_string(),
                style,
            ),
        }
    }

    /// Constructs a new `IndentWithPrefix` by custom label with custom indents.
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
    #[inline]
    pub fn new(prefix: String, indent: usize, prefix_style: Option<DiagnosticStyle>) -> Self {
        Self {
            indent,
            prefix: StyledString::<DiagnosticStyle>::new(prefix, prefix_style),
        }
    }
}

impl Component<DiagnosticStyle> for IndentWithPrefix {
    #[inline]
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
        let indent = self.indent;
        sb.appendl(&format!("{:>indent$}", ""), None);
        self.prefix.format(sb, errs)
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
    symbol: StyledString<DiagnosticStyle>,
}

const DEFAULT_UNDERLINE_LABEL: &str = "^";
impl UnderLine {
    /// Constructs a new `UnderLine` with a default label.
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
    #[inline]
    pub fn new_with_default_label(
        start: usize,
        end: usize,
        style: Option<DiagnosticStyle>,
    ) -> Self {
        Self {
            start,
            end,
            symbol: StyledString::<DiagnosticStyle>::new(
                DEFAULT_UNDERLINE_LABEL.to_string(),
                style,
            ),
        }
    }

    /// Constructs a new `UnderLine` with a custom label.
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
    #[inline]
    pub fn new(start: usize, end: usize, label: String, style: Option<DiagnosticStyle>) -> Self {
        Self {
            start,
            end,
            symbol: StyledString::<DiagnosticStyle>::new(label, style),
        }
    }
}

impl Component<DiagnosticStyle> for UnderLine {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
        match self.start.cmp(&self.end) {
            Ordering::Greater => errs.push(ComponentFormatError::new(
                "UnderLine",
                "Failed to Format UnderLine in One Line.",
            )),
            Ordering::Less => {
                IndentWithPrefix::new("".to_string(), self.start, None).format(sb, errs);
                for _ in self.start..self.end {
                    self.symbol.format(sb, errs);
                }
            }
            Ordering::Equal => {}
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
    /// let code_snippet = CodeSnippet::new(code_span, Arc::new(sm));
    /// code_snippet.format(&mut sb, &mut errs);
    /// ```
    #[inline]
    pub fn new(code_span: Span, source_map: Arc<SourceMap>) -> Self {
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
                            // The line number shown in diagnostic should begin from 1.
                            // The `line.line_index` get from `SourceMap` begin from 0.
                            // So, the line number shown in diagnostic should be equal to line.line_index + 1.
                            let line_index = (line.line_index + 1).to_string();
                            let indent = line_index.len() + 1;
                            IndentWithPrefix::new(line_index, indent, Some(DiagnosticStyle::Url))
                                .format(sb, errs);
                            IndentWithPrefix::default().format(sb, errs);
                            if let Some(line) = sf.get_line(line.line_index) {
                                sb.appendl(&line, None);
                            } else {
                                errs.push(ComponentFormatError::new(
                                    "CodeSnippet",
                                    "Failed to Display Code Snippet.",
                                ))
                            }
                            sb.appendl("\n", None);
                            IndentWithPrefix::new_with_default_label(indent + 1, None)
                                .format(sb, errs);
                            UnderLine::new_with_default_label(
                                line.start_col.0,
                                line.end_col.0,
                                Some(DiagnosticStyle::NeedFix),
                            )
                            .format(sb, errs);
                            // The newline "\n" should not be included at the end of the `CodeSnippet`.
                            // The user can choose whether to add a newline at the end of `CodeSnippet` instead of
                            // having the newline built in at the end of `CodeSnippet`.
                        }
                    }
                    None => errs.push(ComponentFormatError::new(
                        "CodeSnippet",
                        "Failed to Load Source File",
                    )),
                };
            }
            Err(_) => errs.push(ComponentFormatError::new(
                "CodeSnippet",
                "Failed to Display Code Snippet Lines",
            )),
        };
    }
}
