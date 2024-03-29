use super::Options;
use super::Resolver;
use crate::builtin::BUILTIN_FUNCTION_NAMES;
use crate::pre_process::pre_process_program;
use crate::resolver::resolve_program;
use crate::resolver::resolve_program_with_opts;
use crate::resolver::scope::*;
use crate::ty::{Type, TypeKind};
use anyhow::Result;
use kclvm_ast::ast;
use kclvm_ast::pos::ContainsPos;
use kclvm_error::*;
use kclvm_parser::load_program;
use kclvm_parser::parse_file_force_errors;
use kclvm_parser::LoadProgramOptions;
use kclvm_parser::ParseSession;
use kclvm_utils::path::PathPrefix;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

pub fn parse_program(filename: &str) -> Result<ast::Program> {
    let abspath = std::fs::canonicalize(std::path::PathBuf::from(filename)).unwrap();

    let mut prog = ast::Program {
        root: abspath.parent().unwrap().adjust_canonicalization(),
        pkgs: HashMap::new(),
    };

    let mut module = parse_file_force_errors(abspath.to_str().unwrap(), None)?;
    module.filename = filename.to_string();
    module.pkg = kclvm_ast::MAIN_PKG.to_string();
    module.name = kclvm_ast::MAIN_PKG.to_string();

    prog.pkgs
        .insert(kclvm_ast::MAIN_PKG.to_string(), vec![module]);

    Ok(prog)
}

#[test]
fn test_scope() {
    let mut scope = builtin_scope();
    for name in BUILTIN_FUNCTION_NAMES {
        let obj = scope.lookup(name).unwrap();
        let obj_ref = obj.borrow_mut();
        assert!(obj_ref.ty.is_func());
    }
    for name in BUILTIN_FUNCTION_NAMES {
        scope.set_ty(name, Arc::new(Type::ANY));
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
fn test_resolve_program_with_cache() {
    let mut program = parse_program("./src/resolver/test_data/assign.k").unwrap();

    let scope = resolve_program_with_opts(
        &mut program,
        Options {
            merge_program: false,
            type_erasure: false,
            ..Default::default()
        },
        None,
    );
    let cached_scope = Arc::new(Mutex::new(CachedScope::new(&scope, &program)));
    let scope = resolve_program_with_opts(
        &mut program,
        Options {
            merge_program: false,
            type_erasure: false,
            ..Default::default()
        },
        Some(cached_scope),
    );
    assert_eq!(scope.pkgpaths(), vec!["__main__".to_string()]);
    let main_scope = scope.main_scope().unwrap();
    let main_scope = main_scope.borrow_mut();
    assert!(main_scope.lookup("a").is_some());
    assert!(main_scope.lookup("b").is_some());
    assert!(main_scope.lookup("print").is_none());
}

#[test]
fn test_pkg_init_in_schema_resolve() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/pkg_init_in_schema.k"],
        None,
        None,
    )
    .unwrap()
    .program;
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
                assert_eq!(schema_expr.name.node.get_names(), vec!["Name".to_string()]);
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
    let work_dir = "./src/resolver/test_fail_data/";
    let cases = &[
        "attr.k",
        "cannot_find_member_0.k",
        "cannot_find_member_1.k",
        "cannot_find_module.k",
        "comp_clause_error_0.k",
        "comp_clause_error_1.k",
        "comp_clause_error_2.k",
        "comp_clause_error_3.k",
        "comp_clause_error_4.k",
        "config_expr.k",
        "invalid_mixin_0.k",
        "module_optional_select.k",
        "mutable_error_0.k",
        "mutable_error_1.k",
        "unique_key_error_0.k",
        "unique_key_error_1.k",
        "unmatched_index_sign_default_value.k",
        "unmatched_args.k",
        "unmatched_nest_schema_attr_0.k",
        "unmatched_nest_schema_attr_1.k",
        "unmatched_nest_schema_attr_2.k",
        "unmatched_nest_schema_attr_3.k",
        "unmatched_schema_attr_0.k",
        "unmatched_schema_attr_1.k",
        "unmatched_schema_attr_2.k",
        "unmatched_schema_attr_3.k",
    ];
    for case in cases {
        let path = Path::new(work_dir).join(case);
        let mut program = parse_program(&path.to_string_lossy()).unwrap();
        let scope = resolve_program(&mut program);
        assert!(scope.handler.diagnostics.len() > 0, "{}", case);
    }
}

