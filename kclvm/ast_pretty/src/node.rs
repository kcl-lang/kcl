use std::collections::HashSet;

use compiler_base_macros::bug;
use kclvm_ast::{
    ast::{self, CallExpr},
    token::{DelimToken, TokenKind},
    walker::MutSelfTypedResultWalker,
};

use super::{Indentation, Printer};

type ParameterType<'a> = (
    (&'a ast::NodeRef<ast::Identifier>, Option<String>),
    &'a Option<ast::NodeRef<ast::Expr>>,
);

const COMMA_WHITESPACE: &str = ", ";
const IDENTIFIER_REGEX: &str = r#"^\$?[a-zA-Z_]\w*$"#;

macro_rules! interleave {
    ($inter: expr, $f: expr, $seq: expr) => {
        if !$seq.is_empty() {
            $f(&$seq[0]);
            for s in &$seq[1..] {
                $inter();
                $f(s);
            }
        }
    };
}

impl<'p, 'ctx> MutSelfTypedResultWalker<'ctx> for Printer<'p> {
    type Result = ();

    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result {
        for comment in &module.comments {
            self.comments.push_back(comment.clone());
        }
        if let Some(doc) = &module.doc {
            self.write(&doc.node);
            self.write_newline();
        }

        self.stmts(&module.body);
    }

    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        interleave!(
            || self.write(COMMA_WHITESPACE),
            |expr| self.expr(expr),
            expr_stmt.exprs
        );
        self.write_newline_without_fill();
    }

    fn walk_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result {
        self.walk_identifier(&unification_stmt.target.node);
        self.write(": ");
        self.walk_schema_expr(&unification_stmt.value.node);
        self.write_newline_without_fill();
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        self.write("type");
        self.write_space();
        self.walk_identifier(&type_alias_stmt.type_name.node);
        self.write(" = ");
        self.write(&type_alias_stmt.type_value.node);
        self.write_newline_without_fill();
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        for (i, target) in assign_stmt.targets.iter().enumerate() {
            self.walk_identifier(&target.node);
            if i == 0 {
                if let Some(ty) = &assign_stmt.ty {
                    self.write(": ");
                    self.write(&ty.node.to_string());
                }
            }
            self.write(" = ");
        }
        self.expr(&assign_stmt.value);
        self.write_newline_without_fill();
        if matches!(assign_stmt.value.node, ast::Expr::Schema(_)) {
            self.write_newline_without_fill();
        }
    }

    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        self.walk_identifier(&aug_assign_stmt.target.node);
        self.write_space();
        self.write(aug_assign_stmt.op.symbol());
        self.write_space();
        self.expr(&aug_assign_stmt.value);
        self.write_newline_without_fill();
    }

    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        self.write("assert ");
        self.expr(&assert_stmt.test);
        if let Some(if_cond) = &assert_stmt.if_cond {
            self.write(" if ");
            self.expr(if_cond);
        }
        if let Some(msg) = &assert_stmt.msg {
            self.write(COMMA_WHITESPACE);
            self.expr(msg);
        }
        self.write_newline_without_fill();
    }

    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        self.write("if ");
        self.expr(&if_stmt.cond);
        self.write_token(TokenKind::Colon);
        self.write_newline_without_fill();
        self.write_indentation(Indentation::Indent);
        self.stmts(&if_stmt.body);
        self.write_indentation(Indentation::Dedent);

        if !if_stmt.orelse.is_empty() {
            // Check if orelse contains exactly one if statement
            if if_stmt.orelse.len() == 1 {
                if let ast::Stmt::If(elif_stmt) = &if_stmt.orelse[0].node {
                    // Nested if statements need to be considered,
                    // so `el` needs to be preceded by the current indentation.
                    self.fill("el");
                    self.walk_if_stmt(elif_stmt);
                } else {
                    self.fill("else:");
                    self.write_newline_without_fill();
                    self.write_indentation(Indentation::Indent);
                    self.stmts(&if_stmt.orelse);
                    self.write_indentation(Indentation::Dedent);
                }
            } else {
                // Handle multiple else statements
                self.fill("else:");
                self.write_newline_without_fill();
                self.write_indentation(Indentation::Indent);
                self.stmts(&if_stmt.orelse);
                self.write_indentation(Indentation::Dedent);
            }
        } else {
            self.write_newline_without_fill();
        }
    }

    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        self.write("import ");
        self.write(&import_stmt.path.node);
        if let Some(as_name) = &import_stmt.asname {
            self.write(" as ");
            self.write(&as_name.node);
        }
        self.write_newline_without_fill();
    }

    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        interleave!(
            || self.write_newline(),
            |expr: &ast::NodeRef<CallExpr>| {
                self.write("@");
                self.walk_call_expr(&expr.node);
            },
            schema_stmt.decorators
        );
        if !schema_stmt.decorators.is_empty() {
            self.write_newline();
        }
        if schema_stmt.is_mixin {
            self.write("mixin ");
        } else if schema_stmt.is_protocol {
            self.write("protocol ");
        } else {
            self.write("schema ");
        }
        self.write(&schema_stmt.name.node);
        if let Some(args) = &schema_stmt.args {
            self.write("[");
            self.walk_arguments(&args.node);
            self.write("]");
        }
        if let Some(parent_name) = &schema_stmt.parent_name {
            self.write("(");
            self.walk_identifier(&parent_name.node);
            self.write(")");
        }
        if let Some(host_name) = &schema_stmt.for_host_name {
            self.write(" for ");
            self.walk_identifier(&host_name.node);
        }
        self.write_token(TokenKind::Colon);
        self.write_newline_without_fill();
        self.write_indentation(Indentation::Indent);

        if let Some(doc) = &schema_stmt.doc {
            self.fill("");
            self.write(&doc.node);
            self.write_newline_without_fill();
        }

        if !schema_stmt.mixins.is_empty() {
            self.fill("");
            self.write("mixin [");
            self.write_indentation(Indentation::IndentWithNewline);
            interleave!(
                || {
                    self.write(",");
                    self.write_newline();
                },
                |mixin_name: &ast::NodeRef<ast::Identifier>| self.walk_identifier(&mixin_name.node),
                schema_stmt.mixins
            );
            self.write_indentation(Indentation::Dedent);
            self.write_newline();
            self.write("]");
            self.write_newline_without_fill();
        }
        if let Some(index_signature) = &schema_stmt.index_signature {
            self.fill("");
            self.write_token(TokenKind::OpenDelim(DelimToken::Bracket));
            if index_signature.node.any_other {
                self.write_token(TokenKind::DotDotDot);
            }
            if let Some(key_name) = &index_signature.node.key_name {
                self.write(&format!("{}: ", key_name));
            }
            self.write(&index_signature.node.key_ty.node.to_string());
            self.write_token(TokenKind::CloseDelim(DelimToken::Bracket));
            self.write_token(TokenKind::Colon);
            self.write_space();
            self.write(&index_signature.node.value_ty.node.to_string());
            if let Some(value) = &index_signature.node.value {
                self.write(" = ");
                self.expr(value);
            }
            self.write_newline_without_fill();
        }
        self.stmts(&schema_stmt.body);
        self.write_newline_without_fill();
        if !schema_stmt.checks.is_empty() {
            self.fill("check:");
            // Schema check indent
            self.write_indentation(Indentation::IndentWithNewline);
            interleave!(
                || self.write_newline(),
                |check_expr: &ast::NodeRef<ast::CheckExpr>| self.walk_check_expr(&check_expr.node),
                schema_stmt.checks
            );
            self.write_newline_without_fill();
            // Schema check dedent
            self.write_indentation(Indentation::Dedent);
            self.write_newline_without_fill();
        }
        // Schema Stmt dedent
        self.write_indentation(Indentation::Dedent);
    }

    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        interleave!(
            || self.write_newline(),
            |expr: &ast::NodeRef<CallExpr>| {
                self.write("@");
                self.walk_call_expr(&expr.node);
            },
            rule_stmt.decorators
        );
        if !rule_stmt.decorators.is_empty() {
            self.write_newline();
        }
        self.write("rule ");
        self.write(&rule_stmt.name.node);
        if let Some(args) = &rule_stmt.args {
            self.write("[");
            self.walk_arguments(&args.node);
            self.write("]");
        }
        if !rule_stmt.parent_rules.is_empty() {
            self.write("(");
            interleave!(
                || self.write(COMMA_WHITESPACE),
                |identifier: &ast::NodeRef<ast::Identifier>| self.walk_identifier(&identifier.node),
                rule_stmt.parent_rules
            );
            self.write(")");
        }
        if let Some(host_name) = &rule_stmt.for_host_name {
            self.write(" for ");
            self.walk_identifier(&host_name.node);
        }
        self.write_token(TokenKind::Colon);
        // Rule Stmt indent
        self.write_indentation(Indentation::IndentWithNewline);
        if let Some(doc) = &rule_stmt.doc {
            self.write(&doc.node);
            self.write_newline();
        }
        if !rule_stmt.checks.is_empty() {
            interleave!(
                || self.write_newline(),
                |check_expr: &ast::NodeRef<ast::CheckExpr>| self.walk_check_expr(&check_expr.node),
                rule_stmt.checks
            );
            self.write_newline_without_fill();
        }
        // Rule Stmt dedent
        self.write_indentation(Indentation::Dedent);
    }

    fn walk_quant_expr(&mut self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        let in_one_line = false;
        let quant_op_string: String = quant_expr.op.clone().into();
        self.write(&quant_op_string);
        self.write_space();
        interleave!(
            || self.write(COMMA_WHITESPACE),
            |identifier: &ast::NodeRef<ast::Identifier>| self.walk_identifier(&identifier.node),
            quant_expr.variables
        );
        self.write(" in ");
        self.expr(&quant_expr.target);
        self.write(" {");
        if !in_one_line {
            self.write_indentation(Indentation::IndentWithNewline);
        }
        self.expr(&quant_expr.test);
        if let Some(if_cond) = &quant_expr.if_cond {
            self.write(" if ");
            self.expr(if_cond);
        }
        if !in_one_line {
            self.write_indentation(Indentation::DedentWithNewline)
        }
        self.write("}")
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        interleave!(
            || self.write_newline(),
            |expr: &ast::NodeRef<CallExpr>| {
                self.write("@");
                self.walk_call_expr(&expr.node)
            },
            schema_attr.decorators
        );
        if !schema_attr.decorators.is_empty() {
            self.write_newline();
        }
        self.write_attribute(&schema_attr.name);
        if schema_attr.is_optional {
            self.write("?");
        }
        self.write(": ");
        self.write(&schema_attr.ty.node.to_string());
        if let Some(op) = &schema_attr.op {
            let symbol = op.symbol();
            self.write_space();
            self.write(symbol);
            self.write_space();
        }
        if let Some(value) = &schema_attr.value {
            self.expr(value);
        }
        self.write_newline_without_fill();
    }

    fn walk_if_expr(&mut self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        self.expr(&if_expr.body);
        self.write(" if ");
        self.expr(&if_expr.cond);
        self.write(" else ");
        self.expr(&if_expr.orelse);
    }

    fn walk_unary_expr(&mut self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        self.write(unary_expr.op.symbol());
        // Four forms: `+expr`, `-expr`, `~expr`, `not expr`
        // `not expr` needs a space between `not` and `expr`
        if matches!(unary_expr.op, ast::UnaryOp::Not) {
            self.write_space();
        }
        self.expr(&unary_expr.operand);
    }

    fn walk_binary_expr(&mut self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        let symbol = binary_expr.op.symbol();
        self.expr(&binary_expr.left);
        self.write_space();
        self.write(symbol);
        self.write_space();
        self.expr(&binary_expr.right);
    }

    fn walk_selector_expr(&mut self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        self.expr(&selector_expr.value);
        self.write(if selector_expr.has_question {
            "?."
        } else {
            "."
        });
        self.walk_identifier(&selector_expr.attr.node);
    }

    fn walk_call_expr(&mut self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        self.expr(&call_expr.func);
        self.write("(");
        self.write_args_and_kwargs(&call_expr.args, &call_expr.keywords);
        self.write(")");
    }

    fn walk_subscript(&mut self, subscript: &'ctx ast::Subscript) -> Self::Result {
        self.expr(&subscript.value);
        if subscript.has_question {
            self.write("?");
        }
        self.write("[");
        if let Some(index) = &subscript.index {
            self.expr(index);
        } else {
            if let Some(lower) = &subscript.lower {
                self.expr(lower);
            }
            self.write_token(TokenKind::Colon);
            if let Some(upper) = &subscript.upper {
                self.expr(upper);
            }
            self.write_token(TokenKind::Colon);
            if let Some(step) = &subscript.step {
                self.expr(step);
            }
        }
        self.write("]");
    }

    fn walk_paren_expr(&mut self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        self.write_token(TokenKind::OpenDelim(DelimToken::Paren));
        self.expr(&paren_expr.expr);
        self.write_token(TokenKind::CloseDelim(DelimToken::Paren));
    }

    fn walk_list_expr(&mut self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        let line_set = list_expr
            .elts
            .iter()
            .map(|e| e.line)
            .collect::<HashSet<u64>>();
        // There are comments in the configuration block.
        let has_comment = !list_expr.elts.is_empty()
            && list_expr
                .elts
                .iter()
                .map(|e| self.has_comments_on_node(e))
                .all(|r| r);
        // When there are comments in the configuration block, print them as multiline configurations.
        let mut in_one_line = line_set.len() <= 1 && !has_comment;
        if let Some(elt) = list_expr.elts.first() {
            if let ast::Expr::ListIfItem(_) = &elt.node {
                in_one_line = false;
            }
        }
        self.write_token(TokenKind::OpenDelim(DelimToken::Bracket));
        if !in_one_line {
            self.write_indentation(Indentation::IndentWithNewline);
        }
        interleave!(
            || if in_one_line {
                self.write(COMMA_WHITESPACE);
            } else {
                self.write_newline();
            },
            |elt| self.expr(elt),
            list_expr.elts
        );
        if !in_one_line {
            self.write_indentation(Indentation::DedentWithNewline);
        }
        self.write_token(TokenKind::CloseDelim(DelimToken::Bracket));
    }

    fn walk_list_comp(&mut self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        self.write_token(TokenKind::OpenDelim(DelimToken::Bracket));
        self.expr(&list_comp.elt);
        for gen in &list_comp.generators {
            self.walk_comp_clause(&gen.node);
        }
        self.write_token(TokenKind::CloseDelim(DelimToken::Bracket));
    }

    fn walk_list_if_item_expr(
        &mut self,
        list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        self.write("if ");
        self.expr(&list_if_item_expr.if_cond);
        self.write(":");
        self.write_indentation(Indentation::IndentWithNewline);
        interleave!(
            || self.write_newline(),
            |expr| self.expr(expr),
            list_if_item_expr.exprs
        );
        self.write_indentation(Indentation::DedentWithNewline);
        if let Some(orelse) = &list_if_item_expr.orelse {
            match &orelse.node {
                ast::Expr::List(list_expr) => {
                    self.write("else:");
                    self.write_indentation(Indentation::IndentWithNewline);
                    interleave!(
                        || self.write_newline(),
                        |expr| self.expr(expr),
                        list_expr.elts
                    );
                    self.write_indentation(Indentation::Dedent);
                }
                ast::Expr::ListIfItem(_) => {
                    self.write("el");
                    self.expr(orelse);
                }
                _ => bug!("Invalid list if expr orelse node {:?}", orelse.node),
            }
        }
    }

    fn walk_starred_expr(&mut self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        self.write("*");
        self.expr(&starred_expr.value)
    }

    fn walk_dict_comp(&mut self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        self.write_token(TokenKind::OpenDelim(DelimToken::Brace));
        self.expr(match &dict_comp.entry.key {
            Some(key) => key,
            None => bug!("Invalid dict comp key"),
        });
        if !matches!(dict_comp.entry.operation, ast::ConfigEntryOperation::Union) {
            self.write_space();
        }
        self.write(dict_comp.entry.operation.symbol());
        self.write_space();
        self.expr(&dict_comp.entry.value);
        for gen in &dict_comp.generators {
            self.walk_comp_clause(&gen.node);
        }
        self.write_token(TokenKind::CloseDelim(DelimToken::Brace));
    }

    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        self.write("if ");
        self.expr(&config_if_entry_expr.if_cond);
        self.write_token(TokenKind::Colon);
        self.write_indentation(Indentation::IndentWithNewline);
        interleave!(
            || self.write_newline(),
            |entry: &ast::NodeRef<ast::ConfigEntry>| self.write_entry(entry),
            config_if_entry_expr.items
        );
        self.write_indentation(Indentation::DedentWithNewline);
        if let Some(orelse) = &config_if_entry_expr.orelse {
            match &orelse.node {
                ast::Expr::Config(config_expr) => {
                    self.write("else:");
                    self.write_indentation(Indentation::IndentWithNewline);
                    interleave!(
                        || self.write_newline(),
                        |entry: &ast::NodeRef<ast::ConfigEntry>| self.write_entry(entry),
                        config_expr.items
                    );
                    self.write_indentation(Indentation::Dedent);
                }
                ast::Expr::ConfigIfEntry(_) => {
                    self.write("el");
                    self.expr(orelse);
                }
                _ => bug!("Invalid config if expr orelse node {:?}", orelse.node),
            }
        }
    }

    fn walk_comp_clause(&mut self, comp_clause: &'ctx ast::CompClause) -> Self::Result {
        self.write(" for ");
        interleave!(
            || self.write(COMMA_WHITESPACE),
            |target: &ast::NodeRef<ast::Identifier>| self.walk_identifier(&target.node),
            comp_clause.targets
        );
        self.write(" in ");
        self.expr(&comp_clause.iter);
        for if_clause in &comp_clause.ifs {
            self.write(" if ");
            self.expr(if_clause);
        }
    }

    fn walk_schema_expr(&mut self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        self.walk_identifier(&schema_expr.name.node);
        if !schema_expr.args.is_empty() || !schema_expr.kwargs.is_empty() {
            self.write_token(TokenKind::OpenDelim(DelimToken::Paren));
            self.write_args_and_kwargs(&schema_expr.args, &schema_expr.kwargs);
            self.write_token(TokenKind::CloseDelim(DelimToken::Paren));
        }
        self.write_space();
        self.expr(&schema_expr.config)
    }

    fn walk_config_expr(&mut self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        let line_set: HashSet<u64> = config_expr.items.iter().map(|item| item.line).collect();
        // There are comments in the configuration block.
        let has_comment = !config_expr.items.is_empty()
            && config_expr
                .items
                .iter()
                .map(|item| self.has_comments_on_node(item))
                .all(|r| r);
        // When there are comments in the configuration block, print them as multiline configurations.
        let mut in_one_line = line_set.len() <= 1 && !has_comment;
        // When there are complex configuration blocks in the configuration block, print them as multiline configurations.
        if config_expr.items.len() == 1 && in_one_line {
            if let Some(item) = config_expr.items.first() {
                if matches!(
                    &item.node.value.node,
                    ast::Expr::ConfigIfEntry(_) | ast::Expr::Config(_) | ast::Expr::Schema(_)
                ) {
                    in_one_line = false;
                }
            }
        }
        self.write_token(TokenKind::OpenDelim(DelimToken::Brace));
        if !config_expr.items.is_empty() {
            if !in_one_line {
                self.write_indentation(Indentation::IndentWithNewline);
            }
            interleave!(
                || if in_one_line {
                    self.write(COMMA_WHITESPACE);
                } else {
                    self.write_newline();
                },
                |entry: &ast::NodeRef<ast::ConfigEntry>| self.write_entry(entry),
                config_expr.items
            );
            if !in_one_line {
                self.write_indentation(Indentation::DedentWithNewline);
            }
        }
        self.write_token(TokenKind::CloseDelim(DelimToken::Brace));
    }

    fn walk_check_expr(&mut self, check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        self.expr(&check_expr.test);
        if let Some(if_cond) = &check_expr.if_cond {
            self.write(" if ");
            self.expr(if_cond);
        }
        if let Some(msg) = &check_expr.msg {
            self.write(COMMA_WHITESPACE);
            self.expr(msg);
        }
    }

    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        self.write("lambda");
        if let Some(args) = &lambda_expr.args {
            self.write_space();
            self.walk_arguments(&args.node);
        }
        if let Some(ty_str) = &lambda_expr.return_ty {
            self.write_space();
            self.write_token(TokenKind::RArrow);
            self.write_space();
            self.write(&ty_str.node.to_string());
        }
        self.write_space();
        self.write_token(TokenKind::OpenDelim(DelimToken::Brace));
        self.write_newline_without_fill();
        self.write_indentation(Indentation::Indent);

        // lambda body
        self.stmts(&lambda_expr.body);

        self.write_indentation(Indentation::Dedent);
        self.fill("");
        self.write_token(TokenKind::CloseDelim(DelimToken::Brace));
    }

    fn walk_keyword(&mut self, keyword: &'ctx ast::Keyword) -> Self::Result {
        self.walk_identifier(&keyword.arg.node);
        if let Some(value) = &keyword.value {
            self.write("=");
            self.expr(value);
        }
    }

    fn walk_arguments(&mut self, arguments: &'ctx ast::Arguments) -> Self::Result {
        let parameter_zip_list: Vec<ParameterType<'_>> = arguments
            .args
            .iter()
            .zip(
                arguments
                    .ty_list
                    .iter()
                    .map(|ty| ty.clone().map(|n| n.node.to_string())),
            )
            .zip(arguments.defaults.iter())
            .collect();
        interleave!(
            || self.write(COMMA_WHITESPACE),
            |para: &ParameterType<'_>| {
                let ((arg, ty_str), default) = para;
                self.walk_identifier(&arg.node);
                if let Some(ty_str) = ty_str {
                    self.write(&format!(": {}", ty_str));
                }
                if let Some(default) = default {
                    self.write(" = ");
                    self.expr(default);
                }
            },
            parameter_zip_list
        );
    }

    fn walk_compare(&mut self, compare: &'ctx ast::Compare) -> Self::Result {
        self.expr(&compare.left);
        for (op, expr) in compare.ops.iter().zip(compare.comparators.iter()) {
            self.write_space();
            self.write(op.symbol());
            self.write_space();
            self.expr(expr);
        }
    }

    #[inline]
    fn walk_identifier(&mut self, identifier: &'ctx ast::Identifier) -> Self::Result {
        self.write(&identifier.get_name());
    }

    fn walk_number_lit(&mut self, number_lit: &'ctx ast::NumberLit) -> Self::Result {
        match &number_lit.value {
            ast::NumberLitValue::Int(int_val) => self.write(&int_val.to_string()),
            ast::NumberLitValue::Float(float_val) => self.write(&float_val.to_string()),
        }
        // Number suffix e.g., 1Gi
        if let Some(binary_suffix) = &number_lit.binary_suffix {
            self.write(&binary_suffix.value())
        }
    }

    fn walk_string_lit(&mut self, string_lit: &'ctx ast::StringLit) -> Self::Result {
        if !string_lit.raw_value.is_empty() {
            self.write(&string_lit.raw_value)
        } else {
            self.write(&if string_lit.is_long_string {
                format!("\"\"\"{}\"\"\"", string_lit.value.replace('\"', "\\\""))
            } else {
                format!("\"{}\"", string_lit.value.replace('\"', "\\\""))
            });
        }
    }

    #[inline]
    fn walk_name_constant_lit(
        &mut self,
        name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        self.write(name_constant_lit.value.symbol());
    }

    fn walk_joined_string(&mut self, joined_string: &'ctx ast::JoinedString) -> Self::Result {
        let quote_str = if joined_string.is_long_string {
            "\"\"\""
        } else {
            "\""
        };
        self.write(quote_str);
        for value in &joined_string.values {
            match &value.node {
                ast::Expr::StringLit(string_lit) => {
                    self.write(&string_lit.value.replace('\"', "\\\""));
                }
                _ => self.expr(value),
            }
        }
        self.write(quote_str);
    }

    fn walk_formatted_value(&mut self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result {
        self.write("${");
        self.expr(&formatted_value.value);
        if let Some(spec) = &formatted_value.format_spec {
            self.write(&format!(": {}", spec));
        }
        self.write("}");
    }

    fn walk_comment(&mut self, comment: &'ctx ast::Comment) -> Self::Result {
        self.writeln(&comment.text);
        self.fill("");
    }

    fn walk_missing_expr(&mut self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        // Nothing to do
    }
}

