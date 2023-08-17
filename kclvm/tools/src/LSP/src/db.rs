use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_error::Diagnostic;
use kclvm_sema::resolver::scope::ProgramScope;

/// Holds the result of the compile
#[derive(Clone, Default)]
pub struct AnalysisDatabase {
    pub prog: Program,
    pub scope: ProgramScope,
    pub diags: IndexSet<Diagnostic>,
}
