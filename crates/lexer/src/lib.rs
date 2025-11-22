//! Low-level token stream lexer.
//!
//! The purpose of `kclvm_lexer` is similar to [`rustc_lexer`] crate,
//! which separates out pure lexing and language-specific designs.
//!
//! The difference with [`rustc_lexer`] is that here we want to define
//! a more general and wider range of tokens used by more languages.
//!
//! A language-specific lexer is needed to convert the basic token stream
//! into wide tokens consumed by the parser.
//!
//! The purpose of the lexer is to convert raw sources into a labeled sequence
//! of well-known token types. No error reporting on obvious error(e.g., string-not-closed error),
//! instead storing them as flags on the token. Checking of literal content is
//! not performed in this lexer.
//!
//! The main entity of this crate is the [`TokenKind`] enum which represents common
//! lexeme types.
//!
//! [`parser::lexer`]: ../parser/lexer/index.html
//! We want to be able to build this crate with a stable compiler, so no
//! `#![feature]` attributes should be added.

mod cursor;
mod kcl_cursor;
mod number;

#[cfg(test)]
mod tests;

extern crate kclvm_error;

use cursor::EOF_CHAR;

use self::TokenKind::*;
pub use crate::cursor::Cursor;

/// Parsed token.
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
}

impl Token {
    fn new(kind: TokenKind, len: usize) -> Self {
        Token { kind, len }
    }
}

/// Enum representing common lexeme types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    /// "# comment" or "// comment"
    LineComment { doc_style: Option<DocStyle> },

    /// `/* block comment */`
    ///
    /// Block comments can be recursive, so the sequence like `/* /* */`
    /// will not be considered terminated and will result in a parsing error.
    BlockComment {
        doc_style: Option<DocStyle>,
        terminated: bool,
    },

    /// "\t"
    Tab,

    /// " "
    Space,

    /// "\r"
    CarriageReturn,

    /// "\n"
    Newline,

    /// Any other whitespace characters sequence.
    Whitespace,

    /// "ident"
    Ident,

    /// Like the above, but containing invalid unicode codepoints.
    InvalidIdent,

    /// Invalid line continue symbol `\\` without the `\n` followed.
    InvalidLineContinue,

    /// Valid Line continue '\\'
    LineContinue,

    /// "12_u8", "1.0e-40", "b"123"". See `LiteralKind` for more details.
    Literal {
        kind: LiteralKind,
        suffix_start: usize,
    },

    /// ";"
    Semi,

    /// ","
    Comma,

    /// "."
    Dot,

    /// ".."
    DotDot,

    /// "..."
    DotDotDot,

    /// "("
    OpenParen,

    /// ")"
    CloseParen,

    /// "{"
    OpenBrace,

    /// "}"
    CloseBrace,

    /// "["
    OpenBracket,

    /// "]"
    CloseBracket,

    /// "@"
    At,

    /// "#"
    Pound,

    /// "~"
    Tilde,

    /// "?"
    Question,

    /// ":"
    Colon,

    /// "$"
    Dollar,

    /// "="
    Eq,

    /// "!"
    Bang,

    /// "<"
    Lt,

    /// ">"
    Gt,

    /// "=="
    EqEq,

    /// "!="
    BangEq,

    /// ">="
    GtEq,

    /// "<="
    LtEq,

    /// "-"
    Minus,

    /// "&"
    And,

    /// "|"
    Or,

    /// "+"
    Plus,

    /// "*"
    Star,

    /// "/"
    Slash,

    /// "^"
    Caret,

    /// "%"
    Percent,

    /// "**"
    StarStar,

    /// "//"
    SlashSlash,

    /// "<<"
    LtLt,

    /// ">>"
    GtGt,

    /// "+="
    PlusEq,

    /// "-="
    MinusEq,

    /// "*="
    StarEq,

    /// "/="
    SlashEq,

    /// "%="
    PercentEq,

    /// "&="
    AndEq,

    /// "|="
    OrEq,

    /// "^="
    CaretEq,

    /// "**="
    StarStarEq,

    /// "//="
    SlashSlashEq,

    /// "<<="
    LtLtEq,

    /// ">>="
    GtGtEq,

    /// Unknown token, not expected by the lexer, e.g. "№"
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocStyle {
    Outer,
    Inner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiteralKind {
    /// "12", "0o100", "0b120199"
    Int { base: Base, empty_int: bool },
    /// "12.34", "0b100.100"
    Float { base: Base, empty_exponent: bool },
    /// ""abc"", "'abc'", "'''abc'''"
    Str {
        terminated: bool,
        triple_quoted: bool,
    },
    /// True, False
    Bool { terminated: bool },
}

/// Base of numeric literal encoding according to its prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    /// Literal starts with "0b".
    Binary,
    /// Literal starts with "0o".
    Octal,
    /// Literal starts with "0x".
    Hexadecimal,
    /// Literal doesn't contain a prefix.
    Decimal,
}

