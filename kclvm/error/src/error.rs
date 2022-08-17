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

// Error messages for EXXXX errors. Each message should start and end with a
// new line.
register_errors! {
    E1001: ErrorKind::InvalidSyntax, include_str!("./error_codes/E1001.md"),
    E2G22: ErrorKind::TypeError, include_str!("./error_codes/E2G22.md"),
    E2F04: ErrorKind::CannotFindModule, include_str!("./error_codes/E2F04.md"),
    E2L23: ErrorKind::CompileError, include_str!("./error_codes/E2L23.md"),
    E2A31: ErrorKind::IllegalAttributeError, include_str!("./error_codes/E2A31.md"),
    E2L28: ErrorKind::UniqueKeyError, include_str!("./error_codes/E2L28.md"),
    E2D34: ErrorKind::IllegalInheritError, include_str!("./error_codes/E2D34.md"),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    pub code: &'static str,
    pub kind: ErrorKind,
    pub message: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    InvalidSyntax,
    TabError,
    Indentation,
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
        write!(f, "{:?}", self)
    }
}

impl ErrorKind {
    #[allow(dead_code)]
    pub fn name(&self) -> String {
        return format!("{:?}", self);
    }
}

/// Warning information of KCL. Usually something that does not conform to the specification but does not cause an error.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Warning {
    pub code: &'static str,
    pub kind: ErrorKind,
    pub message: Option<&'static str>,
}

// Kind of KCL warning.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WarningKind {
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
///         pos: Position::dummy_pos(),
///         style: Style::LineAndColumn,
///         message: "Module 'a' imported but unused.".to_string(),
///         note: None,
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
        write!(f, "{:?}", self)
    }
}

/// Test warning `name`
/// ```
/// use kclvm_error::*;
/// use kclvm_error::DiagnosticId::Warning;
/// let mut handler = Handler::default();
/// handler.add_warning(WarningKind::UnusedImportWarning, &[
///     Message {
///         pos: Position::dummy_pos(),
///         style: Style::LineAndColumn,
///         message: "Module 'a' imported but unused.".to_string(),
///         note: None,
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
    pub fn name(&self) -> String {
        return format!("{:?}", self);
    }
}
