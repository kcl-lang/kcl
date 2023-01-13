//! 'components.rs' defines all components with style `DiagnosticStyle` that builtin in compiler_base_error.
use std::{cmp::Ordering, sync::Arc};

use super::{style::DiagnosticStyle, Component};
use crate::errors::ComponentFormatError;
use compiler_base_span::{span_to_filename_string, SourceFile, SourceMap, Span};
use rustc_errors::styled_buffer::{StyledBuffer, StyledString};
use rustc_span::LineInfo;

const CODE_LINE_PREFIX: &str = " | ";
const FILE_PATH_PREFIX: &str = "-->";

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
                let indent = self.start;
                format!("{:<indent$}", "").format(sb, errs);
                for _ in self.start..self.end {
                    self.symbol.format(sb, errs);
                }
            }
            Ordering::Equal => {}
        }
    }
}

/// `CodeSnippet` is a component of diagnostic to display code snippets.
///
/// Note:
/// If the span spans multiple lines of code, only the first line of the code will be selected.
///
/// In the text rendered by [`CodeSnippet`], the specific position of the span will be highlighted by an underline.
///
/// Therefore, we recommend that do not use a span with a large scope,
/// the scope of the span should be as small as possible and point to the problem location in the code snippet.
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

impl Component<DiagnosticStyle> for CodeSnippet {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
        match self.source_map.span_to_lines(self.code_span) {
            Ok(affected_lines) => {
                match self
                    .source_map
                    .source_file_by_filename(&span_to_filename_string(
                        &self.code_span,
                        &self.source_map,
                    )) {
                    Some(sf) => {
                        // If the span cross multiple lines of code,
                        // only the first line of the code will be selected.
                        if let Some(line) = affected_lines.lines.first() {
                            let indent = (line.line_index + 1).to_string().len();
                            self.format_file_info(sb, errs, &affected_lines.lines, indent);
                            StyledString::new(
                                format!("{:<indent$}{}\n", "", CODE_LINE_PREFIX),
                                Some(DiagnosticStyle::Url),
                            )
                            .format(sb, errs);
                            self.format_code_line(sb, errs, line, indent, &sf)
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

impl CodeSnippet {
    /// Format a code line in [`CodeSnippet`] into '<line_no> | <src_code_line>'
    ///
    /// <line_no>: The line number of the first line of code in the code snippet.
    /// <src_code_line>: The src code.
    ///
    /// e.g. "12 | int a = 10;"
    fn format_code_line(
        &self,
        sb: &mut StyledBuffer<DiagnosticStyle>,
        errs: &mut Vec<ComponentFormatError>,
        line: &LineInfo,
        indent: usize,
        sf: &SourceFile,
    ) {
        // The line number shown in diagnostic should begin from 1.
        // The `line.line_index` get from `SourceMap` begin from 0.
        // So, the line number shown in diagnostic should be equal to line.line_index + 1.
        let line_index = (line.line_index + 1).to_string();
        StyledString::new(
            format!("{:<indent$}{}", line_index, CODE_LINE_PREFIX),
            Some(DiagnosticStyle::Url),
        )
        .format(sb, errs);

        if let Some(line) = sf.get_line(line.line_index) {
            sb.appendl(&line, None);
        } else {
            errs.push(ComponentFormatError::new(
                "CodeSnippet",
                "Failed to Display Code Snippet.",
            ))
        }
        sb.appendl("\n", None);

        StyledString::new(
            format!("{:<indent$}{}", "", CODE_LINE_PREFIX),
            Some(DiagnosticStyle::Url),
        )
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

    /// Format file information in [`CodeSnippet`] into '--> <file_path>:<line_no>:<col_no>'.
    ///
    /// <file_path>: The full path of the span.
    /// <line_no>: The line number of the first line of code in the code snippet.
    /// <col_no>: The column number of the first line of code in the code snippet.
    ///
    /// e.g. "--> /User/test/file_name.file_extension:1:10"
    fn format_file_info(
        &self,
        sb: &mut StyledBuffer<DiagnosticStyle>,
        errs: &mut Vec<ComponentFormatError>,
        lines: &[LineInfo],
        indent: usize,
    ) {
        let (first_line, first_col) = match lines.first() {
            Some(line) => (line.line_index + 1, line.start_col.0 + 1),
            None => {
                errs.push(ComponentFormatError::new(
                    "CodeSnippet",
                    "Failed to Display Code Snippet.",
                ));
                (0, 0)
            }
        };
        StyledString::new(
            format!("{:>indent$}{}", "", FILE_PATH_PREFIX),
            Some(DiagnosticStyle::Url),
        )
        .format(sb, errs);

        StyledString::new(
            format!(
                " {}:{}:{}\n",
                span_to_filename_string(&self.code_span, &self.source_map),
                first_line,
                first_col
            ),
            Some(DiagnosticStyle::Url),
        )
        .format(sb, errs);
    }
}
