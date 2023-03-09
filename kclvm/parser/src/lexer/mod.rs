//! A KCL lexer.
//!
//! The lexer is built on the low level [`kclvm_lexer`], and works
//! based on the rules defined by the KCL grammar ['./spec/grammar'].
//!
//! It's main responsibilities:
//! 1. Mapping low level [`kclvm_lexer::Token`] tokens into [`kclvm_ast::Token`] tokens,
//! and provide TokenStream to downstream [`kclvm_parser::parser`].
//! 2. Validations on Literals(String, Int, Float).
//! 3. Validations on closure of delim tokens.
//! 4. Validations on indent and dedent.
//!
//! The main differences of tokens between ast and lexer is:
//! 1. AST Affinity, based on unary, binary and other operations.
//! 2. Has Indent and dedent.
//! 3. Don't have some tokens(such as ';', '..', '..=', '<-')

mod indent;
mod string;

#[cfg(test)]
mod tests;

use compiler_base_macros::bug;
use compiler_base_span::{self, span::new_byte_pos, BytePos, Span};
use kclvm_ast::ast::NumberBinarySuffix;
use kclvm_ast::token::{self, CommentKind, Token, TokenKind};
use kclvm_ast::token_stream::TokenStream;
use kclvm_lexer::Base;
use kclvm_span::symbol::Symbol;
pub(crate) use string::str_content_eval;

use self::indent::IndentLevel;
use crate::session::ParseSession;

/// EntryPoint of the lexer.
/// Parse token streams from an input raw string and a fixed start point.
/// Return an iterable token stream.
pub fn parse_token_streams(sess: &ParseSession, src: &str, start_pos: BytePos) -> TokenStream {
    Lexer {
        sess,
        start_pos,
        pos: start_pos,
        end_src_index: src.len(),
        src,
        token: TokenWithIndents::Token {
            token: Token::dummy(),
        },
        indent_cxt: IndentContext {
            delims: Vec::new(),
            tabs: 0,
            spaces: 0,
            new_line_beginning: false,
            indents: vec![Default::default()],
        },
    }
    .into_tokens()
}

/// A token or a token with indent.
enum TokenWithIndents {
    Token {
        token: Token,
    },
    WithIndent {
        token: Token,
        indent: IndentOrDedents,
    },
}

/// A indent or a fixed count of dedents.
enum IndentOrDedents {
    Indent { token: Token },
    Dedents { tokens: Vec<Token> },
}

impl TokenWithIndents {
    pub(crate) fn is_eof(&self) -> bool {
        match self {
            TokenWithIndents::Token { token } => *token == token::Eof,
            TokenWithIndents::WithIndent { token, indent: _ } => *token == token::Eof,
        }
    }

    pub(crate) fn append_to(&self, buf: &mut TokenStreamBuilder) {
        match self {
            TokenWithIndents::Token { token } => {
                buf.push(*token);
            }
            TokenWithIndents::WithIndent { token, indent } => {
                match indent {
                    IndentOrDedents::Indent { token } => {
                        buf.push(*token);
                    }
                    IndentOrDedents::Dedents { tokens } => {
                        for dedent in tokens {
                            buf.push(*dedent);
                        }
                    }
                }

                buf.push(*token);
            }
        }
    }
}

struct Lexer<'a> {
    /// Initial position, read-only.
    start_pos: BytePos,

    /// The absolute offset within the source_map of the current character.
    pos: BytePos,

    /// Stop reading src at this index.
    end_src_index: usize,

    /// Source text to tokenize.
    src: &'a str,

    /// Token
    token: TokenWithIndents,

    /// A on-going context to handle indent/dedents
    indent_cxt: IndentContext,

    /// parse-time session
    pub sess: &'a ParseSession,
}

struct IndentContext {
    /// A new line flag
    new_line_beginning: bool,

    /// Delim stack
    delims: Vec<TokenKind>,

    /// tab counter
    tabs: usize,

    /// space counter
    spaces: usize,

    /// Indents stack
    indents: Vec<IndentLevel>,
}

impl<'a> Lexer<'a> {
    fn into_tokens(mut self) -> TokenStream {
        let mut buf = TokenStreamBuilder::default();
        self.token = self.token();

        while !self.token.is_eof() {
            self.token.append_to(&mut buf);
            self.token = self.token();
        }

        self.eof(&mut buf);
        buf.into_token_stream()
    }

