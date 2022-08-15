pub use rustc_errors::styled_buffer::StyledBuffer;
use rustc_errors::Style;

pub mod components;
pub mod style;

#[cfg(test)]
mod tests;

/// 'Component' specifies the method `format()` that all diagnostic components should implement.
/// 
/// 'Component' decouples 'structure' and 'theme' during formatting diagnostic components.
/// `T: Clone + PartialEq + Eq + Style` is responsible for 'theme' such as colors/fonts in the component formatting.
/// `format()` organizes the 'structure' of diagnostic components.
pub trait Component<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    /// `format()` formats components into `StyledString` and saves them in `StyledBuffer`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// struct ComponentWithStyleLogo {
    ///     text: String
    /// }
    ///
    /// impl Component<DiagnosticStyle> for ComponentWithStyleLogo {
    ///     fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>) {
    ///         // set style
    ///         sb.pushs(&self.text, Some(DiagnosticStyle::Logo));
    ///     }
    /// }
    /// 
    /// ```
    fn format(&self, sb: &mut StyledBuffer<T>);
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
/// // ": this is an error!" - None
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
/// assert_eq!(result.get(0).unwrap().get(2).unwrap().style, None);
/// ```
pub struct Diagnostic<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    components: Vec<Box<dyn Component<T>>>,
}

impl<T> Diagnostic<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    pub fn new() -> Self {
        Diagnostic { components: vec![] }
    }

    pub fn append_component(&mut self, component: Box<dyn Component<T>>) {
        self.components.push(component);
    }

    pub fn prepend_component(&mut self, component: Box<dyn Component<T>>) {
        self.components.insert(0, component);
    }
}

impl<T> Component<T> for Diagnostic<T>
where
    T: Clone + PartialEq + Eq + Style,
{
    fn format(&self, sb: &mut StyledBuffer<T>) {
        for component in &self.components {
            component.format(sb);
        }
    }
}

/// `String` can be considered as a component of diagnostic with no style.
///
/// The result of component `String` rendering is a `String` who has no style.
impl<T> Component<T> for String
where
    T: Clone + PartialEq + Eq + Style,
{
    fn format(&self, sb: &mut StyledBuffer<T>) {
        sb.appendl(&self, None);
    }
}
