use std::sync::Arc;

use indexmap::IndexSet;
use kclvm_error::{Diagnostic, Handler};
use kclvm_parser::{load_program, LoadProgramOptions, ParseSession};
use kclvm_runtime::PanicInfo;
use kclvm_sema::resolver::resolve_program_with_opts;
use std::fs;
use std::path::{Path};
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
/// let (errors, warnings) = lint_files(&["test.k"], None);
/// ```
///
/// - test.k
///
/// ```kcl
/// import math
/// schema Person:
///     age: int
/// ```
///
/// - return
/// ```no_check
/// error: []
/// warning: [
///    Diagnostic {
///        level: Warning
///        messages: [Message {
///            range: (
///                Position {
///                    filename: test.k,
///                    line: 1,
///                    column: None,
///                },
///                Position {
///                    filename: test.k,
///                    line: 1,
///                    column: None,
///                },
///            ),
///            style: Style::Line,
///            message: "Module 'math' imported but unused",
///            note: Some("Consider removing this statement".to_string()),
///        }],
///        code: Some<WarningKind::UnusedImportWarning>,
///     }
/// ]
/// ```
pub fn lint_files(
    files: &[&str],
    opts: Option<LoadProgramOptions>,
) -> (IndexSet<Diagnostic>, IndexSet<Diagnostic>) {
    // Parse AST program.
    let sess = Arc::new(ParseSession::default());
    let mut opts = opts.unwrap_or_default();
    opts.load_plugins = true;
    let mut program = match load_program(sess.clone(), files, Some(opts), None) {
        Ok(p) => p.program,
        Err(err_str) => {
            return Handler::default()
                .add_panic_info(&PanicInfo::from(err_str.to_string()))
                .classification();
        }
    };
    sess.append_diagnostic(
        resolve_program_with_opts(
            &mut program,
            kclvm_sema::resolver::Options {
                merge_program: false,
                ..Default::default()
            },
            None,
        )
        .handler
        .diagnostics,
    )
    .classification()
}

fn collect_kcl_files<P: AsRef<Path>>(dir: P) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "k") {
                files.push(path.to_string_lossy().into_owned());
            } else if path.is_dir() {
                files.extend(collect_kcl_files(path));
            }
        }
    }
    files
}

pub fn lint_all_files(dir: &str, opts: Option<LoadProgramOptions>) -> (IndexSet<Diagnostic>, IndexSet<Diagnostic>) {
    let files = collect_kcl_files(dir);

    let mut all_errors = IndexSet::new();
    let mut all_warnings = IndexSet::new();

    for file in files {
        let file_refs = vec![file.as_str()];
        let (errors, warnings) = lint_files(&file_refs, opts.clone());
        all_errors.extend(errors);
        all_warnings.extend(warnings);
    }

    (all_errors, all_warnings)
}