    fn token(&mut self) -> TokenWithIndents {
        loop {
            let start_src_index = self.src_index(self.pos);
            let text: &str = &self.src[start_src_index..self.end_src_index];

            if text.is_empty() {
                return TokenWithIndents::Token {
                    token: Token::new(token::Eof, self.span(self.pos, self.pos)),
                };
            }

            // fetch next token
            let token = kclvm_lexer::first_token(text);

            // Detect and handle indent cases before lexing on-going token
            let indent = self.lex_indent_context(token.kind);

            let start = self.pos;
            // update pos after token and indent handling
            self.pos = self.pos + new_byte_pos(token.len as u32);

            if let Some(kind) = self.lex_token(token, start) {
                let span = self.span(start, self.pos);

                match indent {
                    Some(iord) => {
                        // return the token with the leading indent/dedents
                        return TokenWithIndents::WithIndent {
                            token: Token::new(kind, span),
                            indent: iord,
                        };
                    }
                    None => {
                        // return the token itself
                        return TokenWithIndents::Token {
                            token: Token::new(kind, span),
                        };
                    }
                }
            }
        }
    }

    /// Turns `kclvm_lexer::TokenKind` into a rich `kclvm_ast::TokenKind`.
    fn lex_token(&mut self, token: kclvm_lexer::Token, start: BytePos) -> Option<TokenKind> {
        Some(match token.kind {
            kclvm_lexer::TokenKind::LineComment { doc_style: _ } => {
                let s = self.str_from(start);
                token::DocComment(CommentKind::Line(Symbol::intern(s)))
            }
            // Whitespace
            kclvm_lexer::TokenKind::Newline => {
                self.indent_cxt.new_line_beginning = true;
                token::Newline
            }
            kclvm_lexer::TokenKind::Tab
            | kclvm_lexer::TokenKind::Space
            | kclvm_lexer::TokenKind::CarriageReturn
            | kclvm_lexer::TokenKind::Whitespace => return None,
            // Identifier
            kclvm_lexer::TokenKind::Ident => {
                let s = self.str_from(start);
                token::Ident(Symbol::intern(s))
            }
            // Literal
            kclvm_lexer::TokenKind::Literal { kind, suffix_start } => {
                let suffix_start = start + new_byte_pos(suffix_start as u32);
                let (kind, symbol, suffix, raw) = self.lex_literal(start, suffix_start, kind);
                token::Literal(token::Lit {
                    kind,
                    symbol,
                    suffix,
                    raw,
                })
            }
            // Unary op
            kclvm_lexer::TokenKind::Tilde => token::UnaryOp(token::UTilde),
            kclvm_lexer::TokenKind::Bang => token::UnaryOp(token::UNot),
            // Binary op
            kclvm_lexer::TokenKind::Plus => token::BinOp(token::Plus),
            kclvm_lexer::TokenKind::Minus => {
                let head = start + new_byte_pos(1);
                let tail = start + new_byte_pos(2);
                if self.has_next_token(head, tail) {
                    let next_tkn = self.str_from_to(head, tail);
                    if next_tkn == ">" {
                        // waste '>' token
                        self.pos = self.pos + new_byte_pos(1);
                        token::RArrow
                    } else {
                        token::BinOp(token::Minus)
                    }
                } else {
                    token::BinOp(token::Minus)
                }
            }
            kclvm_lexer::TokenKind::Star => token::BinOp(token::Star),
            kclvm_lexer::TokenKind::Slash => token::BinOp(token::Slash),
            kclvm_lexer::TokenKind::Percent => token::BinOp(token::Percent),
            kclvm_lexer::TokenKind::StarStar => token::BinOp(token::StarStar),
            kclvm_lexer::TokenKind::SlashSlash => token::BinOp(token::SlashSlash),
            kclvm_lexer::TokenKind::Caret => token::BinOp(token::Caret),
            kclvm_lexer::TokenKind::And => token::BinOp(token::And),
            kclvm_lexer::TokenKind::Or => token::BinOp(token::Or),
            kclvm_lexer::TokenKind::LtLt => token::BinOp(token::Shl),
            kclvm_lexer::TokenKind::GtGt => token::BinOp(token::Shr),
            // Binary op eq
            kclvm_lexer::TokenKind::PlusEq => token::BinOpEq(token::Plus),
            kclvm_lexer::TokenKind::MinusEq => token::BinOpEq(token::Minus),
            kclvm_lexer::TokenKind::StarEq => token::BinOpEq(token::Star),
            kclvm_lexer::TokenKind::SlashEq => token::BinOpEq(token::Slash),
            kclvm_lexer::TokenKind::PercentEq => token::BinOpEq(token::Percent),
            kclvm_lexer::TokenKind::StarStarEq => token::BinOpEq(token::StarStar),
            kclvm_lexer::TokenKind::SlashSlashEq => token::BinOpEq(token::SlashSlash),
            kclvm_lexer::TokenKind::CaretEq => token::BinOpEq(token::Caret),
            kclvm_lexer::TokenKind::AndEq => token::BinOpEq(token::And),
            kclvm_lexer::TokenKind::OrEq => token::BinOpEq(token::Or),
            kclvm_lexer::TokenKind::LtLtEq => token::BinOpEq(token::Shl),
            kclvm_lexer::TokenKind::GtGtEq => token::BinOpEq(token::Shr),
            // Binary cmp
            kclvm_lexer::TokenKind::EqEq => token::BinCmp(token::Eq),
            kclvm_lexer::TokenKind::BangEq => token::BinCmp(token::NotEq),
            kclvm_lexer::TokenKind::Lt => token::BinCmp(token::Lt),
            kclvm_lexer::TokenKind::LtEq => token::BinCmp(token::LtEq),
            kclvm_lexer::TokenKind::Gt => token::BinCmp(token::Gt),
            kclvm_lexer::TokenKind::GtEq => token::BinCmp(token::GtEq),
            // Structural symbols
            kclvm_lexer::TokenKind::At => token::At,
            kclvm_lexer::TokenKind::Dot => token::Dot,
            kclvm_lexer::TokenKind::DotDotDot => token::DotDotDot,
            kclvm_lexer::TokenKind::Comma => token::Comma,
            kclvm_lexer::TokenKind::Colon => token::Colon,
            kclvm_lexer::TokenKind::Dollar => token::Dollar,
            kclvm_lexer::TokenKind::Question => token::Question,
            kclvm_lexer::TokenKind::Eq => token::Assign,
            // Delim tokens
            kclvm_lexer::TokenKind::OpenParen => {
                self.indent_cxt.delims.push(token::OpenDelim(token::Paren));
                token::OpenDelim(token::Paren)
            }
            kclvm_lexer::TokenKind::CloseParen => match self.indent_cxt.delims.pop() {
                // check delim stack
                Some(delim) => match delim {
                    // expected case
                    token::OpenDelim(token::Paren) => token::CloseDelim(token::Paren),
                    // error recovery
                    token::OpenDelim(token::Brace) => {
                        self.sess.struct_span_error_recovery(
                            "error nesting on close paren",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Brace)
                    }
                    // error recovery
                    token::OpenDelim(token::Bracket) => {
                        self.sess.struct_span_error_recovery(
                            "error nesting on close paren",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Bracket)
                    }
                    // impossible case
                    _ => bug!("Impossible!"),
                },
                // error recovery
                None => {
                    self.sess.struct_span_error_recovery(
                        "error nesting on close paren",
                        self.span(start, self.pos),
                    );
                    token::CloseDelim(token::Paren)
                }
            },
            kclvm_lexer::TokenKind::OpenBrace => {
                self.indent_cxt.delims.push(token::OpenDelim(token::Brace));
                token::OpenDelim(token::Brace)
            }
            kclvm_lexer::TokenKind::CloseBrace => match self.indent_cxt.delims.pop() {
                // check delim stack
                Some(delim) => match delim {
                    // expected case
                    token::OpenDelim(token::Brace) => token::CloseDelim(token::Brace),
                    // error recovery
                    token::OpenDelim(token::Paren) => {
                        self.sess.struct_span_error_recovery(
                            "error nesting on close brace",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Paren)
                    }
                    // error recovery
                    token::OpenDelim(token::Bracket) => {
                        self.sess.struct_span_error_recovery(
                            "error nesting on close brace",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Bracket)
                    }
                    // impossible case
                    _ => bug!("Impossible!"),
                },
                // error recovery
                None => {
                    self.sess.struct_span_error_recovery(
                        "error nesting on close brace",
                        self.span(start, self.pos),
                    );
                    token::CloseDelim(token::Brace)
                }
            },
            kclvm_lexer::TokenKind::OpenBracket => {
                self.indent_cxt
                    .delims
                    .push(token::OpenDelim(token::Bracket));
                token::OpenDelim(token::Bracket)
            }
            kclvm_lexer::TokenKind::CloseBracket => match self.indent_cxt.delims.pop() {
                // check delim stack
                Some(delim) => match delim {
                    // expected case
                    token::OpenDelim(token::Bracket) => token::CloseDelim(token::Bracket),
                    // error recovery
                    token::OpenDelim(token::Brace) => {
                        self.sess.struct_span_error_recovery(
                            "error nesting on close bracket",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Brace)
                    }
                    // error recovery
                    token::OpenDelim(token::Paren) => {
                        self.sess.struct_span_error_recovery(
                            "error nesting on close bracket",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Paren)
                    }
                    // impossible case
                    _ => bug!("Impossible!"),
                },
                // error recovery
                None => {
                    self.sess.struct_span_error_recovery(
                        "error nesting on close bracket",
                        self.span(start, self.pos),
                    );
                    token::CloseDelim(token::Bracket)
                }
            },
            kclvm_lexer::TokenKind::LineContinue => return None,
            kclvm_lexer::TokenKind::InvalidLineContinue => self.sess.struct_span_error(
                "unexpected character after line continuation character",
                self.span(start, self.pos),
            ),
            _ => self
                .sess
                .struct_span_error("unknown start of token", self.span(start, self.pos)),
        })
    }

