use std::sync::Arc;

use super::*;
use kclvm_ast::path::get_attr_paths_from_config_expr;
use kclvm_parser::{ParseSession, load_program, parse_file_force_errors};
use kclvm_primitives::IndexMap;

#[test]
fn test_fix_qualified_identifier() {
    let mut module =
        parse_file_force_errors("./src/pre_process/test_data/qualified_identifier.k", None)
            .unwrap();
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
fn test_fix_lit_ty_default_value() {
    let mut module =
        parse_file_force_errors("./src/pre_process/test_data/lit_ty_default_val.k", None).unwrap();
    fix_lit_ty_default_value(&mut module);
    if let ast::Stmt::Schema(schema_stmt) = &module.body[0].node {
        if let ast::Stmt::SchemaAttr(schema_attr) = &schema_stmt.body[0].node {
            assert_eq!(
                schema_attr.value.as_ref().unwrap().node,
                ast::Expr::StringLit(ast::StringLit {
                    is_long_string: false,
                    raw_value: "\"val\"".to_string(),
                    value: "val".to_string(),
                })
            )
        } else {
            panic!("invalid schema attr value")
        }
        if let ast::Stmt::SchemaAttr(schema_attr) = &schema_stmt.body[1].node {
            assert_eq!(
                schema_attr.value.as_ref().unwrap().node,
                ast::Expr::NumberLit(ast::NumberLit {
                    value: ast::NumberLitValue::Int(1),
                    binary_suffix: None,
                })
            )
        } else {
            panic!("invalid schema attr value")
        }
        if let ast::Stmt::SchemaAttr(schema_attr) = &schema_stmt.body[2].node {
            assert_eq!(
                schema_attr.value.as_ref().unwrap().node,
                ast::Expr::NumberLit(ast::NumberLit {
                    value: ast::NumberLitValue::Int(1),
                    binary_suffix: Some(ast::NumberBinarySuffix::Ki),
                })
            )
        } else {
            panic!("invalid schema attr value")
        }
        if let ast::Stmt::SchemaAttr(schema_attr) = &schema_stmt.body[3].node {
            assert_eq!(
                schema_attr.value.as_ref().unwrap().node,
                ast::Expr::NumberLit(ast::NumberLit {
                    value: ast::NumberLitValue::Float(2.0),
                    binary_suffix: None,
                })
            )
        } else {
            panic!("invalid schema attr value")
        }
        if let ast::Stmt::SchemaAttr(schema_attr) = &schema_stmt.body[4].node {
            assert_eq!(
                schema_attr.value.as_ref().unwrap().node,
                ast::Expr::NameConstantLit(ast::NameConstantLit {
                    value: ast::NameConstant::True,
                })
            )
        } else {
            panic!("invalid schema attr value")
        }
        if let ast::Stmt::SchemaAttr(schema_attr) = &schema_stmt.body[5].node {
            assert_eq!(
                schema_attr.value.as_ref().unwrap().node,
                ast::Expr::NameConstantLit(ast::NameConstantLit {
                    value: ast::NameConstant::False,
                })
            )
        } else {
            panic!("invalid schema attr value")
        }
    } else {
        panic!("invalid schema statement")
    }
}

#[test]
fn test_fix_raw_identifier_prefix() {
    let mut module =
        parse_file_force_errors("./src/pre_process/test_data/raw_identifier.k", None).unwrap();
    if let ast::Stmt::Assign(assign_stmt) = &module.body[0].node {
        assert_eq!(assign_stmt.targets[0].node.name.node, "$schema")
    } else {
        panic!("invalid assign statement")
    }
    fix_raw_identifier_prefix(&mut module);
    if let ast::Stmt::Assign(assign_stmt) = &module.body[0].node {
        assert_eq!(assign_stmt.targets[0].node.name.node, "schema")
    } else {
        panic!("invalid assign statement")
    }
    if let ast::Stmt::Schema(schema_stmt) = &module.body[1].node {
        if let ast::Stmt::SchemaAttr(attr) = &schema_stmt.body[0].node {
            assert_eq!(attr.name.node, "name");
        } else {
            panic!("invalid schema attr")
        }
        if let ast::Stmt::SchemaAttr(attr) = &schema_stmt.body[1].node {
            assert_eq!(attr.name.node, "$name");
        } else {
            panic!("invalid schema attr")
        }
    } else {
        panic!("invalid schema statement")
    }
}

#[test]
fn test_transform_multi_assign() {
    let targets = ["a", "b", "c", "d"];
    let mut module =
        parse_file_force_errors("./src/pre_process/test_data/multi_assign.k", None).unwrap();
    if let ast::Stmt::Assign(assign_stmt) = &module.body[1].node {
        assert_eq!(assign_stmt.targets.len(), targets.len());
        for (i, target) in targets.iter().enumerate() {
            assert_eq!(assign_stmt.targets[i].node.get_name(), *target);
        }
    } else {
        panic!("invalid assign statement")
    }
    transform_multi_assign(&mut module);
    for (i, target) in targets.iter().enumerate() {
        if let ast::Stmt::Assign(assign_stmt) = &module.body[i + 1].node {
            assert_eq!(assign_stmt.targets.len(), 1);
            assert_eq!(assign_stmt.targets[0].node.get_name(), *target);
        } else {
            panic!("invalid assign statement")
        }
    }
}

#[test]
fn test_config_merge() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess,
        &[
            "./src/pre_process/test_data/config_merge/def.k",
            "./src/pre_process/test_data/config_merge/config1.k",
            "./src/pre_process/test_data/config_merge/config2.k",
            "./src/pre_process/test_data/config_merge/config2.k",
        ],
        None,
        None,
    )
    .unwrap()
    .program;
    merge_program(&mut program);
    let modules = program.pkgs.get(kclvm_ast::MAIN_PKG).unwrap();
    assert_eq!(modules.len(), 3);
    // Test the module merge result
    let module = modules.last().unwrap();
    let module = program
        .get_module(module)
        .expect("Failed to acquire module lock")
        .expect(&format!("module {:?} not found in program", module));
    if let ast::Stmt::Unification(unification) = &module.body[0].node {
        let schema = &unification.value.node;
        if let ast::Expr::Config(config) = &schema.config.node {
            // 2 contains `name` in `config1.k`, `age` in `config2.k`.
            // person: Person {
            //     name = "Alice"
            //     age = 18
            // }
            assert_eq!(config.items.len(), 2);
            assert_eq!(
                get_attr_paths_from_config_expr(config),
                vec!["name".to_string(), "age".to_string()]
            );
        } else {
            panic!(
                "test failed, expect config expression, got {:?}",
                schema.config
            )
        }
    } else {
        panic!(
            "test failed, expect unification statement, got {:?}",
            module.body[0]
        )
    }
}

