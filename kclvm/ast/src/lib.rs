// Copyright 2021 The KCL Authors. All rights reserved.
use crate::ast::*;

pub mod ast;
pub mod config;
pub mod path;
pub mod token;
pub mod token_stream;
pub mod walker;

#[cfg(test)]
mod tests;

pub const MAIN_PKG: &str = "__main__";

#[macro_export]
macro_rules! node_ref {
    ($node: expr) => {
        NodeRef::new(Node::dummy_node($node))
    };
    ($node: expr, $pos:expr) => {
        NodeRef::new(Node::node_with_pos($node, $pos))
    };
}

#[macro_export]
macro_rules! expr_as {
    ($expr: expr, $expr_enum: path) => {
        if let $expr_enum(x) = ($expr.node as Expr) {
            Some(x)
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! stmt_as {
    ($stmt: expr, $stmt_enum: path) => {
        if let $stmt_enum(x) = ($stmt.node as Stmt) {
            Some(x)
        } else {
            None
        }
    };
}

/// Construct an AssignStmt node with assign_value as value
pub fn build_assign_node(attr_name: &str, assign_value: NodeRef<Expr>) -> NodeRef<Stmt> {
    let iden = node_ref!(Identifier {
        names: vec![attr_name.to_string()],
        pkgpath: String::new(),
        ctx: ExprContext::Store
    });

    node_ref!(Stmt::Assign(AssignStmt {
        value: assign_value,
        targets: vec![iden],
        type_annotation: None,
        ty: None
    }))
}
