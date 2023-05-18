//! GotoDefinition for KCL
//! Github Issue: https://github.com/KusionStack/KCLVM/issues/476
//! Now supports goto definition for the following situation:
//! + variable
//! + schema definition
//! + mixin definition
//! + schema attr
//! + attr type

use indexmap::IndexSet;

use kclvm_ast::ast::{Expr, Identifier, ImportStmt, Node, Program, SchemaExpr, Stmt};

use kclvm_error::Position as KCLPos;

use kclvm_sema::resolver::scope::{ProgramScope, ScopeObject};
use lsp_types::{GotoDefinitionResponse, Url};
use lsp_types::{Location, Range};
use std::path::Path;

use crate::to_lsp::lsp_pos;
use crate::util::{get_pos_from_real_path, get_real_path_from_external, inner_most_expr_in_stmt};

// Navigates to the definition of an identifier.
pub(crate) fn goto_definition(
    program: &Program,
    kcl_pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::GotoDefinitionResponse> {
    match program.pos_to_stmt(kcl_pos) {
        Some(node) => match node.node {
            Stmt::Import(stmt) => goto_def_for_import(&stmt, kcl_pos, prog_scope, program),
            _ => {
                let objs = find_definition_objs(node, kcl_pos, prog_scope);
                let positions = objs
                    .iter()
                    .map(|obj| (obj.start.clone(), obj.end.clone()))
                    .collect();
                positions_to_goto_def_resp(&positions)
            }
        },
        None => None,
    }
}
pub(crate) fn find_definition_objs(
    node: Node<Stmt>,
    kcl_pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Vec<ScopeObject> {
    let (inner_expr, parent) = inner_most_expr_in_stmt(&node.node, kcl_pos, None);
    if let Some(expr) = inner_expr {
        if let Expr::Identifier(id) = expr.node {
            let name = get_identifier_last_name(&id);
            let objs = if let Some(parent) = parent {
                // find schema attr def
                match parent.node {
                    Expr::Schema(schema_expr) => {
                        find_def_of_schema_attr(schema_expr, prog_scope, name)
                    }
                    _ => vec![],
                }
            } else {
                find_objs_in_program_scope(&name, prog_scope)
            };
            return objs;
        }
    }
    vec![]
}

// This function serves as the result of a global search, which may cause duplication.
// It needs to be pruned according to the situation. There are two actions todo:
// + AST Identifier provides location information for each name.
// + Scope provides a method similar to resolve_var of the resolver to replace this function.
pub(crate) fn find_objs_in_program_scope(
    name: &str,
    prog_scope: &ProgramScope,
) -> Vec<ScopeObject> {
    let mut res = vec![];
    for s in prog_scope.scope_map.values() {
        let mut objs = s.borrow().search_obj_by_name(name);
        res.append(&mut objs);
    }

    res
}

// Convert kcl position to GotoDefinitionResponse. This function will convert to
// None, Scalar or Array according to the number of positions
fn positions_to_goto_def_resp(
    positions: &IndexSet<(KCLPos, KCLPos)>,
) -> Option<GotoDefinitionResponse> {
    match positions.len() {
        0 => None,
        1 => {
            let (start, end) = positions.iter().next().unwrap().clone();
            Some(lsp_types::GotoDefinitionResponse::Scalar(Location {
                uri: Url::from_file_path(start.filename.clone()).unwrap(),
                range: Range {
                    start: lsp_pos(&start),
                    end: lsp_pos(&end),
                },
            }))
        }
        _ => {
            let mut res = vec![];
            for (start, end) in positions {
                res.push(Location {
                    uri: Url::from_file_path(start.filename.clone()).unwrap(),
                    range: Range {
                        start: lsp_pos(start),
                        end: lsp_pos(end),
                    },
                })
            }
            Some(lsp_types::GotoDefinitionResponse::Array(res))
        }
    }
}

fn goto_def_for_import(
    stmt: &ImportStmt,
    _kcl_pos: &KCLPos,
    _prog_scope: &ProgramScope,
    program: &Program,
) -> Option<GotoDefinitionResponse> {
    let pkgpath = &stmt.path;
    let mut real_path =
        Path::new(&program.root).join(pkgpath.replace('.', &std::path::MAIN_SEPARATOR.to_string()));
    let mut positions = get_pos_from_real_path(&real_path);

    if positions.is_empty() && !real_path.exists() {
        real_path =
            get_real_path_from_external(&stmt.pkg_name, pkgpath, program.root.clone().into());
    }

    positions = get_pos_from_real_path(&real_path);

    positions_to_goto_def_resp(&positions)
}

// Todo: fix ConfigExpr
// ```kcl
// schema Person:
//     name: str
//     data: Data

// schema Data:
//     id: int

// person = Person {
//     data.id = 1
//     data: {
//         id = 1
//     }
//     data: Data {
//         id = 3
//     }
// }
pub(crate) fn find_def_of_schema_attr(
    schema_expr: SchemaExpr,
    prog_scope: &ProgramScope,
    attr_name: String,
) -> Vec<ScopeObject> {
    let schema_name = get_identifier_last_name(&schema_expr.name.node);
    let mut res = vec![];
    for scope in prog_scope.scope_map.values() {
        let s = scope.borrow();
        if let Some(scope) = s.search_child_scope_by_name(&schema_name) {
            let s = scope.borrow();
            if matches!(s.kind, kclvm_sema::resolver::scope::ScopeKind::Schema(_)) {
                for (attr, obj) in &s.elems {
                    if attr == &attr_name {
                        res.push(obj.borrow().clone());
                    }
                }
            }
        }
    }
    res
}

pub(crate) fn get_identifier_last_name(id: &Identifier) -> String {
    match id.names.len() {
        0 => "".to_string(),
        1 => id.names[0].clone(),
        _ => {
            if id.names.last().unwrap().clone() == *"" {
                // MissingExpr
                id.names.get(id.names.len() - 2).unwrap().clone()
            } else {
                id.names.last().unwrap().clone()
            }
        }
    }
}
