use crate::lint::lint::{Lint, LintArray, LintContext};
use crate::lint::lintpass::LintPass;
use crate::resolver::scope::Scope;
use crate::{declare_lint_pass, resolver::scope::ScopeObjectKind};
use indexmap::IndexSet;
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_error::{Handler, Level, Message, Style, WarningKind};

/// The 'import_position' lint detects import statements that are not declared at the top of file.
/// ### Example
///
/// ```kcl
/// schema Person:
///     name: str
///
/// import foo
///
/// ```
/// ### Explanation
///
/// According to the KCL code style conventions, import statement are always declared at the top of the file.
pub static IMPORT_POSITION: &Lint = &Lint {
    name: stringify!("IMPORT_POSITION"),
    level: Level::Warning,
    desc: "Check for importstmt that are not defined at the top of file",
    code: "W0413",
    note: Some("Consider moving tihs statement to the top of the file"),
};

declare_lint_pass!(ImportPosition => [IMPORT_POSITION]);

impl LintPass for ImportPosition {
    fn check_module(
        &mut self,
        handler: &mut Handler,
        _ctx: &mut LintContext,
        module: &ast::Module,
    ) {
        let mut first_non_importstmt = std::u64::MAX;
        for stmt in &module.body {
            match &stmt.node {
                ast::Stmt::Import(_import_stmt) => {}
                _ => {
                    if stmt.line < first_non_importstmt {
                        first_non_importstmt = stmt.line
                    }
                }
            }
        }
        for stmt in &module.body {
            if let ast::Stmt::Import(_import_stmt) = &stmt.node {
                if stmt.line > first_non_importstmt {
                    handler.add_warning(
                        WarningKind::ImportPositionWarning,
                        &[Message {
                            range: stmt.get_span_pos(),
                            style: Style::Line,
                            message: format!(
                                "Importstmt should be placed at the top of the module"
                            ),
                            note: Some(
                                "Consider moving tihs statement to the top of the file".to_string(),
                            ),
                            suggested_replacement: None,
                        }],
                    );
                }
            }
        }
    }
}

/// The 'unused_import' lint detects import statements that are declared but not used.
///
/// ### Example
///
/// ```kcl
/// import foo
///
/// schema Person:
///     name: str
///
/// ```
/// ### Explanation
///
/// Useless imports can affect the speed of compilation. It is necessary to remove useless imports from the kcl code.
pub static UNUSED_IMPORT: &Lint = &Lint {
    name: stringify!("UNUSED_IMPORT"),
    level: Level::Warning,
    desc: "Check for unused importstmt",
    code: "W0411",
    note: Some("Consider removing this statement"),
};

declare_lint_pass!(UnusedImport => [UNUSED_IMPORT]);

impl LintPass for UnusedImport {
    fn check_scope(&mut self, handler: &mut Handler, _ctx: &mut LintContext, scope: &Scope) {
        let scope_objs = &scope.elems;
        for (_, scope_obj) in scope_objs {
            let scope_obj = scope_obj.borrow();
            if let ScopeObjectKind::Module(m) = &scope_obj.kind {
                for (stmt, has_used) in &m.import_stmts {
                    if !has_used {
                        handler.add_warning(
                            WarningKind::UnusedImportWarning,
                            &[Message {
                                range: stmt.get_span_pos(),
                                style: Style::Line,
                                message: format!("Module '{}' imported but unused", scope_obj.name),
                                note: Some("Consider removing this statement".to_string()),
                                suggested_replacement: None,
                            }],
                        );
                    }
                }
            }
        }
    }
}

/// The 'reimport' lint detects deplicate import statement
/// ### Example
///
/// ```kcl
/// import foo
/// import foo
///
/// schema Person:
///     name: str
///
/// ```
/// ### Explanation
///
/// The import statement should be declared only once
pub static REIMPORT: &Lint = &Lint {
    name: stringify!("REIMPORT"),
    level: Level::Warning,
    desc: "Check for deplicate importstmt",
    code: "W0404",
    note: Some("Consider removing this statement"),
};

declare_lint_pass!(ReImport => [REIMPORT]);

impl LintPass for ReImport {
    fn check_module(
        &mut self,
        handler: &mut Handler,
        _ctx: &mut LintContext,
        module: &ast::Module,
    ) {
        let mut import_names = IndexSet::<String>::new();
        for stmt in &module.body {
            if let ast::Stmt::Import(import_stmt) = &stmt.node {
                if import_names.contains(&import_stmt.path.node) {
                    handler.add_warning(
                        WarningKind::ReimportWarning,
                        &[Message {
                            range: stmt.get_span_pos(),
                            style: Style::Line,
                            message: format!(
                                "Module '{}' is reimported multiple times",
                                &import_stmt.name
                            ),
                            note: Some("Consider removing this statement".to_string()),
                            suggested_replacement: None,
                        }],
                    );
                } else {
                    import_names.insert(import_stmt.path.node.clone());
                }
            }
        }
    }
}
