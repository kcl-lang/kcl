#![allow(dead_code)]

use super::Parser;

use kclvm_ast::ast::{Expr, Node, NodeRef, Type};
use kclvm_ast::token;
use kclvm_ast::token::{BinOpToken, DelimToken, TokenKind};
use kclvm_ast::{ast, expr_as};
use kclvm_span::symbol::{kw, sym};

impl<'a> Parser<'a> {
    /// Syntax:
    ///
    /// type: type_element (OR type_element)*
    /// type_element: schema_type | basic_type | compound_type | literal_type
    /// schema_type: identifier
    /// basic_type: STRING_TYPE | INT_TYPE | FLOAT_TYPE | BOOL_TYPE | ANY_TYPE
    /// compound_type: list_type | dict_type
    /// list_type: LEFT_BRACKETS (type)? RIGHT_BRACKETS
    /// dict_type: LEFT_BRACE (type)? COLON (type)? RIGHT_BRACE
    /// literal_type: string | number | TRUE | FALSE | NONE
    pub(crate) fn parse_type_annotation(&mut self) -> NodeRef<Type> {
        let token = self.token;
        let mut type_node_list = vec![self.parse_type_element()];

        while let TokenKind::BinOp(BinOpToken::Or) = self.token.kind {
            self.bump();
            let t = self.parse_type_element();
            type_node_list.push(t);
        }

        if type_node_list.len() > 1 {
            let mut union_type = ast::UnionType {
                type_elements: Vec::new(),
            };
            for v in type_node_list.iter_mut() {
                union_type.type_elements.push(v.clone());
            }

            Box::new(Node::node(
                Type::Union(union_type.clone()),
                self.sess.struct_token_loc(token, self.prev_token),
            ))
        } else {
            type_node_list[0].clone()
        }
    }

