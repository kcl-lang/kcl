//! The design and implementation of KCL Lint refer to the [rust-lang/rustc](https://github.com/rust-lang/rust) lint and follow Apache License Version 2.0
//!
//! This file is the implementation of KCLLint, which is used to perform some additional checks on KCL code.
//! The main structures of the file are Lint, LintPass, CombinedLintPass and Linter.
//! For details see the: https://github.com/kcl-lang/kcl/issues/109
//!
//! File dependencies：
//! mode  -> combinedlintpass -> lints_def -> lintpass -> lint
//!
//! mode.rs: Definition of `Linter`, the entry for lint check
//! combinedlintpass.rs: `CombinedLintPass` collects all the lints defined in the lints_def.rs
//! lints_def.rs: Defined the various lints and the corresponding lintpasses implementation
//! lintpass.rs: Definition of `Lintpass`
//! lint.rs: Definition of `Lint`
//!               
//! Steps to define a new lint:
//! 1. Define a static instance of the `Lint` structure in lints_def.rs，e.g.,
//!    
//!     ```ignore
//!    pub static IMPORT_POSITION: &Lint = &Lint {
//!         ...
//!    }
//!    ```
//!
//! 2. Define a lintpass, which is used to implement the checking process，e.g.,
//!    
//!     ```ignore
//!    declare_lint_pass!(ImportPosition => [IMPORT_POSITION]);
//!    ```
//!
//!    The `ImportPosition` is the defined LintPass structure and the `IMPORT_POSITION` is the `Lint` structure
//!    defined in step 1. Here is a `LintArray`, which means that multiple lint checks can be implemented
//!    in a single lintpass.
//!
//! 3. Implement the lintpass check process, e.g.,
//!
//!    ```ignore
//!    impl LintPass for ImportPosition {
//!        fn check_module(&mut self, handler: &mut Handler, ctx: &mut LintContext,module: &ast::Module){
//!            ...
//!        }
//!    }
//!    ```
//!
//! 4. Add the `check_*` methods in lintpass to the macro `lint_methods`, or skip it if it exists
//!
//!    ```ignore
//!    macro_rules! lint_methods {
//!        ($macro:path, $args:tt) => (
//!            $macro!($args, [
//!                fn check_module(module: &ast::Module);
//!            ]);
//!        )
//!    }
//!    ```
//!
//! 5. Add the new lintpass to the macro `default_lint_passes` in lintpass.rs , noting that `:` is preceded and followed by
//! the name of the lintpass. e.g.,
//!
//!    ```ignore
//!    macro_rules! default_lint_passes {
//!        ($macro:path, $args:tt) => {
//!            $macro!(
//!                $args,
//!                [
//!                    ImportPosition: ImportPosition,
//!                ]
//!            );
//!        };
//!    }
//!    ```
//!
//! 6. If new `check_*` method was added in step 4, it needs to override the walk_* method in Linter.
//! In addition to calling the self.pass.check_* function, the original walk method in MutSelfWalker
//! should be copied here so that it can continue to traverse the child nodes.

use crate::resolver::{scope::Scope, Resolver};
use kclvm_ast::pos::GetPos;
use kclvm_error::{Handler, Position};
mod combinedlintpass;
mod lint;
mod lintpass;
mod lints_def;
use kclvm_ast::ast;
use kclvm_ast::walker::MutSelfWalker;

pub use self::{combinedlintpass::CombinedLintPass, lint::LintContext, lintpass::LintPass};

/// The struct `Linter` is used to traverse the AST and call the `check_*` method defined in `CombinedLintPass`.
pub struct Linter<T: LintPass> {
    pub pass: T,
    pub handler: Handler,
    pub ctx: LintContext,
}

impl LintContext {
    pub fn dummy_ctx() -> Self {
        LintContext {
            filename: "".to_string(),
            start_pos: Position::dummy_pos(),
            end_pos: Position::dummy_pos(),
        }
    }
}

impl Linter<CombinedLintPass> {
    pub fn new() -> Self {
        Linter::<CombinedLintPass> {
            pass: CombinedLintPass::new(),
            handler: Handler::default(),
            ctx: LintContext::dummy_ctx(),
        }
    }
    pub fn walk_scope(&mut self, scope: &Scope) {
        self.pass
            .check_scope(&mut self.handler, &mut self.ctx, scope);
    }
}

