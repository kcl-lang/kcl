use kclvm_error::{Level, Position};

/// Record the information at `LintContext` when traversing the AST for analysis across AST nodes, e.g., record
/// used importstmt(used_import_names) when traversing `ast::Identifier` and `ast::SchemaAttr`, and detect unused
/// importstmt after traversing the entire module.
pub struct LintContext {
    /// What source file are we in.
    pub filename: String,
    /// Are we resolving the ast node start position.
    pub start_pos: Position,
    /// Are we resolving the ast node end position.
    pub end_pos: Position,
}

/// Definition of `Lint` struct
/// Note that Lint declarations don't carry any "state" - they are merely global identifiers and descriptions of lints.
pub struct Lint {
    /// A string identifier for the lint.
    pub name: &'static str,

    /// Level for the lint.
    pub level: Level,

    /// Description of the lint or the issue it detects.
    /// e.g., "imports that are never used"
    pub desc: &'static str,

    // Error/Warning code
    pub code: &'static str,

    // Suggest methods to fix this problem
    pub note: Option<&'static str>,
}

pub type LintArray = Vec<&'static Lint>;

/// Declares a static `LintArray` and return it as an expression.
#[macro_export]
macro_rules! lint_array {
    ($( $lint:expr ),* ,) => { lint_array!( $($lint),* ) };
    ($( $lint:expr ),*) => {{
        vec![$($lint),*]
    }}
}
