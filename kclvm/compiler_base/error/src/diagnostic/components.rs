//! 'components.rs' defines all components with style `DiagnosticStyle` that builtin in compiler_base_error.
use super::{style::DiagnosticStyle, Component};
use crate::errors::ComponentFormatError;
use rustc_errors::styled_buffer::StyledBuffer;

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
