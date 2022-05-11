use indexmap::IndexMap;

use kclvm_ast::ast;
use kclvm_ast::token::Token;
use kclvm_ast::walker::MutSelfTypedResultWalker;

use super::Printer;

const COMMA_WHITESPACE: &str = ", ";
const INVALID_AST_MSG: &str = "Invalid AST Node";
const TEMP_ROOT: &str = "<root>";

macro_rules! interleave {
    ($inter: stmt, $f: expr, $seq: expr) => {
        if $seq.is_empty() {
            return;
        }
        $f(&$seq[0]);
        for s in &$seq[1..] {
            $inter
            $f(s);
        }
    };
}

impl<'p, 'ctx> MutSelfTypedResultWalker<'ctx> for Printer<'p> {
    type Result = ();

    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result {
        for comment in &module.comments {
            self.comments.push_back(comment.clone());
        }
        self.stmts(&module.body);
    }

    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        interleave!(
            self.write(COMMA_WHITESPACE),
            |expr| self.expr(expr),
            expr_stmt.exprs
        );
        self.writeln("");
    }

    fn walk_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result {
        todo!()
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        todo!()
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        todo!()
    }

    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        todo!()
    }

    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        todo!()
    }

    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        todo!()
    }

    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        todo!()
    }

    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        todo!()
    }

    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        todo!()
    }

    fn walk_quant_expr(&mut self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        todo!()
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        todo!()
    }

    fn walk_if_expr(&mut self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        todo!()
    }

    fn walk_unary_expr(&mut self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        todo!()
    }

    fn walk_binary_expr(&mut self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        todo!()
    }

    fn walk_selector_expr(&mut self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        todo!()
    }

    fn walk_call_expr(&mut self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        todo!()
    }

    fn walk_subscript(&mut self, subscript: &'ctx ast::Subscript) -> Self::Result {
        todo!()
    }

    fn walk_paren_expr(&mut self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        todo!()
    }

    fn walk_list_expr(&mut self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        todo!()
    }

    fn walk_list_comp(&mut self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        todo!()
    }

    fn walk_list_if_item_expr(
        &mut self,
        list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        todo!()
    }

    fn walk_starred_expr(&mut self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        todo!()
    }

    fn walk_dict_comp(&mut self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        todo!()
    }

    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        todo!()
    }

    fn walk_comp_clause(&mut self, comp_clause: &'ctx ast::CompClause) -> Self::Result {
        todo!()
    }

    fn walk_schema_expr(&mut self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        todo!()
    }

    fn walk_config_expr(&mut self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        todo!()
    }

    fn walk_check_expr(&mut self, check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        todo!()
    }

    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        todo!()
    }

    fn walk_keyword(&mut self, keyword: &'ctx ast::Keyword) -> Self::Result {
        todo!()
    }

    fn walk_arguments(&mut self, arguments: &'ctx ast::Arguments) -> Self::Result {
        todo!()
    }

    fn walk_compare(&mut self, compare: &'ctx ast::Compare) -> Self::Result {
        todo!()
    }

    fn walk_identifier(&mut self, identifier: &'ctx ast::Identifier) -> Self::Result {
        todo!()
    }

    fn walk_literal(&mut self, literal: &'ctx ast::Literal) -> Self::Result {
        todo!()
    }

    fn walk_number_lit(&mut self, number_lit: &'ctx ast::NumberLit) -> Self::Result {
        todo!()
    }

    fn walk_string_lit(&mut self, string_lit: &'ctx ast::StringLit) -> Self::Result {
        todo!()
    }

    fn walk_name_constant_lit(
        &mut self,
        name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        todo!()
    }

    fn walk_joined_string(&mut self, joined_string: &'ctx ast::JoinedString) -> Self::Result {
        todo!()
    }

    fn walk_formatted_value(&mut self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result {
        todo!()
    }

    fn walk_comment(&mut self, comment: &'ctx ast::Comment) -> Self::Result {
        self.writeln(&comment.text);
        self.fill("");
    }
}

impl<'p> Printer<'p> {
    // ------------------------------
    // Expr and Stmt walker functions
    // ------------------------------

    pub fn expr(&mut self, expr: &ast::NodeRef<ast::Expr>) {
        self.print_ast_comments(expr);
        self.walk_expr(&expr.node)
    }

    pub fn stmt(&mut self, stmt: &ast::NodeRef<ast::Stmt>) {
        self.fill("");
        self.print_ast_comments(stmt);
        self.walk_stmt(&stmt.node)
    }

    pub fn exprs(&mut self, exprs: &[ast::NodeRef<ast::Expr>]) {
        for expr in exprs {
            self.expr(expr);
        }
    }

    pub fn stmts(&mut self, stmts: &[ast::NodeRef<ast::Stmt>]) {
        for stmt in stmts {
            self.stmt(stmt);
        }
    }
}
