//! A KCL Parser.
//!
//! The parser is built on top of the [`kclvm_parser::lexer`], and ordering KCL tokens
//! [`kclvm_ast::token`] to KCL ast nodes [`kclvm_ast::ast`].
//!
//! The parser follows a LL1 parsing method, which constantly looking for
//! left-side derivation until a terminal token found. Since we hand-written the parser,
//! there is more flexibility in dealing with deduction priorities.
//!
//! KCL syntax elements can be simply divided into statements, expressions and tokens,
//! in which statement consists of expressions and tokens. In expression, operand is the most
//! complex part to enable all kinds of ident, constant, list, dict, config exprs.

#![macro_use]

mod expr;
mod int;
mod module;
mod precedence;
mod stmt;
#[cfg(test)]
mod tests;
mod ty;

use crate::session::ParseSession;

use compiler_base_span::span::new_byte_pos;
use kclvm_ast::ast::{Comment, NodeRef};
use kclvm_ast::token::{CommentKind, Token, TokenKind};
use kclvm_ast::token_stream::{Cursor, TokenStream};
use kclvm_span::symbol::Symbol;

pub struct Parser<'a> {
    /// The current token.
    pub token: Token,
    /// The previous token.
    pub prev_token: Token,
    /// Stream cursor
    cursor: Cursor,
    /// all comments.
    comments: Vec<NodeRef<Comment>>,
    /// parse-time session
    pub sess: &'a ParseSession,
}

impl<'a> Parser<'a> {
    pub fn new(sess: &'a ParseSession, stream: TokenStream) -> Self {
        let (non_comment_tokens, comments) = Parser::split_token_stream(sess, stream);

        let mut parser = Parser {
            token: Token::dummy(),
            prev_token: Token::dummy(),
            cursor: TokenStream::new(non_comment_tokens).cursor(),
            comments,
            sess,
        };

        // bump to the first token
        parser.bump();

        parser
    }

    pub(crate) fn token_span_pos(
        &mut self,
        lo_tok: Token,
        hi_tok: Token,
    ) -> (String, u64, u64, u64, u64) {
        let lo = self.sess.lookup_char_pos(lo_tok.span.lo());
        let hi = self.sess.lookup_char_pos(hi_tok.span.hi());

        let filename: String = format!("{}", lo.file.name.prefer_remapped());
        (
            filename,
            lo.line as u64,
            lo.col.0 as u64,
            hi.line as u64,
            hi.col.0 as u64,
        )
    }

    pub(crate) fn bump(&mut self) {
        self.prev_token = self.token;
        let next = self.cursor.next();

        if let Some(token) = next {
            self.token = token
        }
    }

    pub(crate) fn bump_keyword(&mut self, kw: Symbol) {
        if !self.token.is_keyword(kw) {
            self.sess.struct_token_error(&[kw.into()], self.token);
        }
        self.bump();
    }

    pub(crate) fn bump_token(&mut self, kind: TokenKind) {
        if self.token.kind != kind {
            self.sess.struct_token_error(&[kind.into()], self.token);
        }
        self.bump();
    }

    pub(crate) fn skip_newlines(&mut self) {
        while let TokenKind::Newline = self.token.kind {
            self.bump();
        }
    }
}

impl<'a> Parser<'a> {
    fn split_token_stream(
        sess: &'a ParseSession,
        stream: TokenStream,
    ) -> (Vec<Token>, Vec<NodeRef<Comment>>) {
        let mut comments = Vec::new();
        let mut non_comment_tokens = Vec::new();

        for (i, tok) in stream.iter().enumerate() {
            let prev_token = if i == 0 {
                Token {
                    kind: TokenKind::Dummy,
                    span: kclvm_span::Span::new(new_byte_pos(0), new_byte_pos(0)),
                }
            } else {
                stream[i - 1]
            };

            // eof: add newline
            if tok.kind == TokenKind::Eof {
                // append Newline
                if prev_token.kind != TokenKind::Newline {
                    non_comment_tokens.push(Token {
                        kind: TokenKind::Newline,
                        span: tok.span,
                    });
                }
                non_comment_tokens.push(*tok);
                break;
            }

            // split comments
            if matches!(tok.kind, TokenKind::DocComment(_)) {
                match tok.kind {
                    TokenKind::DocComment(comment_kind) => match comment_kind {
                        CommentKind::Line(x) => {
                            let lo = sess.lookup_char_pos(tok.span.lo());
                            let hi = sess.lookup_char_pos(tok.span.hi());
                            let filename: String = format!("{}", lo.file.name.prefer_remapped());

                            let node = kclvm_ast::ast::Node {
                                node: Comment {
                                    text: x.as_str().to_string(),
                                },
                                filename,
                                line: lo.line as u64,
                                column: lo.col.0 as u64,
                                end_line: hi.line as u64,
                                end_column: hi.col.0 as u64,
                            };

                            comments.push(NodeRef::new(node));
                        }
                    },
                    _ => (),
                }
                continue;
            }

            // normal tokens
            non_comment_tokens.push(*tok);
        }

        (non_comment_tokens, comments)
    }
}
