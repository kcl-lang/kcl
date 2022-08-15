use self::style::DiagnosticStyle;
pub use rustc_errors::styled_buffer::StyledBuffer;

pub mod components;
pub mod style;

#[cfg(test)]
mod tests;

/// 'Component' specifies the method `format()` that all diagnostic components should implement.
pub trait Component {
    /// `format()` formats components into `StyledString` and saves them in `StyledBuffer`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// struct ComponentWithStyleLogo {
    ///     text: String
    /// }
    ///
    /// impl Component for ComponentWithStyleLogo {
    ///     fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>) {
    ///         // set style
    ///         sb.pushs(&self.text, Some(DiagnosticStyle::Logo));
    ///     }
    /// }
    ///
    /// ```
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>);
}

/// `Diagnostic` is a collection of various components,
/// and any data structure that implements `Component` can be a part of `Diagnostic`.
///
/// # Examples
///
/// ```rust
/// # use rustc_errors::styled_buffer::StyledBuffer;
/// # use compiler_base_error::diagnostic::{Diagnostic, components::Label, style::DiagnosticStyle, Component};
///
/// // If you want a diagnostic message “error[E3033]: this is an error!”.
/// let mut diagnostic = Diagnostic::new();
///
/// // First, create a label component wrapped by `Box<>`
/// let err_label = Box::new(Label::Error("E3033".to_string()));
///
/// // Second, add the label component to `Diagnostic`.
/// diagnostic.append_component(err_label);
///
/// // Then, create a string component wrapped by `Box<>`.
/// let msg = Box::new(": this is an error!".to_string());
///
/// // And add it to `Diagnostic`.
/// diagnostic.append_component(msg);
///
/// // Create a `Styledbuffer` to get the result.
/// let mut sb = StyledBuffer::<DiagnosticStyle>::new();
///
/// // Rendering !
/// diagnostic.format(&mut sb);
/// let result = sb.render();
///
/// // “error[E3033]: this is an error!” is only one line.
/// assert_eq!(result.len(), 1);
///
/// // “error[E3033]: this is an error!” has three different style snippets.
///
/// // "error" - DiagnosticStyle::NeedFix
/// // "[E3033]" - DiagnosticStyle::Helpful
/// // ": this is an error!" - DiagnosticStyle::NoStyle
///
/// // `DiagnosticStyle` can be rendered into different text colors and formats when diaplaying.
///
/// assert_eq!(result.get(0).unwrap().len(), 3);
/// assert_eq!(result.get(0).unwrap().get(0).unwrap().text, "error");
/// assert_eq!(result.get(0).unwrap().get(1).unwrap().text, "[E3033]");
/// assert_eq!(result.get(0).unwrap().get(2).unwrap().text, ": this is an error!");
///
/// assert_eq!(result.get(0).unwrap().get(0).unwrap().style, Some(DiagnosticStyle::NeedFix));
/// assert_eq!(result.get(0).unwrap().get(1).unwrap().style, Some(DiagnosticStyle::Helpful));
/// assert_eq!(result.get(0).unwrap().get(2).unwrap().style, Some(DiagnosticStyle::NoStyle));
/// ```
pub struct Diagnostic {
    components: Vec<Box<dyn Component>>,
}

impl Diagnostic {
    pub fn new() -> Self {
        Diagnostic { components: vec![] }
    }

    pub fn append_component(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }

    pub fn prepend_component(&mut self, component: Box<dyn Component>) {
        self.components.insert(0, component);
    }
}

impl Component for Diagnostic {
    fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>) {
        for component in &self.components {
            component.format(sb);
        }
    }
}
