use compiler_base_diagnostic::{
    pendant::*, Diagnostic, DiagnosticBuilder, Message, Position, Sentence,
};

use macros::DiagnosticBuilderMacro;

#[cfg(test)]
mod tests;

// CompilerBase-Error
#[derive(DiagnosticBuilderMacro)]
#[nopendant(
    title,
    msg = "For more information about this error, try `rustc --explain E0999`."
)]
#[error(title, msg = "oh no! this is an error!", code = "E012")] // 目前在这里多写没人管，少写不行，因为是从一个list里面往出找。
#[help(title, code = "E00", msg = "I need help !")]
#[error(msg = "oh no! this is an error!")]
pub struct ThisIsAnErr {
    #[position(msg = "err position")]
    pub pos: Position,
}
