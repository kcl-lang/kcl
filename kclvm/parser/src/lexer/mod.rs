//! A KCL lexer.
//!
//! The lexer is built on the low level [`kclvm_lexer`]
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
use kclvm_ast::token::VALID_SPACES_LENGTH;
use kclvm_ast::token::{self, BinOpToken, CommentKind, Token, TokenKind};
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
        tok_start_pos: start_pos,
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

    /// The start position of the current token.
    tok_start_pos: BytePos,

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
        // In the process of look-behind lexing, it is necessary to check the type of the previous token in 'buf',
        // If the previous token and the current token can form a multi-character token,
        // then the previous token will be popped from 'buf'.
        //
        // Therefore, the method 'self.token()' needs to take the mutable reference of 'buf' as an incoming argument.
        self.token = self.token(&mut buf);

        while !self.token.is_eof() {
            self.token.append_to(&mut buf);
            self.token = self.token(&mut buf);
        }

        self.eof(&mut buf);
        buf.into_token_stream()
    }

    fn token(&mut self, tok_stream_builder: &mut TokenStreamBuilder) -> TokenWithIndents {
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

            // Because of the 'look-behind', the 'start' of the current token becomes a two-way cursor,
            // which can not only move forward, but also move backward when 'look-behind'.
            // Therefore, the value of 'self.tok_start_pos' can be changed in 'self.lex_token()'.
            self.tok_start_pos = self.pos;
            // update pos after token and indent handling
            self.pos = self.pos + new_byte_pos(token.len as u32);

            // In the process of look-behind lexing, it is necessary to check the type of the previous token in 'tok_stream_builder',
            // If the previous token and the current token can form a multi-character token,
            // then the previous token will be popped from 'tok_stream_builder'.
            // Therefore, the method 'self.lex_token()' needs to take the mutable reference of 'tok_stream_builder' as an incoming argument.
            if let Some(kind) = self.lex_token(token, self.tok_start_pos, tok_stream_builder) {
                let span = self.span(self.tok_start_pos, self.pos);

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
    fn lex_token(
        &mut self,
        token: kclvm_lexer::Token,
        start: BytePos,
        tok_stream_builder: &mut TokenStreamBuilder,
    ) -> Option<TokenKind> {
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
            kclvm_lexer::TokenKind::Bang => {
                self.sess.struct_span_error(
                    "invalid token '!', consider using 'not'",
                    self.span(start, self.pos),
                );
                token::UnaryOp(token::UNot)
            }
            // Binary op
            kclvm_lexer::TokenKind::Plus => token::BinOp(token::Plus),
            kclvm_lexer::TokenKind::Minus => token::BinOp(token::Minus),
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
            // If the current token is '>',
            // then lexer need to check whether the previous token is '-',
            // if yes, return token '->', if not return token '>'.
            kclvm_lexer::TokenKind::Gt => match self.look_behind(&token, tok_stream_builder) {
                Some(tok_kind) => tok_kind,
                None => token::BinCmp(token::Gt),
            },
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
                        self.sess.struct_span_error(
                            "error nesting on close paren",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Brace)
                    }
                    // error recovery
                    token::OpenDelim(token::Bracket) => {
                        self.sess.struct_span_error(
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
                    self.sess.struct_span_error(
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
                        self.sess.struct_span_error(
                            "error nesting on close brace",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Paren)
                    }
                    // error recovery
                    token::OpenDelim(token::Bracket) => {
                        self.sess.struct_span_error(
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
                    self.sess.struct_span_error(
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
                        self.sess.struct_span_error(
                            "mismatched closing delimiter",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Brace)
                    }
                    // error recovery
                    token::OpenDelim(token::Paren) => {
                        self.sess.struct_span_error(
                            "mismatched closing delimiter",
                            self.span(start, self.pos),
                        );
                        token::CloseDelim(token::Paren)
                    }
                    // impossible case
                    _ => bug!("Impossible!"),
                },
                // error recovery
                None => {
                    self.sess.struct_span_error(
                        "mismatched closing delimiter",
                        self.span(start, self.pos),
                    );
                    token::CloseDelim(token::Bracket)
                }
            },
            kclvm_lexer::TokenKind::LineContinue => return None,
            kclvm_lexer::TokenKind::InvalidLineContinue => {
                // If we encounter an illegal line continuation character,
                // we will restore it to a normal line continuation character.
                self.sess.struct_span_error(
                    "unexpected character after line continuation character",
                    self.span(start, self.pos),
                );
                return None;
            }
            kclvm_lexer::TokenKind::Semi => {
                // If we encounter an illegal semi token ';', raise a friendly error.
                self.sess.struct_span_error(
                    "the semicolon ';' here is unnecessary, please remove it",
                    self.span(start, self.pos),
                );
                return None;
            }
            _ => {
                self.sess
                    .struct_span_error("unknown start of token", self.span(start, self.pos));
                return None;
            }
        })
    }

    /// From the lexed tokens stack, check whether the token at the top of the stack and the current character can combine a new token.
    /// If yes, lexer will pop the token at the top of the stack and return a new token combined with the token poped and the current character.
    /// If not, return None.
    fn look_behind(
        &mut self,
        tok: &kclvm_lexer::Token,
        tok_stream_builder: &mut TokenStreamBuilder,
    ) -> Option<TokenKind> {
        match tok.kind {
            // Most multi-character tokens are lexed in ['kclvm-lexer'],
            // and the multi-character tokens that need to be lexed in ['kclvm-parser/lexer'] are only token '->'.
            // If a new multi-character token is added later, the corresponding operation can be added here.
            kclvm_lexer::TokenKind::Gt => {
                if tok_stream_builder
                    .pop_if_tok_kind(&TokenKind::BinOp(BinOpToken::Minus))
                    .is_some()
                {
                    // After the previous token pops up, 'self.tok_start_pos' needs to be updated.
                    if self.tok_start_pos >= new_byte_pos(1) {
                        self.tok_start_pos = self.tok_start_pos - new_byte_pos(1);
                        return Some(TokenKind::RArrow);
                    } else {
                        bug!("Internal Bugs: Please connect us to fix it, invalid token start pos")
                    }
                }
            }
            _ => return None,
        }
        None
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
                let start_char = self.char_from(start);
                let (is_raw, quote_char_pos, quote_char) = match start_char {
                    'r' | 'R' => {
                        let pos = start + new_byte_pos(1);
                        (true, pos, self.char_from(pos))
                    }
                    _ => (false, start, start_char),
                };
                if !terminated {
                    self.sess.struct_span_error(
                        "unterminated string",
                        self.span(quote_char_pos, self.pos),
                    )
                }
                // Cut offset before validation.
                let offset: u32 = if triple_quoted {
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
                // For unclosed quote string, cut offset of the string content.
                if !terminated {
                    content_end = content_end + new_byte_pos(if triple_quoted { 3 } else { 1 })
                }
                // If start > end, it is a invalid string content.
                let value = if content_start > content_end {
                    // If get an error string from the eval process,
                    // directly return an empty string.
                    self.sess.struct_span_error(
                        "invalid string syntax",
                        self.span(content_start, self.pos),
                    );
                    "".to_string()
                } else {
                    let string_content = self.str_from_to(content_start, content_end);
                    match str_content_eval(string_content, quote_char, triple_quoted, false, is_raw)
                    {
                        Some(v) => v,
                        None => {
                            // If get an error string from the eval process,
                            // directly return an empty string.
                            self.sess.struct_span_error(
                                "invalid string syntax",
                                self.span(content_start, self.pos),
                            );
                            "".to_string()
                        }
                    }
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
                    );
                    // If it is a empty int, returns number 0.
                    (token::Integer, Symbol::intern("0"), None, None)
                } else {
                    let symbol = if self.validate_literal_int(base, start, suffix_start) {
                        self.symbol_from_to(start, suffix_start)
                    } else {
                        Symbol::intern("0")
                    };

                    let suffix = if suffix_start < self.pos {
                        let suffix_str = self.str_from(suffix_start);
                        // int binary suffix
                        if !NumberBinarySuffix::all_names().contains(&suffix_str) {
                            self.sess.struct_span_error(
                                "invalid int binary suffix",
                                self.span(start, self.pos),
                            );
                            None
                        } else {
                            Some(Symbol::intern(suffix_str))
                        }
                    } else {
                        None
                    };

                    (token::Integer, symbol, suffix, None)
                }
            }

            kclvm_lexer::LiteralKind::Float {
                base,
                empty_exponent,
            } => {
                let symbol = if self.validate_literal_float(base, start, empty_exponent) {
                    self.symbol_from_to(start, suffix_start)
                } else {
                    Symbol::intern("0")
                };
                (token::Float, symbol, None, None)
            }
            kclvm_lexer::LiteralKind::Bool { terminated: _ } => (
                token::Bool,
                self.symbol_from_to(start, suffix_start),
                None,
                None,
            ),
        }
    }

    fn validate_literal_int(
        &self,
        base: Base,
        content_start: BytePos,
        content_end: BytePos,
    ) -> bool {
        let base = match base {
            Base::Binary => 2,
            Base::Octal => 8,
            Base::Hexadecimal => 16,
            Base::Decimal => return true,
        };
        let s = self.str_from_to(content_start + new_byte_pos(2), content_end);
        for (idx, c) in s.char_indices() {
            let idx = idx as u32;
            if c != '_' && c.to_digit(base).is_none() {
                let lo = content_start + new_byte_pos(2 + idx);
                let hi = content_start + new_byte_pos(2 + idx + c.len_utf8() as u32);

                self.sess.struct_span_error(
                    &format!("invalid digit for a base {base} literal, start: {lo}, stop: {hi}"),
                    self.span(lo, self.pos),
                );
                return false;
            }
        }
        true
    }

    fn validate_literal_float(&self, base: Base, start: BytePos, empty_exponent: bool) -> bool {
        if empty_exponent {
            self.sess.struct_span_error(
                "expected at least one digit in exponent",
                self.span(start, self.pos),
            );
            false
        } else {
            match base {
                kclvm_lexer::Base::Hexadecimal
                | kclvm_lexer::Base::Octal
                | kclvm_lexer::Base::Binary => {
                    self.sess.struct_span_error(
                        &format!("{} float literal is not supported", base.describe()),
                        self.span(start, self.pos),
                    );
                    false
                }
                _ => true,
            }
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

    fn symbol_from_to(&self, start: BytePos, end: BytePos) -> Symbol {
        Symbol::intern(self.str_from_to(start, end))
    }

    fn eof(&mut self, buf: &mut TokenStreamBuilder) {
        if !self.indent_cxt.new_line_beginning {
            self.indent_cxt.new_line_beginning = true;
            buf.push(Token::new(token::Newline, self.span(self.pos, self.pos)));
        }

        while self.indent_cxt.indents.len() > 1 {
            self.indent_cxt.indents.pop();
            buf.push(Token::new(
                token::Dedent(VALID_SPACES_LENGTH),
                self.span(self.pos, self.pos),
            ));
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

    /// Pop the token at the top of the stack, and return None if the stack is empty.
    fn pop(&mut self) -> Option<Token> {
        self.buf.pop()
    }

    /// If the token kind at the top of the stack is 'expected_tok_kind',
    /// pop the token and return it, otherwise do nothing and return None.
    fn pop_if_tok_kind(&mut self, expected_tok_kind: &TokenKind) -> Option<Token> {
        if self.peek_tok_kind() == expected_tok_kind {
            self.pop()
        } else {
            None
        }
    }

    /// Peek the kind of the token on the top of the stack.
    fn peek_tok_kind(&self) -> &TokenKind {
        match self.buf.last() {
            Some(tok) => &tok.kind,
            None => &TokenKind::Dummy,
        }
    }
}
