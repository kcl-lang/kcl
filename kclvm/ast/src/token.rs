//! KCL AST Affinity tokens.
//!
//! Tokens are designed based on the KCL AST.
//! Including indent and dedent tokens.
//! Not Include some tokens of low level tokens, such as ';', '..', '..=', '<-'.
pub use BinCmpToken::*;
pub use BinOpToken::*;
pub use DelimToken::*;
pub use LitKind::*;
pub use TokenKind::*;
pub use UnaryOpToken::*;

use compiler_base_span::{Span, DUMMY_SP};
pub use kclvm_span::symbol::{Ident, Symbol};
pub const VALID_SPACES_LENGTH: usize = 0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CommentKind {
    /// "#"
    Line(Symbol),
}

#[derive(Clone, PartialEq, Hash, Debug, Copy)]
pub enum UnaryOpToken {
    /// "~"
    UTilde,

    /// "not"
    UNot,
}

#[derive(Clone, PartialEq, Hash, Debug, Copy)]
pub enum BinOpToken {
    /// "+"
    Plus,

    /// "-"
    Minus,

    /// "*"
    Star,

    /// "/"
    Slash,

    /// "%"
    Percent,

    /// "**"
    StarStar,

    /// "//"
    SlashSlash,

    /// "^"
    Caret,

    /// "&"
    And,

    /// "|"
    Or,

    /// "<<"
    Shl,

    /// ">>"
    Shr,
}

#[derive(Clone, PartialEq, Hash, Debug, Copy)]
pub enum BinCmpToken {
    /// "=="
    Eq,

    /// "!="
    NotEq,

    /// "<"
    Lt,

    /// "<="
    LtEq,

    /// ">"
    Gt,

    /// ">="
    GtEq,
}

/// A delimiter token.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum DelimToken {
    /// A round parenthesis (i.e., `(` or `)`).
    Paren,
    /// A square bracket (i.e., `[` or `]`).
    Bracket,
    /// A curly brace (i.e., `{` or `}`).
    Brace,
    /// An empty delimiter.
    NoDelim,
}

/// A literal token.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Lit {
    pub kind: LitKind,
    pub symbol: Symbol,
    pub suffix: Option<Symbol>,
    pub raw: Option<Symbol>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LitKind {
    Bool,
    Integer,
    Float,
    Str { is_long_string: bool, is_raw: bool },
    None,
    Undefined,
    Err,
}