#[test]
fn test_resolve_program_redefine() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_fail_data/redefine_import/main.k"],
        None,
        None,
    )
    .unwrap()
    .program;

    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 2);
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(
        diag.messages[0].message,
        "the name 's' is defined multiple times, 's' must be defined only once"
    );
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
        "expected int, got {str(key):int(1)}"
    );
}

#[test]
fn test_resolve_program_cycle_reference_fail() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_fail_data/cycle_reference/file1.k"],
        None,
        None,
    )
    .unwrap()
    .program;
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
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/record_used_module.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    let main_scope = scope
        .scope_map
        .get(kclvm_runtime::MAIN_PKG_PATH)
        .unwrap()
        .borrow_mut()
        .clone();
    for (_, obj) in main_scope.elems {
        let obj = obj.borrow_mut().clone();
        if let ScopeObjectKind::Module(m) = obj.kind {
            for (_, used) in m.import_stmts {
                if obj.name == "math" {
                    assert!(!used);
                } else {
                    assert!(used);
                }
            }
        }
    }
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
    assert_eq!(diag.messages[0].range.0.line, 4);
    assert_eq!(diag.messages[0].message, expect_err_msg,);
    let diag = &scope.handler.diagnostics[1];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::IllegalAttributeError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].message, expect_err_msg,);
    assert_eq!(diag.messages[0].range.0.line, 5);
}

#[test]
fn test_resolve_program_unmatched_args_fail() {
    let mut program = parse_program("./src/resolver/test_fail_data/unmatched_args.k").unwrap();
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 3);
    let expect_err_msg = "\"Foo\" takes 1 positional argument but 3 were given";
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].range.0.line, 6);
    assert_eq!(diag.messages[0].message, expect_err_msg);

    let expect_err_msg = "\"f\" takes 1 positional argument but 2 were given";
    let diag = &scope.handler.diagnostics[1];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].range.0.line, 7);
    assert_eq!(diag.messages[0].message, expect_err_msg);

    let expect_err_msg = "\"Foo2\" takes 2 positional arguments but 3 were given";
    let diag = &scope.handler.diagnostics[2];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].range.0.line, 12);
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
    assert_eq!(diag.messages[0].range.0.line, 3);
    assert_eq!(diag.messages[0].message, expect_err_msg);

    let expect_err_msg = "Module 'math' imported but unused";
    let diag = &scope.handler.diagnostics[1];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Warning(WarningKind::UnusedImportWarning))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(diag.messages[0].range.0.line, 1);
    assert_eq!(diag.messages[0].message, expect_err_msg);
}