impl Resolver<'_> {
    /// Iterate the module and run lint checks, generating diagnostics and save them in `lint.handler`
    pub fn lint_check_module(&mut self, module: &ast::Module) {
        self.linter.ctx.filename = module.filename.clone();
        self.linter.walk_module(module);
    }
    /// Recursively iterate the scope and its child scope, run lint checks, generating diagnostics and save them in `lint.handler`
    pub fn lint_check_scope(&mut self, scope: &Scope) {
        self.linter.walk_scope(scope);
        for children in &scope.children {
            self.lint_check_scope(&children.borrow().clone())
        }
    }

    /// Iterate the resolver.scope_map and run lint checks, generating diagnostics and save them in `lint.handler`
    pub fn lint_check_scope_map(&mut self) {
        let scope_map = self.scope_map.clone();
        for (_, scope) in scope_map.iter() {
            self.lint_check_scope(&scope.borrow())
        }
    }
}

macro_rules! walk_set_list {
    ($walker: expr, $method: ident, $list: expr) => {
        for elem in &$list {
            set_pos!($walker, elem);
            $walker.$method(&elem.node)
        }
    };
}

macro_rules! walk_set_if {
    ($walker: expr, $method: ident, $value: expr) => {
        match &$value {
            Some(v) => {
                set_pos!($walker, &v);
                $walker.$method(&v.node);
            }
            None => (),
        }
    };
}

macro_rules! set_pos {
    ($walker: expr, $value: expr) => {
        $walker.set_pos(&$value.get_pos(), &$value.get_end_pos());
    };
}

impl Linter<CombinedLintPass> {
    fn set_pos(&mut self, start_pos: &Position, end_pos: &Position) {
        self.ctx.start_pos = start_pos.clone();
        self.ctx.end_pos = end_pos.clone();
    }
}

impl MutSelfWalker for Linter<CombinedLintPass> {
    fn walk_module(&mut self, module: &ast::Module) {
        self.pass
            .check_module(&mut self.handler, &mut self.ctx, module);
        walk_set_list!(self, walk_stmt, module.body);
    }

