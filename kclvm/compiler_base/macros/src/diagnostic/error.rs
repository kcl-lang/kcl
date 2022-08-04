//！This is a mini error management crate, 
//! mainly responsible for managing exceptions in compiler_base_macros/diagnostic. 
//! It is an internal crate and does not support external calls.

use compiler_base_diagnostic::pendant::HeaderPendant;
use compiler_base_diagnostic::Diagnostic;
use compiler_base_diagnostic::DiagnosticBuilder;
use compiler_base_diagnostic::Message;
use compiler_base_diagnostic::Sentence;


/// UnexpectedAttr: Unsupported sub-attribute used in macro attribute.
/// 
/// # Examples
/// 
/// ```no run
/// 
/// #[error(msg="error msg !", unknown="")] // Unexpected sub-attribute 'unknown' cause the error.
/// ```
pub struct UnexpectedAttr {
    label_name: String,
    attr_name: String,
}

impl UnexpectedAttr {
    pub fn new(label_name: String, attr_name: String) -> Self {
        Self {
            label_name,
            attr_name,
        }
    }
}

impl DiagnosticBuilder for UnexpectedAttr {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "unknown attribute '{}' for #[{}]",
            self.attr_name, self.label_name
        )))
    }
}

/// UnexpectedFieldType: Macro attribute binds to struct field with wrong type.
/// 
/// # Examples
/// 
/// ```no run
/// 
/// struct ErrorType{
///     #[position(msg="error msg !")] 
///     pos: String // The type of 'pos' must be 'Position'.
/// }
/// ```
pub struct UnexpectedFieldType {
    label_name: String,
    field_name: String,
    ty_name: String,
}

impl DiagnosticBuilder for UnexpectedFieldType {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "the type of struct field '{}' bound by #[{}] can only be '{}'",
            self.field_name, self.label_name, self.ty_name
        )))
    }
}

/// MissingAttr: Macro attribute is missing the required sub-attribute.
/// 
/// # Examples
/// 
/// ```no run
/// 
/// #error[code="E0101"] // Sub-attribute 'msg' is required for 'error'.
/// ```
pub struct MissingAttr {
    label_name: String,
    attr_name: String,
}

impl MissingAttr {
    pub fn new(label_name: String, attr_name: String) -> Self {
        Self {
            label_name,
            attr_name,
        }
    }
}

impl DiagnosticBuilder for MissingAttr {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "missing required attribute '{}' in #[{}]",
            self.attr_name, self.label_name
        )))
    }
}

/// UnexpectedDiagnosticType: Some macros only support 'struct'.
/// 
/// # Examples
/// 
/// ```no run
/// 
/// #[derive(DiagnosticBuilderMacro)] // 'DiagnosticBuilderMacro' only support 'struct'.
/// enum ErrorType{
///     ...
/// }
/// ```
pub struct UnexpectedDiagnosticType;

impl UnexpectedDiagnosticType {
    pub fn new() -> Self {
        Self {}
    }
}

impl DiagnosticBuilder for UnexpectedDiagnosticType {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "diagnostic type is only supported to define through 'struct'.",
        )))
    }
}

/// UnexpectedLabel: Unexpected macro attribute is used.
/// 
/// # Examples
/// 
/// ```no run
/// 
/// #[derive(DiagnosticBuilderMacro)] 
/// #[unknown_label()] // 'DiagnosticBuilderMacro' do not has sub-attribute 'unknown_label'.
/// struct ErrorType{
///     ...
/// }
/// ```
pub struct UnexpectedLabel {
    label_name: String,
}
impl UnexpectedLabel {
    pub fn new(label_name: String) -> Self {
        Self { label_name }
    }
}

impl DiagnosticBuilder for UnexpectedLabel {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "unexpected label #[{}]",
            self.label_name,
        )))
    }
}

/// DuplicateAttr: Duplicate sub-attributes are defined.
/// 
/// # Examples
/// 
/// ```no run
/// 
/// #[derive(DiagnosticBuilderMacro)] 
/// #[error(msg="error msg1", msg="error msg2")] // Duplicate sub-attributes 'msg'.
/// struct ErrorType{
///     ...
/// }
/// ```
pub struct DuplicateAttr {
    label_name: String,
    attr_name: String,
}

impl DuplicateAttr {
    pub fn new(attr_name: String, label_name: String) -> Self {
        Self {
            attr_name,
            label_name,
        }
    }
}

impl DiagnosticBuilder for DuplicateAttr {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "attribute '{}' is duplicately defined in #[{}].",
            self.attr_name, self.label_name
        )))
    }
}

/// InternalBug: An internal bug.
pub struct InternalBug;

impl InternalBug {
    pub fn new() -> Self {
        Self {}
    }
}

impl DiagnosticBuilder for InternalBug {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "this is an internal bug, please contact us",
        )))
    }
}

fn construct_one_sentence_diagnostic(msg: Message) -> Diagnostic {
    let mut diag = Diagnostic::new();
    let pendant = HeaderPendant::new("error".to_string(), None);
    let sentence = Sentence::new_sentence_str(Box::new(pendant), msg);
    diag.add_sentence(sentence);
    diag
}
