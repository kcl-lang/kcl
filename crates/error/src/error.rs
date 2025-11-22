//! This module is used to gather all error codes into one place,
//! the goal being to make their maintenance easier.

macro_rules! register_errors {
    ($($ecode:ident: $kind:expr, $message:expr,)*) => (
        pub static ERRORS: &[(&str, Error)] = &[
            $( (stringify!($ecode), Error {
                code: stringify!($ecode),
                kind: $kind,
                message: Some($message),
            }), )*
        ];
        $(pub const $ecode: Error = Error {
            code: stringify!($ecode),
            kind: $kind,
            message: Some($message),
        };)*
    )
}

macro_rules! register_warnings {
    ($($ecode:ident: $kind:expr, $message:expr,)*) => (
        pub static WARNINGS: &[(&str, Warning)] = &[
            $( (stringify!($ecode), Warning {
                code: stringify!($ecode),
                kind: $kind,
                message: Some($message),
            }), )*
        ];
        $(pub const $ecode: Warning = Warning {
            code: stringify!($ecode),
            kind: $kind,
            message: Some($message),
        };)*
    )
}

// Error messages for EXXXX errors. Each message should start and end with a
// new line.
register_errors! {
    // E1XXX Syntax Errors
    E1001: ErrorKind::InvalidSyntax, include_str!("./error_codes/E1001.md"),
    E1002: ErrorKind::TabError, include_str!("./error_codes/E1002.md"),
    E1003: ErrorKind::IndentationError, include_str!("./error_codes/E1003.md"),
    E1I37: ErrorKind::IllegalArgumentSyntax, include_str!("./error_codes/E1I37.md"),
    // E2XXX Compile Errors
    E2G22: ErrorKind::TypeError, include_str!("./error_codes/E2G22.md"),
    E2F04: ErrorKind::CannotFindModule, include_str!("./error_codes/E2F04.md"),
    E2L23: ErrorKind::CompileError, include_str!("./error_codes/E2L23.md"),
    E2A31: ErrorKind::IllegalAttributeError, include_str!("./error_codes/E2A31.md"),
    E2L28: ErrorKind::UniqueKeyError, include_str!("./error_codes/E2L28.md"),
    E2D34: ErrorKind::IllegalInheritError, include_str!("./error_codes/E2D34.md"),
    // E3XXX Runtime Errors
    E3M38: ErrorKind::EvaluationError, include_str!("./error_codes/E2D34.md"),
}

// Error messages for WXXXX errors. Each message should start and end with a
// new line.
register_warnings! {
    W1001: WarningKind::CompilerWarning, include_str!("./warning_codes/W1001.md"),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    pub code: &'static str,
    pub kind: ErrorKind,
    pub message: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    // Syntax Errors
    InvalidSyntax,
    TabError,
    IndentationError,
    IllegalArgumentSyntax,
    // Compile Errors
    CannotFindModule,
    RecursiveLoad,
    FloatOverflow,
    FloatUnderflow,
    IntOverflow,
    InvalidDocstring,
    Deprecated,
    UnKnownDecorator,
    InvalidDecoratorTarget,
    InvalidFormatSpec,
    SchemaCheckFailure,
    IndexSignatureError,
    TypeError,
    NameError,
    ValueError,
    KeyError,
    AttributeError,
    // Runtime Errors
    AssertionError,
    ImmutableError,
    MultiInheritError,
    CycleInheritError,
    IllegalInheritError,
    IllegalAttributeError,
    IllegalParameterError,
    RecursionError,
    PlanError,
    CannotAddMembers,
    CompileError,
    EvaluationError,
    UniqueKeyError,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ErrorKind {
    /// Returns the error name.
    pub fn name(&self) -> String {
        format!("{self:?}")
    }
    /// Returns the error code.
    pub fn code(&self) -> String {
        match ERRORS.iter().find(|&error_pair| error_pair.1.kind == *self) {
            Some(r) => r.0.to_string(),
            None => E1001.code.to_string(),
        }
    }
}

/// Warning information of KCL. Usually something that does not conform to the specification but does not cause an error.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Warning {
    pub code: &'static str,
    pub kind: WarningKind,
    pub message: Option<&'static str>,
}

// Kind of KCL warning.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WarningKind {
    // Compile Warnings
    CompilerWarning,
    UnusedImportWarning,
    ReimportWarning,
    ImportPositionWarning,
}

/// Test warning `fmt`
/// ```
/// use kclvm_error::*;
/// use kclvm_error::DiagnosticId::Warning;
/// let mut handler = Handler::default();
/// handler.add_warning(WarningKind::UnusedImportWarning, &[
///     Message {
///         range: (Position::dummy_pos(), Position::dummy_pos()),
///         style: Style::LineAndColumn,
///         message: "Module 'a' imported but unused.".to_string(),
///         note: None,
///         suggested_replacement: None,
///     }],
/// );
/// for diag in &handler.diagnostics {
///     if let Warning(warningkind) = diag.code.as_ref().unwrap() {
///         println!("{}",warningkind);
///     }     
/// }
/// ```
///
impl std::fmt::Display for WarningKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Test warning `name`
/// ```
/// use kclvm_error::*;
/// use kclvm_error::DiagnosticId::Warning;
/// let mut handler = Handler::default();
/// handler.add_warning(WarningKind::UnusedImportWarning, &[
///     Message {
///         range: (Position::dummy_pos(), Position::dummy_pos()),
///         style: Style::LineAndColumn,
///         message: "Module 'a' imported but unused.".to_string(),
///         note: None,
///         suggested_replacement: None,
///     }],
/// );
/// for diag in &handler.diagnostics {
///     match diag.code.as_ref().unwrap() {
///         Warning(warningkind) => {
///             println!("{}",warningkind.name());
///         }
///         _ => {}
///     }
///     
/// }
/// ```
impl WarningKind {
    /// Returns the warning name.
    pub fn name(&self) -> String {
        format!("{self:?}")
    }
    /// Returns the warning code.
    pub fn code(&self) -> String {
        match WARNINGS
            .iter()
            .find(|&error_pair| error_pair.1.kind == *self)
        {
            Some(r) => r.0.to_string(),
            None => W1001.code.to_string(),
        }
    }
}