    fn walk_expr_stmt(&mut self, expr_stmt: &ast::ExprStmt) {
        for expr in &expr_stmt.exprs {
            set_pos!(self, &expr);
            self.walk_expr(&expr.node)
        }
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &ast::TypeAliasStmt) {
        set_pos!(self, &type_alias_stmt.type_name);
        self.walk_identifier(&type_alias_stmt.type_name.node);
    }
    fn walk_unification_stmt(&mut self, unification_stmt: &ast::UnificationStmt) {
        set_pos!(self, &unification_stmt.target);
        self.walk_identifier(&unification_stmt.target.node);
        set_pos!(self, &unification_stmt.value);
        self.walk_schema_expr(&unification_stmt.value.node);
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &ast::AssignStmt) {
        for target in &assign_stmt.targets {
            set_pos!(self, &target);
            self.walk_identifier(&target.node)
        }
        set_pos!(self, &assign_stmt.value);
        self.walk_expr(&assign_stmt.value.node);
    }
    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &ast::AugAssignStmt) {
        set_pos!(self, &aug_assign_stmt.target);
        self.walk_identifier(&aug_assign_stmt.target.node);
        set_pos!(self, &aug_assign_stmt.value);
        self.walk_expr(&aug_assign_stmt.value.node);
    }
    fn walk_assert_stmt(&mut self, assert_stmt: &ast::AssertStmt) {
        set_pos!(self, &assert_stmt.test);
        self.walk_expr(&assert_stmt.test.node);
        walk_set_if!(self, walk_expr, assert_stmt.if_cond);
        walk_set_if!(self, walk_expr, assert_stmt.msg);
    }
    fn walk_if_stmt(&mut self, if_stmt: &ast::IfStmt) {
        set_pos!(self, &if_stmt.cond);
        self.walk_expr(&if_stmt.cond.node);
        walk_set_list!(self, walk_stmt, if_stmt.body);
        walk_set_list!(self, walk_stmt, if_stmt.orelse);
    }
    fn walk_import_stmt(&mut self, import_stmt: &ast::ImportStmt) {
        // Nothing to do.
        let _ = import_stmt;
    }
    fn walk_schema_attr(&mut self, schema_attr: &ast::SchemaAttr) {
        walk_set_list!(self, walk_call_expr, schema_attr.decorators);
        walk_set_if!(self, walk_expr, schema_attr.value);
    }
    fn walk_schema_stmt(&mut self, schema_stmt: &ast::SchemaStmt) {
        walk_set_if!(self, walk_identifier, schema_stmt.parent_name);
        walk_set_if!(self, walk_identifier, schema_stmt.for_host_name);
        walk_set_if!(self, walk_arguments, schema_stmt.args);
        if let Some(schema_index_signature) = &schema_stmt.index_signature {
            let value = &schema_index_signature.node.value;
            walk_set_if!(self, walk_expr, value);
        }
        walk_set_list!(self, walk_identifier, schema_stmt.mixins);
        walk_set_list!(self, walk_call_expr, schema_stmt.decorators);
        walk_set_list!(self, walk_check_expr, schema_stmt.checks);
        walk_set_list!(self, walk_stmt, schema_stmt.body);
    }
    fn walk_rule_stmt(&mut self, rule_stmt: &ast::RuleStmt) {
        walk_set_list!(self, walk_identifier, rule_stmt.parent_rules);
        walk_set_list!(self, walk_call_expr, rule_stmt.decorators);
        walk_set_list!(self, walk_check_expr, rule_stmt.checks);
        walk_set_if!(self, walk_arguments, rule_stmt.args);
        walk_set_if!(self, walk_identifier, rule_stmt.for_host_name);
    }
    fn walk_quant_expr(&mut self, quant_expr: &ast::QuantExpr) {
        set_pos!(self, &quant_expr.target);
        self.walk_expr(&quant_expr.target.node);
        walk_set_list!(self, walk_identifier, quant_expr.variables);
        set_pos!(self, &quant_expr.test);
        self.walk_expr(&quant_expr.test.node);
        walk_set_if!(self, walk_expr, quant_expr.if_cond);
    }
    fn walk_if_expr(&mut self, if_expr: &ast::IfExpr) {
        set_pos!(self, &if_expr.cond);
        self.walk_expr(&if_expr.cond.node);
        set_pos!(self, &if_expr.body);
        self.walk_expr(&if_expr.body.node);
        set_pos!(self, &if_expr.orelse);
        self.walk_expr(&if_expr.orelse.node);
    }
    fn walk_unary_expr(&mut self, unary_expr: &ast::UnaryExpr) {
        set_pos!(self, &unary_expr.operand);
        self.walk_expr(&unary_expr.operand.node);
    }
    fn walk_binary_expr(&mut self, binary_expr: &ast::BinaryExpr) {
        set_pos!(self, &binary_expr.left);
        self.walk_expr(&binary_expr.left.node);
        set_pos!(self, &binary_expr.right);
        self.walk_expr(&binary_expr.right.node);
    }
    fn walk_selector_expr(&mut self, selector_expr: &ast::SelectorExpr) {
        set_pos!(self, &selector_expr.value);
        self.walk_expr(&selector_expr.value.node);
        set_pos!(self, &selector_expr.attr);
        self.walk_identifier(&selector_expr.attr.node);
    }
    fn walk_call_expr(&mut self, call_expr: &ast::CallExpr) {
        set_pos!(self, &call_expr.func);
        self.walk_expr(&call_expr.func.node);
        walk_set_list!(self, walk_expr, call_expr.args);
        walk_set_list!(self, walk_keyword, call_expr.keywords);
    }
    fn walk_subscript(&mut self, subscript: &ast::Subscript) {
        set_pos!(self, &subscript.value);
        self.walk_expr(&subscript.value.node);
        walk_set_if!(self, walk_expr, subscript.index);
        walk_set_if!(self, walk_expr, subscript.lower);
        walk_set_if!(self, walk_expr, subscript.upper);
        walk_set_if!(self, walk_expr, subscript.step);
    }
    fn walk_paren_expr(&mut self, paren_expr: &ast::ParenExpr) {
        set_pos!(self, &paren_expr.expr);
        self.walk_expr(&paren_expr.expr.node);
    }
    fn walk_list_expr(&mut self, list_expr: &ast::ListExpr) {
        walk_set_list!(self, walk_expr, list_expr.elts);
    }
    fn walk_list_comp(&mut self, list_comp: &ast::ListComp) {
        set_pos!(self, &list_comp.elt);
        self.walk_expr(&list_comp.elt.node);
        walk_set_list!(self, walk_comp_clause, list_comp.generators);
    }
    fn walk_list_if_item_expr(&mut self, list_if_item_expr: &ast::ListIfItemExpr) {
        set_pos!(self, &list_if_item_expr.if_cond);
        self.walk_expr(&list_if_item_expr.if_cond.node);
        walk_set_list!(self, walk_expr, list_if_item_expr.exprs);
        walk_set_if!(self, walk_expr, list_if_item_expr.orelse);
    }
    fn walk_starred_expr(&mut self, starred_expr: &ast::StarredExpr) {
        set_pos!(self, &starred_expr.value);
        self.walk_expr(&starred_expr.value.node);
    }
    fn walk_dict_comp(&mut self, dict_comp: &ast::DictComp) {
        if let Some(key) = &dict_comp.entry.key {
            set_pos!(self, &key);
            self.walk_expr(&key.node);
        }
        set_pos!(self, &dict_comp.entry.value);
        self.walk_expr(&dict_comp.entry.value.node);
        walk_set_list!(self, walk_comp_clause, dict_comp.generators);
    }
    fn walk_config_if_entry_expr(&mut self, config_if_entry_expr: &ast::ConfigIfEntryExpr) {
        set_pos!(self, &config_if_entry_expr.if_cond);
        self.walk_expr(&config_if_entry_expr.if_cond.node);
        for config_entry in &config_if_entry_expr.items {
            walk_set_if!(self, walk_expr, config_entry.node.key);
            set_pos!(self, &config_entry.node.value);
            self.walk_expr(&config_entry.node.value.node);
        }
        walk_set_if!(self, walk_expr, config_if_entry_expr.orelse);
    }
    fn walk_comp_clause(&mut self, comp_clause: &ast::CompClause) {
        walk_set_list!(self, walk_identifier, comp_clause.targets);
        set_pos!(self, &comp_clause.iter);
        self.walk_expr(&comp_clause.iter.node);
        walk_set_list!(self, walk_expr, comp_clause.ifs);
    }
    fn walk_schema_expr(&mut self, schema_expr: &ast::SchemaExpr) {
        set_pos!(self, &schema_expr.name);
        self.walk_identifier(&schema_expr.name.node);
        walk_set_list!(self, walk_expr, schema_expr.args);
        walk_set_list!(self, walk_keyword, schema_expr.kwargs);
        set_pos!(self, &schema_expr.config);
        self.walk_expr(&schema_expr.config.node);
    }
    fn walk_config_expr(&mut self, config_expr: &ast::ConfigExpr) {
        for config_entry in &config_expr.items {
            walk_set_if!(self, walk_expr, config_entry.node.key);
            set_pos!(self, &config_entry.node.value);
            self.walk_expr(&config_entry.node.value.node);
        }
    }
    fn walk_check_expr(&mut self, check_expr: &ast::CheckExpr) {
        set_pos!(self, &check_expr.test);
        self.walk_expr(&check_expr.test.node);
        walk_set_if!(self, walk_expr, check_expr.if_cond);
        walk_set_if!(self, walk_expr, check_expr.msg);
    }
    fn walk_lambda_expr(&mut self, lambda_expr: &ast::LambdaExpr) {
        walk_set_if!(self, walk_arguments, lambda_expr.args);
        walk_set_list!(self, walk_stmt, lambda_expr.body);
    }
    fn walk_keyword(&mut self, keyword: &ast::Keyword) {
        set_pos!(self, &keyword.arg);
        self.walk_identifier(&keyword.arg.node);
        if let Some(v) = &keyword.value {
            set_pos!(self, &v);
            self.walk_expr(&v.node)
        }
    }
    fn walk_arguments(&mut self, arguments: &ast::Arguments) {
        walk_set_list!(self, walk_identifier, arguments.args);
        for default in &arguments.defaults {
            if let Some(d) = default {
                set_pos!(self, d);
                self.walk_expr(&d.node)
            }
        }
    }
    fn walk_compare(&mut self, compare: &ast::Compare) {
        set_pos!(self, &compare.left);
        self.walk_expr(&compare.left.node);
        walk_set_list!(self, walk_expr, compare.comparators);
    }
    fn walk_identifier(&mut self, identifier: &ast::Identifier) {
        // Nothing to do.
        let _ = identifier;
    }
    fn walk_number_lit(&mut self, number_lit: &ast::NumberLit) {
        let _ = number_lit;
    }
    fn walk_string_lit(&mut self, string_lit: &ast::StringLit) {
        // Nothing to do.
        let _ = string_lit;
    }
    fn walk_name_constant_lit(&mut self, name_constant_lit: &ast::NameConstantLit) {
        // Nothing to do.
        let _ = name_constant_lit;
    }
    fn walk_joined_string(&mut self, joined_string: &ast::JoinedString) {
        walk_set_list!(self, walk_expr, joined_string.values);
    }
    fn walk_formatted_value(&mut self, formatted_value: &ast::FormattedValue) {
        set_pos!(self, &formatted_value.value);
        self.walk_expr(&formatted_value.value.node);
    }
    fn walk_comment(&mut self, comment: &ast::Comment) {
        // Nothing to do.
        let _ = comment;
    }
}
