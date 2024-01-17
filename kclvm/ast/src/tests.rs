use crate::node_ref;
use crate::walker::MutSelfMutWalker;
use crate::{ast, ast::*};

/// Construct an AssignStmt node with assign_value as value
fn build_assign_node(attr_name: &str, assign_value: NodeRef<Expr>) -> NodeRef<Stmt> {
    let iden = node_ref!(Identifier {
        names: vec![Node::dummy_node(attr_name.to_string())],
        pkgpath: String::new(),
        ctx: ExprContext::Store
    });

    node_ref!(Stmt::Assign(AssignStmt {
        value: assign_value,
        targets: vec![iden],
        ty: None
    }))
}

fn get_dummy_assign_ast() -> ast::Node<ast::AssignStmt> {
    let filename = "main.k";
    let line = 1;
    let column = 1;
    let end_line = 1;
    let end_column = 2;
    ast::Node::new(
        ast::AssignStmt {
            targets: vec![Box::new(ast::Node::new(
                ast::Identifier {
                    names: vec![Node::dummy_node(String::from("a"))],
                    pkgpath: String::from(filename),
                    ctx: ast::ExprContext::Load,
                },
                String::from(filename),
                line,
                column,
                end_line,
                end_column,
            ))],
            value: Box::new(ast::Node::new(
                ast::Expr::StringLit(ast::StringLit {
                    is_long_string: false,
                    raw_value: String::from("s"),
                    value: String::from("s"),
                }),
                String::from(filename),
                line,
                column,
                end_line,
                end_column,
            )),
            ty: None,
        },
        String::from(filename),
        line,
        column,
        end_line,
        end_column,
    )
}

fn get_dummy_assign_binary_ast() -> ast::Node<ast::AssignStmt> {
    let filename = "main.k";
    let line = 1;
    let column = 1;
    let end_line = 1;
    let end_column = 2;
    ast::Node::new(
        ast::AssignStmt {
            targets: vec![Box::new(ast::Node::new(
                ast::Identifier {
                    names: vec![Node::dummy_node(String::from("a"))],
                    pkgpath: String::from(filename),
                    ctx: ast::ExprContext::Load,
                },
                String::from(filename),
                line,
                column,
                end_line,
                end_column,
            ))],
            value: Box::new(ast::Node::new(
                ast::Expr::Binary(ast::BinaryExpr {
                    op: ast::BinOp::Add,
                    left: Box::new(ast::Node::new(
                        ast::Expr::Identifier(ast::Identifier {
                            names: vec![Node::dummy_node(String::from("a"))],
                            pkgpath: String::from(filename),
                            ctx: ast::ExprContext::Load,
                        }),
                        String::from(filename),
                        line,
                        column,
                        end_line,
                        end_column,
                    )),
                    right: Box::new(ast::Node::new(
                        ast::Expr::Identifier(ast::Identifier {
                            names: vec![Node::dummy_node(String::from("a"))],
                            pkgpath: String::from(filename),
                            ctx: ast::ExprContext::Load,
                        }),
                        String::from(filename),
                        line,
                        column,
                        end_line,
                        end_column,
                    )),
                }),
                String::from(filename),
                line,
                column,
                end_line,
                end_column,
            )),
            ty: None,
        },
        String::from(filename),
        line,
        column,
        end_line,
        end_column,
    )
}

#[test]
fn test_ast_print_assign() {
    let assign_stmt = get_dummy_assign_ast();
    println!("{:?}", assign_stmt);
    let json_str = serde_json::to_string(&assign_stmt).unwrap();
    println!("{:?}", json_str);
}

#[test]
fn test_ast_print_assign_binary() {
    let assign_stmt = get_dummy_assign_binary_ast();
    println!("{:?}", assign_stmt);
    let json_str = serde_json::to_string(&assign_stmt).unwrap();
    println!("{:?}", json_str);
}

#[test]
fn test_mut_walker() {
    pub struct VarMutSelfMutWalker;
    impl<'ctx> MutSelfMutWalker<'ctx> for VarMutSelfMutWalker {
        fn walk_identifier(&mut self, identifier: &'ctx mut ast::Identifier) {
            if identifier.names[0].node == "a" {
                let id_mut = identifier.names.get_mut(0).unwrap();
                id_mut.node = "x".to_string();
            }
        }
    }
    let mut assign_stmt = get_dummy_assign_ast();
    VarMutSelfMutWalker {}.walk_assign_stmt(&mut assign_stmt.node);
    assert_eq!(assign_stmt.node.targets[0].node.names[0].node, "x")
}

