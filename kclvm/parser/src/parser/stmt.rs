#![allow(dead_code)]
#![allow(unused_macros)]

use compiler_base_span::{span::new_byte_pos, BytePos, Span};
use kclvm_ast::token::VALID_SPACES_LENGTH;
use kclvm_ast::token::{CommentKind, DelimToken, LitKind, Token, TokenKind};
use kclvm_ast::{ast::*, expr_as, node_ref};
use kclvm_span::symbol::kw;

use super::Parser;

/// Parser implementation of statements, which consists of expressions and tokens.
/// Parser uses `parse_exprlist` and `parse_expr` in [`kclvm_parser::parser::expr`]
/// to get a expression node, and then concretize it into the specified expression node,
/// and then assemble it into the corresponding statement node.
impl<'a> Parser<'a> {
    /// Syntax:
    /// statement: simple_stmt | compound_stmt
    /// simple_stmt: (assign_stmt | expr_stmt | assert_stmt | import_stmt | type_alias_stmt) NEWLINE
    /// compound_stmt: if_stmt | schema_stmt
    pub(crate) fn parse_stmt(&mut self) -> Option<NodeRef<Stmt>> {
        // skip new lines
        if matches!(self.token.kind, TokenKind::Newline) {
            self.skip_newlines();
        }

        // eof => None
        if matches!(self.token.kind, TokenKind::Eof) {
            return None;
        }

        // compound_stmt
        if let Some(stmt) = self.parse_compound_stmt() {
            return Some(stmt);
        }

        // simple_stmt
        if let Some(stmt) = self.parse_simple_stmt() {
            return Some(stmt);
        }

        self.sess.struct_span_error(
            &format!("unexpected '{:?}'", self.token.kind),
            self.token.span,
        );

        None
    }

    /// Syntax:
    /// simple_stmt: (assign_stmt | expr_stmt | assert_stmt | import_stmt | type_alias_stmt) NEWLINE
    fn parse_compound_stmt(&mut self) -> Option<NodeRef<Stmt>> {
        // skip new lines
        if matches!(self.token.kind, TokenKind::Newline) {
            self.skip_newlines();
        }

        // eof => None
        if matches!(self.token.kind, TokenKind::Eof) {
            return None;
        }

        // if ...
        if self.token.is_keyword(kw::If) {
            return Some(self.parse_if_stmt());
        }

        // @decorators
        let decorators = if matches!(self.token.kind, TokenKind::At) {
            Some(self.parse_decorators())
        } else {
            None
        };

        // schema/mixin/protocol/rule ...
        if self.token.is_keyword(kw::Schema)
            || self.token.is_keyword(kw::Mixin)
            || self.token.is_keyword(kw::Protocol)
        {
            return Some(self.parse_schema_stmt(decorators));
        }
        if self.token.is_keyword(kw::Rule) {
            return Some(self.parse_rule_stmt(decorators));
        }

        None
    }

    /// Syntax:
    /// simple_stmt: (assign_stmt | unification_stmt | expr_stmt | assert_stmt | import_stmt | type_alias_stmt) NEWLINE
    fn parse_simple_stmt(&mut self) -> Option<NodeRef<Stmt>> {
        // skip new lines
        if matches!(self.token.kind, TokenKind::Newline) {
            self.skip_newlines();
        }

        // eof => None
        if matches!(self.token.kind, TokenKind::Eof) {
            return None;
        }

        // import ...
        if self.token.is_keyword(kw::Import) {
            return Some(self.parse_import_stmt());
        }

        // type ...
        if self.token.is_keyword(kw::Type) {
            return Some(self.parse_type_alias_stmt());
        }
        // assert ...
        if self.token.is_keyword(kw::Assert) {
            return Some(self.parse_assert_stmt());
        }

        // unary expr
        if let TokenKind::UnaryOp(_) = self.token.kind {
            return Some(self.parse_expr_stmt());
        }

        if matches!(
            self.token.kind,
            TokenKind::Ident(_) | TokenKind::Literal(_) | TokenKind::OpenDelim(_)
        ) {
            // expr or assign
            self.parse_expr_or_assign_stmt(false)
        } else {
            None
        }
    }

    /// Syntax:
    /// Indent/Dedent
    pub(crate) fn parse_block_stmt_list(
        &mut self,
        open_tok: TokenKind,
        close_tok: TokenKind,
    ) -> Vec<NodeRef<Stmt>> {
        let mut stmt_list = Vec::new();
        self.validate_dedent();
        self.bump_token(open_tok);
        loop {
            if self.token.kind == TokenKind::Eof {
                self.bump();
                break;
            }

            self.validate_dedent();
            if self.token.kind == close_tok {
                self.bump_token(close_tok);
                break;
            }

            if let Some(stmt) = self.parse_stmt() {
                stmt_list.push(stmt);
            } else {
                // Error recovery from panic mode: Once an error is detected (the statement is None),
                // the symbols in the input are continuously discarded (one symbol at a time), until the
                // "synchronous lexical unit" is found (the statement start token e.g., import, schema, etc).
                self.bump();
            }
        }

        self.skip_newlines();
        stmt_list
    }

    /// Syntax:
    /// test: if_expr | simple_expr
    fn parse_expr_stmt(&mut self) -> NodeRef<Stmt> {
        let token = self.token;
        let expr = vec![self.parse_expr()];

        let stmt = node_ref!(
            Stmt::Expr(ExprStmt { exprs: expr }),
            self.token_span_pos(token, self.prev_token)
        );

        self.skip_newlines();
        stmt
    }

