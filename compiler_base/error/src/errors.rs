//! This crate provides all error types used in compiler-base-error.

use std::{error::Error, fmt};

impl Error for ComponentFormatError {}
impl Error for ComponentError {}

/// `ComponentFormatError` will be return when `Component` formatting exception occurs.
/// For more information about `Component`, see doc in 'compiler_base/error/src/diagnostic/mod.rs'
/// and 'compiler_base/error/src/diagnostic/components.rs'.
#[derive(Debug)]
pub struct ComponentFormatError {
    name: String,
    message: String,
}

impl ComponentFormatError {
    /// The constructor of `ComponentFormatError`.
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::errors::ComponentFormatError;
    ///
    /// // If you want to new a `ComponentFormatError`,
    /// // the first arg is the component name, and the second arg is the help info for this error.
    /// let component_format_error = ComponentFormatError::new("name", "The component format failed.");
    ///
    /// let err_fmt = format!("{:?}", component_format_error);
    /// assert_eq!("ComponentFormatError { name: \"name\", message: \"The component format failed.\" }", err_fmt);
    /// ```
    pub fn new(name: &str, msg: &str) -> Self {
        Self {
            name: name.to_string(),
            message: msg.to_string(),
        }
    }

    pub(crate) fn format(&self) -> String {
        format!(
            "Failed to display '{}' on terminal, {}.\n",
            self.name, self.message
        )
    }
}

impl fmt::Display for ComponentFormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// `ComponentError` is a collection of errors in `Component`.
/// For more information about `Component`, see doc in 'compiler_base/error/src/diagnostic/mod.rs'
/// and 'compiler_base/error/src/diagnostic/components.rs'.
///
/// Currently `ComponentError` only supports type `ComponentFormatErrors`, and more types can be added later if needed.
///
/// # Examples
///
/// ```rust
/// # use compiler_base_error::errors::ComponentFormatError;
/// # use compiler_base_error::errors::ComponentError;
///
/// // If you want to new a `ComponentFormatError`,
/// let component_format_error_1 = ComponentFormatError::new("name_1", "The component_1 format failed.");
/// let component_format_error_2 = ComponentFormatError::new("name_2", "The component_1 format failed.");
/// let errs = vec![component_format_error_1, component_format_error_2];
/// let component_format_errors = ComponentError::ComponentFormatErrors(errs);
///
/// let errs_fmt = format!("{:?}", component_format_errors);
/// assert_eq!(
/// "ComponentFormatErrors([ComponentFormatError { name: \"name_1\", message: \"The component_1 format failed.\" }, ComponentFormatError { name: \"name_2\", message: \"The component_1 format failed.\" }])"
/// , errs_fmt)
/// ```
#[derive(Debug)]
pub enum ComponentError {
    ComponentFormatErrors(Vec<ComponentFormatError>),
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComponentError::ComponentFormatErrors(errs) => {
                let mut result = String::new();
                for e in errs {
                    result += &e.format();
                }
                result += "/n";
                write!(f, "{}", result)
            }
        }
    }
}
