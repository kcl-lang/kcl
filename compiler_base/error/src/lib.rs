//! Compiler-Base-Error
//!
//! The idea with `Compiler-Base-Error` is to make a reusable library,
//! by separating out error thorwing and diagnostic diaplaying or other error handling procedures.
//!
//! - Compiler-Base-Error provides `DiagnosticHandler` to diaplay diagnostic.
//! For more information about `DiagnosticHandler`, see doc in 'compiler_base/error/diagnostic/diagnostic_handler.rs'.
//!
//! - TODO(zongz): Compiler-Base-Error provides `ErrorRecover` to recover from errors.

mod diagnostic;
mod emitter;
#[cfg(test)]
mod tests;

pub mod errors;

pub use diagnostic::{
    components, diagnostic_handler, style::DiagnosticStyle, Component, Diagnostic,
};
pub use emitter::{Emitter, TerminalEmitter};
