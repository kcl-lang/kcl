use indexmap::IndexSet;
use kclvm_error::{Diagnostic, Level};
use kclvm_parser::{load_program, LoadProgramOptions};
use kclvm_sema::resolver::resolve_program;

/// Check a set of files, skips execute and divides diagnostics into error and warning
pub fn lint_files(
    files: &[&str],
    opts: Option<LoadProgramOptions>,
) -> (IndexSet<Diagnostic>, IndexSet<Diagnostic>) {
    // Parse AST program.
    let mut program = load_program(&files, opts).unwrap();
    let scope = resolve_program(&mut program);
    let (mut errs, mut warnings) = (IndexSet::new(), IndexSet::new());
    for diag in &scope.diagnostics {
        if diag.level == Level::Error {
            errs.insert(diag.clone());
        } else if diag.level == Level::Warning {
            warnings.insert(diag.clone());
        } else {
            continue;
        }
    }
    (errs, warnings)
}
