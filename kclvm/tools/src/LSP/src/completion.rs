//! Complete for KCL
//! Now supports code completion in treigger mode (triggered when user enters `.`, `:` and `=`), schema attr and global variables
//! and the content of the completion includes:
//! + variable
//! + schema attr name
//! + dot(.)
//!     + import path
//!     + schema attr
//!     + builtin function(str function)
//!     + defitions in pkg
//!     + system module functions
//! + assign(=, :)
//!     + schema attr value
//!     + variable value
//! + new line
//!     + schema init

use std::io;
use std::{fs, path::Path};

use indexmap::IndexSet;
use kclvm_ast::ast::{Expr, ImportStmt, Node, Program, Stmt};
use kclvm_ast::pos::GetPos;
use kclvm_ast::MAIN_PKG;
use kclvm_config::modfile::KCL_FILE_EXTENSION;
use kclvm_sema::pkgpath_without_prefix;

use kclvm_error::Position as KCLPos;
use kclvm_sema::builtin::{
    get_system_member_function_ty, get_system_module_members, STANDARD_SYSTEM_MODULES,
    STRING_MEMBER_FUNCTIONS,
};
use kclvm_sema::resolver::doc::{parse_doc_string, Doc};
use kclvm_sema::resolver::scope::{ProgramScope, ScopeObjectKind};
use kclvm_sema::ty::{FunctionType, SchemaType, Type};
use lsp_types::{CompletionItem, CompletionItemKind};

use crate::goto_def::{find_def, get_identifier_last_name, Definition};
use crate::util::{inner_most_expr_in_stmt, is_in_docstring, is_in_schema_expr};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum KCLCompletionItemKind {
    Function,
    Variable,
    File,
    Dir,
    Schema,
    SchemaAttr,
    Module,
    Doc,
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
            KCLCompletionItemKind::Doc => CompletionItemKind::SNIPPET,
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

fn ty_complete_label(ty: &Type) -> Vec<String> {
    match &ty.kind {
        kclvm_sema::ty::TypeKind::Bool => vec!["True".to_string(), "False".to_string()],
        kclvm_sema::ty::TypeKind::BoolLit(b) => {
            vec![if *b {
                "True".to_string()
            } else {
                "False".to_string()
            }]
        }
        kclvm_sema::ty::TypeKind::IntLit(i) => vec![i.to_string()],
        kclvm_sema::ty::TypeKind::FloatLit(f) => vec![f.to_string()],
        kclvm_sema::ty::TypeKind::Str => vec![r#""""#.to_string()],
        kclvm_sema::ty::TypeKind::StrLit(s) => vec![format!("{:?}", s)],
        kclvm_sema::ty::TypeKind::List(_) => vec!["[]".to_string()],
        kclvm_sema::ty::TypeKind::Dict(_) => vec!["{}".to_string()],
        kclvm_sema::ty::TypeKind::Union(types) => {
            types.iter().flat_map(|ty| ty_complete_label(ty)).collect()
        }
        kclvm_sema::ty::TypeKind::Schema(schema) => {
            vec![format!(
                "{}{}{}",
                if schema.pkgpath.is_empty() || schema.pkgpath == MAIN_PKG {
                    "".to_string()
                } else {
                    format!("{}.", schema.pkgpath.split(".").last().unwrap())
                },
                schema.name,
                "{}"
            )]
        }
        _ => vec![],
    }
}

/// Computes completions at the given position.
pub(crate) fn completion(
    trigger_character: Option<char>,
    program: &Program,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::CompletionResponse> {
    match trigger_character {
        Some(c) => match c {
            '.' => completion_dot(program, pos, prog_scope),
            '=' | ':' => completion_assign(program, pos, prog_scope),
            '\n' => completion_newline(program, pos, prog_scope),
            _ => None,
        },
        None => {
            let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();

            completions.extend(completion_variable(pos, prog_scope));

            completions.extend(completion_attr(program, pos, prog_scope));

            completions.extend(completion_import_builtin_pkg(program, pos, prog_scope));

            Some(into_completion_items(&completions).into())
        }
    }
}

/// Abstraction of CompletionItem in KCL
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
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
            _ => Some(into_completion_items(&get_dot_completion(node, pos, prog_scope)).into()),
        },
        None => None,
    }
}

