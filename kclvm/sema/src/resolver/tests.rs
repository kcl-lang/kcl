use super::Options;
use super::Resolver;
use crate::builtin::BUILTIN_FUNCTION_NAMES;
use crate::pre_process::pre_process_program;
use crate::resolver::resolve_program;
use crate::resolver::scope::*;
use crate::ty::Type;
use compiler_base_session::Session;
use kclvm_ast::ast;
use kclvm_error::*;
use kclvm_parser::{load_program, parse_program};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

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
    let sess = Arc::new(Session::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/pkg_init_in_schema.k"],
        None,
    )
    .unwrap();
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
    let cases = &[
        "./src/resolver/test_fail_data/attr.k",
        "./src/resolver/test_fail_data/cannot_find_module.k",
        "./src/resolver/test_fail_data/config_expr.k",
        "./src/resolver/test_fail_data/module_optional_select.k",
        "./src/resolver/test_fail_data/unmatched_args.k",
    ];
    for case in cases {
        let mut program = parse_program(case).unwrap();
        let scope = resolve_program(&mut program);
        assert!(scope.handler.diagnostics.len() > 0);
    }
}

#[test]
fn test_resolve_program_mismatch_type_fail() {
    let mut program = parse_program("./src/resolver/test_fail_data/config_expr.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 1);
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(diag.code, Some(DiagnosticId::Error(ErrorKind::TypeError)));
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(
        diag.messages[0].message,
        "expect int, got {str(key):int(1)}"
    );
}

#[test]
fn test_resolve_program_cycle_reference_fail() {
    let sess = Arc::new(Session::default());
    let mut program = load_program(
        sess.clone(),
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
        "Module 'file2' imported but unused",
        "Module 'file1' imported but unused",
    ];
    assert_eq!(scope.handler.diagnostics.len(), err_messages.len());
    for (diag, msg) in scope.handler.diagnostics.iter().zip(err_messages.iter()) {
        assert_eq!(diag.messages[0].message, msg.to_string(),);
    }
}

#[test]
fn test_record_used_module() {
    let sess = Arc::new(Session::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/record_used_module.k"],
        None,
    )
    .unwrap();
    let scope = resolve_program(&mut program);
    let main_scope = scope
        .scope_map
        .get(kclvm_runtime::MAIN_PKG_PATH)
        .unwrap()
        .borrow_mut()
        .clone();
    for (_, obj) in main_scope.elems {
        let obj = obj.borrow_mut().clone();
        if obj.kind == ScopeObjectKind::Module {
            if obj.name == "math" {
                assert!(!obj.used);
            } else {
                assert!(obj.used);
            }
        }
    }
}

#[test]
fn test_cannot_find_module() {
    let sess = Arc::new(Session::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_fail_data/cannot_find_module.k"],
        None,
    )
    .unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics[0].messages[0].pos.column, None);
}

#[test]
fn test_resolve_program_illegal_attr_fail() {
    let mut program = parse_program("./src/resolver/test_fail_data/attr.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 2);
    let expect_err_msg = "A attribute must be string type, got 'Data'";
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::IllegalAttributeError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].pos.line, 4);
    assert_eq!(diag.messages[0].message, expect_err_msg,);
    let diag = &scope.handler.diagnostics[1];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::IllegalAttributeError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].message, expect_err_msg,);
    assert_eq!(diag.messages[0].pos.line, 5);
}

#[test]
fn test_resolve_program_unmatched_args_fail() {
    let mut program = parse_program("./src/resolver/test_fail_data/unmatched_args.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 2);
    let expect_err_msg = "\"Foo\" takes 1 positional argument but 3 were given";
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].pos.line, 6);
    assert_eq!(diag.messages[0].message, expect_err_msg);

    let expect_err_msg = "\"f\" takes 1 positional argument but 2 were given";
    let diag = &scope.handler.diagnostics[1];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].pos.line, 7);
    assert_eq!(diag.messages[0].message, expect_err_msg);
}

#[test]
fn test_resolve_program_module_optional_select_fail() {
    let mut program =
        parse_program("./src/resolver/test_fail_data/module_optional_select.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 2);
    let expect_err_msg =
        "For the module type, the use of '?.log' is unnecessary and it can be modified as '.log'";
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].pos.line, 3);
    assert_eq!(diag.messages[0].message, expect_err_msg);

    let expect_err_msg = "Module 'math' imported but unused";
    let diag = &scope.handler.diagnostics[1];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Warning(WarningKind::UnusedImportWarning))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].pos.line, 1);
    assert_eq!(diag.messages[0].message, expect_err_msg);
}

#[test]
fn test_lint() {
    let sess = Arc::new(Session::default());
    let mut program =
        load_program(sess.clone(), &["./src/resolver/test_data/lint.k"], None).unwrap();
    pre_process_program(&mut program);
    let mut resolver = Resolver::new(
        &program,
        Options {
            raise_err: true,
            config_auto_fix: false,
            lint_check: true,
        },
    );
    resolver.resolve_import();
    resolver.check_and_lint(kclvm_ast::MAIN_PKG);

    let root = &program.root.clone();
    let filename = Path::new(&root.clone())
        .join("lint.k")
        .display()
        .to_string();
    let mut handler = Handler::default();
    handler.add_warning(
        WarningKind::ImportPositionWarning,
        &[Message {
            pos: Position {
                filename: filename.clone(),
                line: 10,
                column: None,
            },
            style: Style::Line,
            message: format!("Importstmt should be placed at the top of the module"),
            note: Some("Consider moving tihs statement to the top of the file".to_string()),
        }],
    );
    handler.add_warning(
        WarningKind::ReimportWarning,
        &[Message {
            pos: Position {
                filename: filename.clone(),
                line: 2,
                column: None,
            },
            style: Style::Line,
            message: format!("Module 'a' is reimported multiple times"),
            note: Some("Consider removing this statement".to_string()),
        }],
    );
    handler.add_warning(
        WarningKind::UnusedImportWarning,
        &[Message {
            pos: Position {
                filename,
                line: 1,
                column: None,
            },
            style: Style::Line,
            message: format!("Module 'import_test.a' imported but unused"),
            note: Some("Consider removing this statement".to_string()),
        }],
    );
    for (d1, d2) in resolver
        .linter
        .handler
        .diagnostics
        .iter()
        .zip(handler.diagnostics.iter())
    {
        assert_eq!(d1, d2);
    }
}