    /// Syntax:
    ///
    /// expr_stmt: testlist_expr
    /// testlist_expr: test (COMMA test)*
    ///
    /// assign_stmt: identifier [COLON type] (ASSIGN identifier)* ASSIGN test
    ///   | identifier (COMP_PLUS | COMP_MINUS | COMP_MULTIPLY | COMP_DOUBLE_STAR | COMP_DIVIDE
    ///     | COMP_DOUBLE_DIVIDE | COMP_MOD | COMP_AND | COMP_OR | COMP_XOR | COMP_SHIFT_LEFT
    ///     | COMP_SHIFT_RIGHT | L_OR | L_AND)
    ///     test
    fn parse_expr_or_assign_stmt(&mut self, is_in_schema_stmt: bool) -> Option<NodeRef<Stmt>> {
        let token = self.token;
        let mut targets = vec![self.parse_expr()];

        let mut value_or_target = None;
        let mut type_annotation = None;
        let mut ty = None;

        if let TokenKind::Colon = self.token.kind {
            self.bump_token(TokenKind::Colon);
            let typ = self.parse_type_annotation();

            type_annotation = Some(node_ref!(typ.node.to_string(), typ.pos()));
            // Unification statement
            if let TokenKind::OpenDelim(DelimToken::Brace) = self.token.kind {
                // schema expression without args
                if let Type::Named(ref identifier) = typ.node {
                    let identifier = node_ref!(Expr::Identifier(identifier.clone()), typ.pos());
                    let schema_expr = self.parse_schema_expr(*identifier, token);
                    let mut ident = self.expr_as_identifier(targets[0].clone(), token);
                    ident.ctx = ExprContext::Store;
                    let unification_stmt = UnificationStmt {
                        target: Box::new(Node::node_with_pos(ident, targets[0].pos())),
                        value: Box::new(schema_expr.as_ref().clone().try_into().unwrap()),
                    };
                    self.skip_newlines();
                    return Some(node_ref!(
                        Stmt::Unification(unification_stmt),
                        self.token_span_pos(token, self.prev_token)
                    ));
                }
            }
            ty = Some(typ);
        } else if let TokenKind::BinOpEq(x) = self.token.kind {
            let op = AugOp::from(x);
            self.bump_token(self.token.kind);

            let value = self.parse_expr();
            let mut ident = self.expr_as_identifier(targets[0].clone(), token);
            ident.ctx = ExprContext::Store;

            let t = node_ref!(
                Stmt::AugAssign(AugAssignStmt {
                    target: Box::new(Node::node_with_pos(ident, targets[0].pos())),
                    value,
                    op,
                }),
                self.token_span_pos(token, self.prev_token)
            );

            self.skip_newlines();

            return Some(t);
        }

        while let TokenKind::Assign = self.token.kind {
            self.bump_token(TokenKind::Assign);

            let expr = self.parse_expr();
            if let Some(target) = value_or_target {
                targets.push(target);
            }

            value_or_target = Some(expr);
        }

        if let TokenKind::BinOpEq(x) = self.token.kind {
            if targets.len() == 1 && type_annotation.is_some() && is_in_schema_stmt {
                let aug_op = AugOp::from(x);
                self.bump_token(self.token.kind);
                let value = self.parse_expr();
                if let Expr::Identifier(target) = &targets[0].node {
                    self.skip_newlines();
                    return Some(node_ref!(
                        Stmt::SchemaAttr(SchemaAttr {
                            doc: "".to_string(),
                            name: node_ref!(target.get_name(), targets[0].pos()),
                            ty: ty.unwrap(),
                            op: Some(aug_op),
                            value: Some(value),
                            is_optional: false,
                            decorators: Vec::new(),
                        }),
                        self.token_span_pos(token, self.prev_token)
                    ));
                }
            }
        }

        let stmt_end_token = self.prev_token;

        if let Some(value) = value_or_target {
            let mut pos = targets[0].pos();
            pos.3 = value.end_line;
            pos.4 = value.end_column;

            let targets = targets
                .iter()
                .map(|expr| match &expr.node {
                    Expr::Identifier(x) => {
                        let mut x = x.clone();
                        x.ctx = ExprContext::Store;
                        Box::new(Node::node_with_pos(x, expr.pos()))
                    }
                    _ => {
                        self.sess
                            .struct_token_error(&[TokenKind::ident_value()], self.token);
                        Box::new(expr.into_missing_identifier())
                    }
                })
                .collect();

            self.skip_newlines();

            Some(node_ref!(
                Stmt::Assign(AssignStmt { targets, value, ty }),
                self.token_span_pos(token, stmt_end_token)
            ))
        } else {
            if targets.len() == 1 && type_annotation.is_some() && is_in_schema_stmt {
                if let Expr::Identifier(target) = &targets[0].node {
                    self.skip_newlines();
                    return Some(node_ref!(
                        Stmt::SchemaAttr(SchemaAttr {
                            doc: "".to_string(),
                            name: node_ref!(target.get_names().join("."), targets[0].pos()),
                            ty: ty.unwrap(),
                            op: None,
                            value: None,
                            is_optional: false,
                            decorators: Vec::new(),
                        }),
                        self.token_span_pos(token, stmt_end_token)
                    ));
                }
            }
            if type_annotation.is_none() && !targets.is_empty() {
                let mut pos = targets[0].pos();
                pos.3 = targets.last().unwrap().end_line;
                pos.4 = targets.last().unwrap().end_column;

                let t = Box::new(Node::node_with_pos(
                    Stmt::Expr(ExprStmt {
                        exprs: targets.clone(),
                    }),
                    pos,
                ));

                self.skip_newlines();

                Some(t)
            } else {
                let miss_expr = self.missing_expr();
                self.skip_newlines();
                self.sess
                    .struct_token_error(&[TokenKind::Assign.into()], self.token);
                if type_annotation.is_some() && !targets.is_empty() && !is_in_schema_stmt {
                    let mut pos = targets[0].pos();
                    pos.3 = targets.last().unwrap().end_line;
                    pos.4 = targets.last().unwrap().end_column;

                    let targets: Vec<_> = targets
                        .iter()
                        .map(|expr| match &expr.node {
                            Expr::Identifier(x) => {
                                let mut x = x.clone();
                                x.ctx = ExprContext::Store;
                                Box::new(Node::node_with_pos(x, expr.pos()))
                            }
                            _ => {
                                self.sess
                                    .struct_token_error(&[TokenKind::ident_value()], self.token);
                                Box::new(expr.into_missing_identifier())
                            }
                        })
                        .collect();
                    Some(Box::new(Node::node_with_pos(
                        Stmt::Assign(AssignStmt {
                            targets: targets.clone(),
                            value: miss_expr,
                            ty,
                        }),
                        pos,
                    )))
                } else {
                    None
                }
            }
        }
    }

    /// Syntax:
    /// assert_stmt: ASSERT simple_expr (IF simple_expr)? (COMMA test)?
    fn parse_assert_stmt(&mut self) -> NodeRef<Stmt> {
        let token = self.token;
        self.bump_keyword(kw::Assert);

        let simple_expr = self.parse_simple_expr();
        let if_cond = if self.token.is_keyword(kw::If) {
            self.bump_keyword(kw::If);
            Some(self.parse_simple_expr())
        } else {
            None
        };

        let msg = if let TokenKind::Comma = self.token.kind {
            self.bump_token(TokenKind::Comma);
            Some(self.parse_expr())
        } else {
            None
        };

        let t = node_ref!(
            Stmt::Assert(AssertStmt {
                test: simple_expr,
                if_cond,
                msg,
            }),
            self.token_span_pos(token, self.prev_token)
        );

        self.skip_newlines();

        t
    }

