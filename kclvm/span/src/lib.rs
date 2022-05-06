//! Source positions and related helper functions.
//!
//! Important concepts in this module include:
//!
//! - the *span*, represented by [`Span`] and related types;
//! - interned strings, represented by [`Symbol`]s, with some common symbols available statically in the [`sym`] module.
//!
//! Reference: https://github.com/rust-lang/rust/blob/master/compiler/rustc_span/src/lib.rs

mod session_globals;
pub mod span;
pub mod symbol;

#[cfg(test)]
mod tests;

pub use session_globals::create_session_globals_then;
use session_globals::with_session_globals;
pub use span::{BytePos, Span, DUMMY_SP};
pub use symbol::{Ident, Symbol};

pub type SourceMap = rustc_span::SourceMap;
pub type SourceFile = rustc_span::SourceFile;
pub type FilePathMapping = rustc_span::source_map::FilePathMapping;
pub type Loc = rustc_span::Loc;

#[macro_use]
extern crate kclvm_macros;