#[test]
fn test_lint() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/lint.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let opts = Options::default();
    pre_process_program(&mut program, &opts);
    let mut resolver = Resolver::new(&program, opts);
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
            range: (
                Position {
                    filename: filename.clone(),
                    line: 11,
                    column: Some(0),
                },
                Position {
                    filename: filename.clone(),
                    line: 11,
                    column: Some(20),
                },
            ),
            style: Style::Line,
            message: format!("Importstmt should be placed at the top of the module"),
            note: Some("Consider moving tihs statement to the top of the file".to_string()),
            suggested_replacement: None,
        }],
    );
    handler.add_warning(
        WarningKind::ReimportWarning,
        &[Message {
            range: (
                Position {
                    filename: filename.clone(),
                    line: 2,
                    column: Some(0),
                },
                Position {
                    filename: filename.clone(),
                    line: 2,
                    column: Some(20),
                },
            ),
            style: Style::Line,
            message: format!("Module 'a' is reimported multiple times"),
            note: Some("Consider removing this statement".to_string()),
            suggested_replacement: None,
        }],
    );
    handler.add_warning(
        WarningKind::UnusedImportWarning,
        &[Message {
            range: (
                Position {
                    filename: filename.clone(),
                    line: 1,
                    column: Some(0),
                },
                Position {
                    filename: filename.clone(),
                    line: 1,
                    column: Some(20),
                },
            ),
            style: Style::Line,
            message: format!("Module 'import_test.a' imported but unused"),
            note: Some("Consider removing this statement".to_string()),
            suggested_replacement: None,
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

#[test]
fn test_resolve_schema_doc() {
    let mut program = parse_program("./src/resolver/test_data/doc.k").unwrap();
    let scope = resolve_program(&mut program);
    let main_scope = scope
        .scope_map
        .get(kclvm_runtime::MAIN_PKG_PATH)
        .unwrap()
        .borrow_mut()
        .clone();

    let schema_scope_obj = &main_scope.elems[0].borrow().clone();
    let schema_summary = match &schema_scope_obj.ty.kind {
        TypeKind::Schema(schema_ty) => schema_ty.doc.clone(),
        _ => "".to_string(),
    };

    let schema_scope = &main_scope.children[0];
    let attrs_scope = &schema_scope.borrow().elems;
    assert_eq!("Server is the common user interface for long-running services adopting the best practice of Kubernetes.".to_string(), schema_summary);
    assert_eq!(
        Some(
            "Use this attribute to specify which kind of long-running service you want.
Valid values: Deployment, CafeDeployment.
See also: kusion_models/core/v1/workload_metadata.k."
                .to_string()
        ),
        attrs_scope.get("workloadType").unwrap().borrow().doc
    );
    assert_eq!(
        Some(
            "A Server-level attribute.
The name of the long-running service.
See also: kusion_models/core/v1/metadata.k."
                .to_string()
        ),
        attrs_scope.get("name").unwrap().borrow().doc
    );
    assert_eq!(
        Some(
            "A Server-level attribute.
The labels of the long-running service.
See also: kusion_models/core/v1/metadata.k."
                .to_string()
        ),
        attrs_scope.get("labels").unwrap().borrow().doc
    );
}

#[test]
fn test_pkg_scope() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/pkg_scope.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);

    assert_eq!(scope.scope_map.len(), 2);
    let main_scope = scope
        .scope_map
        .get(kclvm_runtime::MAIN_PKG_PATH)
        .unwrap()
        .borrow_mut()
        .clone();
    let pkg_scope = scope.scope_map.get("pkg").unwrap().borrow_mut().clone();

    let root = &program.root.clone();
    let filename = Path::new(&root.clone())
        .join("pkg_scope.k")
        .display()
        .to_string();

    let pos = Position {
        filename: filename.clone(),
        line: 2,
        column: Some(0),
    };

    assert!(main_scope.contains_pos(&pos));

    let filename = Path::new(&root.clone())
        .join("pkg")
        .join("pkg.k")
        .display()
        .to_string();

    let pos = Position {
        filename: filename.clone(),
        line: 4,
        column: Some(0),
    };

    assert!(pkg_scope.contains_pos(&pos));
}

#[test]
fn test_system_package() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/system_package.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    let main_scope = scope
        .scope_map
        .get(kclvm_runtime::MAIN_PKG_PATH)
        .unwrap()
        .borrow_mut()
        .clone();

    assert!(main_scope.lookup("base64").unwrap().borrow().ty.is_module());
    assert!(main_scope
        .lookup("base64_encode")
        .unwrap()
        .borrow()
        .ty
        .is_func());
    assert!(main_scope
        .lookup("base64_decode")
        .unwrap()
        .borrow()
        .ty
        .is_func());
}

#[test]
fn test_resolve_program_import_suggest() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_fail_data/not_found_suggest/main.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 2);
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(
        diag.messages[0].message,
        "name 's' is not defined, did you mean '[\"s1\"]'?"
    );
}

#[test]
fn test_resolve_assignment_in_lambda() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/assign_in_lambda.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    let main_scope = scope.scope_map.get("__main__").unwrap().clone();
    assert_eq!(main_scope.borrow().children.len(), 1);
    let lambda_scope = main_scope.borrow().children[0].clone();
    assert_eq!(lambda_scope.borrow().elems.len(), 2);
    let images_scope_obj = lambda_scope.borrow().elems.get("images").unwrap().clone();
    assert_eq!(images_scope_obj.borrow().ty.ty_str(), "[str]");
}

#[test]
fn test_resolve_function_with_default_values() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/function_with_default_values.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert!(!scope.handler.has_errors());
    let main_scope = scope.main_scope().unwrap();
    let func = main_scope.borrow().lookup("is_alpha").unwrap();
    assert!(func.borrow().ty.is_func());
    let func_ty = func.borrow().ty.into_func_type();
    assert_eq!(func_ty.params.len(), 3);
    assert_eq!(func_ty.params[0].has_default, false);
    assert_eq!(func_ty.params[1].has_default, true);
    assert_eq!(func_ty.params[2].has_default, true);
}

