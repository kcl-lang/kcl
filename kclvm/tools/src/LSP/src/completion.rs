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
use kclvm_ast::ast::{Expr, ImportStmt, Program, Stmt};
use kclvm_compiler::pkgpath_without_prefix;
use kclvm_config::modfile::KCL_FILE_EXTENSION;

use kclvm_error::Position as KCLPos;
use kclvm_sema::builtin::{
    get_system_module_members, STANDARD_SYSTEM_MODULES, STRING_MEMBER_FUNCTIONS,
};
use kclvm_sema::resolver::scope::ProgramScope;
use lsp_types::CompletionItem;

use crate::goto_def::{get_identifier_last_name, resolve_var};
use crate::util::inner_most_expr_in_stmt;
use crate::util::{fix_missing_identifier, get_pkg_scope};

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
        // todo: Complete identifiers such as attr, variables, types, etc.
        None
    }
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
            _ => {
                let expr = inner_most_expr_in_stmt(&node.node, pos, None).0;
                match expr {
                    Some(node) => {
                        let items = get_completion_items(&node.node, prog_scope);
                        Some(into_completion_items(&items).into())
                    }
                    None => None,
                }
            }
        },
        None => None,
    }
}

fn completion_for_import(
    stmt: &ImportStmt,
    _pos: &KCLPos,
    _prog_scope: &ProgramScope,
    program: &Program,
) -> Option<lsp_types::CompletionResponse> {
    let mut items: IndexSet<String> = IndexSet::new();
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
                    items.insert(filename);
                } else if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == KCL_FILE_EXTENSION {
                            items.insert(
                                path.with_extension("")
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
    }
    Some(into_completion_items(&items).into())
}

fn get_completion_items(expr: &Expr, prog_scope: &ProgramScope) -> IndexSet<String> {
    let mut items = IndexSet::new();
    match expr {
        Expr::Identifier(id) => {
            let name = get_identifier_last_name(&id);
            if !id.pkgpath.is_empty() {
                // standard system module
                if STANDARD_SYSTEM_MODULES.contains(&name.as_str()) {
                    items.extend(
                        get_system_module_members(name.as_str())
                            .iter()
                            .map(|s| s.to_string()),
                    )
                }
                // user module
                if let Some(scope) = prog_scope
                    .scope_map
                    .get(&pkgpath_without_prefix!(id.pkgpath))
                {
                    let scope = scope.borrow();
                    for (name, obj) in &scope.elems {
                        if obj.borrow().ty.is_module() {
                            continue;
                        }
                        items.insert(name.clone());
                    }
                }
                return items;
            }

            let def = resolve_var(
                &fix_missing_identifier(&id.names),
                &get_pkg_scope(&id.pkgpath, &prog_scope.scope_map),
                &prog_scope.scope_map,
            );

            if let Some(def) = def {
                match def {
                    crate::goto_def::Definition::Object(obj) => {
                        match &obj.ty.kind {
                            // builtin (str) functions
                            kclvm_sema::ty::TypeKind::Str => {
                                let binding = STRING_MEMBER_FUNCTIONS;
                                for k in binding.keys() {
                                    items.insert(format!("{}{}", k, "()"));
                                }
                            }
                            // schema attrs
                            kclvm_sema::ty::TypeKind::Schema(schema) => {
                                for k in schema.attrs.keys() {
                                    if k != "__settings__" {
                                        items.insert(k.clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    crate::goto_def::Definition::Scope(_) => {}
                }
            }
        }
        Expr::Selector(select_expr) => {
            let res = get_completion_items(&select_expr.value.node, prog_scope);
            items.extend(res);
        }
        Expr::StringLit(_) => {
            let binding = STRING_MEMBER_FUNCTIONS;
            for k in binding.keys() {
                items.insert(format!("{}{}", k, "()"));
            }
        }
        _ => {}
    }
    items
}

pub(crate) fn into_completion_items(items: &IndexSet<String>) -> Vec<CompletionItem> {
    items
        .iter()
        .map(|item| CompletionItem {
            label: item.clone(),
            ..Default::default()
        })
        .collect()
}
