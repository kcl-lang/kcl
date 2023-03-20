use indexmap::IndexSet;

use std::fs;
use std::path::Path;

use kclvm_ast::ast::{AssignStmt, ImportStmt, Program, Stmt, UnificationStmt};
use kclvm_ast::pos::ContainsPos;
use kclvm_config::modfile::KCL_FILE_EXTENSION;
use kclvm_error::Position as KCLPos;
use kclvm_sema::resolver::scope::ProgramScope;
use lsp_types::{GotoDefinitionResponse, Url};
use lsp_types::{Location, Range};

use crate::to_lsp::lsp_pos;

pub(crate) fn goto_definition(
    program: Program,
    kcl_pos: KCLPos,
    prog_scope: ProgramScope,
) -> Option<lsp_types::GotoDefinitionResponse> {
    match program.pos_to_stmt(&kcl_pos) {
        Some(node) => match node.node {
            Stmt::Unification(stmt) => goto_def_for_unification(stmt, kcl_pos, prog_scope),
            Stmt::Assign(stmt) => goto_def_for_assign(stmt, kcl_pos, prog_scope),
            Stmt::Import(stmt) => goto_def_for_import(stmt, kcl_pos, prog_scope, program),
            _ => {
                // todo
                None
            }
        },
        None => None,
    }
}

fn find_name_in_program_scope(name: &str, prog_scope: ProgramScope) -> IndexSet<(KCLPos, KCLPos)> {
    let mut positions = IndexSet::new();
    let mut scopes = vec![];
    for s in prog_scope.scope_map.values() {
        scopes.push(s.borrow().clone());
    }

    while !scopes.is_empty() {
        let s = scopes.pop().unwrap();
        match s.lookup(&name) {
            Some(obj) => {
                let obj = obj.borrow().clone();
                positions.insert((obj.start, obj.end));
            }
            None => {
                for c in s.children {
                    scopes.push(c.borrow().clone());
                }
            }
        }
    }
    positions
}

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
                        start: lsp_pos(&start),
                        end: lsp_pos(&end),
                    },
                })
            }
            Some(lsp_types::GotoDefinitionResponse::Array(res))
        }
    }
}

fn goto_def_for_unification(
    stmt: UnificationStmt,
    kcl_pos: KCLPos,
    prog_scope: ProgramScope,
) -> Option<GotoDefinitionResponse> {
    let schema_expr = stmt.value.node;
    if schema_expr.name.contains_pos(&kcl_pos) {
        let id = schema_expr.name.node.names.last().unwrap();
        let positions = find_name_in_program_scope(id, prog_scope);
        positions_to_goto_def_resp(&positions)
    } else {
        None
    }
}

fn goto_def_for_assign(
    stmt: AssignStmt,
    kcl_pos: KCLPos,
    prog_scope: ProgramScope,
) -> Option<GotoDefinitionResponse> {
    let id = {
        if let Some(ty) = stmt.type_annotation {
            if ty.contains_pos(&kcl_pos) {
                Some(ty.node)
            } else {
                None
            }
        } else if stmt.value.contains_pos(&kcl_pos) {
            match stmt.value.node {
                kclvm_ast::ast::Expr::Identifier(id) => Some(id.names.last().unwrap().clone()),
                kclvm_ast::ast::Expr::Schema(schema_expr) => {
                    if schema_expr.name.contains_pos(&kcl_pos) {
                        Some(schema_expr.name.node.names.last().unwrap().clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    };
    match id {
        Some(id) => {
            let positions = find_name_in_program_scope(&id, prog_scope);
            positions_to_goto_def_resp(&positions)
        }
        None => None,
    }
}

fn goto_def_for_import(
    stmt: ImportStmt,
    _kcl_pos: KCLPos,
    _prog_scope: ProgramScope,
    program: Program,
) -> Option<GotoDefinitionResponse> {
    let pkgpath = &stmt.path;
    let real_path = Path::new(&program.root).join(pkgpath.replace('.', "/"));
    let mut positions = IndexSet::new();
    let mut k_file = real_path.clone();
    k_file.set_extension(KCL_FILE_EXTENSION);

    if k_file.is_file() {
        let start = KCLPos {
            filename: k_file.to_str().unwrap().to_string(),
            line: 1,
            column: None,
        };
        let end = start.clone();
        positions.insert((start, end));
    } else if real_path.is_dir() {
        if let Ok(entries) = fs::read_dir(real_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(extension) = entry.path().extension() {
                        if extension == KCL_FILE_EXTENSION {
                            let start = KCLPos {
                                filename: entry.path().to_str().unwrap().to_string(),
                                line: 1,
                                column: None,
                            };
                            let end = start.clone();
                            positions.insert((start, end));
                        }
                    }
                }
            }
        }
    }
    positions_to_goto_def_resp(&positions)
}
