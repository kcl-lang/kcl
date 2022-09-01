use std::{error::Error, fmt};

impl Error for ComponentFormatError {}
impl Error for ComponentError {}

#[derive(Debug)]
pub struct ComponentFormatError {
    component_name: String,
    details: String,
}

impl ComponentFormatError {
    pub fn new(name: &str, msg: &str) -> Self {
        Self {
            component_name: name.to_string(),
            details: msg.to_string(),
        }
    }

    pub fn format(&self) -> String {
        format!(
            "Formatting Component {} Failed, {}.\n",
            self.component_name, self.details
        )
    }
}

impl fmt::Display for ComponentFormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[derive(Debug)]
pub enum ComponentError {
    ComponentFormatErrors(Vec<ComponentFormatError>),
    EmitterError,
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComponentError::ComponentFormatErrors(errs) => {
                let mut result = String::new();
                for e in errs {
                    result += &e.format();
                }
                write!(f, "{}", result)
            }
            ComponentError::EmitterError => {
                write!(f, "Emitting Diagnostic Failed")
            }
        }
    }
}
