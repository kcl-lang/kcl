use crate::lint::lint::LintContext;
use crate::resolver::scope::Scope;
use kclvm_ast::ast;
use kclvm_error::Handler;

#[macro_export]
/// A summary of the methods that need to be implemented in lintpass, to be added when constructing new lint
/// lint and lintpass. When defining lintpass, the default implementation of these methods is provided: null
/// check (see macro `expand_default_lint_pass_methods`). So what need to do is to override the specific
/// `check_*` function. Some of the methods are commented out here to avoid useless empty functions, which
/// can be added when needed.
macro_rules! lint_methods {
    ($macro:path, $args:tt) => (
        $macro!($args, [

            fn check_scope(_scope: &Scope);

            fn check_module(_module: &ast::Module);
            /*
            * Stmt
            */

            // fn check_expr_stmt(expr_stmt: &ast::ExprStmt);
            // fn check_unification_stmt(unification_stmt: &ast::UnificationStmt);
            // fn check_type_alias_stmt(type_alias_stmt: &ast::TypeAliasStmt);
            // fn check_assign_stmt(assign_stmt: &ast::AssignStmt);
            // fn check_aug_assign_stmt(aug_assign_stmt: &ast::AugAssignStmt);
            // fn check_assert_stmt(assert_stmt: &ast::AssertStmt);
            // fn check_if_stmt(if_stmt: &ast::IfStmt);
            // fn check_import_stmt(import_stmt: &ast::ImportStmt);
            // fn check_schema_stmt(schema_stmt: &ast::SchemaStmt);
            // fn check_rule_stmt(rule_stmt: &ast::RuleStmt);

            /*
            * Expr
            */

            // fn check_expr(expr: &ast::Node<&ast::Expr>);
            // fn check_quant_expr(quant_expr: &ast::QuantExpr);
            // fn check_schema_attr(schema_attr: &ast::SchemaAttr);
            // fn check_if_expr(if_expr: &ast::IfExpr);
            // fn check_unary_expr(unary_expr: &ast::UnaryExpr);
            // fn check_binary_expr(binary_expr: &ast::BinaryExpr);
            // fn check_selector_expr(selector_expr: &ast::SelectorExpr);
            // fn check_call_expr(call_expr: &ast::CallExpr);
            // fn check_subscript(subscript: &ast::Subscript);
            // fn check_paren_expr(paren_expr: &ast::ParenExpr);
            // fn check_list_expr(list_expr: &ast::ListExpr);
            // fn check_list_comp(list_comp: &ast::ListComp);
            // fn check_list_if_item_expr(list_if_item_expr: &ast::ListIfItemExpr);
            // fn check_starred_expr(starred_expr: &ast::StarredExpr);
            // fn check_dict_comp(dict_comp: &ast::DictComp);
            // fn check_config_if_entry_expr(config_if_entry_expr: &ast::ConfigIfEntryExpr,
            // );
            // fn check_comp_clause(comp_clause: &ast::CompClause);
            // fn check_schema_expr(schema_expr: &ast::SchemaExpr);
            // fn check_config_expr(config_expr: &ast::ConfigExpr);
            // fn check_check_expr(check_expr: &ast::CheckExpr);
            // fn check_lambda_expr(lambda_expr: &ast::LambdaExpr);
            // fn check_keyword(keyword: &ast::Keyword);
            // fn check_arguments(arguments: &ast::Arguments);
            // fn check_compare(compare: &ast::Compare);
            // fn check_identifier(id: &ast::Identifier);
            // fn check_number_lit(number_lit: &ast::NumberLit);
            // fn check_string_lit(string_lit: &ast::StringLit);
            // fn check_name_constant_lit(name_constant_lit: &ast::NameConstantLit);
            // fn check_joined_string(joined_string: &ast::JoinedString);
            // fn check_formatted_value(formatted_value: &ast::FormattedValue);
            // fn check_comment(comment: &ast::Comment);
        ]);
    )
}

/// Provide a default implementation of the methods in lint_methods for each lintpass: null checking
macro_rules! expand_default_lint_pass_methods {
    ($handler:ty, $ctx:ty, [$($(#[$attr:meta])* fn $name:ident($($param:ident: $arg:ty),*);)*]) => (
        $(#[inline(always)] fn $name(&mut self, _handler: &mut $handler, _ctx: &mut $ctx, $($param: $arg),*) {})*
    )
}

/// Definition of `LintPass` trait
macro_rules! declare_default_lint_pass_impl {
    ([], [$($methods:tt)*]) => (
        pub trait LintPass {
            expand_default_lint_pass_methods!(Handler, LintContext, [$($methods)*]);
        }
    )
}

// Define LintPass
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
