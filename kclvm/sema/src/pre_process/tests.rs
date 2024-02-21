use std::sync::Arc;

use super::*;
use indexmap::IndexMap;
use kclvm_ast::path::get_attr_paths_from_config_expr;
use kclvm_parser::{load_program, parse_file_force_errors, ParseSession};

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
fn test_fix_raw_identifier_prefix() {
    let mut module =
        parse_file_force_errors("./src/pre_process/test_data/raw_identifier.k", None).unwrap();
    if let ast::Stmt::Assign(assign_stmt) = &module.body[0].node {
        assert_eq!(assign_stmt.targets[0].node.names[0].node, "$schema")
    } else {
        panic!("invalid assign statement")
    }
    fix_raw_identifier_prefix(&mut module);
    if let ast::Stmt::Assign(assign_stmt) = &module.body[0].node {
        assert_eq!(assign_stmt.targets[0].node.names[0].node, "schema")
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
    let modules = program.pkgs.get_mut(kclvm_ast::MAIN_PKG).unwrap();
    assert_eq!(modules.len(), 4);
    // Test the module merge result
    let module = modules.last().unwrap();
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
    let modules = program.pkgs.get_mut(kclvm_ast::MAIN_PKG).unwrap();
    assert_eq!(modules.len(), 1);
    // Test the module merge result
    let module = modules.first().unwrap();
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
    let mut program = load_program(
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
    let modules = program.pkgs.get_mut(kclvm_ast::MAIN_PKG).unwrap();
    assert_eq!(modules.len(), 3);
    let config1 = &modules[1];
    let config2 = &modules[1];
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
