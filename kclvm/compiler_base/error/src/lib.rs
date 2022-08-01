use diagnostic::Position;
use macros::DiagnosticBuilder;
use diagnostic::DiagnosticBuilder;
use diagnostic::Diagnostic;
use diagnostic::pendant::CodeCtxPendant;
use diagnostic::Sentence;
use diagnostic::Message;
use diagnostic::pendant::HeaderPendant;

#[cfg(test)]
mod tests;

// CompilerBase-Error

#[derive(DiagnosticBuilder)]
#[error(title, msg = "oh no! this is an error!", code = "E0124")]
#[help(title, msg = "I need help !", code = "E0124")]
#[nopendant(msg = "For more information about this error, try `rustc --explain E0999`.")]
#[error(msg = "oh no! this is an error!")]
pub struct ThisIsAnErr {
    #[position(msg = "err position")]
    pub loc: Position,
}