    /// Syntax:
    /// import_stmt: IMPORT dot_name (AS NAME)?
    fn parse_import_stmt(&mut self) -> NodeRef<Stmt> {
        let token = self.token;
        self.bump_keyword(kw::Import);
        let dot_name_token = self.token;

        let mut leading_dot = Vec::new();
        while let TokenKind::DotDotDot = self.token.kind {
            leading_dot.push("...".to_string());
            self.bump_token(TokenKind::DotDotDot);
        }
        while let TokenKind::Dot = self.token.kind {
            leading_dot.push(".".to_string());
            self.bump_token(TokenKind::Dot);
        }
        let dot_name = self.parse_identifier().node;
        let dot_name_end_token = self.prev_token;

        let asname = if self.token.is_keyword(kw::As) {
            self.bump_keyword(kw::As);
            let ident = self.parse_identifier().node;
            match ident.names.len() {
                1 => Some(ident.names[0].clone()),
                _ => {
                    self.sess
                        .struct_span_error("Invalid import asname", self.token.span);
                    None
                }
            }
        } else {
            None
        };

        let mut path = leading_dot.join("");
        path.push_str(dot_name.get_names().join(".").as_str());

        let rawpath = path.clone();
        let path_node = Node::node_with_pos(
            path,
            self.token_span_pos(dot_name_token, dot_name_end_token),
        );

        let name = if let Some(as_name_value) = asname.clone() {
            as_name_value.node
        } else {
            dot_name.get_names().last().unwrap().clone()
        };

        let t = node_ref!(
            Stmt::Import(ImportStmt {
                path: path_node,
                rawpath,
                name,
                asname,
                pkg_name: String::new()
            }),
            self.token_span_pos(token, self.prev_token)
        );

        self.skip_newlines();

        t
    }

    /// Syntax:
    /// type_alias_stmt: "type" NAME ASSIGN type
    fn parse_type_alias_stmt(&mut self) -> NodeRef<Stmt> {
        self.bump_keyword(kw::Type);

        let type_name_pos = self.token;
        let expr = self.parse_expr();
        let type_name = self.expr_as_identifier(expr, type_name_pos);
        let type_name_end = self.prev_token;

        self.bump_token(TokenKind::Assign);

        let typ_pos = self.token;
        let typ = self.parse_type_annotation();
        let typ_end = self.prev_token;

        self.skip_newlines();

        node_ref!(
            Stmt::TypeAlias(TypeAliasStmt {
                type_name: node_ref!(type_name, self.token_span_pos(type_name_pos, type_name_end)),
                type_value: node_ref!(typ.node.to_string(), self.token_span_pos(typ_pos, typ_end)),
                ty: typ,
            }),
            self.token_span_pos(type_name_pos, typ_end)
        )
    }

    /// Syntax:
    /// if_stmt: IF test COLON execution_block (ELIF test COLON execution_block)* (ELSE COLON execution_block)?
    /// execution_block: if_simple_stmt | NEWLINE _INDENT schema_init_stmt+ _DEDENT
    /// if_simple_stmt: (simple_assign_stmt | unification_stmt | expr_stmt | assert_stmt) NEWLINE
    /// schema_init_stmt: if_simple_stmt | if_stmt
    fn parse_if_stmt(&mut self) -> NodeRef<Stmt> {
        let token = self.token;

        // if
        let mut if_stmt = {
            self.bump_keyword(kw::If);

            let cond = self.parse_expr();

            self.bump_token(TokenKind::Colon);

            let body = if self.token.kind != TokenKind::Newline {
                if let Some(stmt) = self.parse_expr_or_assign_stmt(false) {
                    vec![stmt]
                } else {
                    // Error recovery from panic mode: Once an error is detected (the statement is None).
                    self.bump();
                    vec![]
                }
            } else {
                self.skip_newlines();
                self.parse_block_stmt_list(
                    TokenKind::Indent(VALID_SPACES_LENGTH),
                    TokenKind::Dedent(VALID_SPACES_LENGTH),
                )
            };

            IfStmt {
                body,
                cond,
                orelse: Vec::new(),
            }
        };

        if self.token.kind == TokenKind::Newline {
            self.skip_newlines();
        }

        // elif ...
        let mut elif_list = Vec::new();
        while self.token.is_keyword(kw::Elif) {
            let token = self.token;
            self.bump_keyword(kw::Elif);

            let cond = self.parse_expr();

            self.bump_token(TokenKind::Colon);

            let body = if self.token.kind != TokenKind::Newline {
                if let Some(stmt) = self.parse_expr_or_assign_stmt(false) {
                    vec![stmt]
                } else {
                    // Error recovery from panic mode: Once an error is detected (the statement is None).
                    self.bump();
                    vec![]
                }
            } else {
                self.skip_newlines();
                self.parse_block_stmt_list(
                    TokenKind::Indent(VALID_SPACES_LENGTH),
                    TokenKind::Dedent(VALID_SPACES_LENGTH),
                )
            };

            let t = node_ref!(
                IfStmt {
                    body,
                    cond,
                    orelse: Vec::new(),
                },
                self.token_span_pos(token, self.prev_token)
            );

            elif_list.push(t);
        }

        if self.token.kind == TokenKind::Newline {
            self.skip_newlines();
        }

        // else
        if self.token.is_keyword(kw::Else) {
            self.bump_keyword(kw::Else);

            // `else if -> elif` error recovery.
            if self.token.is_keyword(kw::If) {
                self.sess.struct_span_error(
                    "'else if' here is invalid in KCL, consider using the 'elif' keyword",
                    self.token.span,
                );
            } else if self.token.kind != TokenKind::Colon {
                self.sess
                    .struct_token_error(&[TokenKind::Colon.into()], self.token);
            }
            // Bump colon token.
            self.bump();

            let else_body = if self.token.kind != TokenKind::Newline {
                if let Some(stmt) = self.parse_expr_or_assign_stmt(false) {
                    vec![stmt]
                } else {
                    // Error recovery from panic mode: Once an error is detected (the statement is None).
                    self.bump();
                    vec![]
                }
            } else {
                self.skip_newlines();
                self.parse_block_stmt_list(
                    TokenKind::Indent(VALID_SPACES_LENGTH),
                    TokenKind::Dedent(VALID_SPACES_LENGTH),
                )
            };

            if_stmt.orelse = else_body;
        }

        // fix elif-list and else
        while let Some(mut x) = elif_list.pop() {
            x.node.orelse = if_stmt.orelse.clone();
            let pos = if if_stmt.orelse.is_empty() {
                x.clone().pos()
            } else {
                let start_pos = x.clone().pos();
                let end_pos = if_stmt.orelse.last().unwrap().pos();
                (
                    start_pos.0.clone(),
                    start_pos.1,
                    start_pos.2,
                    end_pos.3,
                    end_pos.4,
                )
            };
            let t = Box::new(Node::node_with_pos(Stmt::If(x.node), pos));
            if_stmt.orelse = vec![t];
        }

        let t = node_ref!(
            Stmt::If(if_stmt),
            self.token_span_pos(token, self.prev_token)
        );

        self.skip_newlines();

        t
    }