fn completion_assign(
    program: &Program,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::CompletionResponse> {
    // Get the position of trigger_character '=' or ':'
    let pos = &KCLPos {
        filename: pos.filename.clone(),
        line: pos.line,
        column: pos.column.map(|c| c - 1),
    };

    match program.pos_to_stmt(pos) {
        Some(node) => Some(
            into_completion_items(&get_schema_attr_value_completion(node, pos, prog_scope)).into(),
        ),
        None => None,
    }
}

fn completion_newline(
    program: &Program,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::CompletionResponse> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();
    let pos = &KCLPos {
        filename: pos.filename.clone(),
        line: pos.line - 1,
        column: pos.column,
    };

    match program.pos_to_stmt(pos) {
        Some(node) => {
            let end_pos = node.get_end_pos();
            if let Some((node, schema_expr)) = is_in_schema_expr(program, &end_pos) {
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
            } else if let Some((doc, schema)) = is_in_docstring(program, &pos) {
                let doc = parse_doc_string(&doc.node);
                if doc.summary.is_empty() && doc.attrs.len() == 0 && doc.examples.len() == 0 {
                    // empty docstring, provide total completion
                    let doc_parsed = Doc::new_from_schema_stmt(&schema);
                    let label = doc_parsed.to_doc_string();
                    // generate docstring from doc
                    completions.insert(KCLCompletionItem {
                        label,
                        detail: Some("generate docstring".to_string()),
                        documentation: Some(format!("docstring for {}", schema.name.node.clone())),
                        kind: Some(KCLCompletionItemKind::Doc),
                    });
                }
            }
        }
        None => {}
    }
    Some(into_completion_items(&completions).into())
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

    if let Some((node, schema_expr)) = is_in_schema_expr(program, pos) {
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
                kclvm_sema::resolver::scope::ScopeObjectKind::Definition => {
                    let schema_ty = obj.borrow().ty.clone().into_schema_type();
                    completions.insert(schema_ty_completion_item(&schema_ty));
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
                        kind: Some(KCLCompletionItemKind::Schema),
                    });
                }
            }
        }
    }
    completions
}

