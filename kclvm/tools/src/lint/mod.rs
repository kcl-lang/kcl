use indexmap::IndexSet;
use kclvm_error::{Diagnostic, Level};
use kclvm_parser::{load_program, LoadProgramOptions};
use kclvm_sema::resolver::resolve_program;
#[cfg(test)]
mod tests;

/// KCL Lint tools API, check a set of files, skips execute, divides and returns diagnostics into error and warning
///
/// # Parameters
///
/// `file`: [&str]
///     The File that need to be check
///
/// `opts`: Option<LoadProgramOptions>
///     The compilation parameters of KCL, same as the compilation process
///
/// # Returns
///
/// result: (IndexSet<Diagnostic>, IndexSet<Diagnostic>)
///     Error and warning diagenostics.
///
/// # Examples
///
/// ```no_run
/// use kclvm_tools::lint::lint_files;
/// let (error, warning) = lint_files(&["test.k"], None);
/// ```
///
/// - test.k
///
/// ```kcl
/// import kcl_plugin.hello
/// schema Person:
///     age: int
/// ```
///
/// - return
/// error: []
/// warning: [
///    Diagnostic {
///        level: Warning
///        messages: [Message {
///            pos: Position {
///                filename: test.k,
///                line: 1,
///                column: None,
///            },
///            style: Style::Line,
///            message: "Module 'kcl_plugin.hello' imported but unused",
///            note: Some("Consider removing this statement".to_string()),
///        }],
///        code: Some<WarningKind::UnusedImportWarning>,
///     }
/// ]
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
