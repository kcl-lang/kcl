use kclvm_ast::ast::*;
use kclvm_ast::{token::LitKind, token::TokenKind};
use kclvm_error::Diagnostic;

use super::Parser;

impl<'a> Parser<'_> {
    /// Syntax:
    /// start: (NEWLINE | statement)*
    pub fn parse_module(&mut self) -> Result<Module, Diagnostic> {
        let doc = self.parse_doc();
        let body = self.parse_body()?;
        Ok(Module {
            filename: "".to_string(),
            pkg: "".to_string(),
            name: "".to_string(),
            doc,
            comments: self.comments.clone(),
            body,
        })
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

    fn parse_body(&mut self) -> Result<Vec<NodeRef<Stmt>>, Diagnostic> {
        let mut stmts = Vec::new();

        while let Some(stmt) = self.parse_stmt()? {
            stmts.push(stmt)
        }

        Ok(stmts)
    }
}
