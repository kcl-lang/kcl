use crate::builtin::BUILTIN_FUNCTION_NAMES;
use crate::resolver::resolve_program;
use crate::resolver::scope::*;
use crate::ty::Type;
use kclvm_ast::ast;
use kclvm_error::*;
use kclvm_parser::{load_program, parse_program};
use std::rc::Rc;

#[test]
fn test_scope() {
    let mut scope = builtin_scope();
    for name in BUILTIN_FUNCTION_NAMES {
        let obj = scope.lookup(name).unwrap();
        let obj_ref = obj.borrow_mut();
        assert!(obj_ref.ty.is_func());
    }
    for name in BUILTIN_FUNCTION_NAMES {
        scope.set_ty(name, Rc::new(Type::ANY));
    }
    for name in BUILTIN_FUNCTION_NAMES {
        let obj = scope.lookup(name).unwrap();
        let obj_ref = obj.borrow_mut();
        assert!(obj_ref.ty.is_any());
    }
}

#[test]
fn test_resolve_program() {
    let mut program = parse_program("./src/resolver/test_data/assign.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.pkgpaths(), vec!["__main__".to_string()]);
    let main_scope = scope.main_scope().unwrap();
    let main_scope = main_scope.borrow_mut();
    assert!(main_scope.lookup("a").is_some());
    assert!(main_scope.lookup("b").is_some());
    assert!(main_scope.lookup("print").is_none());
}

#[test]
fn test_pkg_init_in_schema_resolve() {
    let mut program =
        load_program(&["./src/resolver/test_data/pkg_init_in_schema.k"], None).unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(
        scope.pkgpaths(),
        vec!["__main__".to_string(), "pkg".to_string()]
    );
    let module = &program.pkgs["pkg"][0];
    if let ast::Stmt::Schema(schema) = &module.body[1].node {
        if let ast::Stmt::SchemaAttr(attr) = &schema.body[0].node {
            let value = attr.value.as_ref().unwrap();
            if let ast::Expr::Schema(schema_expr) = &value.node {
                assert_eq!(schema_expr.name.node.names, vec!["Name".to_string()]);
            } else {
                panic!("test failed, expect schema expr, got {:?}", value)
            }
        } else {
            panic!(
                "test failed, expect schema attribute, got {:?}",
                schema.body[0]
            )
        }
    } else {
        panic!(
            "test failed, expect schema statement, got {:?}",
            module.body[1]
        )
    }
}

#[test]
fn test_resolve_program_fail() {
    let mut program = parse_program("./src/resolver/test_fail_data/config_expr.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.diagnostics.len(), 1);
    let diag = &scope.diagnostics[0];
    assert_eq!(diag.code, Some(DiagnosticId::Error(ErrorKind::TypeError)));
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].message, "expect int, got {str:int(1)}");
}

#[test]
fn test_resolve_program_cycle_reference_fail() {
    let mut program = load_program(
        &["./src/resolver/test_fail_data/cycle_reference/file1.k"],
        None,
    )
    .unwrap();
    let scope = resolve_program(&mut program);
    let err_messages = [
        "There is a circular import reference between module file1 and file2",
        "There is a circular reference between schema SchemaBase and SchemaSub",
        "There is a circular reference between schema SchemaSub and SchemaBase",
        "There is a circular reference between rule RuleBase and RuleSub",
        "There is a circular reference between rule RuleSub and RuleBase",
    ];
    assert_eq!(scope.diagnostics.len(), err_messages.len());
    for (diag, msg) in scope.diagnostics.iter().zip(err_messages.iter()) {
        assert_eq!(diag.messages[0].message, msg.to_string(),);
    }
}

#[test]
fn test_record_used_module() {
    let mut program =
        load_program(&["./src/resolver/test_data/record_used_module.k"], None).unwrap();
    let scope = resolve_program(&mut program);
    let main_scope = scope
        .scope_map
        .get(kclvm::MAIN_PKG_PATH)
        .unwrap()
        .borrow_mut()
        .clone();
    for (_, obj) in main_scope.elems {
        let obj = obj.borrow_mut().clone();
        if obj.kind == ScopeObjectKind::Module {
            if obj.name == "math" {
                assert_eq!(obj.used, false);
            } else {
                assert_eq!(obj.used, true);
            }
        }
    }
}

#[test]
fn test_cannot_find_module() {
    let mut program = load_program(
        &["./src/resolver/test_fail_data/cannot_find_module.k"],
        None,
    )
    .unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.diagnostics[0].messages[0].pos.column, None);
}

#[test]
fn test_resolve_program_illegal_attr_fail() {
    let mut program = parse_program("./src/resolver/test_fail_data/attr.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.diagnostics.len(), 2);
    let expect_err_msg = "A attribute must be string type, got 'Data'";
    let diag = &scope.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::IllegalAttributeError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].pos.line, 4);
    assert_eq!(diag.messages[0].message, expect_err_msg,);
    let diag = &scope.diagnostics[1];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::IllegalAttributeError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].message, expect_err_msg,);
    assert_eq!(diag.messages[0].pos.line, 5);
}