#[test]
fn test_config_override() {
    let sess = Arc::new(ParseSession::default());
    let mut program = load_program(
        sess,
        &["./src/pre_process/test_data/config_override.k"],
        None,
        None,
    )
    .unwrap()
    .program;
    merge_program(&mut program);
    let modules = program.pkgs.get(kclvm_ast::MAIN_PKG).unwrap();
    assert_eq!(modules.len(), 1);
    // Test the module merge result
    let module = modules.first().unwrap();
    let module = program
        .get_module(module)
        .expect("Failed to acquire module lock")
        .expect(&format!("module {:?} not found in program", module));
    if let ast::Stmt::Unification(unification) = &module.body[2].node {
        let schema = &unification.value.node;
        if let ast::Expr::Config(config) = &schema.config.node {
            // key = Config {
            //     data.key: "value1"
            // }
            assert_eq!(config.items.len(), 1);
            assert_eq!(
                get_attr_paths_from_config_expr(config),
                vec!["key".to_string(), "key.data.key".to_string()]
            );
        } else {
            panic!(
                "test failed, expect config expression, got {:?}",
                schema.config
            )
        }
    } else {
        panic!(
            "test failed, expect unification statement, got {:?}",
            module.body[2]
        )
    }
}

#[test]
fn test_skip_merge_program() {
    let sess = Arc::new(ParseSession::default());
    let program = load_program(
        sess,
        &[
            "./src/pre_process/test_data/config_merge/def.k",
            "./src/pre_process/test_data/config_merge/config1.k",
            "./src/pre_process/test_data/config_merge/config2.k",
        ],
        None,
        None,
    )
    .unwrap()
    .program;
    // skip merge program and save raw config ast node
    // merge_program(&mut program);
    let modules = program.pkgs.get(kclvm_ast::MAIN_PKG).unwrap();
    assert_eq!(modules.len(), 3);
    let config1 = &modules[1];
    let config2 = &modules[1];
    let config1 = program
        .get_module(config1)
        .expect("Failed to acquire module lock")
        .expect(&format!("module {:?} not found in program", config1));
    let config2 = program
        .get_module(config2)
        .expect("Failed to acquire module lock")
        .expect(&format!("module {:?} not found in program", config2));
    if let ast::Stmt::Unification(unification) = &config1.body[0].node {
        let schema = &unification.value.node;
        if let ast::Expr::Config(config) = &schema.config.node {
            assert_eq!(config.items.len(), 1);
        } else {
            panic!(
                "test failed, expect config expression, got {:?}",
                schema.config
            )
        }
    } else {
        panic!(
            "test failed, expect unification statement, got {:?}",
            config1.body[0]
        )
    }

    if let ast::Stmt::Unification(unification) = &config2.body[0].node {
        let schema = &unification.value.node;
        if let ast::Expr::Config(config) = &schema.config.node {
            assert_eq!(config.items.len(), 1);
        } else {
            panic!(
                "test failed, expect config expression, got {:?}",
                schema.config
            )
        }
    } else {
        panic!(
            "test failed, expect unification statement, got {:?}",
            config2.body[0]
        )
    }
}

#[test]
fn test_list_type_validation() {
    let code = r#"
    schema Resource:
        kind: str
        apiGroup: str
        metadata: any
        spec: any

    resource = Resource{
        kind = "Pod"
        apiGroup = "core"
        metadata = {
            name = "test"
        }
    }

    resource2 = Resource{
        kind = "Pod"
        apiGroup = "core"
        metadata = {
            name = "test"
        }
    }

    otherResource = {
        name = "test"
    }

    resourceList: [Resource] = [resource, resource2, otherResource]
    "#;

    let result = parse_file_force_errors("test_list_type_validation.k", Some(code.to_string()));
    assert!(
        result.is_err(),
        "Expected an evaluation error, but the code passed."
    );
}