    /// Syntax:
    /// schema_stmt: [decorators] (SCHEMA|MIXIN|PROTOCOL) NAME
    ///   [LEFT_BRACKETS [schema_arguments] RIGHT_BRACKETS]
    ///   [LEFT_PARENTHESES identifier (COMMA identifier)* RIGHT_PARENTHESES]
    ///   [for_host] COLON NEWLINE [schema_body]
    fn parse_schema_stmt(&mut self, decorators: Option<Vec<NodeRef<CallExpr>>>) -> NodeRef<Stmt> {
        let token = self.token;

        // schema decorators
        let decorators = match decorators {
            Some(v) => v,
            None => Vec::new(),
        };

        // schema|mixin|protocol
        let mut is_mixin = false;
        let mut is_protocol = false;
        {
            if self.token.is_keyword(kw::Mixin) {
                self.bump_keyword(kw::Mixin);
                is_mixin = true;
            } else if self.token.is_keyword(kw::Protocol) {
                self.bump_keyword(kw::Protocol);
                is_protocol = true;
            } else {
                self.bump_keyword(kw::Schema);
            }
        }

        // schema Name
        let name_expr = self.parse_identifier();
        let name_pos = name_expr.pos();
        let name = name_expr.node;
        let name = node_ref!(name.get_names().join("."), name_pos);

        if name
            .node
            .ends_with(kclvm_sema::resolver::global::MIXIN_SUFFIX)
        {
            is_mixin = true;
        } else if name
            .node
            .ends_with(kclvm_sema::resolver::global::PROTOCOL_SUFFIX)
        {
            is_protocol = true;
        }

        // schema Name[args...]
        let args = if let TokenKind::OpenDelim(DelimToken::Bracket) = self.token.kind {
            self.parse_parameters(
                &[TokenKind::OpenDelim(DelimToken::Bracket)],
                &[TokenKind::CloseDelim(DelimToken::Bracket)],
                true,
            )
        } else {
            None
        };

        // schema Name [args...](Base)
        let parent_name = if let TokenKind::OpenDelim(DelimToken::Paren) = self.token.kind {
            self.bump_token(TokenKind::OpenDelim(DelimToken::Paren));
            let expr = self.parse_identifier();
            let expr_pos = expr.pos();
            let base_schema_name = expr.node;
            self.bump_token(TokenKind::CloseDelim(DelimToken::Paren));
            Some(node_ref!(base_schema_name, expr_pos))
        } else {
            None
        };

        // schema Name [args...](Base) for SomeProtocol
        let for_host_name = if self.token.is_keyword(kw::For) {
            self.bump_keyword(kw::For);
            let token = self.token;
            let expr = self.parse_expr();
            let expr_pos = expr.pos();
            let ident = self.expr_as_identifier(expr, token);
            Some(node_ref!(ident, expr_pos))
        } else {
            None
        };

        self.bump_token(TokenKind::Colon);

        self.skip_newlines();

        if let TokenKind::Indent(VALID_SPACES_LENGTH) = self.token.kind {
            let body = self.parse_schema_body();

            let mut pos = self.token_span_pos(token, self.prev_token);
            // Insert a fake newline character to expand the scope of the schema，
            // used to complete the schema attributes at the end of the file
            // FIXME: fix in lsp
            if let TokenKind::Eof = self.prev_token.kind {
                pos.3 += 1;
                pos.4 = 0;
            }

            node_ref!(
                Stmt::Schema(SchemaStmt {
                    doc: body.doc,
                    name,
                    parent_name,
                    for_host_name,
                    is_mixin,
                    is_protocol,
                    args,
                    mixins: body.mixins,
                    body: body.body,
                    decorators,
                    checks: body.checks,
                    index_signature: body.index_signature,
                }),
                pos
            )
        } else {
            let pos = self.token_span_pos(token, self.prev_token);
            node_ref!(
                Stmt::Schema(SchemaStmt {
                    doc: None,
                    name,
                    parent_name,
                    for_host_name,
                    is_mixin,
                    is_protocol,
                    args,
                    mixins: vec![],
                    body: vec![],
                    decorators,
                    checks: vec![],
                    index_signature: None,
                }),
                pos
            )
        }
    }

    /// Syntax:
    /// decorators: (AT decorator_expr NEWLINE)+
    fn parse_decorators(&mut self) -> Vec<NodeRef<CallExpr>> {
        let mut decorators = Vec::new();
        while let TokenKind::At = self.token.kind {
            self.bump_token(TokenKind::At);

            let expr = self.parse_expr();
            let expr_pos = expr.pos();
            match expr.node {
                Expr::Identifier(x) => {
                    decorators.push(node_ref!(
                        CallExpr {
                            func: node_ref!(Expr::Identifier(x), expr_pos.clone()),
                            args: Vec::new(),
                            keywords: Vec::new(),
                        },
                        expr_pos
                    ));
                }
                Expr::Call(x) => {
                    decorators.push(node_ref!(x, expr_pos));
                }
                _ => self
                    .sess
                    .struct_token_error(&[TokenKind::ident_value()], self.token),
            };

            self.skip_newlines();
        }

        self.skip_newlines();
        decorators
    }