    fn lex_literal(
        &self,
        start: BytePos,
        suffix_start: BytePos,
        kind: kclvm_lexer::LiteralKind,
    ) -> (token::LitKind, Symbol, Option<Symbol>, Option<Symbol>) {
        match kind {
            kclvm_lexer::LiteralKind::Str {
                terminated,
                triple_quoted,
            } => {
                if !terminated {
                    self.sess
                        .struct_span_error("unterminated string", self.span(start, self.pos))
                }

                let start_char = self.char_from(start);
                let (is_raw, quote_char) = match start_char {
                    'r' | 'R' => (true, self.char_from(start + new_byte_pos(1))),
                    _ => (false, start_char),
                };

                // cut offset before validation
                let offset = if triple_quoted {
                    if is_raw {
                        4
                    } else {
                        3
                    }
                } else if is_raw {
                    2
                } else {
                    1
                };

                let content_start = start + new_byte_pos(offset);
                let mut content_end = suffix_start - new_byte_pos(offset);
                if is_raw {
                    content_end = content_end + new_byte_pos(1);
                }
                let string_content = self.str_from_to(content_start, content_end);
                let value = match str_content_eval(
                    string_content,
                    quote_char,
                    triple_quoted,
                    false,
                    is_raw,
                ) {
                    Some(v) => v,
                    None => self.sess.struct_span_error(
                        "Invalid string syntax",
                        self.span(content_start, self.pos),
                    ),
                };

                (
                    token::Str {
                        is_long_string: triple_quoted,
                        is_raw,
                    },
                    Symbol::intern(&value),
                    None,
                    Some(self.symbol_from_to(start, suffix_start)),
                )
            }
            kclvm_lexer::LiteralKind::Int { base, empty_int } => {
                if empty_int {
                    self.sess.struct_span_error(
                        "no valid digits found for number",
                        self.span(start, self.pos),
                    )
                } else {
                    self.validate_literal_int(base, start, suffix_start);

                    let suffix = if suffix_start < self.pos {
                        let suffix_str = self.str_from(suffix_start);
                        // int binary suffix
                        if !NumberBinarySuffix::all_names().contains(&suffix_str) {
                            self.sess.struct_span_error(
                                "invalid int binary suffix",
                                self.span(start, self.pos),
                            )
                        }
                        Some(Symbol::intern(suffix_str))
                    } else {
                        None
                    };

                    (
                        token::Integer,
                        self.symbol_from_to(start, suffix_start),
                        suffix,
                        None,
                    )
                }
            }

            kclvm_lexer::LiteralKind::Float {
                base,
                empty_exponent,
            } => {
                self.validate_literal_float(base, start, empty_exponent);
                (
                    token::Float,
                    self.symbol_from_to(start, suffix_start),
                    None,
                    None,
                )
            }
            kclvm_lexer::LiteralKind::Bool { terminated: _ } => (
                token::Bool,
                self.symbol_from_to(start, suffix_start),
                None,
                None,
            ),
        }
    }

