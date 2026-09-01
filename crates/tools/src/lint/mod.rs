use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use kcl_error::{Diagnostic, Handler};
use kcl_parser::{LoadProgramOptions, ParseSession, get_kcl_files, load_program};
use kcl_primitives::IndexSet;
use kcl_runtime::PanicInfo;
use kcl_sema::resolver::resolve_program_with_opts;
#[cfg(test)]
mod tests;

/// KCL Lint tools API, check a set of files, skips execute, divides and returns diagnostics into error and warning
///
/// # Parameters
///
/// `file`: [&str]
///     The File that need to be check. A path ending with `/...` (e.g. `./...`)
///     recursively lints every package under the root directory, like
///     `go build ./...`: all `.k` files are collected, grouped by their
///     directory (one directory is one package) and each package is linted
///     as its own program, so packages that are not imported by the entry
///     file are also checked and name clashes between packages are allowed.
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
/// use kcl_tools::lint::lint_files;
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
    if files.iter().any(|f| recursive_root(f).is_some()) {
        return lint_all_packages(files, opts);
    }
    lint_package(files, opts)
}

/// Expand `./...`-style patterns and lint every package found under their roots.
///
/// All the `.k` files collected from the patterns (plus the plain paths given
/// alongside them) are grouped by parent directory, since one directory is one
/// KCL package, and each group is linted as its own program. This mirrors
/// `go build ./...`: packages unreachable from any entry file are still
/// checked, while symbols defined in distinct packages never collide.
fn lint_all_packages(
    files: &[&str],
    opts: Option<LoadProgramOptions>,
) -> (IndexSet<Diagnostic>, IndexSet<Diagnostic>) {
    let mut packages: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    for file in files {
        match recursive_root(file) {
            Some(root) => match get_kcl_files(root, true) {
                Ok(kcl_files) => {
                    for kcl_file in kcl_files {
                        packages
                            .entry(
                                Path::new(&kcl_file)
                                    .parent()
                                    .unwrap_or(Path::new(""))
                                    .to_path_buf(),
                            )
                            .or_default()
                            .push(kcl_file);
                    }
                }
                Err(err) => {
                    return Handler::default()
                        .add_panic_info(&PanicInfo::from(err.to_string()))
                        .classification();
                }
            },
            None => {
                packages
                    .entry(
                        Path::new(file)
                            .parent()
                            .unwrap_or(Path::new(""))
                            .to_path_buf(),
                    )
                    .or_default()
                    .push(file.to_string());
            }
        }
    }
    let mut errors = IndexSet::default();
    let mut warnings = IndexSet::default();
    for (_, package) in packages.iter_mut() {
        package.sort();
        package.dedup();
        let entries: Vec<&str> = package.iter().map(|f| f.as_str()).collect();
        let (errs, warns) = lint_package(&entries, opts.clone());
        errors.extend(errs);
        warnings.extend(warns);
    }
    (errors, warnings)
}

/// For a `./...`-style path, return the root directory the pattern scans;
/// return `None` for ordinary paths.
fn recursive_root(path: &str) -> Option<&str> {
    if Path::new(path).file_name()? == "..." {
        let root = Path::new(path).parent()?.to_str()?;
        Some(if root.is_empty() { "." } else { root })
    } else {
        None
    }
}

#[allow(clippy::arc_with_non_send_sync)]
fn lint_package(
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
            kcl_sema::resolver::Options {
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
