//! 'pendant.rs' defines all pendants that builtin in compiler_base_error.
use super::{style::DiagnosticStyle, Formatter};
use rustc_errors::styled_buffer::StyledBuffer;

/// LabelPendant: A pendant to shown some label messages for diagnostics.
///
/// e.g.
/// error: this is an error!
/// warning[W0011]: this is an warning!
/// note: this is note.
///
/// 'error', 'warning[W0011]' and 'note' are 'LabelPendant'.
///
/// And `LabelPendant` currently supported text format rendering style:
///
/// - "error" => DiagnosticStyle::NeedFix,
/// - "warning" => DiagnosticStyle::NeedAttention,
/// - "help" => DiagnosticStyle::Helpful,
/// - "note" => DiagnosticStyle::Important,
/// - other  => DiagnosticStyle::NoStyle,
pub struct LabelPendant {
    /// It's just an icon for the language. e.g. Rust、Java or KCL
    logo: Option<String>,
    /// A short label message for an exception. e.g. error, warning, help.
    diag_label: String,
    /// Code for the current exception type. e.g. E1010, W0091
    diag_code: Option<String>,
}

impl LabelPendant {
    pub fn new(diag_label: String, diag_code: Option<String>) -> Self {
        Self {
            logo: None,
            diag_label,
            diag_code,
        }
    }

    pub fn set_logo(&mut self, logo: String) -> &mut Self {
        self.logo = Some(logo);
        self
    }

    pub fn get_logo(&self) -> String {
        self.logo.clone().unwrap()
    }

    pub fn get_label_style_by_label_text(&self, label_text: &str) -> DiagnosticStyle {
        match label_text {
            "error" => DiagnosticStyle::NeedFix,
            "warning" => DiagnosticStyle::NeedAttention,
            "help" => DiagnosticStyle::Helpful,
            "note" => DiagnosticStyle::Important,
            _ => DiagnosticStyle::NoStyle,
        }
    }
}

impl Formatter for LabelPendant {
    /// format `LabelPendant` to string 'logo diag_label[diag_code]'
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::pendant::LabelPendant;
    /// # use crate::compiler_base_error::diagnostic::Formatter;
    /// # use compiler_base_error::diagnostic::style::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// let label_pendant = LabelPendant::new(
    ///     "error".to_string(), Some("E0986".to_string())
    /// );
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let result = label_pendant.format(&mut sb);
    /// ```
    /// the text of `result` will be "error[EO986]".
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::pendant::LabelPendant;
    /// # use crate::compiler_base_error::diagnostic::Formatter;
    /// # use compiler_base_error::diagnostic::style::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// let mut label_pendant = LabelPendant::new(
    ///     "error".to_string(), Some("E0986".to_string())
    /// );
    ///
    /// label_pendant.set_logo("KCL".to_string());
    ///
    /// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
    /// let result = label_pendant.format(&mut sb);
    /// ```
    /// the text of `result` will be "KCL error[EO986]".
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>) {
        if let Some(logo) = &self.logo {
            sb.pushs(&logo, Some(DiagnosticStyle::Logo));
            sb.appendl(" ", Some(DiagnosticStyle::NoStyle));
        }

        let label_text = self.diag_label.as_str();

        let style = self.get_label_style_by_label_text(label_text);
        sb.appendl(label_text, Some(style));

        // e.g. "error[E1010]"
        if let Some(c) = &self.diag_code {
            sb.appendl("[", Some(DiagnosticStyle::Helpful));
            sb.appendl(c.as_str(), Some(DiagnosticStyle::Helpful));
            sb.appendl("]", Some(DiagnosticStyle::Helpful));
        }
        sb.appendl(":", Some(DiagnosticStyle::NoStyle));
    }
}
