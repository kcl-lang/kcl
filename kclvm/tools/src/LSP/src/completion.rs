//! Complete for KCL
//! Github Issue: https://github.com/kcl-lang/kcl/issues/476
//! Now supports code completion in treigger mode (triggered when user enters `.`),
//! and the content of the completion includes:
//!  + import path
//!  + schema attr
//!  + builtin function(str function)
//!  + defitions in pkg
//!  + system module functions

use std::io;
use std::{fs, path::Path};

use indexmap::IndexSet;
use kclvm_ast::ast::{Expr, ImportStmt, Node, Program, Stmt};
use kclvm_ast::pos::GetPos;
use kclvm_compiler::pkgpath_without_prefix;
use kclvm_config::modfile::KCL_FILE_EXTENSION;

use kclvm_error::Position as KCLPos;
use kclvm_sema::builtin::{
    get_system_member_function_ty, get_system_module_members, STANDARD_SYSTEM_MODULES,
    STRING_MEMBER_FUNCTIONS,
};
use kclvm_sema::resolver::scope::{ProgramScope, ScopeObjectKind};
use kclvm_sema::ty::FunctionType;
use lsp_types::{CompletionItem, CompletionItemKind};

use crate::goto_def::{find_def, get_identifier_last_name, Definition};
use crate::util::{inner_most_expr_in_stmt, is_in_schema};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum KCLCompletionItemKind {
    Function,
    Variable,
    File,
    Dir,
    Schema,
    SchemaAttr,
    Module,
}

impl Into<CompletionItemKind> for KCLCompletionItemKind {
    fn into(self) -> CompletionItemKind {
        match self {
            KCLCompletionItemKind::Function => CompletionItemKind::FUNCTION,
            KCLCompletionItemKind::Variable => CompletionItemKind::VARIABLE,
            KCLCompletionItemKind::File => CompletionItemKind::FILE,
            KCLCompletionItemKind::Schema => CompletionItemKind::CLASS,
            KCLCompletionItemKind::SchemaAttr => CompletionItemKind::FIELD,
            KCLCompletionItemKind::Module => CompletionItemKind::MODULE,
            KCLCompletionItemKind::Dir => CompletionItemKind::FOLDER,
        }
    }
}

fn func_ty_complete_label(func_name: &String, func_type: &FunctionType) -> String {
    format!(
        "{}({})",
        func_name,
        func_type
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<Vec<String>>()
            .join(", "),
    )
}

/// Computes completions at the given position.
pub(crate) fn completion(
    trigger_character: Option<char>,
    program: &Program,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::CompletionResponse> {
    if let Some('.') = trigger_character {
        completion_dot(program, pos, prog_scope)
    } else {
        let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();

        completions.extend(completion_variable(pos, prog_scope));

        completions.extend(completion_attr(program, pos, prog_scope));

        completions.extend(completion_import_builtin_pkg(program, pos, prog_scope));

        Some(into_completion_items(&completions).into())
    }
}

/// Abstraction of CompletionItem in KCL
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub(crate) struct KCLCompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub kind: Option<KCLCompletionItemKind>,
}

fn completion_dot(
    program: &Program,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::CompletionResponse> {
    // Get the position of trigger_character '.'
    let pos = &KCLPos {
        filename: pos.filename.clone(),
        line: pos.line,
        column: pos.column.map(|c| c - 1),
    };

    match program.pos_to_stmt(pos) {
        Some(node) => match node.node {
            Stmt::Import(stmt) => completion_for_import(&stmt, pos, prog_scope, program),
            _ => Some(into_completion_items(&get_completion(node, pos, prog_scope)).into()),
        },
        None => None,
    }
}

fn completion_import_builtin_pkg(
    program: &Program,
    pos: &KCLPos,
    _prog_scope: &ProgramScope,
) -> IndexSet<KCLCompletionItem> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
    // completion position not contained in import stmt
    // import <space>  <cursor>
    // |             | |  <- input `m` here for complete `math`
    // |<----------->| <- import stmt only contains this range, so we need to check the beginning of line
    let line_start_pos = &KCLPos {
        filename: pos.filename.clone(),
        line: pos.line,
        column: Some(0),
    };

    if let Some(node) = program.pos_to_stmt(line_start_pos) {
        if let Stmt::Import(_) = node.node {
            completions.extend(STANDARD_SYSTEM_MODULES.iter().map(|s| KCLCompletionItem {
                label: s.to_string(),
                detail: None,
                documentation: None,
                kind: Some(KCLCompletionItemKind::Module),
            }))
        }
    }
    completions
}