    /// Syntax:
    /// schema_arguments: schema_argument (COMMA schema_argument)*
    /// schema_argument: NAME [COLON type] [ASSIGN test]
    pub(crate) fn parse_parameters(
        &mut self,
        open_tokens: &[TokenKind],
        close_tokens: &[TokenKind],
        bump_close: bool,
    ) -> Option<NodeRef<Arguments>> {
        let mut has_open_token = false;

        let token = self.token;

        for token in open_tokens {
            if *token == self.token.kind {
                self.bump_token(*token);
                has_open_token = true;
                break;
            }
        }
        if !open_tokens.is_empty() && !has_open_token {
            return None;
        }
        let mut args = Arguments {
            args: Vec::new(),
            defaults: Vec::new(),
            ty_list: Vec::new(),
        };

        loop {
            let marker = self.mark();

            let mut has_close_token = false;
            for token in close_tokens {
                if *token == self.token.kind {
                    if bump_close {
                        self.bump_token(*token);
                    }
                    has_close_token = true;
                    break;
                }
            }
            if has_close_token {
                break;
            }

            if matches!(self.token.kind, TokenKind::Newline | TokenKind::Eof) {
                let expect_tokens: Vec<String> = close_tokens
                    .iter()
                    .map(|t| <kclvm_ast::token::TokenKind as Into<String>>::into(*t))
                    .collect();
                self.sess.struct_token_error(&expect_tokens, self.token);
                break;
            }

            let name_pos = self.token;
            let name = self.parse_identifier().node;
            let name_end = self.prev_token;

            let name = node_ref!(name, self.token_span_pos(name_pos, name_end));

            let type_annotation_node = if let TokenKind::Colon = self.token.kind {
                self.bump_token(TokenKind::Colon);
                let typ = self.parse_type_annotation();
                Some(typ)
            } else {
                None
            };

            let default_value = if let TokenKind::Assign = self.token.kind {
                self.bump_token(TokenKind::Assign);
                Some(self.parse_expr())
            } else {
                None
            };

            args.args.push(name);
            args.ty_list.push(type_annotation_node);
            args.defaults.push(default_value);
            // Parameter interval comma
            if let TokenKind::Comma = self.token.kind {
                self.bump();
            }

            self.drop(marker);
        }

        self.skip_newlines();

        Some(node_ref!(args, self.token_span_pos(token, self.prev_token)))
    }

    /// Syntax:
    /// schema_body: _INDENT (string NEWLINE)* [mixin_stmt]
    ///   (schema_attribute_stmt|schema_init_stmt|schema_index_signature)*
    ///   [check_block] _DEDENT
    ///
    /// schema_attribute_stmt: attribute_stmt NEWLINE
    /// attribute_stmt: [decorators] NAME [QUESTION] COLON type [(ASSIGN|COMP_OR) test]
    ///
    /// schema_init_stmt: if_simple_stmt | if_stmt
    ///   if_stmt: IF test COLON execution_block (ELIF test COLON execution_block)* (ELSE COLON execution_block)?
    ///     execution_block: if_simple_stmt | NEWLINE _INDENT schema_init_stmt+ _DEDENT
    ///   if_simple_stmt: (simple_assign_stmt | unification_stmt | expr_stmt | assert_stmt) NEWLINE
    ///
    /// schema_index_signature:
    ///   LEFT_BRACKETS [NAME COLON] [ELLIPSIS] basic_type RIGHT_BRACKETS
    ///   COLON type [ASSIGN test] NEWLINE
    fn parse_schema_body(&mut self) -> SchemaStmt {
        self.validate_dedent();
        self.bump_token(TokenKind::Indent(VALID_SPACES_LENGTH));

        // doc string when it is not a string-like attribute statement.
        let body_doc = if let Some(peek) = self.cursor.peek() {
            if matches!(peek.kind, TokenKind::Colon) {
                None
            } else {
                self.parse_doc()
            }
        } else {
            self.parse_doc()
        };

        // mixin
        let body_mixins = if self.token.is_keyword(kw::Mixin) {
            let mixins = self.parse_mixins();
            self.skip_newlines();
            mixins
        } else {
            Vec::new()
        };

        // body
        let mut body_body = Vec::new();
        let mut body_index_signature = None;

        loop {
            let marker = self.mark();
            self.validate_dedent();
            if matches!(
                self.token.kind,
                TokenKind::Dedent(VALID_SPACES_LENGTH) | TokenKind::Eof
            ) || self.token.is_keyword(kw::Check)
            {
                break;
            }
            // assert stmt
            else if self.token.is_keyword(kw::Assert) {
                body_body.push(self.parse_assert_stmt());
                continue;
            }
            // if stmt
            else if self.token.is_keyword(kw::If) {
                body_body.push(self.parse_if_stmt());
                continue;
            }
            // schema_attribute_stmt: string COLON type_annotation
            else if self.token.is_string_lit() {
                if let Some(peek) = self.cursor.peek() {
                    if let TokenKind::Colon = peek.kind {
                        let token = self.token;
                        let attr = self.parse_schema_attribute();
                        body_body.push(node_ref!(
                            Stmt::SchemaAttr(attr),
                            self.token_span_pos(token, self.prev_token)
                        ));
                        continue;
                    }
                }
            }
            // schema_attribute_stmt
            else if let TokenKind::At = self.token.kind {
                let token = self.token;
                let attr = self.parse_schema_attribute();
                body_body.push(node_ref!(
                    Stmt::SchemaAttr(attr),
                    self.token_span_pos(token, self.prev_token)
                ));
                continue;
            }
            if let Some(peek) = self.cursor.peek() {
                if let TokenKind::Question = peek.kind {
                    let token = self.token;
                    let attr = self.parse_schema_attribute();
                    body_body.push(node_ref!(
                        Stmt::SchemaAttr(attr),
                        self.token_span_pos(token, self.prev_token)
                    ));
                    continue;
                }
            }

            // schema_index_signature or list
            if let TokenKind::OpenDelim(DelimToken::Bracket) = self.token.kind {
                let token = self.token;

                let (index_sig, or_list_expr) = self.parse_schema_index_signature_or_list();

                if let Some(x) = index_sig {
                    if body_index_signature.is_some() {
                        self.sess.struct_span_error(
                            "duplicate schema index signature definitions, only one is allowed in the schema",
                            token.span,
                        );
                    }
                    body_index_signature =
                        Some(node_ref!(x, self.token_span_pos(token, self.prev_token)));
                } else if let Some(list_expr) = or_list_expr {
                    let stmt = Stmt::Expr(ExprStmt {
                        exprs: vec![list_expr],
                    });
                    body_body.push(node_ref!(stmt, self.token_span_pos(token, self.prev_token)));
                } else {
                    self.sess.struct_span_error(
                        &format!(
                            "expected a index signature or list expression here, got {}",
                            Into::<String>::into(self.token)
                        ),
                        self.token.span,
                    );
                }

                self.skip_newlines();
                continue;
            }

            // expr or attr
            if let Some(x) = self.parse_expr_or_assign_stmt(true) {
                if let Stmt::SchemaAttr(attr) = &x.node {
                    body_body.push(node_ref!(Stmt::SchemaAttr(attr.clone()), x.pos()));
                    continue;
                }

                if let Stmt::Assign(assign) = x.node.clone() {
                    if assign.targets.len() == 1 {
                        let ident = assign.targets[0].clone().node;
                        if assign.ty.is_some() {
                            body_body.push(node_ref!(
                                Stmt::SchemaAttr(SchemaAttr {
                                    doc: "".to_string(),
                                    name: node_ref!(
                                        ident.get_names().join("."),
                                        assign.targets[0].pos()
                                    ),
                                    ty: assign.ty.unwrap(),
                                    op: Some(AugOp::Assign),
                                    value: Some(assign.value),
                                    is_optional: false,
                                    decorators: Vec::new(),
                                }),
                                x.pos()
                            ));
                            continue;
                        };
                    }
                }

                body_body.push(x);
            } else {
                // Error recovery from panic mode: Once an error is detected (the statement is None).
                self.bump();
            }
            self.drop(marker);
        }

        // check_block
        let body_checks = self.parse_schema_check_block();
        self.validate_dedent();
        self.bump_token(TokenKind::Dedent(VALID_SPACES_LENGTH));
        self.skip_newlines();

        SchemaStmt {
            doc: body_doc,
            mixins: body_mixins,
            body: body_body,
            checks: body_checks,
            index_signature: body_index_signature,

            name: Box::new(Node {
                id: AstIndex::default(),
                node: "".to_string(),
                filename: "".to_string(),
                line: 0,
                column: 0,
                end_line: 0,
                end_column: 0,
            }),
            parent_name: None,
            for_host_name: None,
            is_mixin: false,
            is_protocol: false,
            args: None,
            decorators: Vec::new(),
        }
    }

