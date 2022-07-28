#![feature(let_chains)]
#![feature(once_cell)]
#![feature(rustc_attrs)]
#![feature(type_alias_impl_trait)]
use std::borrow::Cow;
pub use unic_langid::{langid, LanguageIdentifier};

#[cfg(test)]
mod tests;

/// Identifier for the Fluent message/attribute corresponding to a diagnostic message.
type FluentId = Cow<'static, str>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DiagnosticMessage {
    Str(String),
    FluentIdentifier(FluentId, Option<FluentId>),
}

impl DiagnosticMessage {
    pub fn to_string(&self) -> String {
        "todo".to_string()
    }

    pub fn with_subdiagnostic_message(&self, sub: SubdiagnosticMessage) -> Self {
        let attr = match sub {
            SubdiagnosticMessage::Str(s) => return DiagnosticMessage::Str(s.clone()),
            SubdiagnosticMessage::FluentIdentifier(id) => {
                return DiagnosticMessage::FluentIdentifier(id, None);
            }
            SubdiagnosticMessage::FluentAttr(attr) => attr,
        };

        match self {
            DiagnosticMessage::Str(s) => DiagnosticMessage::Str(s.clone()),
            DiagnosticMessage::FluentIdentifier(id, _) => {
                DiagnosticMessage::FluentIdentifier(id.clone(), Some(attr))
            }
        }
    }

    pub fn expect_str(&self) -> &str {
        match self {
            DiagnosticMessage::Str(s) => s,
            _ => panic!("expected non-translatable diagnostic message"),
        }
    }
}
pub enum SubdiagnosticMessage {
    Str(String),
    FluentIdentifier(FluentId),
    FluentAttr(FluentId),
}

/// TODO: generated from macros
#[allow(non_upper_case_globals)]
#[doc(hidden)]
pub mod fluent {
    pub static DEFAULT_LOCALE_RESOURCES: &'static [&'static str] = &[
        "../locales/en-US/borrowck.ftl",
        "../locales/en-US/builtin_macros.ftl",
        "../locales/en-US/lint.ftl",
        "../locales/en-US/parser.ftl",
        "../locales/en-US/privacy.ftl",
        "../locales/en-US/typeck.ftl",
    ];

    pub mod typeck {
        use std::borrow::Cow;
        pub const resource_id: Cow<str> = std::borrow::Cow::Borrowed("typeck");
        pub const field_multiply_specified_in_initializer: crate::DiagnosticMessage =
            crate::DiagnosticMessage::FluentIdentifier(
                std::borrow::Cow::Borrowed("typeck-field-multiply-specified-in-initializer"),
                None,
            );
        pub const label: crate::SubdiagnosticMessage =
            crate::SubdiagnosticMessage::FluentAttr(std::borrow::Cow::Borrowed("label"));
        pub const label_previous_use: crate::SubdiagnosticMessage =
            crate::SubdiagnosticMessage::FluentAttr(std::borrow::Cow::Borrowed(
                "previous-use-label",
            ));
    }
}