/// Complete schema attr
///
/// ```no_run
/// p = Person {
///     n<cursor>
/// }
/// ```
/// complete to
/// ```no_run
/// p = Person {
///     name<cursor>
/// }
/// ```
fn completion_attr(
    program: &Program,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> IndexSet<KCLCompletionItem> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();

    if let Some((node, schema_expr)) = is_in_schema(program, pos) {
        let schema_def = find_def(node, &schema_expr.name.get_end_pos(), prog_scope);
        if let Some(schema) = schema_def {
            if let Definition::Object(obj, _) = schema {
                let schema_type = obj.ty.into_schema_type();
                completions.extend(schema_type.attrs.iter().map(|(name, attr)| {
                    KCLCompletionItem {
                        label: name.clone(),
                        detail: Some(format!("{}: {}", name, attr.ty.ty_str())),
                        documentation: attr.doc.clone(),
                        kind: Some(KCLCompletionItemKind::SchemaAttr),
                    }
                }));
            }
        }
    }
    completions
}

/// Complete all usable scope obj in inner_most_scope
fn completion_variable(pos: &KCLPos, prog_scope: &ProgramScope) -> IndexSet<KCLCompletionItem> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
    if let Some(inner_most_scope) = prog_scope.inner_most_scope(pos) {
        for (name, obj) in inner_most_scope.all_usable_objects() {
            match &obj.borrow().kind {
                kclvm_sema::resolver::scope::ScopeObjectKind::Module(module) => {
                    for stmt in &module.import_stmts {
                        match &stmt.0.node {
                            Stmt::Import(import_stmt) => {
                                completions.insert(KCLCompletionItem {
                                    label: import_stmt.name.clone(),
                                    detail: None,
                                    documentation: None,
                                    kind: Some(KCLCompletionItemKind::Module),
                                });
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    completions.insert(KCLCompletionItem {
                        label: name,
                        detail: Some(format!(
                            "{}: {}",
                            obj.borrow().name,
                            obj.borrow().ty.ty_str()
                        )),
                        documentation: obj.borrow().doc.clone(),
                        kind: Some(KCLCompletionItemKind::Variable),
                    });
                }
            }
        }
    }
    completions
}

fn completion_for_import(
    stmt: &ImportStmt,
    _pos: &KCLPos,
    _prog_scope: &ProgramScope,
    program: &Program,
) -> Option<lsp_types::CompletionResponse> {
    let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();
    let pkgpath = &stmt.path;
    let real_path =
        Path::new(&program.root).join(pkgpath.replace('.', &std::path::MAIN_SEPARATOR.to_string()));
    if real_path.is_dir() {
        if let Ok(entries) = fs::read_dir(real_path) {
            let mut entries = entries
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, io::Error>>()
                .unwrap();
            entries.sort();
            for path in entries {
                let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                if path.is_dir() {
                    items.insert(KCLCompletionItem {
                        label: filename,
                        detail: None,
                        documentation: None,
                        kind: Some(KCLCompletionItemKind::Dir),
                    });
                } else if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == KCL_FILE_EXTENSION {
                            items.insert(KCLCompletionItem {
                                label: path
                                    .with_extension("")
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string(),
                                detail: None,
                                documentation: None,
                                kind: Some(KCLCompletionItemKind::File),
                            });
                        }
                    }
                }
            }
        }
    }
    Some(into_completion_items(&items).into())
}

