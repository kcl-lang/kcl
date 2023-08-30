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
use kclvm_config::modfile::KCL_FILE_EXTENSION;

use kclvm_error::Position as KCLPos;
use kclvm_sema::builtin::{
    get_system_module_members, STANDARD_SYSTEM_MODULES, STRING_MEMBER_FUNCTIONS,
};
use kclvm_sema::resolver::scope::{ProgramScope, ScopeObjectKind};
use lsp_types::CompletionItem;

use crate::goto_def::{find_def, get_identifier_last_name, Definition};
use crate::util::{inner_most_expr_in_stmt, is_in_schema};

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

        Some(into_completion_items(&completions).into())
    }
}

/// Abstraction of CompletionItem in KCL
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub(crate) struct KCLCompletionItem {
    pub label: String,
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

/// Complete schema attr
fn completion_attr(
    program: &Program,
    pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> IndexSet<KCLCompletionItem> {
    let mut completions: IndexSet<KCLCompletionItem> = IndexSet::new();

    if let Some((node, schema_expr)) = is_in_schema(program, pos) {
        let schema_def = find_def(node, &schema_expr.name.get_end_pos(), prog_scope);
        if let Some(schema) = schema_def {
            if let Definition::Object(obj) = schema {
                let schema_type = obj.ty.into_schema_type();
                completions.extend(schema_type.attrs.keys().map(|attr| KCLCompletionItem {
                    label: attr.clone(),
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
                    completions.insert(KCLCompletionItem {
                        label: module.name.clone(),
                    });
                }
                _ => {
                    completions.insert(KCLCompletionItem { label: name });
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
                    items.insert(KCLCompletionItem { label: filename });
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
                    if !id.pkgpath.is_empty() && STANDARD_SYSTEM_MODULES.contains(&name.as_str()) {
                        items.extend(
                            get_system_module_members(name.as_str())
                                .iter()
                                .map(|s| KCLCompletionItem {
                                    label: s.to_string(),
                                })
                                .collect::<IndexSet<KCLCompletionItem>>(),
                        )
                    }

                    let def = find_def(stmt, pos, prog_scope);

                    if let Some(def) = def {
                        match def {
                            crate::goto_def::Definition::Object(obj) => {
                                match &obj.ty.kind {
                                    // builtin (str) functions
                                    kclvm_sema::ty::TypeKind::Str => {
                                        let binding = STRING_MEMBER_FUNCTIONS;
                                        for k in binding.keys() {
                                            items.insert(KCLCompletionItem {
                                                label: format!("{}{}", k, "()"),
                                            });
                                        }
                                    }
                                    // schema attrs
                                    kclvm_sema::ty::TypeKind::Schema(schema) => {
                                        for k in schema.attrs.keys() {
                                            if k != "__settings__" {
                                                items
                                                    .insert(KCLCompletionItem { label: k.clone() });
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            crate::goto_def::Definition::Scope(s) => {
                                for (name, obj) in &s.elems {
                                    if let ScopeObjectKind::Module(_) = obj.borrow().kind {
                                        continue;
                                    } else {
                                        items.insert(KCLCompletionItem {
                                            label: name.clone(),
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
                    let binding = STRING_MEMBER_FUNCTIONS;
                    for k in binding.keys() {
                        items.insert(KCLCompletionItem {
                            label: format!("{}{}", k, "()"),
                        });
                    }
                }
                Expr::Config(config_expr) => match parent {
                    Some(schema_expr) => {
                        if let Expr::Schema(schema_expr) = schema_expr.node {
                            let schema_def =
                                find_def(stmt, &schema_expr.name.get_end_pos(), prog_scope);
                            if let Some(schema) = schema_def {
                                match schema {
                                    Definition::Object(obj) => {
                                        let schema_type = obj.ty.into_schema_type();
                                        items.extend(
                                            schema_type
                                                .attrs
                                                .keys()
                                                .map(|s| KCLCompletionItem {
                                                    label: s.to_string(),
                                                })
                                                .collect::<IndexSet<KCLCompletionItem>>(),
                                        );
                                    }
                                    Definition::Scope(_) => {}
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
            ..Default::default()
        })
        .collect()
}
