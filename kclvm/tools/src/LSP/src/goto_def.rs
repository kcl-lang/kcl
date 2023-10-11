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

use kclvm_sema::builtin::{get_system_member_function_ty, STRING_MEMBER_FUNCTIONS};
use kclvm_sema::resolver::scope::{
    builtin_scope, ProgramScope, Scope, ScopeObject, ScopeObjectKind,
};
use kclvm_sema::ty::{DictType, SchemaType};
use lsp_types::{GotoDefinitionResponse, Url};
use lsp_types::{Location, Range};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::to_lsp::lsp_pos;
use crate::util::{
    fix_missing_identifier, get_pkg_scope, get_pos_from_real_path, get_real_path_from_external,
    inner_most_expr_in_stmt,
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

#[derive(Debug)]
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
        id.names = fix_missing_identifier(&names);
        if !id.pkgpath.is_empty() {
            id.names[0].node = pkgpath_without_prefix!(id.pkgpath);
        }
        id
    }

    let (inner_expr, parent) = inner_most_expr_in_stmt(&node.node, kcl_pos, None);
    if let Some(expr) = inner_expr {
        match expr.node {
            Expr::Identifier(id) => {
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
                                    Definition::Object(obj) => match &obj.ty.kind {
                                        kclvm_sema::ty::TypeKind::Schema(schema_type) => {
                                            return find_attr_in_schema(
                                                &schema_type,
                                                &id.names,
                                                &prog_scope.scope_map,
                                            )
                                        }
                                        _ => {}
                                    },
                                    Definition::Scope(_) => {}
                                }
                            }
                        }
                    }
                    None => {
                        if let Some(inner_most_scope) = prog_scope.inner_most_scope(kcl_pos) {
                            return resolve_var(
                                &id.names,
                                &inner_most_scope,
                                &prog_scope.scope_map,
                            );
                        }
                    }
                }
            }
            Expr::Selector(select_expr) => {
                if select_expr.attr.contains_pos(kcl_pos) {
                    let value_def = find_def(node, &select_expr.value.get_end_pos(), prog_scope);
                    let id = select_expr.attr;
                    match value_def {
                        Some(def) => match def {
                            Definition::Object(obj) => match &obj.ty.kind {
                                kclvm_sema::ty::TypeKind::Schema(schema_type) => {
                                    return find_attr_in_schema(
                                        &schema_type,
                                        &id.node.names,
                                        &prog_scope.scope_map,
                                    )
                                }
                                _ => {}
                            },
                            Definition::Scope(_) => {}
                        },
                        None => {
                            if let Some(inner_most_scope) = prog_scope.inner_most_scope(kcl_pos) {
                                return resolve_var(
                                    &id.node.names,
                                    &inner_most_scope,
                                    &prog_scope.scope_map,
                                );
                            }
                        }
                    }
                }
            }
            Expr::Config(_) | Expr::ConfigIfEntry(_) => match parent {
                Some(schema_expr) => {
                    if let Expr::Schema(schema_expr) = schema_expr.node {
                        return find_def(node, &schema_expr.name.get_end_pos(), prog_scope);
                    }
                }
                None => {}
            },
            _ => {}
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
                Some(obj) => match &obj.borrow().kind {
                    kclvm_sema::resolver::scope::ScopeObjectKind::Module(_) => {
                        match &obj.borrow().ty.kind {
                            kclvm_sema::ty::TypeKind::Module(module_ty) => match module_ty.kind {
                                kclvm_sema::ty::ModuleKind::User => scope_map
                                    .get(&pkgpath_without_prefix!(module_ty.pkgpath))
                                    .map(|scope| Definition::Scope(scope.borrow().clone())),
                                kclvm_sema::ty::ModuleKind::System => {
                                    Some(Definition::Object(obj.borrow().clone()))
                                }
                                kclvm_sema::ty::ModuleKind::Plugin => None,
                            },
                            _ => None,
                        }
                    }
                    _ => Some(Definition::Object(obj.borrow().clone())),
                },
                None => match builtin_scope().lookup(&name) {
                    Some(obj) => {
                        let mut obj = obj.borrow().clone();
                        let doc = {
                            match &obj.ty.kind {
                                kclvm_sema::ty::TypeKind::Function(func) => Some(func.doc.clone()),
                                _ => None,
                            }
                        };
                        obj.kind = ScopeObjectKind::FunctionCall;
                        obj.doc = doc;
                        obj.start = node_names[0].get_pos();
                        obj.end = node_names[0].get_end_pos();
                        Some(Definition::Object(obj))
                    }
                    None => None,
                },
            }
        }
        _ => {
            let name = names[0].clone();
            match current_scope.lookup(&name) {
                Some(obj) => match &obj.borrow().ty.kind {
                    kclvm_sema::ty::TypeKind::Schema(schema_type) => {
                        find_attr_in_schema(schema_type, &node_names[1..], scope_map)
                    }
                    kclvm_sema::ty::TypeKind::Module(module_ty) => match module_ty.kind {
                        kclvm_sema::ty::ModuleKind::User => {
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
                        kclvm_sema::ty::ModuleKind::System => {
                            if node_names.len() == 2 {
                                let func_name_node = node_names[1].clone();
                                let func_name = func_name_node.node.clone();
                                let ty = get_system_member_function_ty(&name, &func_name);
                                match &ty.kind {
                                    kclvm_sema::ty::TypeKind::Function(func_ty) => {
                                        return Some(Definition::Object(ScopeObject {
                                            name: func_name,
                                            start: func_name_node.get_pos(),
                                            end: func_name_node.get_end_pos(),
                                            ty: ty.clone(),
                                            kind: ScopeObjectKind::FunctionCall,
                                            doc: Some(func_ty.doc.clone()),
                                        }))
                                    }
                                    _ => return None,
                                }
                            }
                            None
                        }
                        kclvm_sema::ty::ModuleKind::Plugin => None,
                    },
                    kclvm_sema::ty::TypeKind::Dict(DictType { attrs, .. }) => {
                        let key_name = names[1].clone();
                        match attrs.get(&key_name) {
                            Some(attr) => {
                                let start_pos = attr.range.0.clone();
                                for (_, scope) in scope_map {
                                    match scope.borrow().inner_most(&start_pos) {
                                        Some(inner_most_scope) => {
                                            return resolve_var(
                                                &node_names[1..],
                                                &inner_most_scope,
                                                scope_map,
                                            )
                                        }
                                        None => continue,
                                    }
                                }
                                None
                            }
                            None => None,
                        }
                    }
                    kclvm_sema::ty::TypeKind::Str => {
                        if names.len() == 2 {
                            let func_name_node = node_names[1].clone();
                            let func_name = func_name_node.node.clone();
                            match STRING_MEMBER_FUNCTIONS.get(&func_name) {
                                Some(ty) => match &ty.kind {
                                    kclvm_sema::ty::TypeKind::Function(func_ty) => {
                                        return Some(Definition::Object(ScopeObject {
                                            name: func_name,
                                            start: func_name_node.get_pos(),
                                            end: func_name_node.get_end_pos(),
                                            ty: Rc::new(ty.clone()),
                                            kind: ScopeObjectKind::FunctionCall,
                                            doc: Some(func_ty.doc.clone()),
                                        }))
                                    }
                                    // unreachable
                                    _ => {}
                                },
                                None => {}
                            }
                        }
                        None
                    }
                    _ => None,
                },
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
                return resolve_var(names, &child_scope, scope_map);
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

#[cfg(test)]
mod tests {
    use super::goto_definition;
    use crate::tests::{compare_goto_res, compile_test_file};
    use indexmap::IndexSet;
    use kclvm_error::Position as KCLPos;
    use proc_macro_crate::bench_test;
    use std::path::PathBuf;

    #[test]
    #[bench_test]
    fn goto_import_pkg_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");
        let pos = KCLPos {
            filename: file,
            line: 1,
            column: Some(10),
        };

        let res = goto_definition(&program, &pos, &prog_scope);
        let mut expeced_files = IndexSet::new();
        let path_str = path.to_str().unwrap();
        let test_files = [
            "src/test_data/goto_def_test/pkg/schema_def1.k",
            "src/test_data/goto_def_test/pkg/schema_def.k",
        ];
        expeced_files.insert(format!("{}/{}", path_str, test_files[0]));
        expeced_files.insert(format!("{}/{}", path_str, test_files[1]));

        match res.unwrap() {
            lsp_types::GotoDefinitionResponse::Array(arr) => {
                assert_eq!(expeced_files.len(), arr.len());
                for loc in arr {
                    let got_path = loc.uri.path().to_string();
                    assert!(expeced_files.contains(&got_path));
                }
            }
            _ => {
                unreachable!("test error")
            }
        }
    }

    #[test]
    #[bench_test]
    fn goto_import_file_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto import file: import .pkg.schema_def
        let pos = KCLPos {
            filename: file,
            line: 2,
            column: Some(10),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        match res.unwrap() {
            lsp_types::GotoDefinitionResponse::Scalar(loc) => {
                let got_path = loc.uri.path();
                assert_eq!(got_path, expected_path.to_str().unwrap())
            }
            _ => {
                unreachable!("test error")
            }
        }
    }

    #[test]
    #[bench_test]
    fn goto_pkg_prefix_def_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        // test goto pkg prefix def: p = pkg.Person {  <- pkg
        let pos = KCLPos {
            filename: file,
            line: 4,
            column: Some(7),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        let mut expeced_files = IndexSet::new();
        let path_str = path.to_str().unwrap();
        let test_files = [
            "src/test_data/goto_def_test/pkg/schema_def1.k",
            "src/test_data/goto_def_test/pkg/schema_def.k",
        ];
        expeced_files.insert(format!("{}/{}", path_str, test_files[0]));
        expeced_files.insert(format!("{}/{}", path_str, test_files[1]));

        match res.unwrap() {
            lsp_types::GotoDefinitionResponse::Array(arr) => {
                assert_eq!(expeced_files.len(), arr.len());
                for loc in arr {
                    let got_path = loc.uri.path().to_string();
                    assert!(expeced_files.contains(&got_path));
                }
            }
            _ => {
                unreachable!("test error")
            }
        }
    }

    #[test]
    #[bench_test]
    fn goto_schema_def_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto schema definition: p = pkg.Person <- Person
        let pos = KCLPos {
            filename: file,
            line: 4,
            column: Some(11),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
        );
    }

    #[test]
    #[bench_test]
    fn goto_var_def_in_config_and_config_if_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 67,
            column: Some(36),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 65, 11, 65, 14));

        let pos = KCLPos {
            filename: file.clone(),
            line: 67,
            column: Some(44),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 65, 16, 65, 21));

        let pos = KCLPos {
            filename: file.clone(),
            line: 64,
            column: Some(11),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 69, 6, 69, 10));

        let pos = KCLPos {
            filename: file.clone(),
            line: 67,
            column: Some(10),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 69, 6, 69, 10));
    }

    #[test]
    #[bench_test]
    fn goto_var_def_in_dict_comp_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 77,
            column: Some(68),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 76, 143, 76, 145));

        let pos = KCLPos {
            filename: file.clone(),
            line: 77,
            column: Some(61),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 76, 143, 76, 145));
    }

    #[test]
    fn goto_dict_key_def_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 26,
            column: Some(24),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 8, 4, 8, 8),
        );

        let pos = KCLPos {
            filename: file.clone(),
            line: 59,
            column: Some(28),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 18, 4, 18, 8));
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_def_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto schema attr definition: name: "alice"
        let pos = KCLPos {
            filename: file,
            line: 5,
            column: Some(7),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 4, 4, 4, 8),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_def_test1() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/goto_def.k");

        // test goto schema attr definition, goto name in: s = p2.n.name
        let pos = KCLPos {
            filename: file,
            line: 30,
            column: Some(12),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 18, 4, 18, 8),
        );
    }

    #[test]
    #[bench_test]
    fn test_goto_identifier_names() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/goto_def.k");

        // test goto p2 in: s = p2.n.name
        let pos = KCLPos {
            filename: file.clone(),
            line: 30,
            column: Some(5),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 23, 0, 23, 2),
        );

        // test goto n in: s = p2.n.name
        let pos = KCLPos {
            filename: file.clone(),
            line: 30,
            column: Some(8),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 21, 1, 21, 2),
        );

        // test goto name in: s = p2.n.name
        let pos = KCLPos {
            filename: file,
            line: 30,
            column: Some(12),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 18, 4, 18, 8),
        );
    }

    #[test]
    #[bench_test]
    fn goto_identifier_def_test() {
        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        // test goto identifier definition: p1 = p
        let pos = KCLPos {
            filename: file.to_string(),
            line: 9,
            column: Some(6),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 3, 0, 3, 1));
    }

    #[test]
    #[bench_test]
    fn goto_assign_type_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto schema attr definition: name: "alice"
        let pos = KCLPos {
            filename: file.clone(),
            line: 38,
            column: Some(17),
        };

        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 33, 0, 37, 0));
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test() {
        // test goto schema attr type definition: p1: pkg.Person
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 12,
            column: Some(15),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test1() {
        // test goto schema attr type definition: p2: [pkg.Person]
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 13,
            column: Some(15),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test3() {
        // test goto schema attr type definition: p3: {str: pkg.Person}
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 14,
            column: Some(22),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test4() {
        // test goto schema attr type definition(Person): p4: pkg.Person | pkg.Person1
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 15,
            column: Some(17),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test5() {
        // test goto schema attr type definition(Person1): p4: pkg.Person | pkg.Person1
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def1.k");

        let pos = KCLPos {
            filename: file,
            line: 15,
            column: Some(28),
        };
        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 0, 2, 13),
        );
    }

    #[test]
    #[bench_test]
    fn goto_local_var_def_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto local var def
        let pos = KCLPos {
            filename: file.clone(),
            line: 47,
            column: Some(11),
        };

        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 43, 4, 43, 9));

        let pos = KCLPos {
            filename: file.clone(),
            line: 49,
            column: Some(11),
        };

        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 43, 4, 43, 9));

        let pos = KCLPos {
            filename: file.clone(),
            line: 51,
            column: Some(11),
        };

        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 43, 4, 43, 9));
    }

    #[test]
    #[bench_test]
    fn complex_select_goto_def() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, prog_scope, _) =
            compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 52,
            column: Some(22),
        };

        let res = goto_definition(&program, &pos, &prog_scope);
        compare_goto_res(res, (&file, 43, 4, 43, 9));
    }
}
