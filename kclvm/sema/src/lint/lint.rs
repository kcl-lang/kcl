//! This file is the implementation of KCLLint, which is used to perform some additional checks on KCL code.
//! The main structures of the file are Lint, LintPass, CombinedLintPass and Linter.
//! For details see the: https://github.com/KusionStack/KCLVM/issues/109
//!
//! Steps to define a new lint:
//! 1. Define a static instance of the `Lint` structure，e.g.,
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
//! 5. Add the new lintpass to the macro `default_lint_passes`, noting that `:` is preceded and followed by
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

use crate::lint::lint_def::ImportPosition;
use crate::lint::lint_def::ReImport;
use crate::lint::lint_def::UnusedImport;
use crate::resolver::pos::GetPos;
use crate::resolver::scope::builtin_scope;
use crate::resolver::scope::Scope;
use crate::resolver::Resolver;
use kclvm_ast::ast;
use kclvm_ast::walker::MutSelfWalker;
use kclvm_error::*;

/// A summary of the methods that need to be implemented in lintpass, to be added when constructing new lint
/// lint and lintpass. When defining lintpass, the default implementation of these methods is provided: null
/// check (see macro `expand_default_lint_pass_methods`). So what need to do is to override the specific
/// `check_*` function. Some of the methods are commented out here to avoid useless empty functions, which
/// can be added when needed.
#[macro_export]
macro_rules! lint_methods {
    ($macro:path, $args:tt) => (
        $macro!($args, [

            fn check_scope(scope: &Scope);

            fn check_module(module: &ast::Module);
            /*
            * Stmt
            */

            // fn check_stmt(stmt: ast::Node<ast::Stmt>);
            // fn check_expr_stmt(expr_stmt: ast::ExprStmt);
            // fn check_unification_stmt(unification_stmt: ast::UnificationStmt);
            // fn check_type_alias_stmt(type_alias_stmt: ast::TypeAliasStmt);
            // fn check_assign_stmt(assign_stmt: ast::AssignStmt);
            // fn check_aug_assign_stmt(aug_assign_stmt: ast::AugAssignStmt);
            // fn check_assert_stmt(assert_stmt: ast::AssertStmt);
            // fn check_if_stmt(if_stmt: ast::IfStmt);
            fn check_import_stmt(import_stmt: &ast::ImportStmt);
            // fn check_schema_stmt(schema_stmt: ast::SchemaStmt);
            // fn check_rule_stmt(rule_stmt: ast::RuleStmt);

            /*
            * Expr
            */

            // fn check_expr(expr: ast::Node<ast::Expr>);
            // fn check_quant_expr(quant_expr: ast::QuantExpr);
            // fn check_schema_attr(schema_attr: &ast::SchemaAttr);
            // fn check_if_expr(if_expr: ast::IfExpr);
            // fn check_unary_expr(unary_expr: ast::UnaryExpr);
            // fn check_binary_expr(binary_expr: ast::BinaryExpr);
            // fn check_selector_expr(selector_expr: ast::SelectorExpr);
            // fn check_call_expr(call_expr: ast::CallExpr);
            // fn check_subscript(subscript: ast::Subscript);
            // fn check_paren_expr(paren_expr: ast::ParenExpr);
            // fn check_list_expr(list_expr: ast::ListExpr);
            // fn check_list_comp(list_comp: ast::ListComp);
            // fn check_list_if_item_expr(list_if_item_expr: ast::ListIfItemExpr);
            // fn check_starred_expr(starred_expr: ast::StarredExpr);
            // fn check_dict_comp(dict_comp: ast::DictComp);
            // fn check_config_if_entry_expr(config_if_entry_expr: ast::ConfigIfEntryExpr,
            // );
            // fn check_comp_clause(comp_clause: ast::CompClause);
            // fn check_schema_expr(schema_expr: ast::SchemaExpr);
            // fn check_config_expr(config_expr: ast::ConfigExpr);
            // fn check_check_expr(check_expr: ast::CheckExpr);
            // fn check_lambda_expr(lambda_expr: ast::LambdaExpr);
            // fn check_keyword(keyword: ast::Keyword);
            // fn check_arguments(arguments: ast::Arguments);
            // fn check_compare(compare: ast::Compare);
            // fn check_identifier(id: &ast::Identifier);
            // fn check_number_lit(number_lit: ast::NumberLit);
            // fn check_string_lit(string_lit: ast::StringLit);
            // fn check_name_constant_lit(name_constant_lit: ast::NameConstantLit);
            // fn check_joined_string(joined_string: ast::JoinedString);
            // fn check_formatted_value(formatted_value: ast::FormattedValue);
            // fn check_comment(comment: ast::Comment);
        ]);
    )
}

