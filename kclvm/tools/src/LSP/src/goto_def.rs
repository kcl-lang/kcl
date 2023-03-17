use kclvm_ast::ast::{AssignStmt, Program, Stmt, UnificationStmt};
use kclvm_ast::pos::ContainsPos;
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
            _ => None,
        },
        None => None,
    }
}

pub(crate) fn goto_def_for_unification(
    stmt: UnificationStmt,
    kcl_pos: KCLPos,
    prog_scope: ProgramScope,
) -> Option<GotoDefinitionResponse> {
    let schema_expr = stmt.value.node;
    if schema_expr.name.contains_pos(&kcl_pos)

    {
        let id = schema_expr.name.node.names.last().unwrap();
        let positions = find_name_in_program_scope(id, prog_scope);
        positions_to_goto_def_resp(&positions)
    } else {
        None
    }
}

pub(crate) fn goto_def_for_assign(
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
        } else if stmt.value.contains_pos(&kcl_pos){
            match stmt.value.node {
                kclvm_ast::ast::Expr::Identifier(id) => Some(id.names.last().unwrap().clone()),
                kclvm_ast::ast::Expr::Schema(schema_expr) => {
                    if schema_expr.name.contains_pos(&kcl_pos) {
                        Some(schema_expr.name.node.names.last().unwrap().clone())
                    } else {
                        None
                    }
                },
                _ => None

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

pub(crate) fn find_name_in_program_scope(
    name: &String,
    prog_scope: ProgramScope,
) -> Vec<(KCLPos, KCLPos)> {
    let mut positions = vec![];
    let mut scopes = vec![];
    for s in prog_scope.scope_map.values() {
        scopes.push(s.borrow().clone());
    }

    while !scopes.is_empty() {
        let s = scopes.pop().unwrap();
        match s.lookup(&name) {
            Some(obj) => {
                let obj = obj.borrow().clone();
                positions.push((obj.start, obj.end));
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

pub(crate) fn positions_to_goto_def_resp(
    positions: &Vec<(KCLPos, KCLPos)>,
) -> Option<GotoDefinitionResponse> {
    match positions.len() {
        0 => None,
        1 => {
            let (start, end) = positions[0].clone();
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
