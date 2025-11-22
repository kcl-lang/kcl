use kcl_ast::token::{Token, TokenKind};

use kcl_span::symbol::kw;

#[repr(i32)]
#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Precedence {
    /// lowest place holder
    Lowest,
    /// as
    As,
    /// logic or ||
    LogicOr,
    /// logic and &&
    LogicAnd,
    /// ==, !=
    Equals,
    /// in, not it
    InOrNotIn,
    /// is, is not
    IsOrIsNot,
    /// >, <, >=, <=
    LessGreater,
    /// |
    BitOr,
    /// ^     
    BitXor,
    /// &
    BitAnd,
    /// >>, <<
    Shift,
    /// +, -
    Sum,
    /// *, /, % //
    Product,
    /// **
    Power,
    /// +X, -X, !X
    Prefix,
}

impl From<Token> for Precedence {
    fn from(tok: Token) -> Self {
        if tok.is_keyword(kw::As) {
            return Precedence::As;
        }
        match tok.kind {
            TokenKind::UnaryOp(_) => Precedence::Prefix,
            TokenKind::BinOp(ot) => match ot {
                kcl_ast::token::BinOpToken::Plus | kcl_ast::token::BinOpToken::Minus => {
                    Precedence::Sum
                }
                kcl_ast::token::BinOpToken::Star
                | kcl_ast::token::BinOpToken::Slash
                | kcl_ast::token::BinOpToken::Percent
                | kcl_ast::token::BinOpToken::SlashSlash => Precedence::Product,
                kcl_ast::token::BinOpToken::StarStar => Precedence::Power,
                kcl_ast::token::BinOpToken::Caret => Precedence::BitXor,
                kcl_ast::token::BinOpToken::And => Precedence::BitAnd,
                kcl_ast::token::BinOpToken::Or => Precedence::BitOr,
                kcl_ast::token::BinOpToken::Shl | kcl_ast::token::BinOpToken::Shr => {
                    Precedence::Shift
                }
            },
            TokenKind::BinCmp(ct) => match ct {
                kcl_ast::token::BinCmpToken::Eq | kcl_ast::token::BinCmpToken::NotEq => {
                    Precedence::Equals
                }
                kcl_ast::token::BinCmpToken::Lt
                | kcl_ast::token::BinCmpToken::LtEq
                | kcl_ast::token::BinCmpToken::Gt
                | kcl_ast::token::BinCmpToken::GtEq => Precedence::LessGreater,
            },
            _ => {
                if tok.is_keyword(kw::Or) {
                    Precedence::LogicOr
                } else if tok.is_keyword(kw::And) {
                    Precedence::LogicAnd
                } else if tok.is_keyword(kw::In) {
                    Precedence::InOrNotIn
                } else if tok.is_keyword(kw::Is) {
                    Precedence::IsOrIsNot
                } else {
                    Precedence::Lowest
                }
            }
        }
    }
}