/// Definition of `Lint` struct
/// Note that Lint declarations don't carry any "state" - they are merely global identifiers and descriptions of lints.
pub struct Lint {
    /// A string identifier for the lint.
    pub name: &'static str,

    /// Level for the lint.
    pub level: Level,

    /// Description of the lint or the issue it detects.
    /// e.g., "imports that are never used"
    pub desc: &'static str,

    // Error/Warning code
    pub code: &'static str,

    // Suggest methods to fix this problem
    pub note: Option<&'static str>,
}

pub type LintArray = Vec<&'static Lint>;

/// Declares a static `LintArray` and return it as an expression.
#[macro_export]
macro_rules! lint_array {
    ($( $lint:expr ),* ,) => { lint_array!( $($lint),* ) };
    ($( $lint:expr ),*) => {{
        vec![$($lint),*]
    }}
}

/// Provide a default implementation of the methods in lint_methods for each lintpass: null checking
#[macro_export]
macro_rules! expand_default_lint_pass_methods {
    ($handler:ty, $ctx:ty, [$($(#[$attr:meta])* fn $name:ident($($param:ident: $arg:ty),*);)*]) => (
        $(#[inline(always)] fn $name(&mut self, handler: &mut $handler, ctx: &mut $ctx, $($param: $arg),*) {})*
    )
}

/// Definition of `LintPass` trait
#[macro_export]
macro_rules! declare_default_lint_pass_impl {
    ([], [$($methods:tt)*]) => (
        pub trait LintPass {
            expand_default_lint_pass_methods!(Handler, LintContext, [$($methods)*]);
        }
    )
}

lint_methods!(declare_default_lint_pass_impl, []);

/// The macro to define the LintPass and bind a set of corresponding Lint.
///
/// Here is a `LintArray`, which means that multiple lint checks can be implemented in a single lintpass.
#[macro_export]
macro_rules! declare_lint_pass {
    ($(#[$m:meta])* $name:ident => [$($lint:expr),* $(,)?]) => {
        $(#[$m])* #[derive(Copy, Clone)] pub struct $name;
        $crate::impl_lint_pass!($name => [$($lint),*]);
    };
}

/// Implements `LintPass for $ty` with the given list of `Lint` statics.
#[macro_export]
macro_rules! impl_lint_pass {
    ($ty:ty => [$($lint:expr),* $(,)?]) => {
        impl $ty {
            pub fn get_lints() -> LintArray { $crate::lint_array!($($lint),*) }
        }
    };
}

/// Call the `check_*` method of each lintpass in CombinedLintLass.check_*.
/// ```ignore
///     fn check_ident(&mut self, handler: &mut Handler, ctx: &mut LintContext, id: &ast::Identifier, ){
///         self.LintPassA.check_ident(handler, ctx, id);
///         self.LintPassB.check_ident(handler, ctx, id);
///         ...
///     }
/// ```
#[macro_export]
macro_rules! expand_combined_lint_pass_method {
    ([$($passes:ident),*], $self: ident, $name: ident, $params:tt) => ({
        $($self.$passes.$name $params;)*
    })
}

/// Expand all methods defined in macro `lint_methods` in the `CombinedLintLass`.
///
/// ```ignore
///     fn check_ident(&mut self, handler: &mut Handler, ctx: &mut LintContext, id: &ast::Identifier){};
///     fn check_stmt(&mut self, handler: &mut Handler, ctx: &mut LintContext, module: &ast::Module){};
///     ...
///  ```
#[macro_export]
macro_rules! expand_combined_lint_pass_methods {
    ($handler:ty, $ctx:ty, $passes:tt, [$($(#[$attr:meta])* fn $name:ident($($param:ident: $arg:ty),*);)*]) => (
        $(fn $name(&mut self, handler: &mut $handler, ctx: &mut $ctx, $($param: $arg),*) {
            expand_combined_lint_pass_method!($passes, self, $name, (handler, ctx, $($param),*));
        })*
    )
}

/// Expand all definitions of `CombinedLintPass`. The results are as follows：
///
/// ```ignore
/// pub struct CombinedLintPass {
///     LintPassA: LintPassA;
///     LintPassB: LintPassB;
///     ...
/// }
///
/// impl CombinedLintPass{
///     pub fn new() -> Self {
///        Self {
///            LintPassA: LintPassA,
///            LintPassB: LintPassB,
///            ...
///        }
///     }
///     pub fn get_lints() -> LintArray {
///         let mut lints = Vec::new();
///         lints.extend_from_slice(&LintPassA::get_lints());
///         lints.extend_from_slice(&LintPassB::get_lints());
///         ...
///         lints
///      }
///  }
///
/// impl LintPass for CombinedLintPass {
///     fn check_ident(&mut self, handler: &mut Handler, ctx: &mut LintContext, id: &ast::Identifier, ){
///         self.LintPassA.check_ident(handler, ctx, id);
///         self.LintPassB.check_ident(handler, ctx, id);
///         ...
///     }
///     fn check_stmt(&mut self, handler: &mut Handler ctx: &mut LintContext, module: &ast::Module){
///         self.LintPassA.check_stmt(handler, ctx, stmt);
///         self.LintPassB.check_stmt(handler, ctx, stmt);
///         ...
///     }
///     ...
/// }
/// ```
#[macro_export]
macro_rules! declare_combined_lint_pass {
    ([$v:vis $name:ident, [$($passes:ident: $constructor:expr,)*]], $methods:tt) => (
        #[allow(non_snake_case)]
        $v struct $name {
            $($passes: $passes,)*
        }

        impl $name {
            $v fn new() -> Self {
                Self {
                    $($passes: $constructor,)*
                }
            }

            $v fn get_lints() -> LintArray {
                let mut lints = Vec::new();
                $(lints.extend_from_slice(&$passes::get_lints());)*
                lints
            }
        }

        impl LintPass for $name {
            expand_combined_lint_pass_methods!(Handler, LintContext,[$($passes),*], $methods);
        }
    )
}

#[macro_export]
macro_rules! default_lint_passes {
    ($macro:path, $args:tt) => {
        $macro!(
            $args,
            [
                ImportPosition: ImportPosition,
                UnusedImport: UnusedImport,
                ReImport: ReImport,
            ]
        );
    };
}

#[macro_export]
macro_rules! declare_combined_default_pass {
    ([$name:ident], $passes:tt) => (
        lint_methods!(declare_combined_lint_pass, [pub $name, $passes]);
    )
}

// Define `CombinedLintPass`.
default_lint_passes!(declare_combined_default_pass, [CombinedLintPass]);

/// The struct `Linter` is used to traverse the AST and call the `check_*` method defined in `CombinedLintPass`.
pub struct Linter<T: LintPass> {
    pub pass: T,
    pub handler: Handler,
    pub ctx: LintContext,
}

/// Record the information at `LintContext` when traversing the AST for analysis across AST nodes, e.g., record
/// used importstmt(used_import_names) when traversing `ast::Identifier` and `ast::SchemaAttr`, and detect unused
/// importstmt after traversing the entire module.
pub struct LintContext {
    /// What source file are we in.
    pub filename: String,
    /// Stores all the registered lint definitions
    pub lintstore: LintArray,
    /// Symbol table, copied from resolver
    pub scope: Scope,
    /// Are we resolving the ast node start position.
    pub start_pos: Position,
    /// Are we resolving the ast node end position.
    pub end_pos: Position,
    // /// Module name and importstmt in it.
    // pub import_names: IndexMap<String, IndexSet<String>>,
}

impl LintContext {
    pub fn dummy_ctx() -> Self {
        LintContext {
            filename: "".to_string(),
            lintstore: CombinedLintPass::get_lints(),
            scope: builtin_scope(),
            start_pos: Position::dummy_pos(),
            end_pos: Position::dummy_pos(),
            // import_names: IndexMap::new(),
        }
    }
}

impl Linter<CombinedLintPass> {
    pub fn new(handler: Handler, ctx: LintContext) -> Self {
        Linter::<CombinedLintPass> {
            pass: CombinedLintPass::new(),
            handler,
            ctx,
        }
    }
    pub fn walk_scope(&mut self, scope: &Scope) {
        self.pass
            .check_scope(&mut self.handler, &mut self.ctx, scope);
    }
}

impl Resolver<'_> {
    pub fn lint_check_module(&mut self, module: &ast::Module) {
        self.linter.ctx.filename = module.filename.clone();
        self.linter.walk_module(module);
    }

    pub fn lint_check_scope(&mut self, scope: &Scope) {
        self.linter.walk_scope(scope);
        for children in &scope.children {
            self.lint_check_scope(&children.borrow().clone())
        }
    }

    pub fn lint_check_scopes(&mut self) {
        let scope_map = self.scope_map.clone();
        for (_, scope) in scope_map.iter() {
            self.lint_check_scope(&scope.borrow())
        }
    }
}

#[macro_export]
macro_rules! walk_set_list {
    ($walker: expr, $method: ident, $list: expr) => {
        for elem in &$list {
            set_pos!($walker, elem);
            $walker.$method(&elem.node)
        }
    };
}

#[macro_export]
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

#[macro_export]
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
        self.pass
            .check_import_stmt(&mut self.handler, &mut self.ctx, import_stmt);
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
                set_pos!(self, &d);
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
