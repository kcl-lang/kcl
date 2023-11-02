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
//!
//! The parser error recovery strategy design is [here](https://github.com/kcl-lang/kcl/issues/420).

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

use compiler_base_span::span::{new_byte_pos, BytePos};
use kclvm_ast::ast::{Comment, NodeRef, PosTuple};
use kclvm_ast::token::{CommentKind, Token, TokenKind};
use kclvm_ast::token_stream::{Cursor, TokenStream};
use kclvm_span::symbol::Symbol;

/// The parser is built on top of the [`kclvm_parser::lexer`], and ordering KCL tokens
/// [`kclvm_ast::token`] to KCL ast nodes [`kclvm_ast::ast`].
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

/// The DropMarker is used to mark whether to discard the token Mark whether to discard the token.
/// The principle is to store the index of the token in the token stream. When there is no index
/// change during the parse process, it is discarded and an error is output
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct DropMarker(usize);

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

    /// Get an AST position from the token pair (lo_tok, hi_tok).
    #[inline]
    pub(crate) fn token_span_pos(&mut self, lo_tok: Token, hi_tok: Token) -> PosTuple {
        self.byte_pos_to_pos(lo_tok.span.lo(), hi_tok.span.hi())
    }

    /// Get an AST position from the byte pos pair (lo, hi).
    pub(crate) fn byte_pos_to_pos(&mut self, lo: BytePos, hi: BytePos) -> PosTuple {
        let lo = self.sess.lookup_char_pos(lo);
        let hi = self.sess.lookup_char_pos(hi);

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

    /// Whether the parser has the next token in the token stream.
    #[inline]
    pub(crate) fn has_next(&mut self) -> bool {
        self.cursor.next().is_some()
    }

    #[inline]
    /// Whether the parser has the next token in the token stream.
    pub(crate) fn peek_has_next(&mut self) -> bool {
        self.cursor.peek().is_some()
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

    /// Mark the token index.
    pub(crate) fn mark(&mut self) -> DropMarker {
        DropMarker(self.cursor.index())
    }

    /// Decide to discard token according to the current index.
    pub(crate) fn drop(&mut self, marker: DropMarker) -> bool {
        if marker.0 == self.cursor.index() {
            let token_str: String = self.token.into();
            self.sess.struct_span_error(
                &format!("expected expression got {}", token_str),
                self.token.span,
            );
            self.bump();
            true
        } else {
            false
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
                if let TokenKind::DocComment(comment_kind) = tok.kind {
                    match comment_kind {
                        CommentKind::Line(x) => {
                            let lo = sess.lookup_char_pos(tok.span.lo());
                            let hi = sess.lookup_char_pos(tok.span.hi());
                            let filename: String = format!("{}", lo.file.name.prefer_remapped());

                            let node = kclvm_ast::ast::Node {
                                id: kclvm_ast::ast::AstIndex::default(),
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
                    }
                }
                continue;
            }

            // normal tokens
            non_comment_tokens.push(*tok);
        }

        (non_comment_tokens, comments)
    }
}
