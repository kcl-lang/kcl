//! GotoDefinition for KCL
//! Github Issue: https://github.com/kcl-lang/kcl/issues/476
//! Now supports goto definition for the following situation:
//! + variable
//! + schema definition
//! + mixin definition
//! + schema attr
//! + attr type

use indexmap::{IndexMap, IndexSet};
use kclvm_ast::pos::{ContainsPos, GetPos};

use kclvm_ast::ast::{Expr, Identifier, ImportStmt, Node, Program, Stmt};
use kclvm_compiler::pkgpath_without_prefix;
use kclvm_error::Position as KCLPos;

use kclvm_sema::resolver::scope::{ProgramScope, Scope, ScopeObject};
use kclvm_sema::ty::SchemaType;
use lsp_types::{GotoDefinitionResponse, Url};
use lsp_types::{Location, Range};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::to_lsp::lsp_pos;
use crate::util::{
    get_pkg_scope, get_pos_from_real_path, get_real_path_from_external, inner_most_expr_in_stmt,
};

// Navigates to the definition of an identifier.
pub(crate) fn goto_definition(
    program: &Program,
    kcl_pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<lsp_types::GotoDefinitionResponse> {
    match program.pos_to_stmt(kcl_pos) {
        Some(node) => match node.node {
            Stmt::Import(stmt) => goto_def_for_import(&stmt, kcl_pos, prog_scope, program),
            _ => match find_def(node.clone(), kcl_pos, prog_scope) {
                Some(def) => positions_to_goto_def_resp(&def.get_positions()),
                None => None,
            },
        },
        None => None,
    }
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

pub enum Definition {
    Object(ScopeObject),
    Scope(Scope),
}

impl Definition {
    pub(crate) fn get_positions(&self) -> IndexSet<(KCLPos, KCLPos)> {
        let mut positions = IndexSet::new();
        match self {
            Definition::Object(obj) => {
                positions.insert((obj.start.clone(), obj.end.clone()));
            }
            Definition::Scope(scope) => match &scope.kind {
                kclvm_sema::resolver::scope::ScopeKind::Package(filenames) => {
                    for file in filenames {
                        let dummy_pos = KCLPos {
                            filename: file.clone(),
                            line: 1,
                            column: None,
                        };
                        positions.insert((dummy_pos.clone(), dummy_pos));
                    }
                }
                _ => {
                    positions.insert((scope.start.clone(), scope.end.clone()));
                }
            },
        }
        positions
    }
}

pub(crate) fn find_def(
    node: Node<Stmt>,
    kcl_pos: &KCLPos,
    prog_scope: &ProgramScope,
) -> Option<Definition> {
    fn pre_process_identifier(id: Node<Identifier>, pos: &KCLPos) -> Identifier {
        if !id.contains_pos(pos) && id.node.names.is_empty() {
            return id.node.clone();
        }

        let mut id = id.node.clone();
        let mut names = vec![];
        for name in id.names {
            names.push(name.clone());
            if name.contains_pos(pos) {
                break;
            }
        }
        id.names = names;
        if !id.pkgpath.is_empty() {
            id.names[0].node = pkgpath_without_prefix!(id.pkgpath);
        }
        id
    }

    let (inner_expr, parent) = inner_most_expr_in_stmt(&node.node, kcl_pos, None);
    if let Some(expr) = inner_expr {
        if let Expr::Identifier(id) = expr.node {
            let id_node = Node::node_with_pos(
                id.clone(),
                (
                    expr.filename,
                    expr.line,
                    expr.column,
                    expr.end_line,
                    expr.end_column,
                ),
            );
            let id = pre_process_identifier(id_node, kcl_pos);
            match parent {
                Some(schema_expr) => {
                    if let Expr::Schema(schema_expr) = schema_expr.node {
                        let schema_def =
                            find_def(node, &schema_expr.name.get_end_pos(), prog_scope);
                        if let Some(schema) = schema_def {
                            match schema {
                                Definition::Object(obj) => {
                                    let schema_type = obj.ty.into_schema_type();
                                    return find_attr_in_schema(
                                        &schema_type,
                                        &id.names,
                                        &prog_scope.scope_map,
                                    );
                                }
                                Definition::Scope(_) => {
                                    //todo
                                }
                            }
                        }
                    }
                }
                None => {
                    for (_, scope) in &prog_scope.scope_map {
                        match scope.borrow().inner_most(kcl_pos) {
                            Some(s) => return resolve_var(&id.names, &s, &prog_scope.scope_map),
                            None => continue,
                        }
                    }
                }
            }
        }
    }
    None
}

/// Similar to vars.rs/resolver_var, find a ScopeObj corresponding to the definition of identifier
pub(crate) fn resolve_var(
    node_names: &[Node<String>],
    current_scope: &Scope,
    scope_map: &IndexMap<String, Rc<RefCell<Scope>>>,
) -> Option<Definition> {
    let names = node_names
        .iter()
        .map(|node| node.node.clone())
        .collect::<Vec<String>>();
    match names.len() {
        0 => None,
        1 => {
            let name = names[0].clone();
            match current_scope.lookup(&name) {
                Some(obj) => match obj.borrow().kind {
                    kclvm_sema::resolver::scope::ScopeObjectKind::Module => {
                        match scope_map.get(&name) {
                            Some(scope) => Some(Definition::Scope(scope.borrow().clone())),
                            None => None,
                        }
                    }
                    _ => Some(Definition::Object(obj.borrow().clone())),
                },
                None => None,
            }
        }
        _ => {
            let name = names[0].clone();
            match current_scope.lookup(&name) {
                Some(obj) => {
                    match &obj.borrow().ty.kind {
                        kclvm_sema::ty::TypeKind::Schema(schema_type) => {
                            find_attr_in_schema(schema_type, &node_names[1..], scope_map)
                        }
                        kclvm_sema::ty::TypeKind::Module(module_ty) => {
                            match scope_map.get(&pkgpath_without_prefix!(module_ty.pkgpath)) {
                                Some(scope) => {
                                    return resolve_var(
                                        &node_names[1..],
                                        &scope.borrow(),
                                        scope_map,
                                    );
                                }
                                None => None,
                            }
                        }
                        kclvm_sema::ty::TypeKind::Dict(_, _) => {
                            // Todo: find key def in dict
                            None
                        }
                        _ => None,
                    }
                }
                None => None,
            }
        }
    }
}

pub fn find_attr_in_schema(
    schema_type: &SchemaType,
    names: &[Node<String>],
    scope_map: &IndexMap<String, Rc<RefCell<Scope>>>,
) -> Option<Definition> {
    let schema_pkg_scope = get_pkg_scope(&schema_type.pkgpath, scope_map);
    let names = if schema_type.pkgpath.is_empty() {
        &names[1..]
    } else {
        names
    };
    for child in &schema_pkg_scope.children {
        let child_scope = child.borrow();
        if let kclvm_sema::resolver::scope::ScopeKind::Schema(schema_name) = &child_scope.kind {
            if schema_name == &schema_type.name {
                return resolve_var(&names, &child_scope, scope_map);
            }
        }
    }
    None
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

pub(crate) fn get_identifier_last_name(id: &Identifier) -> String {
    match id.names.len() {
        0 => "".to_string(),
        1 => id.names[0].node.clone(),
        _ => {
            if id.names.last().unwrap().node == *"" {
                // MissingExpr
                id.names.get(id.names.len() - 2).unwrap().node.clone()
            } else {
                id.names.last().unwrap().node.clone()
            }
        }
    }
}