#[test]
fn test_assignment_type_annotation_check_in_lambda() {
    let sess = Arc::new(ParseSession::default());
    let opts = LoadProgramOptions::default();
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/annotation_check_assignment.k"],
        Some(opts),
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 0);
}

#[test]
fn test_resolve_lambda_assignment_diagnostic() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_fail_data/lambda_ty_error.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 1);
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(diag.code, Some(DiagnosticId::Error(ErrorKind::TypeError)));
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(
        diag.messages[0].message,
        "expected (int, int) -> int, got (int, int) -> str"
    );
}

#[test]
fn test_ty_check_in_dict_assign_to_schema() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/attr_ty_check.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 2);
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(diag.code, Some(DiagnosticId::Error(ErrorKind::TypeError)));
    assert_eq!(diag.messages.len(), 2);
    assert_eq!(diag.messages[0].message, "expected int, got str(1)");
    assert_eq!(
        diag.messages[1].message,
        "variable is defined here, its type is int, but got str(1)"
    );
}

#[test]
fn test_pkg_not_found_suggestion() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/pkg_not_found_suggestion.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 4);
    let diag = &scope.handler.diagnostics[1];
    assert_eq!(diag.code, Some(DiagnosticId::Suggestions));
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(
        diag.messages[0].message,
        "try 'kcl mod add k9s' to download the package not found"
    );
    let diag = &scope.handler.diagnostics[2];
    assert_eq!(diag.code, Some(DiagnosticId::Suggestions));
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(
        diag.messages[0].message,
        "find more package on 'https://artifacthub.io'"
    );
}

#[test]
fn undef_lambda_param() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/undef_lambda_param.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 1);

    let root = &program.root.clone();
    let filename = Path::new(&root.clone())
        .join("undef_lambda_param.k")
        .display()
        .to_string();

    let range = scope.handler.diagnostics[0].messages[0].range.clone();

    assert_eq!(
        range,
        (
            Position {
                filename: filename.clone(),
                line: 1,
                column: Some(10),
            },
            Position {
                filename: filename.clone(),
                line: 1,
                column: Some(15),
            }
        )
    );
}

#[test]
fn test_schema_params_count() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/schema_params_miss.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    assert_eq!(scope.handler.diagnostics.len(), 1);
    let diag = &scope.handler.diagnostics[0];
    assert_eq!(
        diag.code,
        Some(DiagnosticId::Error(ErrorKind::CompileError))
    );
    assert_eq!(diag.messages.len(), 1);
    assert_eq!(
        diag.messages[0].message,
        "expected 1 positional argument, found 0"
    );
}

#[test]
fn test_set_ty_in_lambda() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/ty_in_lambda.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    assert_eq!(
        resolve_program(&mut program)
            .main_scope()
            .unwrap()
            .borrow()
            .lookup("result")
            .unwrap()
            .borrow()
            .ty
            .clone()
            .ty_str(),
        "{str:str}"
    );
}

#[test]
fn test_pkg_asname() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess.clone(),
        &["./src/resolver/test_data/pkg_asname/pkg_asname.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    let scope = resolve_program(&mut program);
    let diags = scope.handler.diagnostics;
    assert_eq!(diags.len(), 6);
    assert_eq!(diags[0].messages[0].message, "name 'pkg' is not defined");
    assert_eq!(diags[2].messages[0].message, "name 'subpkg' is not defined");
}

#[test]
fn test_builtin_file_invalid() {
    let test_cases = [
        (
            "./src/resolver/test_data/test_builtin/read.k",
            "expected 1 positional argument, found 0",
        ),
        (
            "./src/resolver/test_data/test_builtin/glob.k",
            "expected 1 positional argument, found 0",
        ),
    ];

    for (file, expected_message) in &test_cases {
        let sess = Arc::new(ParseSession::default());
        let mut program = load_program(sess.clone(), &[file], None, None)
            .unwrap()
            .program;
        let scope = resolve_program(&mut program);
        let diags = scope.handler.diagnostics;
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].messages[0].message, *expected_message);
    }
}
