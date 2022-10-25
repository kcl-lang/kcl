use crate::errors::ComponentFormatError;
pub use rustc_errors::styled_buffer::{StyledBuffer, StyledString};
use rustc_errors::Style;
use std::fmt::Debug;

pub mod components;
pub mod diagnostic_handler;
pub mod diagnostic_message;
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
    /// # use compiler_base_error::errors::ComponentFormatError;
    /// # use compiler_base_error::Component;
    /// # use compiler_base_error::DiagnosticStyle;
    /// # use rustc_errors::styled_buffer::StyledBuffer;
    ///
    /// struct ComponentWithStyleLogo {
    ///     text: String
    /// }
    ///
    /// impl Component<DiagnosticStyle> for ComponentWithStyleLogo {
    ///     fn format(&self, sb: &mut StyledBuffer<DiagnosticStyle>, errs: &mut Vec<ComponentFormatError>) {
    ///         // set style
    ///         sb.pushs(&self.text, Some(DiagnosticStyle::Logo));
    ///     }
    /// }
    ///
    /// ```
    fn format(&self, sb: &mut StyledBuffer<T>, errs: &mut Vec<ComponentFormatError>);
}

/// `Diagnostic` is a collection of various components,
/// and any data structure that implements `Component` can be a part of `Diagnostic`.
///
/// # Examples
///
/// ```rust
/// # use rustc_errors::styled_buffer::StyledBuffer;
/// # use compiler_base_error::components::Label;
/// # use compiler_base_error::DiagnosticStyle;
/// # use compiler_base_error::Diagnostic;
/// # use compiler_base_error::Component;
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
/// // Create an error set for collecting errors.
/// let mut errs = vec![];
///
/// // Rendering !
/// diagnostic.format(&mut sb, &mut errs);
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
#[derive(Default)]
pub struct Diagnostic<T>
where
    T: Clone + PartialEq + Eq + Style + Debug,
{
    components: Vec<Box<dyn Component<T>>>,
}

impl<T> Debug for Diagnostic<T>
where
    T: Clone + PartialEq + Eq + Style + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut diag_fmt = String::new();
        for component in &self.components {
            let mut s_sb = StyledBuffer::<T>::new();
            let mut s_errs = vec![];
            component.format(&mut s_sb, &mut s_errs);
            diag_fmt.push_str(&format!("{:?}\n", s_sb.render()));
        }
        write!(f, "{}", diag_fmt)
    }
}

impl<T> PartialEq for Diagnostic<T>
where
    T: Clone + PartialEq + Eq + Style + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl<T> Diagnostic<T>
where
    T: Clone + PartialEq + Eq + Style + Debug,
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
    T: Clone + PartialEq + Eq + Style + Debug,
{
    fn format(&self, sb: &mut StyledBuffer<T>, errs: &mut Vec<ComponentFormatError>) {
        for component in &self.components {
            component.format(sb, errs);
        }
    }
}

/// `String` can be considered as a component of diagnostic with no style.
///
/// The result of component `String` rendering is a `String` who has no style.
impl<T> Component<T> for String
where
    T: Clone + PartialEq + Eq + Style + Debug,
{
    fn format(&self, sb: &mut StyledBuffer<T>, _: &mut Vec<ComponentFormatError>) {
        sb.appendl(self, None);
    }
}