#[test]
fn test_try_from_for_stringlit() {
    let str_lit = ast::StringLit::try_from("test_str".to_string()).unwrap();
    let json_str = serde_json::to_string(&str_lit).unwrap();

    let str_expected =
        r#"{"is_long_string":false,"raw_value":"\"test_str\"","value":"test_str"}"#.to_string();
    assert_eq!(str_expected, json_str);
}

#[test]
fn test_try_from_for_nameconstant() {
    let name_cons = ast::NameConstant::try_from(true).unwrap();
    let json_str = serde_json::to_string(&name_cons).unwrap();
    assert_eq!("\"True\"", json_str);

    let name_cons = ast::NameConstant::try_from(false).unwrap();
    let json_str = serde_json::to_string(&name_cons).unwrap();
    assert_eq!("\"False\"", json_str);
}

#[test]
fn test_filter_schema_with_no_schema() {
    let ast_mod = Module {
        filename: "".to_string(),
        pkg: "".to_string(),
        doc: Some(node_ref!("".to_string())),
        name: "".to_string(),
        body: vec![],
        comments: vec![],
    };
    let schema_stmts = ast_mod.filter_schema_stmt_from_module();
    assert_eq!(schema_stmts.len(), 0);
}

#[test]
fn test_filter_schema_with_one_schema() {
    let mut ast_mod = Module {
        filename: "".to_string(),
        pkg: "".to_string(),
        doc: Some(node_ref!("".to_string())),
        name: "".to_string(),
        body: vec![],
        comments: vec![],
    };
    let mut gen_schema_stmts = gen_schema_stmt(1);
    ast_mod.body.append(&mut gen_schema_stmts);
    let schema_stmts = ast_mod.filter_schema_stmt_from_module();
    assert_eq!(schema_stmts.len(), 1);
    assert_eq!(schema_stmts[0].node.name.node, "schema_stmt_0".to_string());
}

#[test]
fn test_filter_schema_with_mult_schema() {
    let mut ast_mod = Module {
        filename: "".to_string(),
        pkg: "".to_string(),
        doc: Some(node_ref!("".to_string())),
        name: "".to_string(),
        body: vec![],
        comments: vec![],
    };
    let mut gen_schema_stmts = gen_schema_stmt(10);
    ast_mod.body.append(&mut gen_schema_stmts);
    let schema_stmts = ast_mod.filter_schema_stmt_from_module();
    assert_eq!(schema_stmts.len(), 10);
    for i in 0..10 {
        assert_eq!(
            schema_stmts[i].node.name.node,
            "schema_stmt_".to_string() + &i.to_string()
        )
    }
}

#[test]
fn test_build_assign_stmt() {
    let test_expr = node_ref!(ast::Expr::Identifier(Identifier {
        names: vec![
            Node::dummy_node("name1".to_string()),
            Node::dummy_node("name2".to_string())
        ],
        pkgpath: "test".to_string(),
        ctx: ast::ExprContext::Load
    }));
    let assgin_stmt = build_assign_node("test_attr_name", test_expr);

    if let ast::Stmt::Assign(ref assign) = assgin_stmt.node {
        if let ast::Expr::Identifier(ref iden) = &assign.value.node {
            assert_eq!(iden.names.len(), 2);
            assert_eq!(iden.names[0].node, "name1".to_string());
            assert_eq!(iden.names[1].node, "name2".to_string());
            assert_eq!(iden.pkgpath, "test".to_string());
            match iden.ctx {
                ast::ExprContext::Load => {}
                _ => {
                    assert!(false);
                }
            }
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
}

fn gen_schema_stmt(count: i32) -> Vec<NodeRef<ast::Stmt>> {
    let mut schema_stmts = Vec::new();
    for c in 0..count {
        schema_stmts.push(node_ref!(ast::Stmt::Schema(SchemaStmt {
            doc: Some(node_ref!("".to_string())),
            name: node_ref!("schema_stmt_".to_string() + &c.to_string()),
            parent_name: None,
            for_host_name: None,
            is_mixin: false,
            is_protocol: false,
            args: None,
            mixins: vec![],
            body: vec![],
            decorators: vec![],
            checks: vec![],
            index_signature: None
        })))
    }
    schema_stmts
}
