//!

use std::collections::HashMap;

use crate::util::loader::LoaderKind;

use super::expr_builder::ExprBuilder;
use kclvm_ast::{
    ast::{
        AssignStmt, Expr, ExprContext, Identifier, Module, Node, NodeRef, Program, SchemaStmt, Stmt,
    },
    node_ref,
};
use kclvm_runner::{execute, ExecProgramArgs};

const TMP_FILE: &str = "validationTempKCLCode.k";

pub fn validate(
    schema_name: Option<String>,
    attribute_name: &str,
    validated_file_path: String,
    validated_file_kind: LoaderKind,
    kcl_path: Option<&str>,
    kcl_code: Option<String>,
) -> Result<bool, String> {
    let k_path = match kcl_path {
        Some(path) => path,
        None => TMP_FILE,
    };

    let mut module: Module = match kclvm_parser::parse_file(&k_path, kcl_code) {
        Ok(ast_m) => ast_m,
        Err(err_msg) => return Err(err_msg),
    };

    let schemas = filter_schema_stmt(&module);
    let schema_name = match schema_name {
        Some(name) => Some(name),
        None => match schemas.get(0) {
            Some(schema) => Some(schema.name.node.clone()),
            None => None,
        },
    };

    let expr_builder =
        match ExprBuilder::new_with_file_path(validated_file_kind, validated_file_path) {
            Ok(builder) => builder,
            Err(_) => return Err("Failed to load validated file.".to_string()),
        };

    let validated_expr = match expr_builder.build(schema_name) {
        Ok(expr) => expr,
        Err(_) => return Err("Failed to load validated file.".to_string()),
    };

    let assign_stmt = build_assign(attribute_name, validated_expr);

    module.body.insert(0, assign_stmt);

    match eval_ast(module) {
        Ok(res) => Ok(true),
        Err(err) => Err(err),
    }
}

fn build_assign(attr_name: &str, node: NodeRef<Expr>) -> NodeRef<Stmt> {
    node_ref!(Stmt::Assign(AssignStmt {
        targets: vec![node_ref!(Identifier {
            names: vec![attr_name.to_string()],
            pkgpath: String::new(),
            ctx: ExprContext::Store,
        })],
        value: node,
        type_annotation: None,
        ty: None,
    }))
}

const MAIN_PKG_NAME: &str = "__main__";

fn eval_ast(mut m: Module) -> Result<String, String> {
    m.pkg = MAIN_PKG_NAME.to_string();

    let mut pkgs = HashMap::new();
    pkgs.insert(MAIN_PKG_NAME.to_string(), vec![m]);

    let prog = Program {
        root: MAIN_PKG_NAME.to_string(),
        main: MAIN_PKG_NAME.to_string(),
        pkgs,
        cmd_args: vec![],
        cmd_overrides: vec![],
    };

    execute(prog, 0, &ExecProgramArgs::default())
}

fn filter_schema_stmt(module: &Module) -> Vec<&SchemaStmt> {
    let mut result = vec![];
    for stmt in &module.body {
        if let Stmt::Schema(s) = &stmt.node {
            result.push(s);
        }
    }

    result
}