    /// Syntax:
    /// mixin_stmt: MIXIN LEFT_BRACKETS [mixins | multiline_mixins] RIGHT_BRACKETS NEWLINE
    /// multiline_mixins: NEWLINE _INDENT mixins NEWLINE _DEDENT
    /// mixins: identifier (COMMA (NEWLINE mixins | identifier))*
    fn parse_mixins(&mut self) -> Vec<NodeRef<Identifier>> {
        self.bump_keyword(kw::Mixin);

        let mut mixins = Vec::new();

        self.bump_token(TokenKind::OpenDelim(DelimToken::Bracket));

        // NEWLINE _INDENT
        let has_newline = if self.token.kind == TokenKind::Newline {
            self.skip_newlines();
            if self.token.kind == TokenKind::Indent(VALID_SPACES_LENGTH) {
                self.bump();
            } else {
                self.sess.struct_token_error(
                    &[TokenKind::Indent(VALID_SPACES_LENGTH).into()],
                    self.token,
                )
            }
            true
        } else {
            false
        };

        loop {
            self.validate_dedent();
            if matches!(
                self.token.kind,
                TokenKind::CloseDelim(DelimToken::Bracket) | TokenKind::Dedent(VALID_SPACES_LENGTH)
            ) {
                break;
            }
            let expr = self.parse_identifier();
            let expr_pos = expr.pos();
            let ident = expr.node;
            mixins.push(node_ref!(ident, expr_pos));
            if let TokenKind::Comma = self.token.kind {
                self.bump();
            }
            if let TokenKind::Newline = self.token.kind {
                self.skip_newlines()
            }
        }

        // _DEDENT
        if has_newline {
            self.validate_dedent();
            if self.token.kind == TokenKind::Dedent(VALID_SPACES_LENGTH) {
                self.bump();
            } else {
                self.sess.struct_token_error(
                    &[TokenKind::Dedent(VALID_SPACES_LENGTH).into()],
                    self.token,
                )
            }
        }

        self.bump_token(TokenKind::CloseDelim(DelimToken::Bracket));

        mixins
    }

    /// Syntax:
    /// schema_attribute_stmt: attribute_stmt NEWLINE
    /// attribute_stmt: [decorators] (identifier | string) [QUESTION] COLON type [(ASSIGN|COMP_OR) test]
    fn parse_schema_attribute(&mut self) -> SchemaAttr {
        let doc = "".to_string();

        // @decorators
        let decorators = if matches!(self.token.kind, TokenKind::At) {
            let decorators = self.parse_decorators();
            self.skip_newlines();
            decorators
        } else {
            Vec::new()
        };

        // Parse schema identifier-like or string-like attributes
        let name = if let Some(name) = self.parse_string_attribute() {
            name
        } else {
            let name_expr = self.parse_identifier();
            let name_pos = name_expr.pos();
            let name = name_expr.node;
            node_ref!(name.get_names().join("."), name_pos)
        };

        // Parse attribute optional annotation `?`
        let is_optional = if let TokenKind::Question = self.token.kind {
            self.bump_token(TokenKind::Question);
            true
        } else {
            false
        };

        // Bump the schema attribute annotation token `:`
        self.bump_token(TokenKind::Colon);

        // Parse the schema attribute type annotation.
        let ty = self.parse_type_annotation();

        let op = if self.token.kind == TokenKind::Assign {
            self.bump_token(TokenKind::Assign);
            Some(AugOp::Assign)
        } else if let TokenKind::BinOpEq(x) = self.token.kind {
            self.bump_token(self.token.kind);
            Some(x.into())
        } else {
            None
        };

        let value = if op.is_some() {
            Some(self.parse_expr())
        } else {
            None
        };
        self.skip_newlines();
        SchemaAttr {
            doc,
            name,
            ty,
            op,
            value,
            is_optional,
            decorators,
        }
    }

    /// Syntax:
    /// schema_index_signature:
    ///   LEFT_BRACKETS [NAME COLON] [ELLIPSIS] basic_type RIGHT_BRACKETS
    ///   COLON type [ASSIGN test] NEWLINE
    fn parse_schema_index_signature_or_list(
        &mut self,
    ) -> (Option<SchemaIndexSignature>, Option<NodeRef<Expr>>) {
        self.bump_token(TokenKind::OpenDelim(DelimToken::Bracket));
        let peek_token_kind = match self.cursor.peek() {
            Some(token) => token.kind,
            None => TokenKind::Eof,
        };
        let peek2_token_kind = match self.cursor.peek2() {
            Some(token) => token.kind,
            None => TokenKind::Eof,
        };
        match (self.token.kind, peek_token_kind) {
            // Maybe a list expr or a list comp.
            (TokenKind::Newline | TokenKind::CloseDelim(DelimToken::Bracket), _) => {
                (None, Some(self.parse_list_expr(false)))
            }
            (TokenKind::Ident(_), _)
                if !matches!(peek_token_kind, TokenKind::Colon)
                    && !matches!(peek2_token_kind, TokenKind::Colon) =>
            {
                (None, Some(self.parse_list_expr(false)))
            }
            // Maybe a schema index signature.
            (TokenKind::DotDotDot, _) if matches!(peek_token_kind, TokenKind::Ident(_)) => {
                (Some(self.parse_schema_index_signature()), None)
            }
            (TokenKind::Ident(_), _) => (Some(self.parse_schema_index_signature()), None),
            _ => (None, None),
        }
    }

