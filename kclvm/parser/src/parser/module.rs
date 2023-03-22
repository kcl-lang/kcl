use kclvm_ast::ast::*;
use kclvm_ast::{token::LitKind, token::TokenKind};

use super::Parser;

impl<'a> Parser<'a> {
    /// Syntax:
    /// start: (NEWLINE | statement)*
    pub fn parse_module(&mut self) -> Module {
        let doc = self.parse_doc();
        let body = self.parse_body();
        Module {
            filename: "".to_string(),
            pkg: "".to_string(),
            name: "".to_string(),
            doc,
            comments: self.comments.clone(),
            body,
        }
    }

    fn parse_doc(&mut self) -> String {
        if let TokenKind::Literal(lit) = self.token.kind {
            if let LitKind::Str { is_long_string, .. } = lit.kind {
                if is_long_string {
                    let doc = format!("{:?}", self.token);
                    self.bump();
                    return doc;
                }
            }
        }
        "".to_string()
    }

    fn parse_body(&mut self) -> Vec<NodeRef<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            if matches!(self.token.kind, TokenKind::Eof) {
                self.bump();
                break;
            }

            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                // Error recovery from panic mode: Once an error is detected (the statement is None),
                // the symbols in the input are continuously discarded (one symbol at a time), until the
                // "synchronous lexical unit" is found (the statement start token e.g., import, schema, etc).
                self.bump();
            }
        }
        stmts
    }
}
