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
mod schema;
mod stmt;
#[cfg(test)]
mod tests;
mod ty;

use crate::session::ParseSession;

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
        let (non_comment_tokens, comments) = Parser::split_token_stream(&sess, stream);

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
        use rustc_span::Pos;
        let lo = self.sess.source_map.lookup_char_pos(lo_tok.span.lo());
        let hi = self.sess.source_map.lookup_char_pos(hi_tok.span.hi());

        let filename: String = format!("{}", lo.file.name.prefer_remapped());
        (
            filename,
            lo.line as u64,
            lo.col.to_usize() as u64,
            hi.line as u64,
            hi.col.to_usize() as u64,
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
            if let TokenKind::Ident(ident) = self.token.kind {
                self.sess.struct_span_error(
                    &format!(
                        "bump keyword failed: expect={}, got={:?} # ident={}",
                        kw.to_ident_string(),
                        self.token,
                        ident
                    ),
                    self.token.span,
                );
            } else {
                self.sess.struct_span_error(
                    &format!(
                        "bump keyword failed: expect={}, {:?}",
                        kw.to_ident_string(),
                        self.token
                    ),
                    self.token.span,
                );
            }
        }
        self.bump();
    }

    pub(crate) fn bump_token(&mut self, kind: TokenKind) {
        if self.token.kind != kind {
            if let TokenKind::Ident(ident) = self.token.kind {
                self.sess.struct_span_error(
                    &format!(
                        "bump token failed: expect={:?}, got={:?} # ident={}",
                        kind, self.token, ident
                    ),
                    self.token.span,
                );
            } else {
                self.sess.struct_span_error(
                    &format!("bump token failed: expect={:?}, {:?}", kind, self.token),
                    self.token.span,
                );
            }
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
        use rustc_span::BytePos;

        let mut comments = Vec::new();
        let mut non_comment_tokens = Vec::new();

        for (i, tok) in stream.iter().enumerate() {
            let prev_token = if i == 0 {
                Token {
                    kind: TokenKind::Dummy,
                    span: kclvm_span::Span::new(BytePos(0), BytePos(0)),
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
                            use rustc_span::Pos;
                            let lo = sess.source_map.lookup_char_pos(tok.span.lo());
                            let hi = sess.source_map.lookup_char_pos(tok.span.hi());
                            let filename: String = format!("{}", lo.file.name.prefer_remapped());

                            let node = kclvm_ast::ast::Node {
                                node: Comment {
                                    text: x.as_str().to_string(),
                                },
                                filename: filename,
                                line: lo.line as u64,
                                column: lo.col.to_usize() as u64,
                                end_line: hi.line as u64,
                                end_column: hi.col.to_usize() as u64,
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