/// Complete schema name
///
/// ```no_run
/// p = P<cursor>
/// ```
/// complete to
/// ```no_run
/// p = Person(param1, param2){}<cursor>
/// ```
fn schema_ty_completion_item(schema_ty: &SchemaType) -> KCLCompletionItem {
    let param = schema_ty.func.params.clone();
    let label = format!(
        "{}{}{}",
        schema_ty.name.clone(),
        if param.is_empty() {
            "".to_string()
        } else {
            format!(
                "({})",
                param
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        },
        "{}"
    );
    let detail = {
        let mut details = vec![];
        details.push(schema_ty.schema_ty_signature_str());
        details.push("Attributes:".to_string());
        for (name, attr) in &schema_ty.attrs {
            details.push(format!(
                "{}{}:{}",
                name,
                if attr.is_optional { "?" } else { "" },
                format!(" {}", attr.ty.ty_str()),
            ));
        }
        details.join("\n")
    };
    KCLCompletionItem {
        label,
        detail: Some(detail),
        documentation: Some(schema_ty.doc.clone()),
        kind: Some(KCLCompletionItemKind::Schema),
    }
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

/// Get completion items for trigger '=' or ':'
/// Now, just completion for schema attr value
pub(crate) fn get_schema_attr_value_completion(
    stmt: Node<Stmt>,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> IndexSet<KCLCompletionItem> {
    let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();
    let (expr, _) = inner_most_expr_in_stmt(&stmt.node, pos, None);
    if let Some(node) = expr {
        if let Expr::Identifier(_) = node.node {
            let def = find_def(stmt, pos, prog_scope);
            if let Some(def) = def {
                match def {
                    crate::goto_def::Definition::Object(obj, _) => match obj.kind {
                        ScopeObjectKind::Attribute => {
                            let ty = obj.ty;
                            items.extend(ty_complete_label(&ty).iter().map(|label| {
                                KCLCompletionItem {
                                    label: format!(" {}", label),
                                    detail: Some(format!("{}: {}", obj.name, ty.ty_str())),
                                    kind: Some(KCLCompletionItemKind::Variable),
                                    documentation: obj.doc.clone(),
                                }
                            }))
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    items
}

/// Get completion items for trigger '.'
pub(crate) fn get_dot_completion(
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
                                    match obj.borrow().kind {
                                        ScopeObjectKind::Definition => {
                                            items.insert(schema_ty_completion_item(
                                                &obj.borrow().ty.into_schema_type(),
                                            ));
                                        }
                                        ScopeObjectKind::Module(_) => continue,
                                        _ => {
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
                }
                Expr::Selector(select_expr) => {
                    let res =
                        get_dot_completion(stmt, &select_expr.value.get_end_pos(), prog_scope);
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
    use lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse};
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
        let (file, program, prog_scope, _, gs) =
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
            "subpkg", "math", "Person{}", "P{}", "p", "p1", "p2", "p3", "p4", "aaaa",
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

        expected_labels.extend(["name", "age"]);
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn dot_completion_test() {
        let (file, program, prog_scope, _, gs) =
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

        let expected_labels: Vec<&str> = vec!["Person1{}"];
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

        let expected_labels: Vec<&str> = vec!["a"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn dot_completion_test_without_dot() {
        let (file, program, prog_scope, _, gs) =
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

        let expected_labels: Vec<&str> = vec!["Person1{}"];
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

        let expected_labels: Vec<&str> = vec!["a"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn import_builtin_package() {
        let (file, program, prog_scope, _, gs) =
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

    #[test]
    #[bench_test]
    fn attr_value_completion() {
        let (file, program, prog_scope, _, gs) =
            compile_test_file("src/test_data/completion_test/assign/completion.k");

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 14,
            column: Some(6),
        };

        let got = completion(Some(':'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" True", " False"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 16,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" \"abc\"", " \"def\""];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 18,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" []"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 20,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" 1"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 22,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" True"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 24,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" {}"];
        assert_eq!(got_labels, expected_labels);

        let pos = KCLPos {
            filename: file.to_owned(),
            line: 26,
            column: Some(6),
        };
        let got = completion(Some(':'), &program, &pos, &prog_scope).unwrap();
        let got_labels: Vec<String> = match got {
            CompletionResponse::Array(arr) => arr.iter().map(|item| item.label.clone()).collect(),
            CompletionResponse::List(_) => panic!("test failed"),
        };
        let expected_labels: Vec<&str> = vec![" subpkg.Person1{}"];
        assert_eq!(got_labels, expected_labels);
    }

    #[test]
    #[bench_test]
    fn schema_sig_completion() {
        let (file, program, prog_scope, _, gs) =
            compile_test_file("src/test_data/completion_test/schema/schema.k");

        // test completion for builtin packages
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 7,
            column: Some(5),
        };

        let got = completion(None, &program, &pos, &prog_scope).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert_eq!(
                    arr[1],
                    CompletionItem {
                        label: "Person(b){}".to_string(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some(
                            "__main__\n\nschema Person[b: int](Base)\nAttributes:\nc: int"
                                .to_string()
                        ),
                        documentation: Some(lsp_types::Documentation::String("".to_string())),
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn schema_attr_newline_completion() {
        let (file, program, prog_scope, _, _) =
            compile_test_file("src/test_data/completion_test/newline/newline.k");

        // test completion for builtin packages
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 8,
            column: Some(4),
        };

        let got = completion(Some('\n'), &program, &pos, &prog_scope).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert_eq!(
                    arr[0],
                    CompletionItem {
                        label: "c".to_string(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some("c: int".to_string()),
                        documentation: None,
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }

    #[test]
    fn schema_docstring_newline_completion() {
        let (file, program, prog_scope, _, _) =
            compile_test_file("src/test_data/completion_test/newline/docstring_newline.k");

        // test completion for builtin packages
        let pos = KCLPos {
            filename: file.to_owned(),
            line: 3,
            column: Some(4),
        };

        let got = completion(Some('\n'), &program, &pos, &prog_scope).unwrap();
        match got {
            CompletionResponse::Array(arr) => {
                assert_eq!(
                    arr[0],
                    CompletionItem {
                        label: "\n\nAttributes\n---------\nname: \nworkloadType: \nreplica: \n\nExamples\n--------\n".to_string(),
                        detail: Some("generate docstring".to_string()),
                        kind: Some(CompletionItemKind::SNIPPET),
                        documentation: Some(lsp_types::Documentation::String("docstring for Server".to_string())),
                        ..Default::default()
                    }
                )
            }
            CompletionResponse::List(_) => panic!("test failed"),
        }
    }
}
