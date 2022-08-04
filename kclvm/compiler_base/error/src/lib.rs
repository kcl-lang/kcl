use std::panic;

use compiler_base_diagnostic::{
    pendant::*, Diagnostic, DiagnosticBuilder, Message, Position, Sentence, emitter::{Emitter, EmitterWriter},
};
use compiler_base_macros::DiagnosticBuilderMacro;

#[cfg(test)]
mod tests;

// CompilerBase-Error
#[derive(DiagnosticBuilderMacro)]
#[nopendant(
    title,
    msg = "For more information about this error, try `rustc --explain E0999`."
)]
#[error(title, msg = "oh no! this is an error!", code = "E012")]
#[help(title, code = "E000", msg = "I need help !")]
#[error(msg = "oh no! this is an error!")]
pub struct ThisIsAnErr {
    #[position(msg = "err position")]
    pub pos: Position,
}



