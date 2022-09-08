//! 'style.rs' defines all styles that needed in compiler_base_error.
use rustc_errors::Style;
use termcolor::{Color, ColorSpec};

/// 'DiagnosticStyle' defines all the styles that needed when displaying diagnostic message.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticStyle {
    Logo,
    NeedFix,
    NeedAttention,
    Helpful,
    Important,
    Url,
}

impl Style for DiagnosticStyle {
    fn render_style_to_color_spec(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();
        match self {
            DiagnosticStyle::Logo => {}
            DiagnosticStyle::NeedFix => {
                spec.set_fg(Some(Color::Red))
                    .set_intense(true)
                    .set_bold(true);
            }
            DiagnosticStyle::NeedAttention => {
                spec.set_fg(Some(Color::Yellow))
                    .set_intense(true)
                    .set_bold(true);
            }
            DiagnosticStyle::Helpful => {
                spec.set_fg(Some(Color::Green))
                    .set_intense(true)
                    .set_bold(true);
            }
            DiagnosticStyle::Important => {
                spec.set_fg(Some(Color::Cyan))
                    .set_intense(true)
                    .set_bold(true);
            }
            DiagnosticStyle::Url => {
                spec.set_fg(Some(Color::Blue))
                    .set_intense(true)
                    .set_bold(true);
            }
        }
        spec
    }
}

impl DiagnosticStyle {
    /// Check if a `ColorSpec` is corresponding to the `DiagnosticStyle`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rustc_errors::Style;
    /// # use compiler_base_error::DiagnosticStyle;
    ///
    /// let mut color_spec = DiagnosticStyle::NeedFix.render_style_to_color_spec();
    /// assert!(DiagnosticStyle::NeedFix.check_is_expected_colorspec(&color_spec));
    ///
    /// color_spec.set_bold(false);
    /// assert!(!DiagnosticStyle::NeedFix.check_is_expected_colorspec(&color_spec));
    /// ```
    pub fn check_is_expected_colorspec(&self, spec: &ColorSpec) -> bool {
        match self {
            DiagnosticStyle::Logo => true,
            DiagnosticStyle::NeedFix => {
                spec.fg() == Some(&Color::Red) && spec.intense() && spec.bold()
            }
            DiagnosticStyle::NeedAttention => {
                spec.fg() == Some(&Color::Yellow) && spec.intense() && spec.bold()
            }
            DiagnosticStyle::Helpful => {
                spec.fg() == Some(&Color::Green) && spec.intense() && spec.bold()
            }
            DiagnosticStyle::Important => {
                spec.fg() == Some(&Color::Cyan) && spec.intense() && spec.bold()
            }
            DiagnosticStyle::Url => {
                spec.fg() == Some(&Color::Blue) && spec.intense() && spec.bold()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod test_style {
        use crate::diagnostic::style::DiagnosticStyle;
        use rustc_errors::Style;

        #[test]
        fn test_render_style_to_color_spec() {
            let color_spec = DiagnosticStyle::NeedFix.render_style_to_color_spec();
            assert!(DiagnosticStyle::NeedFix.check_is_expected_colorspec(&color_spec));

            let color_spec = DiagnosticStyle::NeedAttention.render_style_to_color_spec();
            assert!(DiagnosticStyle::NeedAttention.check_is_expected_colorspec(&color_spec));

            let color_spec = DiagnosticStyle::Helpful.render_style_to_color_spec();
            assert!(DiagnosticStyle::Helpful.check_is_expected_colorspec(&color_spec));

            let color_spec = DiagnosticStyle::Important.render_style_to_color_spec();
            assert!(DiagnosticStyle::Important.check_is_expected_colorspec(&color_spec));

            let color_spec = DiagnosticStyle::Logo.render_style_to_color_spec();
            assert!(DiagnosticStyle::Logo.check_is_expected_colorspec(&color_spec));

            let mut color_spec = DiagnosticStyle::Url.render_style_to_color_spec();
            assert!(DiagnosticStyle::Url.check_is_expected_colorspec(&color_spec));

            color_spec.set_bold(false);
            assert!(!DiagnosticStyle::Url.check_is_expected_colorspec(&color_spec));
        }
    }
}