impl<'p> Printer<'p> {
    pub fn write_args_and_kwargs(
        &mut self,
        args: &[ast::NodeRef<ast::Expr>],
        kwargs: &[ast::NodeRef<ast::Keyword>],
    ) {
        interleave!(|| self.write(COMMA_WHITESPACE), |arg| self.expr(arg), args);
        if !args.is_empty() && !kwargs.is_empty() {
            self.write(COMMA_WHITESPACE);
        }
        interleave!(
            || self.write(COMMA_WHITESPACE),
            |kwarg: &ast::NodeRef<ast::Keyword>| self.walk_keyword(&kwarg.node),
            kwargs
        );
    }

    pub fn write_entry(&mut self, item: &ast::NodeRef<ast::ConfigEntry>) {
        match &item.node.key {
            Some(key) => {
                let print_right_brace_count = self.write_config_key(key);
                if item.node.insert_index >= 0 {
                    self.write(&format!("[{}]", item.node.insert_index));
                }
                if !matches!(item.node.operation, ast::ConfigEntryOperation::Union) {
                    self.write_space();
                }
                self.write(item.node.operation.symbol());
                self.write_space();
                self.expr(&item.node.value);
                self.write(&"}".repeat(print_right_brace_count));
            }
            None => {
                if !matches!(&item.node.value.node, ast::Expr::ConfigIfEntry(_)) {
                    self.write("**");
                }
                self.expr(&item.node.value)
            }
        };
    }

