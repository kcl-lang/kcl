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
pub use span::{BytePos, Span, SpanData, DUMMY_SP};

pub type SourceMap = rustc_span::SourceMap;
pub type SourceFile = rustc_span::SourceFile;
pub type FilePathMapping = rustc_span::source_map::FilePathMapping;
pub type Loc = rustc_span::Loc;

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
