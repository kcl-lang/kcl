//! Source positions and related helper functions.
//!
//! Important concepts in this module include:
//!
//! - the *span*, represented by [`Span`] and related types;
//! - interned strings, represented by [`Symbol`]s, with some common symbols available statically in the [`sym`] module.
//!
//! Reference: https://github.com/rust-lang/rust/blob/master/compiler/rustc_span/src/lib.rs

pub mod span;
pub use rustc_span::fatal_error;
use span::new_byte_pos;
pub use span::{BytePos, Span, SpanData, DUMMY_SP};

pub type SourceMap = rustc_span::SourceMap;
pub type SourceFile = rustc_span::SourceFile;
pub type FilePathMapping = rustc_span::source_map::FilePathMapping;
pub type Loc = rustc_span::Loc;

/// New a `Span`.
///
/// # Examples
///
/// ```
/// use compiler_base_span::new_span;
/// let span = new_span(0, 0);
/// ```
#[inline]
pub fn new_span(lo: u32, hi: u32) -> Span {
    Span::new(new_byte_pos(lo), new_byte_pos(hi))
}

/// Get the filename from `SourceMap` by `Span`.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_span::{span_to_filename_string, span::new_byte_pos, FilePathMapping, SourceMap};
/// # use rustc_span::SpanData;
/// # use std::path::PathBuf;
/// # use std::fs;
///
/// // 1. You need to hold a `SourceMap` at first.
/// let filename = fs::canonicalize(&PathBuf::from("./src/test_datas/code_snippet")).unwrap().display().to_string();
/// let src = std::fs::read_to_string(filename.clone()).unwrap();
/// let sm = SourceMap::new(FilePathMapping::empty());
/// sm.new_source_file(PathBuf::from(filename.clone()).into(), src.to_string());
///
/// // 2. You got the span in `SourceMap`.
/// let code_span = SpanData {
///     lo: new_byte_pos(21),
///     hi: new_byte_pos(22),
/// }.span();
///
/// // 3. You can got the filename by `span_to_filename_string()`.
/// assert_eq!(filename, span_to_filename_string(&code_span, &sm));
/// ```
#[inline]
pub fn span_to_filename_string(span: &Span, sm: &SourceMap) -> String {
    format!("{}", sm.span_to_filename(*span).prefer_remapped())
}

/// Get the position information from `SourceMap` by `Span`.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_span::{span_to_filename_string, span::new_byte_pos, FilePathMapping, SourceMap};
/// # use compiler_base_span::span_to_position_info;
/// # use rustc_span::SpanData;
/// # use std::path::PathBuf;
/// # use std::fs;
///
/// // 1. You need to hold a `SourceMap` at first.
/// const CARGO_ROOT: &str = env!("CARGO_MANIFEST_DIR");
/// let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
/// cargo_file_path.push("src/test_datas/code_snippet");
/// let filename = cargo_file_path.to_str().unwrap();

/// let src = std::fs::read_to_string(filename.clone()).unwrap();
/// let sm = SourceMap::new(FilePathMapping::empty());
/// sm.new_source_file(PathBuf::from(filename.clone()).into(), src);
///
/// // 2. You got the span in `SourceMap`.
/// let span = SpanData {
///     lo: new_byte_pos(10),
///     hi: new_byte_pos(50),
/// }.span();
///
/// // 3. You can got the position information by `span_to_position_info()`.
/// assert_eq!(span_to_position_info(&span, &sm), format!("{}{}", filename, ":1:11: 2:30"));
/// ```
#[inline]
pub fn span_to_position_info(span: &Span, sm: &SourceMap) -> String {
    sm.span_to_diagnostic_string(*span)
}

/// Get the position start line from `SourceMap` by `Span`.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_span::{span_to_filename_string, span::new_byte_pos, FilePathMapping, SourceMap};
/// # use compiler_base_span::span_to_start_line;
/// # use rustc_span::SpanData;
/// # use std::path::PathBuf;
/// # use std::fs;
///
/// // 1. You need to hold a `SourceMap` at first.
/// const CARGO_ROOT: &str = env!("CARGO_MANIFEST_DIR");
/// let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
/// cargo_file_path.push("src/test_datas/code_snippet");
/// let filename = cargo_file_path.to_str().unwrap();

/// let src = std::fs::read_to_string(filename.clone()).unwrap();
/// let sm = SourceMap::new(FilePathMapping::empty());
/// sm.new_source_file(PathBuf::from(filename.clone()).into(), src);
///
/// // 2. You got the span in `SourceMap`.
/// let span = SpanData {
///     lo: new_byte_pos(10),
///     hi: new_byte_pos(50),
/// }.span();
///
/// // 3. You can got the position information by `span_to_start_line()`.
/// assert_eq!(span_to_start_line(&span, &sm), 1);
/// ```
#[inline]
pub fn span_to_start_line(span: &Span, sm: &SourceMap) -> usize {
    sm.lookup_char_pos(span.lo()).line
}