impl From<LitKind> for String {
    fn from(val: LitKind) -> Self {
        let s = match val {
            Bool => "bool",
            Integer => "int",
            Float => "float",
            Str { .. } => "str",
            None => "None",
            Undefined => "Undefined",
            Err => "error",
        };

        s.to_string()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TokenKind {
    /* Expression-operator symbols. */
    UnaryOp(UnaryOpToken),
    BinOp(BinOpToken),
    BinOpEq(BinOpToken),
    BinCmp(BinCmpToken),

    /* Structural symbols */
    /// '@'
    At,
    /// '.'
    Dot,
    /// '...'
    DotDotDot,
    /// ','
    Comma,
    /// ':'
    Colon,
    /// '->'
    RArrow,
    /// '$'
    Dollar,
    /// '?'
    Question,
    /// '='
    Assign,
    /// An opening delimiter (e.g., `{`).
    OpenDelim(DelimToken),
    /// A closing delimiter (e.g., `}`).
    CloseDelim(DelimToken),

    /* Literals */
    Literal(Lit),

    /// Identifier token.
    Ident(Symbol),

    /// A comment token.
    DocComment(CommentKind),

    /// '\t' or ' '
    Indent(usize),

    /// Remove an indent
    Dedent(usize),

    /// '\n'
    Newline,

    Dummy,

    Eof,
}

impl TokenKind {
    pub fn ident_value() -> String {
        "identifier".to_string()
    }

    pub fn literal_value() -> String {
        "literal".to_string()
    }
}

impl From<TokenKind> for String {
    fn from(val: TokenKind) -> Self {
        let s = match val {
            UnaryOp(unary_op) => match unary_op {
                UTilde => "~",
                UNot => "not",
            },
            BinOp(bin_op) => match bin_op {
                Plus => "+",
                Minus => "-",
                Star => "*",
                Slash => "/",
                Percent => "%",
                StarStar => "**",
                SlashSlash => "//",
                Caret => "^",
                And => "&",
                Or => "|",
                Shl => "<<",
                Shr => ">>",
            },
            BinOpEq(bin_op_eq) => match bin_op_eq {
                Plus => "+=",
                Minus => "-=",
                Star => "*=",
                Slash => "/=",
                Percent => "%=",
                StarStar => "**=",
                SlashSlash => "//=",
                Caret => "^=",
                And => "&=",
                Or => "|=",
                Shl => "<<=",
                Shr => ">>=",
            },
            BinCmp(bin_cmp) => match bin_cmp {
                Eq => "==",
                NotEq => "!=",
                Lt => "<",
                LtEq => "<=",
                Gt => ">",
                GtEq => ">=",
            },
            At => "@",
            Dot => ".",
            DotDotDot => "...",
            Comma => ",",
            Colon => ":",
            RArrow => "->",
            Dollar => "$",
            Question => "?",
            Assign => "=",
            OpenDelim(delim) => match delim {
                Paren => "(",
                Bracket => "[",
                Brace => "{",
                NoDelim => "open_no_delim",
            },
            CloseDelim(delim) => match delim {
                Paren => ")",
                Bracket => "]",
                Brace => "}",
                NoDelim => "close_no_delim",
            },
            Literal(lit) => match lit.kind {
                Bool => "bool",
                Integer => "integer",
                Float => "float",
                Str { .. } => "string",
                None => "None",
                Undefined => "Undefined",
                Err => "err",
            },
            TokenKind::Ident(_) => "identifier",
            DocComment(kind) => match kind {
                CommentKind::Line(_) => "inline_comment",
            },
            Indent(_) => "indent",
            Dedent(_) => "dedent",
            Newline => "newline",
            Dummy => "dummy",
            Eof => "eof",
        };
        s.to_string()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl From<Token> for String {
    fn from(val: Token) -> Self {
        match val.kind {
            Literal(lk) => {
                let sym = lk.symbol.as_str();

                match lk.suffix {
                    Some(suf) => sym + &suf.as_str(),
                    _other_none => sym,
                }
            }
            _ => val.kind.into(),
        }
    }
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }

    /// Some token that will be thrown away later.
    pub fn dummy() -> Self {
        Token::new(TokenKind::Dummy, DUMMY_SP)
    }

    /// Returns an identifier if this token is an identifier.
    pub fn ident(&self) -> Option<Ident> {
        match self.kind {
            Ident(name) => Some(Ident::new(name, self.span)),
            _ => std::option::Option::None,
        }
    }

    pub fn is_keyword(&self, kw: Symbol) -> bool {
        self.run_on_ident(|id| id.name == kw)
    }

    /// Whether the token is a string literal token.
    pub fn is_string_lit(&self) -> bool {
        match self.kind {
            TokenKind::Literal(lit) => {
                if let LitKind::Str { .. } = lit.kind {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn run_on_ident(&self, pred: impl FnOnce(Ident) -> bool) -> bool {
        match self.ident() {
            Some(id) => pred(id),
            _ => false,
        }
    }

    /// Whether the token kind is in the recovery token set, when meets errors, drop it.
    #[inline]
    pub fn is_in_recovery_set(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Indent(VALID_SPACES_LENGTH) | TokenKind::Dummy
        )
    }
}

impl PartialEq<TokenKind> for Token {
    fn eq(&self, rhs: &TokenKind) -> bool {
        self.kind == *rhs
    }
}
