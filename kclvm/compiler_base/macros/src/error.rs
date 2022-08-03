use compiler_base_diagnostic::pendant::HeaderPendant;
use compiler_base_diagnostic::Diagnostic;
use compiler_base_diagnostic::DiagnosticBuilder;
use compiler_base_diagnostic::Message;
use compiler_base_diagnostic::Sentence;

pub struct MissingField {
    label_name: String,
}

impl DiagnosticBuilder for MissingField {
    fn into_diagnostic(self) -> Diagnostic {
        construct_one_sentence_diagnostic(Message::Str(format!(
            "#[{}] needs to be bound to a struct field",
            self.label_name
        )))
    }
}

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