    fn validate_literal_int(&self, base: Base, content_start: BytePos, content_end: BytePos) {
        let base = match base {
            Base::Binary => 2,
            Base::Octal => 8,
            Base::Hexadecimal => 16,
            _ => return,
        };
        let s = self.str_from_to(content_start + new_byte_pos(2), content_end);
        for (idx, c) in s.char_indices() {
            let idx = idx as u32;
            if c != '_' && c.to_digit(base).is_none() {
                let lo = content_start + new_byte_pos(2 + idx);
                let hi = content_start + new_byte_pos(2 + idx + c.len_utf8() as u32);

                self.sess.struct_span_error(
                    &format!(
                        "invalid digit for a base {} literal, start: {}, stop: {}",
                        base, lo, hi
                    ),
                    self.span(lo, self.pos),
                )
            }
        }
    }

    fn validate_literal_float(&self, base: Base, start: BytePos, empty_exponent: bool) {
        if empty_exponent {
            self.sess.struct_span_error(
                "expected at least one digit in exponent",
                self.span(start, self.pos),
            )
        }

        match base {
            kclvm_lexer::Base::Hexadecimal => self.sess.struct_span_error(
                "hexadecimal float literal is not supported",
                self.span(start, self.pos),
            ),
            kclvm_lexer::Base::Octal => self.sess.struct_span_error(
                "octal float literal is not supported",
                self.span(start, self.pos),
            ),
            kclvm_lexer::Base::Binary => self.sess.struct_span_error(
                "binary float literal is not supported",
                self.span(start, self.pos),
            ),
            _ => (),
        }
    }