impl Base {
    /// Returns the description string of the numeric literal base.
    pub fn describe(&self) -> &'static str {
        match self {
            Base::Binary => "binary",
            Base::Octal => "octal",
            Base::Hexadecimal => "hexadecimal",
            Base::Decimal => "decimal",
        }
    }
}

/// Parses the first token from the provided input string.
pub fn first_token(input: &str) -> Token {
    debug_assert!(!input.is_empty());
    Cursor::new(input).token()
}

/// Creates an iterator that produces tokens from the input string.
pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        if cursor.is_eof() {
            None
        } else {
            cursor.reset_len_consumed();
            Some(cursor.token())
        }
    })
}

pub trait ICursor: ITokenCursor + ICommentCursor {}

// Cursor trait to read one token from char stream.
pub trait ITokenCursor {
    fn token(&mut self) -> Token;
}

// Cursor trait to read comment.
// Line comment and block comment should be considered here.
pub trait ICommentCursor {
    // If we encounter a comment.
    // Returns true if exists, otherwise returns false.
    fn try_comment_magic(&self, _c: char) -> bool {
        false
    }

    // Eat it if so.
    // This mehod **MUST** be called after 'try_comment_magic'.
    // No gurantee to ensure the correctness if no comment here,
    // and return 'Unknown' if it happens.
    fn eat_comment(&mut self) -> TokenKind {
        Unknown
    }
}

// Cursor trait to read string.
// Simple string, raw string, unicode string, multi-lines string,
// and more string cases should be considered here.
pub trait IStringCursor {
    // If we encounter a string.
    // Returns true if exists, otherwise returns false.
    fn try_string_magic(&self, _c: char) -> bool {
        false
    }

    // Eat it if so.
    // This mehod **MUST** be called after 'try_string_magic'.
    // No gurantee to ensure the correctness if no string here,
    // and return 'Unknown' if it happens.
    // For example, no identifier check if no string found.
    fn eat_string(&mut self, _c: char) -> TokenKind {
        Unknown
    }
}

// Cursor trait to read identifier.
// Simple identifier, raw identifier, and more identifier cases should be considered here.
pub trait IIdentCursor {
    // If we encounter a identifier.
    // Returns true if exists, otherwise returns false.
    fn try_ident_magic(&self, _c: char) -> bool {
        false
    }

    // Eat it if so.
    // This method **MUST** be called after 'try_ident_magic'.
    // No guarantee to ensure the correctness if no identifier here,
    // and return 'Unknown' if it happens.
    fn eat_ident(&mut self) -> TokenKind {
        Unknown
    }
}

/// True if `c` is considered a whitespace.
pub fn is_whitespace(c: char) -> bool {
    match c {
        // Usual ASCII suspects
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
            => true,
        _ => false,
    }
}

