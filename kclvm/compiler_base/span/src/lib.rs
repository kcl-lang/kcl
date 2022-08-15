//! Source positions and related helper functions.
//!
//! Important concepts in this module include:
//!
//! - the *span*, represented by [`Span`] and related types;
//! - interned strings, represented by [`Symbol`]s, with some common symbols available statically in the [`sym`] module.
//!
//! Reference: https://github.com/rust-lang/rust/blob/master/compiler/rustc_span/src/lib.rs

pub mod span;
pub use span::{BytePos, Span, DUMMY_SP};

pub type SourceMap = rustc_span::SourceMap;
pub type SourceFile = rustc_span::SourceFile;
pub type FilePathMapping = rustc_span::source_map::FilePathMapping;
pub type Loc = rustc_span::Loc;