pub(crate) fn get_completion(
    stmt: Node<Stmt>,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> IndexSet<KCLCompletionItem> {
    let (expr, parent) = inner_most_expr_in_stmt(&stmt.node, pos, None);
    match expr {
        Some(node) => {
            let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();
            match node.node {
                Expr::Identifier(id) => {
                    let name = get_identifier_last_name(&id);
                    let def = find_def(stmt, pos, prog_scope);
                    if let Some(def) = def {
                        match def {
                            crate::goto_def::Definition::Object(obj, _) => {
                                match &obj.ty.kind {
                                    // builtin (str) functions
                                    kclvm_sema::ty::TypeKind::Str => {
                                        let funcs = STRING_MEMBER_FUNCTIONS;
                                        for (name, ty) in funcs.iter() {
                                            items.insert(KCLCompletionItem {
                                                label: func_ty_complete_label(
                                                    name,
                                                    &ty.into_function_ty(),
                                                ),
                                                detail: Some(ty.ty_str()),
                                                documentation: ty.ty_doc(),
                                                kind: Some(KCLCompletionItemKind::Function),
                                            });
                                        }
                                    }
                                    // schema attrs, but different from `completion_attr`, here complete for
                                    // ```
                                    // n = Person.n<cursor>
                                    // ```
                                    // complete to
                                    // ```
                                    // n = Person.name
                                    // ```
                                    kclvm_sema::ty::TypeKind::Schema(schema) => {
                                        for (name, attr) in &schema.attrs {
                                            if name != "__settings__" {
                                                items.insert(KCLCompletionItem {
                                                    label: name.clone(),
                                                    detail: Some(format!(
                                                        "{}: {}",
                                                        name,
                                                        attr.ty.ty_str()
                                                    )),
                                                    documentation: attr.doc.clone(),
                                                    kind: Some(KCLCompletionItemKind::SchemaAttr),
                                                });
                                            }
                                        }
                                    }

                                    kclvm_sema::ty::TypeKind::Module(module) => match module.kind {
                                        kclvm_sema::ty::ModuleKind::User => {
                                            match prog_scope
                                                .scope_map
                                                .get(&pkgpath_without_prefix!(module.pkgpath))
                                            {
                                                Some(scope) => {
                                                    items.extend(scope.borrow().elems.keys().map(
                                                        |k| KCLCompletionItem {
                                                            label: k.clone(),
                                                            detail: None,
                                                            documentation: None,
                                                            kind: Some(
                                                                KCLCompletionItemKind::Variable,
                                                            ),
                                                        },
                                                    ))
                                                }
                                                None => {}
                                            }
                                        }
                                        kclvm_sema::ty::ModuleKind::System => {
                                            let funcs = get_system_module_members(name.as_str());
                                            for func in funcs {
                                                let ty = get_system_member_function_ty(&name, func);
                                                let func_ty =
                                                    get_system_member_function_ty(&name, func)
                                                        .into_function_ty();
                                                items.insert(KCLCompletionItem {
                                                    label: func_ty_complete_label(
                                                        &func.to_string(),
                                                        &func_ty,
                                                    ),
                                                    detail: Some(
                                                        func_ty
                                                            .func_signature_str(&func.to_string())
                                                            .to_string(),
                                                    ),
                                                    documentation: ty.ty_doc(),
                                                    kind: Some(KCLCompletionItemKind::Function),
                                                });
                                            }
                                        }
                                        kclvm_sema::ty::ModuleKind::Plugin => {}
                                    },
                                    _ => {}
                                }
                            }
                            crate::goto_def::Definition::Scope(s, _) => {
                                for (name, obj) in &s.elems {
                                    if let ScopeObjectKind::Module(_) = obj.borrow().kind {
                                        continue;
                                    } else {
                                        items.insert(KCLCompletionItem {
                                            label: name.clone(),
                                            detail: None,
                                            documentation: None,
                                            kind: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                Expr::Selector(select_expr) => {
                    let res = get_completion(stmt, &select_expr.value.get_end_pos(), prog_scope);
                    items.extend(res);
                }
                Expr::StringLit(_) => {
                    let funcs = STRING_MEMBER_FUNCTIONS;
                    for (name, ty) in funcs.iter() {
                        items.insert(KCLCompletionItem {
                            label: func_ty_complete_label(name, &ty.into_function_ty()),
                            detail: Some(ty.ty_str()),
                            documentation: ty.ty_doc(),
                            kind: Some(KCLCompletionItemKind::Function),
                        });
                    }
                }
                Expr::Config(_) => match parent {
                    Some(schema_expr) => {
                        if let Expr::Schema(schema_expr) = schema_expr.node {
                            let schema_def =
                                find_def(stmt, &schema_expr.name.get_end_pos(), prog_scope);
                            if let Some(schema) = schema_def {
                                match schema {
                                    Definition::Object(obj, _) => {
                                        let schema_type = obj.ty.into_schema_type();
                                        items.extend(
                                            schema_type
                                                .attrs
                                                .iter()
                                                .map(|(name, attr)| KCLCompletionItem {
                                                    label: name.clone(),
                                                    detail: Some(format!(
                                                        "{}: {}",
                                                        name,
                                                        attr.ty.ty_str()
                                                    )),
                                                    documentation: attr.doc.clone(),
                                                    kind: Some(KCLCompletionItemKind::SchemaAttr),
                                                })
                                                .collect::<IndexSet<KCLCompletionItem>>(),
                                        );
                                    }
                                    Definition::Scope(_, _) => {}
                                }
                            }
                        }
                    }
                    None => {}
                },
                _ => {}
            }

            items
        }
        None => IndexSet::new(),
    }
}

pub(crate) fn into_completion_items(items: &IndexSet<KCLCompletionItem>) -> Vec<CompletionItem> {
    items
        .iter()
        .map(|item| CompletionItem {
            label: item.label.clone(),
            detail: item.detail.clone(),
            documentation: item
                .documentation
                .clone()
                .map(|doc| lsp_types::Documentation::String(doc)),
            kind: item.kind.clone().map(|kind| kind.into()),
            ..Default::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use indexmap::IndexSet;
    use kclvm_error::Position as KCLPos;
    use kclvm_sema::builtin::{MATH_FUNCTION_TYPES, STRING_MEMBER_FUNCTIONS};
    use lsp_types::CompletionResponse;
    use proc_macro_crate::bench_test;

    use crate::{
        completion::{
            completion, func_ty_complete_label, into_completion_items, KCLCompletionItem,
            KCLCompletionItemKind,
        },
        tests::compile_test_file,
    };

    #[test]
    #[bench_test]
    fn var_completion_test() {
        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/completion_test/dot/completion.k");

        // test completion for var
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 26,
            column: Some(5),
        };

        let got = completion(None, &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let mut expected_labels: Vec<&str> = vec![
            "", // generate from error recovery of "pkg."
            "subpkg", "math", "Person", "P", "p", "p1", "p2", "p3", "p4", "aaaa",
        ];

        assert_eq!(got_labels, expected_labels);

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 24,
            column: Some(4),
        };

        let got = completion(None, &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        expected_labels.extend(["__settings__", "name", "age"]);
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn dot_completion_test() {
        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/completion_test/dot/completion.k");

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 12,
            column: Some(7),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["name", "age"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 14,
            column: Some(12),
        };

        // test completion for str builtin function
        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_function_ty()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        // test completion for import pkg path
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["file1", "file2", "subpkg"];
        assert_eq!(got_labels, expected_labels);

        // test completion for import pkg' schema
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 16,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["Person1"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 19,
            column: Some(5),
        };
        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = MATH_FUNCTION_TYPES
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_function_ty()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        // test completion for literal str builtin function
        let pos = KCLPos {
            filename: file.clone(),
            line: 21,
            column: Some(4),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_function_ty()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file,
            line: 30,
            column: Some(11),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["__settings__", "a"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn dot_completion_test_without_dot() {
        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/completion_test/without_dot/completion.k");
        // let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();

        // test completion for schema attr
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 12,
            column: Some(7),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["name", "age"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 14,
            column: Some(12),
        };

        // test completion for str builtin function
        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_function_ty()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        // test completion for import pkg path
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["file1", "file2", "subpkg"];
        assert_eq!(got_labels, expected_labels);

        // test completion for import pkg' schema
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 16,
            column: Some(12),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["Person1"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 19,
            column: Some(5),
        };
        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<String> = MATH_FUNCTION_TYPES
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_function_ty()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        // test completion for literal str builtin function
        let pos = KCLPos {
            filename: file.clone(),
            line: 21,
            column: Some(4),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<String> = STRING_MEMBER_FUNCTIONS
            .iter()
            .map(|(name, ty)| func_ty_complete_label(name, &ty.into_function_ty()))
            .collect();
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file,
            line: 30,
            column: Some(11),
        };

        let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };

        let expected_labels: Vec<&str> = vec!["__settings__", "a"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn import_builtin_package() {
        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/completion_test/import/import.k");
        let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();

        // test completion for builtin packages
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 1,
            column: Some(8),
        };

        let got = completion(None, &program, &pos, &prog_scope).unwrap();

        items.extend(
            [
                "", // generate from error recovery
                "collection",
                "net",
                "manifests",
                "math",
                "datetime",
                "regex",
                "yaml",
                "json",
                "crypto",
                "base64",
                "units",
            ]
            .iter()
            .map(|name| KCLCompletionItem {
                label: name.to_string(),
                kind: Some(KCLCompletionItemKind::Module),
                detail: None,
                documentation: None,
            })
            .collect::<IndexSet<KCLCompletionItem>>(),
        );
        let expect: CompletionResponse = into_completion_items(&items).into();
        assert_eq!(got, expect);
    }
}
