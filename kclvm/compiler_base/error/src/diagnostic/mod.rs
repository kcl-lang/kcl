use self::style::DiagnosticStyle;
use rustc_errors::styled_buffer::StyledBuffer;

pub mod pendant;
pub mod style;

#[cfg(test)]
mod tests;

/// 'Formatter' specifies the method `format()` that all Pendants/Sentences/SentenceMessage should implement.
pub trait Formatter {
    /// `format()` formats `Pendant` into `StyledString` and saves them in `StyledBuffer`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// struct PendantWithStyleLogo{
    ///     text: String
    /// }
    ///
    /// impl Pendant for PendantWithStyleLogo{
    ///     fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>){
    ///         // set style
    ///         sb.pushs(&self.text, Some(DiagnosticStyle::Logo));
    ///     }
    /// }
    ///
    /// ```
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>);
}

/// `Sentence` consists of `pendant` and `sentence_message`.
/// `pendant` is optional.
///
/// e.g.
/// Sentence 1: error[E0999]: oh no! this is an error!
///     pendant: error[E0999]:
///     sentence_message: oh no! this is an error!
///
/// Sentence 2:
/// --> mycode.X:3:5
///  |
///3 |     a:int
///  |     ^ error here!
///     pendant:
///     --> mycode.X:3:5
///       |
///     3 |     a:int
///       |     ^
///     sentence content: error here!
///
/// Sentence 3: error: aborting due to previous error.
///     pendant: error
///     sentence content: aborting due to previous error.
///
/// Sentence 4: For more information about this error.
///     pendant: -
///     sentence content: For more information about this error.
///
/// `Sentence` supports nesting.
///
/// e.g.
/// --> mycode.X:3:5
///  |
///3 |     a:int
///  |     ^ help: error here!
///
/// "help: error here!" is another `Sentence` whose `pendant` is "help" and `sentence_message` is "error here!".
pub struct Sentence {
    pendant: Option<Box<dyn Formatter>>,
    sentence_message: Box<dyn Formatter>,
}

impl Sentence {
    pub fn new_sentence_str(
        pendant: Box<dyn Formatter>,
        sentence_message: Box<dyn Formatter>,
    ) -> Self {
        Self {
            pendant: Some(pendant),
            sentence_message,
        }
    }

    pub fn new_nopendant_sentence(sentence_message: Box<dyn Formatter>) -> Self {
        Self {
            pendant: None,
            sentence_message,
        }
    }
}

impl Formatter for Sentence {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>) {
        if let Some(p) = &self.pendant {
            p.format(sb);
        }
        self.sentence_message.format(sb)
    }
}

/// `String` is a type of `sentence_message` supported by `Sentence`.
///
/// `sentence_message` of type `String` will be append to the end line of the `StyledBuffer`.
impl Formatter for String {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>) {
        sb.appendl(&self, Some(DiagnosticStyle::NoStyle));
    }
}