    fn span(&self, lo: BytePos, hi: BytePos) -> Span {
        Span::new(lo, hi)
    }

    #[inline]
    fn src_index(&self, pos: BytePos) -> usize {
        (pos - self.start_pos).0 as usize
    }

    /// Char at `pos` in the source
    fn char_from(&self, pos: BytePos) -> char {
        self.src.as_bytes()[self.src_index(pos)] as char
    }

    /// Slice of the source text from `start` up to but excluding `self.pos`,
    /// meaning the slice does not include the character `self.ch`.
    fn str_from(&self, start: BytePos) -> &str {
        self.str_from_to(start, self.pos)
    }

    /// Slice of the source text spanning from `start` up to but excluding `end`.
    fn str_from_to(&self, start: BytePos, end: BytePos) -> &str {
        &self.src[self.src_index(start)..self.src_index(end)]
    }

    fn has_next_token(&self, start: BytePos, end: BytePos) -> bool {
        if self.src_index(start) > self.src_index(end) || self.src_index(end) > self.src.len() {
            false
        } else {
            true
        }
    }

    fn symbol_from_to(&self, start: BytePos, end: BytePos) -> Symbol {
        Symbol::intern(self.str_from_to(start, end))
    }

    fn eof(&mut self, buf: &mut TokenStreamBuilder) {
        let start = self.pos;

        if !self.indent_cxt.delims.is_empty() {
            self.sess.struct_span_error_recovery(
                "Unclosed nesting at the end of the file",
                self.span(start, self.pos),
            );

            // Add CloseDelims
            while !self.indent_cxt.delims.is_empty() {
                match self.indent_cxt.delims.pop() {
                    Some(token::OpenDelim(token::Paren)) => buf.push(Token::new(
                        token::CloseDelim(token::Paren),
                        self.span(self.pos, self.pos),
                    )),
                    Some(token::OpenDelim(token::Brace)) => buf.push(Token::new(
                        token::CloseDelim(token::Brace),
                        self.span(self.pos, self.pos),
                    )),
                    Some(token::OpenDelim(token::Bracket)) => buf.push(Token::new(
                        token::CloseDelim(token::Bracket),
                        self.span(self.pos, self.pos),
                    )),
                    _ => {
                        self.sess.struct_span_error_recovery(
                            "Unknown delim at the end of the file",
                            self.span(start, self.pos),
                        );
                    }
                }
            }
        }

        if !self.indent_cxt.new_line_beginning {
            self.indent_cxt.new_line_beginning = true;
            buf.push(Token::new(token::Newline, self.span(self.pos, self.pos)));
        }

        while self.indent_cxt.indents.len() > 1 {
            self.indent_cxt.indents.pop();
            buf.push(Token::new(token::Dedent, self.span(self.pos, self.pos)));
        }

        buf.push(Token::new(token::Eof, self.span(self.pos, self.pos)));
    }
}

#[derive(Default)]
struct TokenStreamBuilder {
    buf: Vec<Token>,
}

impl TokenStreamBuilder {
    fn push(&mut self, token: Token) {
        self.buf.push(token)
    }

    fn into_token_stream(self) -> TokenStream {
        TokenStream::new(self.buf)
    }
}
