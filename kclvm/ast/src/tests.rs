use crate::ast;
use crate::walker::MutSelfMutWalker;

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
                    names: vec![String::from("a")],
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
            type_annotation: None,
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
                    names: vec![String::from("a")],
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
                    op: ast::BinOrCmpOp::Bin(ast::BinOp::Add),
                    left: Box::new(ast::Node::new(
                        ast::Expr::Identifier(ast::Identifier {
                            names: vec![String::from("a")],
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
                            names: vec![String::from("a")],
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
            type_annotation: None,
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
            if identifier.names[0] == "a" {
                let id_mut = identifier.names.get_mut(0).unwrap();
                *id_mut = "x".to_string();
            }
        }
    }
    let mut assign_stmt = get_dummy_assign_ast();
    VarMutSelfMutWalker {}.walk_assign_stmt(&mut assign_stmt.node);
    assert_eq!(assign_stmt.node.targets[0].node.names[0], "x")
}