    /// Syntax:
    /// schema_index_signature:
    ///   LEFT_BRACKETS [NAME COLON] [ELLIPSIS] basic_type RIGHT_BRACKETS
    ///   COLON type [ASSIGN test] NEWLINE
    fn parse_schema_index_signature(&mut self) -> SchemaIndexSignature {
        let (key_name, key_ty, any_other) = if matches!(self.token.kind, TokenKind::DotDotDot) {
            // bump token `...`
            self.bump();
            (None, self.parse_type_annotation(), true)
        } else {
            let token = self.token;
            let expr = self.parse_identifier_expr();
            let ident = self.expr_as_identifier(expr.clone(), token);
            let pos = expr.pos();
            let key_name = ident.get_names().join(".");

            if let TokenKind::CloseDelim(DelimToken::Bracket) = self.token.kind {
                (None, node_ref!(Type::Named(ident), pos), false)
            } else {
                self.bump_token(TokenKind::Colon);
                let any_other = if let TokenKind::DotDotDot = self.token.kind {
                    self.bump();
                    true
                } else {
                    false
                };
                (Some(key_name), self.parse_type_annotation(), any_other)
            }
        };

        self.bump_token(TokenKind::CloseDelim(DelimToken::Bracket));
        self.bump_token(TokenKind::Colon);

        let value_ty = self.parse_type_annotation();

        let value = if let TokenKind::Assign = self.token.kind {
            self.bump();
            Some(self.parse_expr())
        } else {
            None
        };

        self.skip_newlines();

        SchemaIndexSignature {
            key_name,
            key_ty,
            value_ty,
            value,
            any_other,
        }
    }

    /// Syntax:
    /// check_block: CHECK COLON NEWLINE _INDENT check_expr+ _DEDENT
    /// check_expr: simple_expr [IF simple_expr] [COMMA primary_expr] NEWLINE
    fn parse_schema_check_block(&mut self) -> Vec<NodeRef<CheckExpr>> {
        let mut check_expr_list = Vec::new();

        if self.token.is_keyword(kw::Check) {
            self.bump_keyword(kw::Check);
            self.bump_token(TokenKind::Colon);
            self.skip_newlines();
            self.bump_token(TokenKind::Indent(VALID_SPACES_LENGTH));
            while self.token.kind != TokenKind::Dedent(VALID_SPACES_LENGTH) {
                let marker = self.mark();
                if matches!(self.token.kind, TokenKind::Eof) {
                    self.sess
                        .struct_token_error(&[TokenKind::Newline.into()], self.token);
                    break;
                }

                let expr = self.parse_check_expr();
                let expr_pos = expr.pos();
                let check_expr = expr_as!(expr, Expr::Check).unwrap();
                check_expr_list.push(node_ref!(check_expr, expr_pos));

                self.skip_newlines();
                self.drop(marker);
            }
            self.validate_dedent();
            self.bump_token(TokenKind::Dedent(VALID_SPACES_LENGTH));
        }

        check_expr_list
    }