impl<'a> ITokenCursor for Cursor<'a> {
    fn token(&mut self) -> Token {
        let char = self.bump().unwrap_or(EOF_CHAR);

        let token_kind = match char {
            // Comment or block comment, or a simple token
            c if self.try_comment_magic(c) => self.eat_comment(),

            // Whitespace sequence.
            c if is_whitespace(c) => Whitespace,

            // Various of string. E.g., quoted string, raw string.
            c if self.try_string_magic(c) => self.eat_string(c),

            // Identifier (this should be checked after other variant that can
            // start as identifier).
            c if self.try_ident_magic(c) => self.eat_ident(),

            // Numeric literal.
            c @ '0'..='9' => {
                let kind = self.number(c);
                let suffix_start = self.len_consumed();
                self.eat_lit_suffix(); // In case we have some suffix

                TokenKind::Literal { kind, suffix_start }
            }

            // '\r' will be considered as a 'WhiteSpace'.
            '\u{0009}' => Tab,
            '\u{0020}' => Space,
            '\u{000A}' => Newline,

            ';' => Semi,
            ',' => Comma,
            '.' => match (self.peek(), self.peek1th()) {
                ('.', '.') => {
                    self.bump();
                    self.bump();
                    DotDotDot
                }
                _ => Dot,
            },
            '(' => OpenParen,
            ')' => CloseParen,
            '{' => OpenBrace,
            '}' => CloseBrace,
            '[' => OpenBracket,
            ']' => CloseBracket,
            '@' => At,
            '#' => Pound,
            '~' => Tilde,
            '?' => Question,
            ':' => Colon,
            '$' => Dollar,
            '=' => match self.peek() {
                '=' => {
                    self.bump();
                    EqEq
                }
                _ => Eq,
            },
            '!' => match self.peek() {
                '=' => {
                    self.bump();
                    BangEq
                }
                _ => Bang,
            },
            '<' => match self.peek() {
                '=' => {
                    self.bump();
                    LtEq
                }
                '<' => {
                    self.bump();
                    match self.peek() {
                        '=' => {
                            self.bump();
                            LtLtEq
                        }
                        _ => LtLt,
                    }
                }
                _ => Lt,
            },
            '>' => match self.peek() {
                '=' => {
                    self.bump();
                    GtEq
                }
                '>' => {
                    self.bump();
                    match self.peek() {
                        '=' => {
                            self.bump();
                            GtGtEq
                        }
                        _ => GtGt,
                    }
                }
                _ => Gt,
            },
            '-' => match self.peek() {
                '=' => {
                    self.bump();
                    MinusEq
                }
                _ => Minus,
            },
            '&' => match self.peek() {
                '=' => {
                    self.bump();
                    AndEq
                }
                _ => And,
            },
            '|' => match self.peek() {
                '=' => {
                    self.bump();
                    OrEq
                }
                _ => Or,
            },
            '+' => match self.peek() {
                '=' => {
                    self.bump();
                    PlusEq
                }
                _ => Plus,
            },
            '*' => match self.peek() {
                '*' => {
                    self.bump();
                    match self.peek() {
                        '=' => {
                            self.bump();
                            StarStarEq
                        }
                        _ => StarStar,
                    }
                }
                '=' => {
                    self.bump();
                    StarEq
                }
                _ => Star,
            },
            '/' => match self.peek() {
                '/' => {
                    self.bump();
                    match self.peek() {
                        '=' => {
                            self.bump();
                            SlashSlashEq
                        }
                        _ => SlashSlash,
                    }
                }
                '=' => {
                    self.bump();
                    SlashEq
                }
                _ => Slash,
            },
            '^' => match self.peek() {
                '=' => {
                    self.bump();
                    CaretEq
                }
                _ => Caret,
            },
            '%' => match self.peek() {
                '=' => {
                    self.bump();
                    PercentEq
                }
                _ => Percent,
            },
            '\\' => match self.peek() {
                '\n' => {
                    self.bump();
                    LineContinue
                }
                EOF_CHAR => LineContinue,
                _ => InvalidLineContinue,
            },
            // Identifier starting with an emoji. Only lexed for graceful error recovery.
            c if !c.is_ascii() && unic_emoji_char::is_emoji(c) => {
                self.fake_ident_or_unknown_prefix()
            }
            _ => Unknown,
        };
        Token::new(token_kind, self.len_consumed())
    }
}

impl<'a> Cursor<'a> {
    // Eats the suffix of the literal, e.g. 'Ki', 'M', etc.
    fn eat_lit_suffix(&mut self) {
        if !rustc_lexer::is_id_start(self.peek()) {
            return;
        }
        self.bump();

        self.eat_while(rustc_lexer::is_id_continue);
    }

    fn fake_ident_or_unknown_prefix(&mut self) -> TokenKind {
        // Start is already eaten, eat the rest of identifier.
        self.eat_while(|c| {
            rustc_lexer::is_id_continue(c)
                || (!c.is_ascii() && unic_emoji_char::is_emoji(c))
                || c == '\u{200d}'
        });
        // Known prefixes must have been handled earlier. So if
        // we see a prefix here, it is definitely an unknown prefix.
        InvalidIdent
    }
}