    fn write_config_key(&mut self, key: &ast::NodeRef<ast::Expr>) -> usize {
        match &key.node {
            ast::Expr::Identifier(identifier) => {
                self.hook.pre(self, super::ASTNode::Expr(key));
                self.write_ast_comments(key);
                // Judge contains string or dot identifier, e.g., "x-y-z" and "a.b.c"
                let names = &identifier.names;

                let re = fancy_regex::Regex::new(IDENTIFIER_REGEX).unwrap();
                let need_right_brace = !names.iter().all(|n| re.is_match(&n.node).unwrap_or(false));
                let count = if need_right_brace {
                    self.write(
                        &names
                            .iter()
                            .map(|n| format!("{:?}", n.node))
                            .collect::<Vec<String>>()
                            .join(": {"),
                    );
                    names.len() - 1
                } else {
                    self.expr(key);
                    0
                };
                self.hook.post(self, super::ASTNode::Expr(key));
                count
            }
            _ => {
                self.expr(key);
                0
            }
        }
    }

    fn write_attribute(&mut self, attr: &ast::NodeRef<String>) {
        let re = fancy_regex::Regex::new(IDENTIFIER_REGEX).unwrap();
        let need_quote = !re.is_match(&attr.node).unwrap();
        if need_quote {
            self.write(&format!("{:?}", attr.node));
        } else {
            self.write(&attr.node);
        };
    }
}