    /// Syntax:
    /// rule_stmt: [decorators] RULE NAME [LEFT_BRACKETS [schema_arguments] RIGHT_BRACKETS] [LEFT_PARENTHESES identifier (COMMA identifier)* RIGHT_PARENTHESES] [for_host] COLON NEWLINE [rule_body]
    /// rule_body: _INDENT (string NEWLINE)* check_expr+ _DEDENT
    fn parse_rule_stmt(&mut self, decorators: Option<Vec<NodeRef<CallExpr>>>) -> NodeRef<Stmt> {
        let token = self.token;
        self.bump_keyword(kw::Rule);

        let decorators = if let Some(x) = decorators {
            x
        } else {
            Vec::new()
        };

        let name_expr = self.parse_identifier_expr();
        let name_pos = name_expr.pos();
        let name = expr_as!(name_expr, Expr::Identifier).unwrap();
        let name = node_ref!(name.get_names().join("."), name_pos);

        let args = if let TokenKind::OpenDelim(DelimToken::Bracket) = self.token.kind {
            self.parse_parameters(
                &[TokenKind::OpenDelim(DelimToken::Bracket)],
                &[TokenKind::CloseDelim(DelimToken::Bracket)],
                true,
            )
        } else {
            None
        };

        let mut parent_rules = vec![];
        if self.token.kind == TokenKind::OpenDelim(DelimToken::Paren) {
            self.bump();
            loop {
                if let TokenKind::CloseDelim(DelimToken::Paren) = self.token.kind {
                    self.bump();
                    break;
                }

                if matches!(self.token.kind, TokenKind::Newline | TokenKind::Eof) {
                    self.sess.struct_token_error(
                        &[
                            TokenKind::CloseDelim(DelimToken::Paren).into(),
                            TokenKind::Comma.into(),
                        ],
                        self.token,
                    );
                    break;
                }

                let token = self.token;
                let expr = self.parse_expr();
                let expr_pos = expr.pos();
                let rule_name = self.expr_as_identifier(expr, token);
                parent_rules.push(node_ref!(rule_name, expr_pos));

                if !matches!(
                    self.token.kind,
                    TokenKind::Comma
                        | TokenKind::CloseDelim(DelimToken::Paren)
                        | TokenKind::Newline
                        | TokenKind::Eof
                ) {
                    self.sess.struct_token_error(
                        &[
                            TokenKind::Comma.into(),
                            TokenKind::CloseDelim(DelimToken::Paren).into(),
                        ],
                        self.token,
                    );
                    self.bump();
                }

                if matches!(self.token.kind, TokenKind::Comma) {
                    self.bump()
                }
            }
        }

        let for_host_name = if self.token.is_keyword(kw::For) {
            self.bump_keyword(kw::For);
            let token = self.token;
            let expr = self.parse_expr();
            let expr_pos = expr.pos();
            let ident = self.expr_as_identifier(expr, token);
            Some(node_ref!(ident, expr_pos))
        } else {
            None
        };

        self.bump_token(TokenKind::Colon);
        self.skip_newlines();
        self.bump_token(TokenKind::Indent(VALID_SPACES_LENGTH));

        // doc string
        let body_doc = match self.token.kind {
            TokenKind::Literal(lit) => {
                if let LitKind::Str { .. } = lit.kind {
                    let doc_expr = self.parse_str_expr(lit);
                    self.skip_newlines();
                    match &doc_expr.node {
                        Expr::StringLit(str) => {
                            Some(node_ref!(str.raw_value.clone(), doc_expr.pos()))
                        }
                        Expr::JoinedString(str) => {
                            Some(node_ref!(str.raw_value.clone(), doc_expr.pos()))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        let mut check_expr_list = vec![];
        self.validate_dedent();
        while self.token.kind != TokenKind::Dedent(VALID_SPACES_LENGTH) {
            let marker = self.mark();
            if matches!(self.token.kind, TokenKind::Eof) {
                self.sess
                    .struct_token_error(&[TokenKind::Newline.into()], self.token);
                break;
            }

            let expr = self.parse_check_expr();
            let expr_pos = expr.pos();
            let check_expr = expr_as!(expr, Expr::Check).unwrap();
            check_expr_list.push(node_ref!(check_expr, expr_pos));

            self.skip_newlines();
            self.drop(marker);
        }
        self.validate_dedent();
        self.bump_token(TokenKind::Dedent(VALID_SPACES_LENGTH));

        let pos = self.token_span_pos(token, self.prev_token);

        node_ref!(
            Stmt::Rule(RuleStmt {
                doc: body_doc,
                name,
                parent_rules,
                decorators,
                checks: check_expr_list,
                args,
                for_host_name,
            }),
            pos
        )
    }

    pub(crate) fn parse_string_attribute(&mut self) -> Option<NodeRef<String>> {
        match self.token.kind {
            TokenKind::Literal(lit) => {
                if let LitKind::Str { .. } = lit.kind {
                    let str_expr = self.parse_str_expr(lit);
                    match &str_expr.node {
                        Expr::StringLit(str) => Some(node_ref!(str.value.clone(), str_expr.pos())),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn parse_joined_string(
        &mut self,
        s: &StringLit,
        pos: BytePos,
    ) -> Option<JoinedString> {
        // skip raw string
        if s.raw_value.starts_with(['r', 'R']) {
            return None;
        }
        if !s.value.contains("${") {
            return None;
        }

        let start_pos = if s.is_long_string {
            pos + new_byte_pos(3)
        } else {
            pos + new_byte_pos(1)
        };

        let mut joined_value = JoinedString {
            is_long_string: s.is_long_string,
            raw_value: s.raw_value.clone(),
            values: Vec::new(),
        };

        fn parse_expr(this: &mut Parser, src: &str, start_pos: BytePos) -> NodeRef<Expr> {
            use crate::lexer::parse_token_streams;
            // The string interpolation end pos.
            let end_pos = start_pos + new_byte_pos(src.len() as u32);
            // Skip the start '${' and end '}'
            let src = &src[2..src.len() - 1];
            if src.is_empty() {
                this.sess.struct_span_error(
                    "string interpolation expression can not be empty",
                    Span::new(start_pos, end_pos),
                );
            }

            // Expression start pos, and skip the start '${'.
            let start_pos = start_pos + new_byte_pos(2);

            let stream = parse_token_streams(this.sess, src, start_pos);

            let mut parser = Parser {
                token: Token::dummy(),
                prev_token: Token::dummy(),
                cursor: stream.cursor(),
                comments: Vec::new(),
                sess: this.sess,
            };

            // bump to the first token
            parser.bump();

            let expr = parser.parse_expr();

            let mut formatted_value = FormattedValue {
                is_long_string: false,
                value: expr,
                format_spec: None,
            };

            if let TokenKind::Colon = parser.token.kind {
                // bump the format spec interval token `:`.
                parser.bump();
                if let TokenKind::DocComment(CommentKind::Line(symbol)) = parser.token.kind {
                    formatted_value.format_spec = Some(symbol.as_str());
                } else {
                    this.sess.struct_span_error(
                        "invalid joined string spec without #",
                        parser.token.span,
                    );
                }
                // Whether there is syntax error or not, bump the joined string spec token.
                parser.bump();
            }

            // The token pair (lo, hi).
            let lo = start_pos;
            let hi = start_pos + new_byte_pos(src.len() as u32);
            // Bump the expression endline.
            parser.skip_newlines();
            // If there are still remaining tokens, it indicates that an
            // unexpected expression has occurred here.
            if !src.is_empty() && parser.has_next() {
                parser.sess.struct_span_error(
                    &format!("invalid string interpolation expression: '{src}'"),
                    Span::new(lo, hi),
                )
            }

            node_ref!(
                Expr::FormattedValue(formatted_value),
                parser.byte_pos_to_pos(lo, hi)
            )
        }

        let data = s.value.as_str();
        let mut off: usize = 0;
        loop {
            if let Some(i) = data[off..].find("${") {
                if let Some(j) = data[off + i..].find('}') {
                    let lo: usize = off + i;
                    let hi: usize = off + i + j + 1;

                    let s0 = &data[off..lo];
                    let s1 = &data[lo..hi];

                    let s0_expr = node_ref!(Expr::StringLit(StringLit {
                        is_long_string: false,
                        raw_value: s0.to_string(),
                        value: s0.to_string().replace("$$", "$"),
                    }));

                    let s1_expr = parse_expr(self, s1, start_pos + new_byte_pos(lo as u32));

                    if !s0.is_empty() {
                        joined_value.values.push(s0_expr);
                    }
                    joined_value.values.push(s1_expr);

                    off = hi;
                    continue;
                } else {
                    self.sess
                        .struct_span_error("invalid joined string", self.token.span);
                    joined_value
                        .values
                        .push(node_ref!(Expr::StringLit(StringLit {
                            is_long_string: false,
                            raw_value: data[off..].to_string(),
                            value: data[off..].to_string(),
                        })));
                    break;
                }
            } else {
                if off >= s.value.as_str().len() {
                    break;
                }

                // todo: fix pos
                joined_value
                    .values
                    .push(node_ref!(Expr::StringLit(StringLit {
                        is_long_string: false,
                        raw_value: data[off..].to_string(),
                        value: data[off..].to_string().replace("$$", "$"),
                    })));
                break;
            }
        }

        Some(joined_value)
    }
}
