use kclvm_error::{diagnostic::Range, Position};

use crate::ast;

pub trait ContainsPos {
    /// Check if current scope or node contains a position.
    fn contains_pos(&self, pos: &Position) -> bool;
}

pub trait GetPos {
    /// Get start and end position from node.
    fn get_span_pos(&self) -> Range {
        (self.get_pos(), self.get_end_pos())
    }
    /// Get start pos from node.
    fn get_pos(&self) -> Position;
    /// Get end pos from node.
    fn get_end_pos(&self) -> Position;
}

impl<T> ContainsPos for ast::Node<T> {
    fn contains_pos(&self, pos: &Position) -> bool {
        let (start_pos, end_pos) = self.get_span_pos();
        start_pos.less_equal(pos) && pos.less_equal(&end_pos)
    }
}

impl ContainsPos for Range {
    fn contains_pos(&self, pos: &Position) -> bool {
        self.0.filename == pos.filename && self.0.less_equal(pos) && pos.less_equal(&self.1)
    }
}

impl<T> GetPos for ast::Node<T> {
    fn get_pos(&self) -> Position {
        Position {
            filename: self.filename.clone(),
            line: self.line,
            column: Some(self.column),
        }
    }

    fn get_end_pos(&self) -> Position {
        Position {
            filename: self.filename.clone(),
            line: self.end_line,
            column: Some(self.end_column),
        }
    }
}

impl GetPos for ast::SchemaStmt {
    fn get_pos(&self) -> Position {
        match self.decorators.first() {
            Some(decorator) => decorator.get_pos(),
            None => self.name.get_pos(),
        }
    }

    fn get_end_pos(&self) -> Position {
        match self.checks.last() {
            Some(check_expr) => check_expr.get_end_pos(),
            None => match self.body.last() {
                Some(body_stmt) => body_stmt.get_end_pos(),
                None => match self.mixins.last() {
                    Some(mixin) => mixin.get_end_pos(),
                    None => self.name.get_end_pos(),
                },
            },
        }
    }
}

impl GetPos for ast::RuleStmt {
    fn get_pos(&self) -> Position {
        match self.decorators.first() {
            Some(decorator) => decorator.get_pos(),
            None => self.name.get_pos(),
        }
    }

    fn get_end_pos(&self) -> Position {
        match self.checks.last() {
            Some(check_expr) => check_expr.get_end_pos(),
            None => self.name.get_end_pos(),
        }
    }
}
