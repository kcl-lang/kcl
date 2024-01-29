//! Copyright The KCL Authors. All rights reserved.

pub mod ast;
pub mod config;
pub mod path;
pub mod pos;
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
