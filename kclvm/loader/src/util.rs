use std::collections::HashMap;

use kclvm_ast::ast;
use kclvm_ast_pretty::{print_ast_node, ASTNode};
use kclvm_sema::eval::str_literal_eval;

pub(crate) fn get_call_args_bool(
    call_expr: &ast::CallExpr,
    index: usize,
    key: Option<&str>,
) -> bool {
    let val = get_call_args_string(call_expr, index, key);
    val == "True" || val == "true"
}

pub(crate) fn get_call_args_strip_string(
    call_expr: &ast::CallExpr,
    index: usize,
    key: Option<&str>,
) -> String {
    let value = get_call_args_string(call_expr, index, key);
    match str_literal_eval(&value, false, false) {
        Some(value) => value,
        None => value,
    }
}

pub(crate) fn get_call_args_string(
    call_expr: &ast::CallExpr,
    index: usize,
    key: Option<&str>,
) -> String {
    let (args, kwargs) = arguments_to_string(&call_expr.args, &call_expr.keywords);
    if let Some(key) = key {
        if let Some(val) = kwargs.get(key) {
            return val.to_string();
        }
    }
    if index < args.len() {
        return args[index].to_string();
    }
    "".to_string()
}

/// Print call arguments to argument vector and keyword mapping.
pub fn arguments_to_string(
    args: &[ast::NodeRef<ast::Expr>],
    kwargs: &[ast::NodeRef<ast::Keyword>],
) -> (Vec<String>, HashMap<String, String>) {
    (
        args.iter()
            .map(|a| print_ast_node(ASTNode::Expr(a)))
            .collect(),
        kwargs
            .iter()
            .map(|a| {
                (
                    a.node.arg.node.get_name(),
                    a.node
                        .value
                        .as_ref()
                        .map(|v| print_ast_node(ASTNode::Expr(v)))
                        .unwrap_or_default(),
                )
            })
            .collect(),
    )
}