    fn parse_type_element(&mut self) -> NodeRef<Type> {
        let token = self.token;

        // any
        if self.token.is_keyword(kw::Any) {
            let t = Type::Any;
            self.bump_keyword(kw::Any);
            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        }
        // lit: true/false
        else if self.token.is_keyword(kw::True) {
            self.bump_keyword(kw::True);
            return Box::new(Node::node(
                Type::Literal(ast::LiteralType::Bool(true)),
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        } else if self.token.is_keyword(kw::False) {
            self.bump_keyword(kw::False);
            return Box::new(Node::node(
                Type::Literal(ast::LiteralType::Bool(false)),
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        }
        // basic type
        else if self.token.is_keyword(sym::bool) {
            let t = Type::Basic(ast::BasicType::Bool);
            self.bump_keyword(sym::bool);
            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        } else if self.token.is_keyword(sym::int) {
            let t = Type::Basic(ast::BasicType::Int);
            self.bump_keyword(sym::int);
            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        } else if self.token.is_keyword(sym::float) {
            let t = Type::Basic(ast::BasicType::Float);
            self.bump_keyword(sym::float);
            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        } else if self.token.is_keyword(sym::str) {
            let t = Type::Basic(ast::BasicType::Str);
            self.bump_keyword(sym::str);
            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        }

        // named type
        if let TokenKind::Ident(_) = self.token.kind {
            let ident = self.parse_identifier_expr();
            let ident = expr_as!(ident, Expr::Identifier).unwrap();
            let t = Type::Named(ident);
            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        }
        // lit type
        else if let TokenKind::Literal(lit) = self.token.kind {
            let t = match lit.kind {
                token::LitKind::Bool => {
                    if lit.symbol == kw::True {
                        ast::LiteralType::Bool(true)
                    } else if lit.symbol == kw::False {
                        ast::LiteralType::Bool(false)
                    } else {
                        self.sess
                            .struct_token_error(&[kw::True.into(), kw::False.into()], self.token);
                        ast::LiteralType::Bool(false)
                    }
                }
                token::LitKind::Integer => {
                    let v = lit.symbol.as_str().parse::<i64>().unwrap();
                    if let Some(suffix) = lit.suffix {
                        let x = ast::NumberBinarySuffix::try_from(suffix.as_str().as_str());
                        ast::LiteralType::Int(ast::IntLiteralType {
                            value: v,
                            suffix: Some(x.unwrap()),
                        })
                    } else {
                        ast::LiteralType::Int(ast::IntLiteralType {
                            value: v,
                            suffix: None,
                        })
                    }
                }
                token::LitKind::Float => {
                    let v = lit.symbol.as_str().parse::<f64>().unwrap();
                    ast::LiteralType::Float(v)
                }
                token::LitKind::Str { .. } => ast::LiteralType::Str(lit.symbol.as_str()),
                _ => {
                    if self.token.is_keyword(kw::True) {
                        ast::LiteralType::Bool(true)
                    } else if self.token.is_keyword(kw::False) {
                        ast::LiteralType::Bool(false)
                    } else {
                        self.sess
                            .struct_token_error(&[kw::True.into(), kw::False.into()], self.token);
                        ast::LiteralType::Bool(false)
                    }
                }
            };

            let t = Type::Literal(t);

            self.bump();

            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        }
        // [type]
        else if let TokenKind::OpenDelim(DelimToken::Bracket) = self.token.kind {
            self.bump_token(TokenKind::OpenDelim(DelimToken::Bracket));

            if let TokenKind::CloseDelim(DelimToken::Bracket) = self.token.kind {
                self.bump();
                let t = Type::List(ast::ListType { inner_type: None });

                return Box::new(Node::node(
                    t,
                    self.sess.struct_token_loc(token, self.prev_token),
                ));
            } else {
                let elem_type = self.parse_type_annotation();
                let t = Type::List(ast::ListType {
                    inner_type: Some(elem_type),
                });

                self.bump_token(TokenKind::CloseDelim(DelimToken::Bracket));

                return Box::new(Node::node(
                    t,
                    self.sess.struct_token_loc(token, self.prev_token),
                ));
            }
        }
        // {key:value}
        else if let TokenKind::OpenDelim(DelimToken::Brace) = self.token.kind {
            self.bump_token(TokenKind::OpenDelim(DelimToken::Brace));

            let key_type = if let TokenKind::Colon = self.token.kind {
                None
            } else {
                Some(self.parse_type_annotation())
            };

            self.bump_token(TokenKind::Colon);

            let value_type = if let TokenKind::CloseDelim(DelimToken::Brace) = self.token.kind {
                None
            } else {
                Some(self.parse_type_annotation())
            };

            let t = Type::Dict(ast::DictType {
                key_type,
                value_type,
            });

            self.bump_token(TokenKind::CloseDelim(DelimToken::Brace));

            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        }
        // (type) -> type
        else if let TokenKind::OpenDelim(DelimToken::Paren) = self.token.kind {
            self.bump_token(TokenKind::OpenDelim(DelimToken::Paren));
            let mut params_type = vec![];
            // Parse all the params type until the params list end ')'
            while self.token.kind != TokenKind::CloseDelim(DelimToken::Paren)
                && self.peek_has_next()
            {
                params_type.push(self.parse_type_annotation());
                // All the params type should be separated by ','
                if let TokenKind::Comma = self.token.kind {
                    self.bump_token(TokenKind::Comma);
                }
            }
            // If there is no params type, set it to None
            let params_ty = if params_type.is_empty() {
                None
            } else {
                Some(params_type)
            };

            self.bump_token(TokenKind::CloseDelim(DelimToken::Paren));
            // If there is a return type, parse it
            // Return type start with '->'
            let ret_ty = if let TokenKind::RArrow = self.token.kind {
                self.bump_token(TokenKind::RArrow);
                Some(self.parse_type_annotation())
            } else {
                None
            };

            let t = Type::Function(ast::FunctionType { params_ty, ret_ty });

            return Box::new(Node::node(
                t,
                self.sess.struct_token_loc(token, self.prev_token),
            ));
        }
        // Expect type tokens
        self.sess.struct_token_error(
            &[
                kw::Any.into(),
                sym::bool.into(),
                sym::int.into(),
                sym::float.into(),
                sym::str.into(),
                kw::True.into(),
                kw::False.into(),
                TokenKind::ident_value(),
                TokenKind::literal_value(),
                TokenKind::OpenDelim(DelimToken::Bracket).into(),
                TokenKind::OpenDelim(DelimToken::Brace).into(),
                TokenKind::CloseDelim(DelimToken::Paren).into(),
            ],
            self.token,
        );
        self.bump();
        Box::new(Node::node(
            Type::Any,
            self.sess.struct_token_loc(token, self.prev_token),
        ))
    }
}
