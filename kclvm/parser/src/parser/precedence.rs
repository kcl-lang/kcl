use kclvm_ast::token::{Token, TokenKind};

use kclvm_span::symbol::kw;

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
                kclvm_ast::token::BinOpToken::Plus | kclvm_ast::token::BinOpToken::Minus => {
                    Precedence::Sum
                }
                kclvm_ast::token::BinOpToken::Star
                | kclvm_ast::token::BinOpToken::Slash
                | kclvm_ast::token::BinOpToken::Percent
                | kclvm_ast::token::BinOpToken::SlashSlash => Precedence::Product,
                kclvm_ast::token::BinOpToken::StarStar => Precedence::Power,
                kclvm_ast::token::BinOpToken::Caret => Precedence::BitXor,
                kclvm_ast::token::BinOpToken::And => Precedence::BitAnd,
                kclvm_ast::token::BinOpToken::Or => Precedence::BitOr,
                kclvm_ast::token::BinOpToken::Shl | kclvm_ast::token::BinOpToken::Shr => {
                    Precedence::Shift
                }
            },
            TokenKind::BinCmp(ct) => match ct {
                kclvm_ast::token::BinCmpToken::Eq | kclvm_ast::token::BinCmpToken::NotEq => {
                    Precedence::Equals
                }
                kclvm_ast::token::BinCmpToken::Lt
                | kclvm_ast::token::BinCmpToken::LtEq
                | kclvm_ast::token::BinCmpToken::Gt
                | kclvm_ast::token::BinCmpToken::GtEq => Precedence::LessGreater,
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