/// Get the position start column from `SourceMap` by `Span`.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_span::{span_to_filename_string, span::new_byte_pos, FilePathMapping, SourceMap};
/// # use compiler_base_span::span_to_start_column;
/// # use rustc_span::SpanData;
/// # use std::path::PathBuf;
/// # use std::fs;
///
/// // 1. You need to hold a `SourceMap` at first.
/// const CARGO_ROOT: &str = env!("CARGO_MANIFEST_DIR");
/// let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
/// cargo_file_path.push("src/test_datas/code_snippet");
/// let filename = cargo_file_path.to_str().unwrap();

/// let src = std::fs::read_to_string(filename.clone()).unwrap();
/// let sm = SourceMap::new(FilePathMapping::empty());
/// sm.new_source_file(PathBuf::from(filename.clone()).into(), src);
///
/// // 2. You got the span in `SourceMap`.
/// let span = SpanData {
///     lo: new_byte_pos(10),
///     hi: new_byte_pos(50),
/// }.span();
///
/// // 3. You can got the position information by `span_to_start_column()`.
/// assert_eq!(span_to_start_column(&span, &sm), 11);
/// ```
#[inline]
pub fn span_to_start_column(span: &Span, sm: &SourceMap) -> usize {
    sm.lookup_char_pos(span.lo()).col_display + 1
}

/// Get the position end line from `SourceMap` by `Span`.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_span::{span_to_filename_string, span::new_byte_pos, FilePathMapping, SourceMap};
/// # use compiler_base_span::span_to_end_line;
/// # use rustc_span::SpanData;
/// # use std::path::PathBuf;
/// # use std::fs;
///
/// // 1. You need to hold a `SourceMap` at first.
/// const CARGO_ROOT: &str = env!("CARGO_MANIFEST_DIR");
/// let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
/// cargo_file_path.push("src/test_datas/code_snippet");
/// let filename = cargo_file_path.to_str().unwrap();

/// let src = std::fs::read_to_string(filename.clone()).unwrap();
/// let sm = SourceMap::new(FilePathMapping::empty());
/// sm.new_source_file(PathBuf::from(filename.clone()).into(), src);
///
/// // 2. You got the span in `SourceMap`.
/// let span = SpanData {
///     lo: new_byte_pos(10),
///     hi: new_byte_pos(50),
/// }.span();
///
/// // 3. You can got the position information by `span_to_end_line()`.
/// assert_eq!(span_to_end_line(&span, &sm), 2);
/// ```
#[inline]
pub fn span_to_end_line(span: &Span, sm: &SourceMap) -> usize {
    sm.lookup_char_pos(span.hi()).line
}

/// Get the position end column from `SourceMap` by `Span`.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_span::{span_to_filename_string, span::new_byte_pos, FilePathMapping, SourceMap};
/// # use compiler_base_span::span_to_end_column;
/// # use rustc_span::SpanData;
/// # use std::path::PathBuf;
/// # use std::fs;
///
/// // 1. You need to hold a `SourceMap` at first.
/// const CARGO_ROOT: &str = env!("CARGO_MANIFEST_DIR");
/// let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
/// cargo_file_path.push("src/test_datas/code_snippet");
/// let filename = cargo_file_path.to_str().unwrap();

/// let src = std::fs::read_to_string(filename.clone()).unwrap();
/// let sm = SourceMap::new(FilePathMapping::empty());
/// sm.new_source_file(PathBuf::from(filename.clone()).into(), src);
///
/// // 2. You got the span in `SourceMap`.
/// let span = SpanData {
///     lo: new_byte_pos(10),
///     hi: new_byte_pos(50),
/// }.span();
///
/// // 3. You can got the position information by `span_to_end_column()`.
/// assert_eq!(span_to_end_column(&span, &sm), 30);
/// ```
#[inline]
pub fn span_to_end_column(span: &Span, sm: &SourceMap) -> usize {
    sm.lookup_char_pos(span.hi()).col_display + 1
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{
        span::new_byte_pos, span_to_end_column, span_to_end_line, span_to_position_info,
        span_to_start_column, span_to_start_line, FilePathMapping, SourceMap, Span,
    };

    const CARGO_ROOT: &str = env!("CARGO_MANIFEST_DIR");

    #[test]
    fn test_span_util_functions() {
        let mut cargo_file_path = PathBuf::from(CARGO_ROOT);
        cargo_file_path.push("src/test_datas/code_snippet");
        let full_path = cargo_file_path.to_str().unwrap();

        let src = std::fs::read_to_string(full_path.clone()).unwrap();
        let sm = SourceMap::new(FilePathMapping::empty());
        sm.new_source_file(PathBuf::from(full_path.clone()).into(), src);

        let span = Span::new(new_byte_pos(10), new_byte_pos(50));

        assert_eq!(span_to_start_line(&span, &sm), 1);
        assert_eq!(span_to_end_line(&span, &sm), 2);

        assert_eq!(span_to_start_column(&span, &sm), 11);
        assert_eq!(span_to_end_column(&span, &sm), 30);

        assert_eq!(
            span_to_position_info(&span, &sm),
            format!("{}{}", full_path, ":1:11: 2:30")
        );
    }
}
