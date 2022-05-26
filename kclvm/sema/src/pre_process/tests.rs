use super::*;
use indexmap::IndexMap;
use kclvm_parser::parse_file;

#[test]
fn test_fix_qualified_identifier() {
    let mut module =
        parse_file("./src/pre_process/test_data/qualified_identifier.k", None).unwrap();
    fix_qualified_identifier(&mut module, &mut IndexMap::default());
    if let ast::Stmt::Assign(assign_stmt) = &module.body[1].node {
        if let ast::Expr::Identifier(identifier) = &assign_stmt.value.node {
            assert_eq!(identifier.pkgpath, "pkg")
        } else {
            panic!("invalid assign statement value")
        }
    } else {
        panic!("invalid assign statement")
    }
}

#[test]
fn test_fix_raw_identifier_prefix() {
    let mut module = parse_file("./src/pre_process/test_data/raw_identifier.k", None).unwrap();
    if let ast::Stmt::Assign(assign_stmt) = &module.body[0].node {
        assert_eq!(assign_stmt.targets[0].node.names[0], "$schema")
    } else {
        panic!("invalid assign statement")
    }
    fix_raw_identifier_prefix(&mut module);
    if let ast::Stmt::Assign(assign_stmt) = &module.body[0].node {
        assert_eq!(assign_stmt.targets[0].node.names[0], "schema")
    } else {
        panic!("invalid assign statement")
    }
}