impl<'p> Printer<'p> {
    // ------------------------------
    // Expr and Stmt walker functions
    // ------------------------------

    pub fn expr(&mut self, expr: &ast::NodeRef<ast::Expr>) {
        self.hook.pre(self, super::ASTNode::Expr(expr));
        self.write_ast_comments(expr);
        self.walk_expr(&expr.node);
        self.hook.post(self, super::ASTNode::Expr(expr));
    }

    pub fn stmt(&mut self, stmt: &ast::NodeRef<ast::Stmt>) {
        self.hook.pre(self, super::ASTNode::Stmt(stmt));
        self.fill("");
        self.write_ast_comments(stmt);
        self.walk_stmt(&stmt.node);
        self.hook.post(self, super::ASTNode::Stmt(stmt));
    }

    pub fn exprs(&mut self, exprs: &[ast::NodeRef<ast::Expr>]) {
        for expr in exprs {
            self.expr(expr);
        }
    }

    pub fn stmts(&mut self, stmts: &[ast::NodeRef<ast::Stmt>]) {
        let mut prev_stmt: Option<ast::Stmt> = None;
        for stmt in stmts {
            let import_stmt_alter = match (prev_stmt.as_ref(), stmt.as_ref().node.to_owned()) {
                (Some(ast::Stmt::Import(_)), ast::Stmt::Import(_)) => false,
                (Some(ast::Stmt::Import(_)), _) => true,
                _ => false,
            };
            if import_stmt_alter {
                self.write_newline();
            }
            self.stmt(stmt);
            prev_stmt = Some(stmt.node.to_owned());
        }
    }
}
