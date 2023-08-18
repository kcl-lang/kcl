//! AST walker. Each overridden walk method has full control over what
//! happens with its node, it can do its own traversal of the node's children,
//! call `walker::walk_*` to apply the default traversal algorithm, or prevent
//! deeper traversal by doing nothing.
//!
//! According to different usage scenarios, walker can be roughly divided into two categories
//! - Whether we need to modify the AST. If so, it means that we need a 'MutWalker'.
//!   For example, when we need to pre process and desugate AST, which we can use 'MutWalker'.
//! - Whether each AST traversing method the ast node needs to return a value.
//!   If so, it means that we need a `TypedResultWalker`. For example, when we need to traverse
//!   the ast for type checking, we expect each ast node to calculate a type return value,
//!   which we can use `TypedResultWalker`.

use super::ast;

#[macro_export]
macro_rules! walk_list {
    ($walker: expr, $method: ident, $list: expr) => {
        for elem in &$list {
            $walker.$method(&elem.node)
        }
    };
}

#[macro_export]
macro_rules! walk_if {
    ($walker: expr, $method: ident, $value: expr) => {
        match &$value {
            Some(v) => $walker.$method(&v.node),
            None => (),
        }
    };
}

#[macro_export]
macro_rules! walk_list_mut {
    ($walker: ident, $method: ident, $list: expr) => {
        for elem in $list.iter_mut() {
            $walker.$method(&mut elem.node)
        }
    };
}

#[macro_export]
macro_rules! walk_if_mut {
    ($walker: ident, $method: ident, $value: expr) => {
        match $value.as_deref_mut() {
            Some(v) => $walker.$method(&mut v.node),
            None => (),
        }
    };
}

/// Each method of the `TypedResultWalker` trait
/// returns a `Result`
pub trait TypedResultWalker<'ctx>: Sized {
    type Result;

    /*
     * Module
     */

    fn walk_module(&self, module: &'ctx ast::Module) -> Self::Result;

    /*
     * Stmt
     */

    fn walk_stmt(&self, stmt: &'ctx ast::Node<ast::Stmt>) -> Self::Result;
    fn walk_expr_stmt(&self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result;
    fn walk_unification_stmt(&self, unification_stmt: &'ctx ast::UnificationStmt) -> Self::Result;
    fn walk_type_alias_stmt(&self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result;
    fn walk_assign_stmt(&self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result;
    fn walk_aug_assign_stmt(&self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result;
    fn walk_assert_stmt(&self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result;
    fn walk_if_stmt(&self, if_stmt: &'ctx ast::IfStmt) -> Self::Result;
    fn walk_import_stmt(&self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result;
    fn walk_schema_stmt(&self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result;
    fn walk_rule_stmt(&self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result;

    /*
     * Expr
     */

    fn walk_expr(&self, expr: &'ctx ast::Node<ast::Expr>) -> Self::Result;
    fn walk_quant_expr(&self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result;
    fn walk_schema_attr(&self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result;
    fn walk_if_expr(&self, if_expr: &'ctx ast::IfExpr) -> Self::Result;
    fn walk_unary_expr(&self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result;
    fn walk_binary_expr(&self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result;
    fn walk_selector_expr(&self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result;
    fn walk_call_expr(&self, call_expr: &'ctx ast::CallExpr) -> Self::Result;
    fn walk_subscript(&self, subscript: &'ctx ast::Subscript) -> Self::Result;
    fn walk_paren_expr(&self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result;
    fn walk_list_expr(&self, list_expr: &'ctx ast::ListExpr) -> Self::Result;
    fn walk_list_comp(&self, list_comp: &'ctx ast::ListComp) -> Self::Result;
    fn walk_list_if_item_expr(&self, list_if_item_expr: &'ctx ast::ListIfItemExpr) -> Self::Result;
    fn walk_starred_expr(&self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result;
    fn walk_dict_comp(&self, dict_comp: &'ctx ast::DictComp) -> Self::Result;
    fn walk_config_if_entry_expr(
        &self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result;
    fn walk_comp_clause(&self, comp_clause: &'ctx ast::CompClause) -> Self::Result;
    fn walk_schema_expr(&self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result;
    fn walk_config_expr(&self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result;
    fn walk_check_expr(&self, check_expr: &'ctx ast::CheckExpr) -> Self::Result;
    fn walk_lambda_expr(&self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result;
    fn walk_keyword(&self, keyword: &'ctx ast::Keyword) -> Self::Result;
    fn walk_arguments(&self, arguments: &'ctx ast::Arguments) -> Self::Result;
    fn walk_compare(&self, compare: &'ctx ast::Compare) -> Self::Result;
    fn walk_identifier(&self, identifier: &'ctx ast::Identifier) -> Self::Result;
    fn walk_number_lit(&self, number_lit: &'ctx ast::NumberLit) -> Self::Result;
    fn walk_string_lit(&self, string_lit: &'ctx ast::StringLit) -> Self::Result;
    fn walk_name_constant_lit(&self, name_constant_lit: &'ctx ast::NameConstantLit)
        -> Self::Result;
    fn walk_joined_string(&self, joined_string: &'ctx ast::JoinedString) -> Self::Result;
    fn walk_formatted_value(&self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result;
    fn walk_comment(&self, comment: &'ctx ast::Comment) -> Self::Result;
    fn walk_missing_expr(&self, missing_expr: &'ctx ast::MissingExpr) -> Self::Result;
}

/// Each method of the `MutSelfTypedResultWalker` trait returns a typed result.
/// We can use it to calculate some values while traversing the AST and return
/// them through methods. For example, in the process of type checking, we can
/// use it to calculate the type return value of each ast node.
pub trait MutSelfTypedResultWalker<'ctx>: Sized {
    type Result;

    /*
     * Module
     */

    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result;

    /*
     * Stmt
     */

    fn walk_stmt(&mut self, stmt: &'ctx ast::Stmt) -> Self::Result {
        match stmt {
            ast::Stmt::TypeAlias(type_alias) => self.walk_type_alias_stmt(type_alias),
            ast::Stmt::Expr(expr_stmt) => self.walk_expr_stmt(expr_stmt),
            ast::Stmt::Unification(unification_stmt) => {
                self.walk_unification_stmt(unification_stmt)
            }
            ast::Stmt::Assign(assign_stmt) => self.walk_assign_stmt(assign_stmt),
            ast::Stmt::AugAssign(aug_assign_stmt) => self.walk_aug_assign_stmt(aug_assign_stmt),
            ast::Stmt::Assert(assert_stmt) => self.walk_assert_stmt(assert_stmt),
            ast::Stmt::If(if_stmt) => self.walk_if_stmt(if_stmt),
            ast::Stmt::Import(import_stmt) => self.walk_import_stmt(import_stmt),
            ast::Stmt::SchemaAttr(schema_attr) => self.walk_schema_attr(schema_attr),
            ast::Stmt::Schema(schema_stmt) => self.walk_schema_stmt(schema_stmt),
            ast::Stmt::Rule(rule_stmt) => self.walk_rule_stmt(rule_stmt),
        }
    }
    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result;
    fn walk_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result;
    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result;
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result;
    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result;
    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result;
    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) -> Self::Result;
    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result;
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result;
    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result;

    /*
     * Expr
     */

    fn walk_expr(&mut self, expr: &'ctx ast::Expr) -> Self::Result {
        match expr {
            ast::Expr::Identifier(identifier) => self.walk_identifier(identifier),
            ast::Expr::Unary(unary_expr) => self.walk_unary_expr(unary_expr),
            ast::Expr::Binary(binary_expr) => self.walk_binary_expr(binary_expr),
            ast::Expr::If(if_expr) => self.walk_if_expr(if_expr),
            ast::Expr::Selector(selector_expr) => self.walk_selector_expr(selector_expr),
            ast::Expr::Call(call_expr) => self.walk_call_expr(call_expr),
            ast::Expr::Paren(paren_expr) => self.walk_paren_expr(paren_expr),
            ast::Expr::Quant(quant_expr) => self.walk_quant_expr(quant_expr),
            ast::Expr::List(list_expr) => self.walk_list_expr(list_expr),
            ast::Expr::ListIfItem(list_if_item_expr) => {
                self.walk_list_if_item_expr(list_if_item_expr)
            }
            ast::Expr::ListComp(list_comp) => self.walk_list_comp(list_comp),
            ast::Expr::Starred(starred_expr) => self.walk_starred_expr(starred_expr),
            ast::Expr::DictComp(dict_comp) => self.walk_dict_comp(dict_comp),
            ast::Expr::ConfigIfEntry(config_if_entry_expr) => {
                self.walk_config_if_entry_expr(config_if_entry_expr)
            }
            ast::Expr::CompClause(comp_clause) => self.walk_comp_clause(comp_clause),
            ast::Expr::Schema(schema_expr) => self.walk_schema_expr(schema_expr),
            ast::Expr::Config(config_expr) => self.walk_config_expr(config_expr),
            ast::Expr::Check(check) => self.walk_check_expr(check),
            ast::Expr::Lambda(lambda) => self.walk_lambda_expr(lambda),
            ast::Expr::Subscript(subscript) => self.walk_subscript(subscript),
            ast::Expr::Keyword(keyword) => self.walk_keyword(keyword),
            ast::Expr::Arguments(arguments) => self.walk_arguments(arguments),
            ast::Expr::Compare(compare) => self.walk_compare(compare),
            ast::Expr::NumberLit(number_lit) => self.walk_number_lit(number_lit),
            ast::Expr::StringLit(string_lit) => self.walk_string_lit(string_lit),
            ast::Expr::NameConstantLit(name_constant_lit) => {
                self.walk_name_constant_lit(name_constant_lit)
            }
            ast::Expr::JoinedString(joined_string) => self.walk_joined_string(joined_string),
            ast::Expr::FormattedValue(formatted_value) => {
                self.walk_formatted_value(formatted_value)
            }
            ast::Expr::Missing(miss_expr) => self.walk_missing_expr(miss_expr),
        }
    }
    fn walk_quant_expr(&mut self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result;
    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result;
    fn walk_if_expr(&mut self, if_expr: &'ctx ast::IfExpr) -> Self::Result;
    fn walk_unary_expr(&mut self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result;
    fn walk_binary_expr(&mut self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result;
    fn walk_selector_expr(&mut self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result;
    fn walk_call_expr(&mut self, call_expr: &'ctx ast::CallExpr) -> Self::Result;
    fn walk_subscript(&mut self, subscript: &'ctx ast::Subscript) -> Self::Result;
    fn walk_paren_expr(&mut self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result;
    fn walk_list_expr(&mut self, list_expr: &'ctx ast::ListExpr) -> Self::Result;
    fn walk_list_comp(&mut self, list_comp: &'ctx ast::ListComp) -> Self::Result;
    fn walk_list_if_item_expr(
        &mut self,
        list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result;
    fn walk_starred_expr(&mut self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result;
    fn walk_dict_comp(&mut self, dict_comp: &'ctx ast::DictComp) -> Self::Result;
    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result;
    fn walk_comp_clause(&mut self, comp_clause: &'ctx ast::CompClause) -> Self::Result;
    fn walk_schema_expr(&mut self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result;
    fn walk_config_expr(&mut self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result;
    fn walk_check_expr(&mut self, check_expr: &'ctx ast::CheckExpr) -> Self::Result;
    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result;
    fn walk_keyword(&mut self, keyword: &'ctx ast::Keyword) -> Self::Result;
    fn walk_arguments(&mut self, arguments: &'ctx ast::Arguments) -> Self::Result;
    fn walk_compare(&mut self, compare: &'ctx ast::Compare) -> Self::Result;
    fn walk_identifier(&mut self, identifier: &'ctx ast::Identifier) -> Self::Result;
    fn walk_number_lit(&mut self, number_lit: &'ctx ast::NumberLit) -> Self::Result;
    fn walk_string_lit(&mut self, string_lit: &'ctx ast::StringLit) -> Self::Result;
    fn walk_name_constant_lit(
        &mut self,
        name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result;
    fn walk_joined_string(&mut self, joined_string: &'ctx ast::JoinedString) -> Self::Result;
    fn walk_formatted_value(&mut self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result;
    fn walk_comment(&mut self, comment: &'ctx ast::Comment) -> Self::Result;
    fn walk_missing_expr(&mut self, missing_expr: &'ctx ast::MissingExpr) -> Self::Result;
}

/// Each method of the `MutSelfMutWalker` trait returns void type.
/// We can use it to traverse the AST and modify it at the same time.
/// Unlike `MutSelfTypedResultWalker`, each method of `MutSelfMutWalker` has no return value.
pub trait MutSelfMutWalker<'ctx> {
    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx mut ast::ExprStmt) {
        for expr in expr_stmt.exprs.iter_mut() {
            self.walk_expr(&mut expr.node)
        }
    }
    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx mut ast::TypeAliasStmt) {
        self.walk_identifier(&mut type_alias_stmt.type_name.node);
        self.walk_type(&mut type_alias_stmt.ty.node);
    }
    fn walk_unification_stmt(&mut self, unification_stmt: &'ctx mut ast::UnificationStmt) {
        self.walk_identifier(&mut unification_stmt.target.node);
        self.walk_schema_expr(&mut unification_stmt.value.node);
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx mut ast::AssignStmt) {
        for target in assign_stmt.targets.iter_mut() {
            self.walk_identifier(&mut target.node)
        }
        self.walk_expr(&mut assign_stmt.value.node);
        walk_if_mut!(self, walk_type, assign_stmt.ty);
    }
    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx mut ast::AugAssignStmt) {
        self.walk_identifier(&mut aug_assign_stmt.target.node);
        self.walk_expr(&mut aug_assign_stmt.value.node);
    }
    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx mut ast::AssertStmt) {
        self.walk_expr(&mut assert_stmt.test.node);
        walk_if_mut!(self, walk_expr, assert_stmt.if_cond);
        walk_if_mut!(self, walk_expr, assert_stmt.msg);
    }
    fn walk_if_stmt(&mut self, if_stmt: &'ctx mut ast::IfStmt) {
        self.walk_expr(&mut if_stmt.cond.node);
        walk_list_mut!(self, walk_stmt, if_stmt.body);
        walk_list_mut!(self, walk_stmt, if_stmt.orelse);
    }
    fn walk_import_stmt(&mut self, _import_stmt: &'ctx mut ast::ImportStmt) {
        // Nothing to do
    }
    fn walk_schema_attr(&mut self, schema_attr: &'ctx mut ast::SchemaAttr) {
        walk_list_mut!(self, walk_call_expr, schema_attr.decorators);
        walk_if_mut!(self, walk_expr, schema_attr.value);
        self.walk_type(&mut schema_attr.ty.node);
    }

    fn walk_type(&mut self, ty: &'ctx mut ast::Type) {
        match ty {
            ast::Type::Named(id) => self.walk_identifier(id),
            ast::Type::List(list_ty) => {
                if let Some(ty) = &mut list_ty.inner_type {
                    self.walk_type(&mut ty.node)
                }
            }
            ast::Type::Dict(dict_ty) => {
                if let Some(ty) = &mut dict_ty.key_type {
                    self.walk_type(&mut ty.node)
                }
                if let Some(ty) = &mut dict_ty.value_type {
                    self.walk_type(&mut ty.node)
                }
            }
            ast::Type::Union(union_ty) => {
                union_ty
                    .type_elements
                    .iter_mut()
                    .for_each(|ty| self.walk_type(&mut ty.node));
            }
            _ => {}
        }
    }
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx mut ast::SchemaStmt) {
        walk_if_mut!(self, walk_identifier, schema_stmt.parent_name);
        walk_if_mut!(self, walk_identifier, schema_stmt.for_host_name);
        walk_if_mut!(self, walk_arguments, schema_stmt.args);
        if let Some(schema_index_signature) = schema_stmt.index_signature.as_deref_mut() {
            let value = &mut schema_index_signature.node.value;
            walk_if_mut!(self, walk_expr, value);
        }
        walk_list_mut!(self, walk_identifier, schema_stmt.mixins);
        walk_list_mut!(self, walk_call_expr, schema_stmt.decorators);
        walk_list_mut!(self, walk_check_expr, schema_stmt.checks);
        walk_list_mut!(self, walk_stmt, schema_stmt.body);
    }
    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx mut ast::RuleStmt) {
        walk_list_mut!(self, walk_identifier, rule_stmt.parent_rules);
        walk_list_mut!(self, walk_call_expr, rule_stmt.decorators);
        walk_list_mut!(self, walk_check_expr, rule_stmt.checks);
        walk_if_mut!(self, walk_arguments, rule_stmt.args);
        walk_if_mut!(self, walk_identifier, rule_stmt.for_host_name);
    }
    fn walk_quant_expr(&mut self, quant_expr: &'ctx mut ast::QuantExpr) {
        self.walk_expr(&mut quant_expr.target.node);
        walk_list_mut!(self, walk_identifier, quant_expr.variables);
        self.walk_expr(&mut quant_expr.test.node);
        walk_if_mut!(self, walk_expr, quant_expr.if_cond);
    }
    fn walk_if_expr(&mut self, if_expr: &'ctx mut ast::IfExpr) {
        self.walk_expr(&mut if_expr.cond.node);
        self.walk_expr(&mut if_expr.body.node);
        self.walk_expr(&mut if_expr.orelse.node);
    }
    fn walk_unary_expr(&mut self, unary_expr: &'ctx mut ast::UnaryExpr) {
        self.walk_expr(&mut unary_expr.operand.node);
    }
    fn walk_binary_expr(&mut self, binary_expr: &'ctx mut ast::BinaryExpr) {
        self.walk_expr(&mut binary_expr.left.node);
        self.walk_expr(&mut binary_expr.right.node);
    }
    fn walk_selector_expr(&mut self, selector_expr: &'ctx mut ast::SelectorExpr) {
        self.walk_expr(&mut selector_expr.value.node);
        self.walk_identifier(&mut selector_expr.attr.node);
    }
    fn walk_call_expr(&mut self, call_expr: &'ctx mut ast::CallExpr) {
        self.walk_expr(&mut call_expr.func.node);
        walk_list_mut!(self, walk_expr, call_expr.args);
        walk_list_mut!(self, walk_keyword, call_expr.keywords);
    }
    fn walk_subscript(&mut self, subscript: &'ctx mut ast::Subscript) {
        self.walk_expr(&mut subscript.value.node);
        walk_if_mut!(self, walk_expr, subscript.index);
        walk_if_mut!(self, walk_expr, subscript.lower);
        walk_if_mut!(self, walk_expr, subscript.upper);
        walk_if_mut!(self, walk_expr, subscript.step);
    }
    fn walk_paren_expr(&mut self, paren_expr: &'ctx mut ast::ParenExpr) {
        self.walk_expr(&mut paren_expr.expr.node);
    }
    fn walk_list_expr(&mut self, list_expr: &'ctx mut ast::ListExpr) {
        walk_list_mut!(self, walk_expr, list_expr.elts);
    }
    fn walk_list_comp(&mut self, list_comp: &'ctx mut ast::ListComp) {
        self.walk_expr(&mut list_comp.elt.node);
        walk_list_mut!(self, walk_comp_clause, list_comp.generators);
    }
    fn walk_list_if_item_expr(&mut self, list_if_item_expr: &'ctx mut ast::ListIfItemExpr) {
        self.walk_expr(&mut list_if_item_expr.if_cond.node);
        walk_list_mut!(self, walk_expr, list_if_item_expr.exprs);
        walk_if_mut!(self, walk_expr, list_if_item_expr.orelse);
    }
    fn walk_starred_expr(&mut self, starred_expr: &'ctx mut ast::StarredExpr) {
        self.walk_expr(&mut starred_expr.value.node);
    }
    fn walk_dict_comp(&mut self, dict_comp: &'ctx mut ast::DictComp) {
        if let Some(key) = &mut dict_comp.entry.key {
            self.walk_expr(&mut key.node);
        }
        self.walk_expr(&mut dict_comp.entry.value.node);
        walk_list_mut!(self, walk_comp_clause, dict_comp.generators);
    }
    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx mut ast::ConfigIfEntryExpr,
    ) {
        self.walk_expr(&mut config_if_entry_expr.if_cond.node);
        for config_entry in config_if_entry_expr.items.iter_mut() {
            walk_if_mut!(self, walk_expr, config_entry.node.key);
            self.walk_expr(&mut config_entry.node.value.node);
        }
        walk_if_mut!(self, walk_expr, config_if_entry_expr.orelse);
    }
    fn walk_comp_clause(&mut self, comp_clause: &'ctx mut ast::CompClause) {
        walk_list_mut!(self, walk_identifier, comp_clause.targets);
        self.walk_expr(&mut comp_clause.iter.node);
        walk_list_mut!(self, walk_expr, comp_clause.ifs);
    }
    fn walk_schema_expr(&mut self, schema_expr: &'ctx mut ast::SchemaExpr) {
        self.walk_identifier(&mut schema_expr.name.node);
        walk_list_mut!(self, walk_expr, schema_expr.args);
        walk_list_mut!(self, walk_keyword, schema_expr.kwargs);
        self.walk_expr(&mut schema_expr.config.node);
    }
    fn walk_config_expr(&mut self, config_expr: &'ctx mut ast::ConfigExpr) {
        for config_entry in config_expr.items.iter_mut() {
            walk_if_mut!(self, walk_expr, config_entry.node.key);
            self.walk_expr(&mut config_entry.node.value.node);
        }
    }
    fn walk_check_expr(&mut self, check_expr: &'ctx mut ast::CheckExpr) {
        self.walk_expr(&mut check_expr.test.node);
        walk_if_mut!(self, walk_expr, check_expr.if_cond);
        walk_if_mut!(self, walk_expr, check_expr.msg);
    }
    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx mut ast::LambdaExpr) {
        walk_if_mut!(self, walk_arguments, lambda_expr.args);
        walk_list_mut!(self, walk_stmt, lambda_expr.body);
        walk_if_mut!(self, walk_type, lambda_expr.return_ty);
    }
    fn walk_keyword(&mut self, keyword: &'ctx mut ast::Keyword) {
        self.walk_identifier(&mut keyword.arg.node);
        if let Some(v) = keyword.value.as_deref_mut() {
            self.walk_expr(&mut v.node)
        }
    }
    fn walk_arguments(&mut self, arguments: &'ctx mut ast::Arguments) {
        walk_list_mut!(self, walk_identifier, arguments.args);
        for default in arguments.defaults.iter_mut() {
            if let Some(d) = default.as_deref_mut() {
                self.walk_expr(&mut d.node)
            }
        }
        for ty in arguments.ty_list.iter_mut() {
            if let Some(ty) = ty.as_deref_mut() {
                self.walk_type(&mut ty.node);
            }
        }
    }
    fn walk_compare(&mut self, compare: &'ctx mut ast::Compare) {
        self.walk_expr(&mut compare.left.node);
        walk_list_mut!(self, walk_expr, compare.comparators);
    }
    fn walk_identifier(&mut self, identifier: &'ctx mut ast::Identifier) {
        // Nothing to do.
        let _ = identifier;
    }
    fn walk_number_lit(&mut self, number_lit: &'ctx mut ast::NumberLit) {
        let _ = number_lit;
    }
    fn walk_string_lit(&mut self, string_lit: &'ctx mut ast::StringLit) {
        // Nothing to do.
        let _ = string_lit;
    }
    fn walk_name_constant_lit(&mut self, name_constant_lit: &'ctx mut ast::NameConstantLit) {
        // Nothing to do.
        let _ = name_constant_lit;
    }
    fn walk_joined_string(&mut self, joined_string: &'ctx mut ast::JoinedString) {
        walk_list_mut!(self, walk_expr, joined_string.values);
    }
    fn walk_formatted_value(&mut self, formatted_value: &'ctx mut ast::FormattedValue) {
        self.walk_expr(&mut formatted_value.value.node);
    }
    fn walk_comment(&mut self, comment: &'ctx mut ast::Comment) {
        // Nothing to do.
        let _ = comment;
    }
    fn walk_missing_expr(&mut self, missing_expr: &'ctx mut ast::MissingExpr) {
        // Nothing to do.
        let _ = missing_expr;
    }
    fn walk_module(&mut self, module: &'ctx mut ast::Module) {
        walk_list_mut!(self, walk_stmt, module.body)
    }
    fn walk_stmt(&mut self, stmt: &'ctx mut ast::Stmt) {
        match stmt {
            ast::Stmt::TypeAlias(type_alias) => self.walk_type_alias_stmt(type_alias),
            ast::Stmt::Expr(expr_stmt) => self.walk_expr_stmt(expr_stmt),
            ast::Stmt::Unification(unification_stmt) => {
                self.walk_unification_stmt(unification_stmt)
            }
            ast::Stmt::Assign(assign_stmt) => self.walk_assign_stmt(assign_stmt),
            ast::Stmt::AugAssign(aug_assign_stmt) => self.walk_aug_assign_stmt(aug_assign_stmt),
            ast::Stmt::Assert(assert_stmt) => self.walk_assert_stmt(assert_stmt),
            ast::Stmt::If(if_stmt) => self.walk_if_stmt(if_stmt),
            ast::Stmt::Import(import_stmt) => self.walk_import_stmt(import_stmt),
            ast::Stmt::SchemaAttr(schema_attr) => self.walk_schema_attr(schema_attr),
            ast::Stmt::Schema(schema_stmt) => self.walk_schema_stmt(schema_stmt),
            ast::Stmt::Rule(rule_stmt) => self.walk_rule_stmt(rule_stmt),
        }
    }
    fn walk_expr(&mut self, expr: &'ctx mut ast::Expr) {
        match expr {
            ast::Expr::Identifier(identifier) => self.walk_identifier(identifier),
            ast::Expr::Unary(unary_expr) => self.walk_unary_expr(unary_expr),
            ast::Expr::Binary(binary_expr) => self.walk_binary_expr(binary_expr),
            ast::Expr::If(if_expr) => self.walk_if_expr(if_expr),
            ast::Expr::Selector(selector_expr) => self.walk_selector_expr(selector_expr),
            ast::Expr::Call(call_expr) => self.walk_call_expr(call_expr),
            ast::Expr::Paren(paren_expr) => self.walk_paren_expr(paren_expr),
            ast::Expr::Quant(quant_expr) => self.walk_quant_expr(quant_expr),
            ast::Expr::List(list_expr) => self.walk_list_expr(list_expr),
            ast::Expr::ListIfItem(list_if_item_expr) => {
                self.walk_list_if_item_expr(list_if_item_expr)
            }
            ast::Expr::ListComp(list_comp) => self.walk_list_comp(list_comp),
            ast::Expr::Starred(starred_expr) => self.walk_starred_expr(starred_expr),
            ast::Expr::DictComp(dict_comp) => self.walk_dict_comp(dict_comp),
            ast::Expr::ConfigIfEntry(config_if_entry_expr) => {
                self.walk_config_if_entry_expr(config_if_entry_expr)
            }
            ast::Expr::CompClause(comp_clause) => self.walk_comp_clause(comp_clause),
            ast::Expr::Schema(schema_expr) => self.walk_schema_expr(schema_expr),
            ast::Expr::Config(config_expr) => self.walk_config_expr(config_expr),
            ast::Expr::Check(check) => self.walk_check_expr(check),
            ast::Expr::Lambda(lambda) => self.walk_lambda_expr(lambda),
            ast::Expr::Subscript(subscript) => self.walk_subscript(subscript),
            ast::Expr::Keyword(keyword) => self.walk_keyword(keyword),
            ast::Expr::Arguments(arguments) => self.walk_arguments(arguments),
            ast::Expr::Compare(compare) => self.walk_compare(compare),
            ast::Expr::NumberLit(number_lit) => self.walk_number_lit(number_lit),
            ast::Expr::StringLit(string_lit) => self.walk_string_lit(string_lit),
            ast::Expr::NameConstantLit(name_constant_lit) => {
                self.walk_name_constant_lit(name_constant_lit)
            }
            ast::Expr::JoinedString(joined_string) => self.walk_joined_string(joined_string),
            ast::Expr::FormattedValue(formatted_value) => {
                self.walk_formatted_value(formatted_value)
            }
            ast::Expr::Missing(missing_expr) => self.walk_missing_expr(missing_expr),
        }
    }
}

/// Each method of the `Walker` trait is a hook to be potentially
/// overridden. Each method's default implementation recursively visits
/// the substructure of the input via the corresponding `walk` method;
/// e.g., the `walk_item` method by default calls `walker::walk_item`.
///
/// If you want to ensure that your code handles every variant
/// explicitly, you need to override each method. (And you also need
/// to monitor future changes to `Walker` in case a new method with a
/// new default implementation gets introduced.)
pub trait Walker<'ctx>: TypedResultWalker<'ctx> {
    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx ast::ExprStmt) {
        walk_expr_stmt(self, expr_stmt);
    }
    fn walk_unification_stmt(&mut self, unification_stmt: &'ctx ast::UnificationStmt) {
        walk_unification_stmt(self, unification_stmt);
    }
    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) {
        walk_type_alias_stmt(self, type_alias_stmt);
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) {
        walk_assign_stmt(self, assign_stmt);
    }
    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx ast::AugAssignStmt) {
        walk_aug_assign_stmt(self, aug_assign_stmt);
    }
    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx ast::AssertStmt) {
        walk_assert_stmt(self, assert_stmt);
    }
    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) {
        walk_if_stmt(self, if_stmt);
    }
    fn walk_import_stmt(&mut self, import_stmt: &'ctx ast::ImportStmt) {
        walk_import_stmt(self, import_stmt);
    }
    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) {
        walk_schema_attr(self, schema_attr);
    }
    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) {
        walk_schema_stmt(self, schema_stmt);
    }
    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) {
        walk_rule_stmt(self, rule_stmt);
    }
    fn walk_quant_expr(&mut self, quant_expr: &'ctx ast::QuantExpr) {
        walk_quant_expr(self, quant_expr);
    }
    fn walk_if_expr(&mut self, if_expr: &'ctx ast::IfExpr) {
        walk_if_expr(self, if_expr);
    }
    fn walk_unary_expr(&mut self, unary_expr: &'ctx ast::UnaryExpr) {
        walk_unary_expr(self, unary_expr);
    }
    fn walk_binary_expr(&mut self, binary_expr: &'ctx ast::BinaryExpr) {
        walk_binary_expr(self, binary_expr);
    }
    fn walk_selector_expr(&mut self, selector_expr: &'ctx ast::SelectorExpr) {
        walk_selector_expr(self, selector_expr);
    }
    fn walk_call_expr(&mut self, call_expr: &'ctx ast::CallExpr) {
        walk_call_expr(self, call_expr);
    }
    fn walk_subscript(&mut self, subscript: &'ctx ast::Subscript) {
        walk_subscript(self, subscript);
    }
    fn walk_paren_expr(&mut self, paren_expr: &'ctx ast::ParenExpr) {
        walk_paren_expr(self, paren_expr);
    }
    fn walk_list_expr(&mut self, list_expr: &'ctx ast::ListExpr) {
        walk_list_expr(self, list_expr);
    }
    fn walk_list_comp(&mut self, list_comp: &'ctx ast::ListComp) {
        walk_list_comp(self, list_comp);
    }
    fn walk_list_if_item_expr(&mut self, list_if_item_expr: &'ctx ast::ListIfItemExpr) {
        walk_list_if_item_expr(self, list_if_item_expr);
    }
    fn walk_starred_expr(&mut self, starred_expr: &'ctx ast::StarredExpr) {
        walk_starred_expr(self, starred_expr);
    }
    fn walk_dict_comp(&mut self, dict_comp: &'ctx ast::DictComp) {
        walk_dict_comp(self, dict_comp);
    }
    fn walk_config_if_entry_expr(&mut self, config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr) {
        walk_config_if_entry_expr(self, config_if_entry_expr);
    }
    fn walk_comp_clause(&mut self, comp_clause: &'ctx ast::CompClause) {
        walk_comp_clause(self, comp_clause);
    }
    fn walk_schema_expr(&mut self, schema_expr: &'ctx ast::SchemaExpr) {
        walk_schema_expr(self, schema_expr);
    }
    fn walk_config_expr(&mut self, config_expr: &'ctx ast::ConfigExpr) {
        walk_config_expr(self, config_expr);
    }
    fn walk_check_expr(&mut self, check_expr: &'ctx ast::CheckExpr) {
        walk_check_expr(self, check_expr);
    }
    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx ast::LambdaExpr) {
        walk_lambda_expr(self, lambda_expr);
    }
    fn walk_keyword(&mut self, keyword: &'ctx ast::Keyword) {
        walk_keyword(self, keyword);
    }
    fn walk_arguments(&mut self, arguments: &'ctx ast::Arguments) {
        walk_arguments(self, arguments);
    }
    fn walk_compare(&mut self, compare: &'ctx ast::Compare) {
        walk_compare(self, compare);
    }
    fn walk_identifier(&mut self, identifier: &'ctx ast::Identifier) {
        walk_identifier(self, identifier);
    }
    fn walk_number_lit(&mut self, number_lit: &'ctx ast::NumberLit) {
        walk_number_lit(self, number_lit);
    }
    fn walk_string_lit(&mut self, string_lit: &'ctx ast::StringLit) {
        walk_string_lit(self, string_lit);
    }
    fn walk_name_constant_lit(&mut self, name_constant_lit: &'ctx ast::NameConstantLit) {
        walk_name_constant_lit(self, name_constant_lit);
    }
    fn walk_joined_string(&mut self, joined_string: &'ctx ast::JoinedString) {
        walk_joined_string(self, joined_string);
    }
    fn walk_formatted_value(&mut self, formatted_value: &'ctx ast::FormattedValue) {
        walk_formatted_value(self, formatted_value);
    }
    fn walk_comment(&mut self, comment: &'ctx ast::Comment) {
        walk_comment(self, comment);
    }
    fn walk_missing_expr(&mut self, missing_expr: &'ctx ast::MissingExpr);
    fn walk_module(&mut self, module: &'ctx ast::Module) {
        walk_module(self, module);
    }
    fn walk_stmt(&mut self, stmt: &'ctx ast::Stmt) {
        walk_stmt(self, stmt)
    }
    fn walk_expr(&mut self, expr: &'ctx ast::Expr) {
        walk_expr(self, expr)
    }
}

pub fn walk_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, expr: &'ctx ast::Expr) {
    match expr {
        ast::Expr::Identifier(identifier) => walker.walk_identifier(identifier),
        ast::Expr::Unary(unary_expr) => walker.walk_unary_expr(unary_expr),
        ast::Expr::Binary(binary_expr) => walker.walk_binary_expr(binary_expr),
        ast::Expr::If(if_expr) => walker.walk_if_expr(if_expr),
        ast::Expr::Selector(selector_expr) => walker.walk_selector_expr(selector_expr),
        ast::Expr::Call(call_expr) => walker.walk_call_expr(call_expr),
        ast::Expr::Paren(paren_expr) => walker.walk_paren_expr(paren_expr),
        ast::Expr::Quant(quant_expr) => walker.walk_quant_expr(quant_expr),
        ast::Expr::List(list_expr) => walker.walk_list_expr(list_expr),
        ast::Expr::ListIfItem(list_if_item_expr) => {
            walker.walk_list_if_item_expr(list_if_item_expr)
        }
        ast::Expr::ListComp(list_comp) => walker.walk_list_comp(list_comp),
        ast::Expr::Starred(starred_expr) => walker.walk_starred_expr(starred_expr),
        ast::Expr::DictComp(dict_comp) => walker.walk_dict_comp(dict_comp),
        ast::Expr::ConfigIfEntry(config_if_entry_expr) => {
            walker.walk_config_if_entry_expr(config_if_entry_expr)
        }
        ast::Expr::CompClause(comp_clause) => walker.walk_comp_clause(comp_clause),
        ast::Expr::Schema(schema_expr) => walker.walk_schema_expr(schema_expr),
        ast::Expr::Config(config_expr) => walker.walk_config_expr(config_expr),
        ast::Expr::Check(check) => walker.walk_check_expr(check),
        ast::Expr::Lambda(lambda) => walker.walk_lambda_expr(lambda),
        ast::Expr::Subscript(subscript) => walker.walk_subscript(subscript),
        ast::Expr::Keyword(keyword) => walker.walk_keyword(keyword),
        ast::Expr::Arguments(arguments) => walker.walk_arguments(arguments),
        ast::Expr::Compare(compare) => walker.walk_compare(compare),
        ast::Expr::NumberLit(number_lit) => walker.walk_number_lit(number_lit),
        ast::Expr::StringLit(string_lit) => walker.walk_string_lit(string_lit),
        ast::Expr::NameConstantLit(name_constant_lit) => {
            walker.walk_name_constant_lit(name_constant_lit)
        }
        ast::Expr::JoinedString(joined_string) => walker.walk_joined_string(joined_string),
        ast::Expr::FormattedValue(formatted_value) => walker.walk_formatted_value(formatted_value),
        ast::Expr::Missing(missing_expr) => walker.walk_missing_expr(missing_expr),
    }
}

pub fn walk_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, stmt: &'ctx ast::Stmt) {
    match stmt {
        ast::Stmt::TypeAlias(type_alias) => walker.walk_type_alias_stmt(type_alias),
        ast::Stmt::Expr(expr_stmt) => walker.walk_expr_stmt(expr_stmt),
        ast::Stmt::Unification(unification_stmt) => walker.walk_unification_stmt(unification_stmt),
        ast::Stmt::Assign(assign_stmt) => walker.walk_assign_stmt(assign_stmt),
        ast::Stmt::AugAssign(aug_assign_stmt) => walker.walk_aug_assign_stmt(aug_assign_stmt),
        ast::Stmt::Assert(assert_stmt) => walker.walk_assert_stmt(assert_stmt),
        ast::Stmt::If(if_stmt) => walker.walk_if_stmt(if_stmt),
        ast::Stmt::Import(import_stmt) => walker.walk_import_stmt(import_stmt),
        ast::Stmt::SchemaAttr(schema_attr) => walker.walk_schema_attr(schema_attr),
        ast::Stmt::Schema(schema_stmt) => walker.walk_schema_stmt(schema_stmt),
        ast::Stmt::Rule(rule_stmt) => walker.walk_rule_stmt(rule_stmt),
    }
}

pub fn walk_expr_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, expr_stmt: &'ctx ast::ExprStmt) {
    walk_list!(walker, walk_expr, expr_stmt.exprs);
}

pub fn walk_unification_stmt<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    unification_stmt: &'ctx ast::UnificationStmt,
) {
    walker.walk_identifier(&unification_stmt.target.node);
    walker.walk_schema_expr(&unification_stmt.value.node);
}

pub fn walk_type_alias_stmt<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    type_alias_stmt: &'ctx ast::TypeAliasStmt,
) {
    walker.walk_identifier(&type_alias_stmt.type_name.node);
}

pub fn walk_assign_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, assign_stmt: &'ctx ast::AssignStmt) {
    walk_list!(walker, walk_identifier, assign_stmt.targets);
    walker.walk_expr(&assign_stmt.value.node);
}

pub fn walk_aug_assign_stmt<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    aug_assign_stmt: &'ctx ast::AugAssignStmt,
) {
    walker.walk_identifier(&aug_assign_stmt.target.node);
    walker.walk_expr(&aug_assign_stmt.value.node);
}

pub fn walk_assert_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, assert_stmt: &'ctx ast::AssertStmt) {
    walker.walk_expr(&assert_stmt.test.node);
    walk_if!(walker, walk_expr, assert_stmt.if_cond);
    walk_if!(walker, walk_expr, assert_stmt.msg);
}

pub fn walk_if_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, if_stmt: &'ctx ast::IfStmt) {
    walker.walk_expr(&if_stmt.cond.node);
    walk_list!(walker, walk_stmt, if_stmt.body);
    walk_list!(walker, walk_stmt, if_stmt.orelse);
}

pub fn walk_import_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, import_stmt: &'ctx ast::ImportStmt) {
    let _ = walker;
    let _ = import_stmt;
}

pub fn walk_schema_attr<'ctx, V: Walker<'ctx>>(walker: &mut V, schema_attr: &'ctx ast::SchemaAttr) {
    walk_list!(walker, walk_call_expr, schema_attr.decorators);
    walk_if!(walker, walk_expr, schema_attr.value);
}

pub fn walk_schema_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, schema_stmt: &'ctx ast::SchemaStmt) {
    walk_if!(walker, walk_identifier, schema_stmt.parent_name);
    walk_if!(walker, walk_identifier, schema_stmt.for_host_name);
    walk_if!(walker, walk_arguments, schema_stmt.args);
    if let Some(schema_index_signature) = &schema_stmt.index_signature {
        walk_schema_index_signature(walker, &schema_index_signature.node);
    }
    walk_list!(walker, walk_identifier, schema_stmt.mixins);
    walk_list!(walker, walk_call_expr, schema_stmt.decorators);
    walk_list!(walker, walk_check_expr, schema_stmt.checks);
    walk_list!(walker, walk_stmt, schema_stmt.body);
}

pub fn walk_rule_stmt<'ctx, V: Walker<'ctx>>(walker: &mut V, rule_stmt: &'ctx ast::RuleStmt) {
    walk_list!(walker, walk_identifier, rule_stmt.parent_rules);
    walk_list!(walker, walk_call_expr, rule_stmt.decorators);
    walk_list!(walker, walk_check_expr, rule_stmt.checks);
    walk_if!(walker, walk_arguments, rule_stmt.args);
    walk_if!(walker, walk_identifier, rule_stmt.for_host_name);
}

pub fn walk_quant_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, quant_expr: &'ctx ast::QuantExpr) {
    walker.walk_expr(&quant_expr.target.node);
    walk_list!(walker, walk_identifier, quant_expr.variables);
    walker.walk_expr(&quant_expr.test.node);
    walk_if!(walker, walk_expr, quant_expr.if_cond);
}

pub fn walk_schema_index_signature<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    schema_index_signature: &'ctx ast::SchemaIndexSignature,
) {
    walk_if!(walker, walk_expr, schema_index_signature.value);
}

pub fn walk_if_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, if_expr: &'ctx ast::IfExpr) {
    walker.walk_expr(&if_expr.cond.node);
    walker.walk_expr(&if_expr.body.node);
    walker.walk_expr(&if_expr.orelse.node);
}

pub fn walk_unary_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, unary_expr: &'ctx ast::UnaryExpr) {
    walker.walk_expr(&unary_expr.operand.node);
}

pub fn walk_binary_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, binary_expr: &'ctx ast::BinaryExpr) {
    walker.walk_expr(&binary_expr.left.node);
    walker.walk_expr(&binary_expr.right.node);
}

pub fn walk_selector_expr<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    selector_expr: &'ctx ast::SelectorExpr,
) {
    walker.walk_expr(&selector_expr.value.node);
    walker.walk_identifier(&selector_expr.attr.node);
}

pub fn walk_call_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, call_expr: &'ctx ast::CallExpr) {
    walker.walk_expr(&call_expr.func.node);
    walk_list!(walker, walk_expr, call_expr.args);
    walk_list!(walker, walk_keyword, call_expr.keywords);
}

pub fn walk_subscript<'ctx, V: Walker<'ctx>>(walker: &mut V, subscript: &'ctx ast::Subscript) {
    walker.walk_expr(&subscript.value.node);
    walk_if!(walker, walk_expr, subscript.index);
    walk_if!(walker, walk_expr, subscript.lower);
    walk_if!(walker, walk_expr, subscript.upper);
    walk_if!(walker, walk_expr, subscript.step);
}

pub fn walk_paren_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, paren_expr: &'ctx ast::ParenExpr) {
    walker.walk_expr(&paren_expr.expr.node);
}

pub fn walk_list_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, list_expr: &'ctx ast::ListExpr) {
    walk_list!(walker, walk_expr, list_expr.elts);
}

pub fn walk_list_comp<'ctx, V: Walker<'ctx>>(walker: &mut V, list_comp: &'ctx ast::ListComp) {
    walker.walk_expr(&list_comp.elt.node);
    walk_list!(walker, walk_comp_clause, list_comp.generators);
}

pub fn walk_list_if_item_expr<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    list_if_item_expr: &'ctx ast::ListIfItemExpr,
) {
    walker.walk_expr(&list_if_item_expr.if_cond.node);
    walk_list!(walker, walk_expr, list_if_item_expr.exprs);
    walk_if!(walker, walk_expr, list_if_item_expr.orelse);
}

pub fn walk_starred_expr<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    starred_expr: &'ctx ast::StarredExpr,
) {
    walker.walk_expr(&starred_expr.value.node);
}

pub fn walk_dict_comp<'ctx, V: Walker<'ctx>>(walker: &mut V, dict_comp: &'ctx ast::DictComp) {
    if let Some(key) = &dict_comp.entry.key {
        walker.walk_expr(&key.node);
    }
    walker.walk_expr(&dict_comp.entry.value.node);
    walk_list!(walker, walk_comp_clause, dict_comp.generators);
}

pub fn walk_config_if_entry_expr<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
) {
    walker.walk_expr(&config_if_entry_expr.if_cond.node);
    for config_entry in &config_if_entry_expr.items {
        walk_if!(walker, walk_expr, config_entry.node.key);
        walker.walk_expr(&config_entry.node.value.node);
    }
    walk_if!(walker, walk_expr, config_if_entry_expr.orelse);
}

pub fn walk_comp_clause<'ctx, V: Walker<'ctx>>(walker: &mut V, comp_clause: &'ctx ast::CompClause) {
    walk_list!(walker, walk_identifier, comp_clause.targets);
    walker.walk_expr(&comp_clause.iter.node);
    walk_list!(walker, walk_expr, comp_clause.ifs);
}

pub fn walk_schema_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, schema_expr: &'ctx ast::SchemaExpr) {
    walker.walk_identifier(&schema_expr.name.node);
    walk_list!(walker, walk_expr, schema_expr.args);
    walk_list!(walker, walk_keyword, schema_expr.kwargs);
    walker.walk_expr(&schema_expr.config.node);
}

pub fn walk_config_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, config_expr: &'ctx ast::ConfigExpr) {
    for config_entry in &config_expr.items {
        walk_if!(walker, walk_expr, config_entry.node.key);
        walker.walk_expr(&config_entry.node.value.node);
    }
}

pub fn walk_check_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, check_expr: &'ctx ast::CheckExpr) {
    walker.walk_expr(&check_expr.test.node);
    walk_if!(walker, walk_expr, check_expr.if_cond);
    walk_if!(walker, walk_expr, check_expr.msg);
}

pub fn walk_lambda_expr<'ctx, V: Walker<'ctx>>(walker: &mut V, lambda_expr: &'ctx ast::LambdaExpr) {
    walk_if!(walker, walk_arguments, lambda_expr.args);
    walk_list!(walker, walk_stmt, lambda_expr.body);
}

pub fn walk_keyword<'ctx, V: Walker<'ctx>>(walker: &mut V, keyword: &'ctx ast::Keyword) {
    walker.walk_identifier(&keyword.arg.node);
    match &keyword.value {
        Some(v) => walker.walk_expr(&v.node),
        None => (),
    }
}

pub fn walk_arguments<'ctx, V: Walker<'ctx>>(walker: &mut V, arguments: &'ctx ast::Arguments) {
    walk_list!(walker, walk_identifier, arguments.args);
    for default in &arguments.defaults {
        if let Some(d) = default.as_ref() {
            walker.walk_expr(&d.node)
        }
    }
}

pub fn walk_compare<'ctx, V: Walker<'ctx>>(walker: &mut V, compare: &'ctx ast::Compare) {
    walker.walk_expr(&compare.left.node);
    walk_list!(walker, walk_expr, compare.comparators);
}

pub fn walk_identifier<'ctx, V: Walker<'ctx>>(walker: &mut V, identifier: &'ctx ast::Identifier) {
    // Nothing to do.
    let _ = walker;
    let _ = identifier;
}

pub fn walk_number_lit<'ctx, V: Walker<'ctx>>(walker: &mut V, number_lit: &'ctx ast::NumberLit) {
    // Nothing to do.
    let _ = walker;
    let _ = number_lit;
}

pub fn walk_string_lit<'ctx, V: Walker<'ctx>>(walker: &mut V, string_lit: &'ctx ast::StringLit) {
    // Nothing to do.
    let _ = walker;
    let _ = string_lit;
}

pub fn walk_name_constant_lit<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    name_constant_lit: &'ctx ast::NameConstantLit,
) {
    // Nothing to do.
    let _ = walker;
    let _ = name_constant_lit;
}

pub fn walk_joined_string<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    joined_string: &'ctx ast::JoinedString,
) {
    walk_list!(walker, walk_expr, joined_string.values);
}

pub fn walk_formatted_value<'ctx, V: Walker<'ctx>>(
    walker: &mut V,
    formatted_value: &'ctx ast::FormattedValue,
) {
    walker.walk_expr(&formatted_value.value.node);
}

pub fn walk_comment<'ctx, V: Walker<'ctx>>(walker: &mut V, comment: &'ctx ast::Comment) {
    // Nothing to do.
    let _ = walker;
    let _ = comment;
}

pub fn walk_module<'ctx, V: Walker<'ctx>>(walker: &mut V, module: &'ctx ast::Module) {
    walk_list!(walker, walk_stmt, module.body)
}

/// Each method of the `MutSelfWalker` trait returns void type and does not need to modify the AST.
/// We can use it to traverse the AST and do some check at the same time, For example, in the process
/// of lint checking, we can use it to check each AST node and generate diagnostcs.
pub trait MutSelfWalker {
    fn walk_expr_stmt(&mut self, expr_stmt: &ast::ExprStmt) {
        for expr in &expr_stmt.exprs {
            self.walk_expr(&expr.node)
        }
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &ast::TypeAliasStmt) {
        self.walk_identifier(&type_alias_stmt.type_name.node);
    }
    fn walk_unification_stmt(&mut self, unification_stmt: &ast::UnificationStmt) {
        self.walk_identifier(&unification_stmt.target.node);
        self.walk_schema_expr(&unification_stmt.value.node);
    }
    fn walk_assign_stmt(&mut self, assign_stmt: &ast::AssignStmt) {
        for target in &assign_stmt.targets {
            self.walk_identifier(&target.node)
        }
        self.walk_expr(&assign_stmt.value.node);
    }
    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &ast::AugAssignStmt) {
        self.walk_identifier(&aug_assign_stmt.target.node);
        self.walk_expr(&aug_assign_stmt.value.node);
    }
    fn walk_assert_stmt(&mut self, assert_stmt: &ast::AssertStmt) {
        self.walk_expr(&assert_stmt.test.node);
        walk_if!(self, walk_expr, assert_stmt.if_cond);
        walk_if!(self, walk_expr, assert_stmt.msg);
    }
    fn walk_if_stmt(&mut self, if_stmt: &ast::IfStmt) {
        self.walk_expr(&if_stmt.cond.node);
        walk_list!(self, walk_stmt, if_stmt.body);
        walk_list!(self, walk_stmt, if_stmt.orelse);
    }
    fn walk_import_stmt(&mut self, _import_stmt: &ast::ImportStmt) {
        // Nothing to do
    }
    fn walk_schema_attr(&mut self, schema_attr: &ast::SchemaAttr) {
        walk_list!(self, walk_call_expr, schema_attr.decorators);
        walk_if!(self, walk_expr, schema_attr.value);
    }
    fn walk_schema_stmt(&mut self, schema_stmt: &ast::SchemaStmt) {
        walk_if!(self, walk_identifier, schema_stmt.parent_name);
        walk_if!(self, walk_identifier, schema_stmt.for_host_name);
        walk_if!(self, walk_arguments, schema_stmt.args);
        if let Some(schema_index_signature) = &schema_stmt.index_signature {
            let value = &schema_index_signature.node.value;
            walk_if!(self, walk_expr, value);
        }
        walk_list!(self, walk_identifier, schema_stmt.mixins);
        walk_list!(self, walk_call_expr, schema_stmt.decorators);
        walk_list!(self, walk_check_expr, schema_stmt.checks);
        walk_list!(self, walk_stmt, schema_stmt.body);
    }
    fn walk_rule_stmt(&mut self, rule_stmt: &ast::RuleStmt) {
        walk_list!(self, walk_identifier, rule_stmt.parent_rules);
        walk_list!(self, walk_call_expr, rule_stmt.decorators);
        walk_list!(self, walk_check_expr, rule_stmt.checks);
        walk_if!(self, walk_arguments, rule_stmt.args);
        walk_if!(self, walk_identifier, rule_stmt.for_host_name);
    }
    fn walk_quant_expr(&mut self, quant_expr: &ast::QuantExpr) {
        self.walk_expr(&quant_expr.target.node);
        walk_list!(self, walk_identifier, quant_expr.variables);
        self.walk_expr(&quant_expr.test.node);
        walk_if!(self, walk_expr, quant_expr.if_cond);
    }
    fn walk_if_expr(&mut self, if_expr: &ast::IfExpr) {
        self.walk_expr(&if_expr.cond.node);
        self.walk_expr(&if_expr.body.node);
        self.walk_expr(&if_expr.orelse.node);
    }
    fn walk_unary_expr(&mut self, unary_expr: &ast::UnaryExpr) {
        self.walk_expr(&unary_expr.operand.node);
    }
    fn walk_binary_expr(&mut self, binary_expr: &ast::BinaryExpr) {
        self.walk_expr(&binary_expr.left.node);
        self.walk_expr(&binary_expr.right.node);
    }
    fn walk_selector_expr(&mut self, selector_expr: &ast::SelectorExpr) {
        self.walk_expr(&selector_expr.value.node);
        self.walk_identifier(&selector_expr.attr.node);
    }
    fn walk_call_expr(&mut self, call_expr: &ast::CallExpr) {
        self.walk_expr(&call_expr.func.node);
        walk_list!(self, walk_expr, call_expr.args);
        walk_list!(self, walk_keyword, call_expr.keywords);
    }
    fn walk_subscript(&mut self, subscript: &ast::Subscript) {
        self.walk_expr(&subscript.value.node);
        walk_if!(self, walk_expr, subscript.index);
        walk_if!(self, walk_expr, subscript.lower);
        walk_if!(self, walk_expr, subscript.upper);
        walk_if!(self, walk_expr, subscript.step);
    }
    fn walk_paren_expr(&mut self, paren_expr: &ast::ParenExpr) {
        self.walk_expr(&paren_expr.expr.node);
    }
    fn walk_list_expr(&mut self, list_expr: &ast::ListExpr) {
        walk_list!(self, walk_expr, list_expr.elts);
    }
    fn walk_list_comp(&mut self, list_comp: &ast::ListComp) {
        self.walk_expr(&list_comp.elt.node);
        walk_list!(self, walk_comp_clause, list_comp.generators);
    }
    fn walk_list_if_item_expr(&mut self, list_if_item_expr: &ast::ListIfItemExpr) {
        self.walk_expr(&list_if_item_expr.if_cond.node);
        walk_list!(self, walk_expr, list_if_item_expr.exprs);
        walk_if!(self, walk_expr, list_if_item_expr.orelse);
    }
    fn walk_starred_expr(&mut self, starred_expr: &ast::StarredExpr) {
        self.walk_expr(&starred_expr.value.node);
    }
    fn walk_dict_comp(&mut self, dict_comp: &ast::DictComp) {
        if let Some(key) = &dict_comp.entry.key {
            self.walk_expr(&key.node);
        }
        self.walk_expr(&dict_comp.entry.value.node);
        walk_list!(self, walk_comp_clause, dict_comp.generators);
    }
    fn walk_config_if_entry_expr(&mut self, config_if_entry_expr: &ast::ConfigIfEntryExpr) {
        self.walk_expr(&config_if_entry_expr.if_cond.node);
        for config_entry in &config_if_entry_expr.items {
            walk_if!(self, walk_expr, config_entry.node.key);
            self.walk_expr(&config_entry.node.value.node);
        }
        walk_if!(self, walk_expr, config_if_entry_expr.orelse);
    }
    fn walk_comp_clause(&mut self, comp_clause: &ast::CompClause) {
        walk_list!(self, walk_identifier, comp_clause.targets);
        self.walk_expr(&comp_clause.iter.node);
        walk_list!(self, walk_expr, comp_clause.ifs);
    }
    fn walk_schema_expr(&mut self, schema_expr: &ast::SchemaExpr) {
        self.walk_identifier(&schema_expr.name.node);
        walk_list!(self, walk_expr, schema_expr.args);
        walk_list!(self, walk_keyword, schema_expr.kwargs);
        self.walk_expr(&schema_expr.config.node);
    }
    fn walk_config_expr(&mut self, config_expr: &ast::ConfigExpr) {
        for config_entry in &config_expr.items {
            walk_if!(self, walk_expr, config_entry.node.key);
            self.walk_expr(&config_entry.node.value.node);
        }
    }
    fn walk_check_expr(&mut self, check_expr: &ast::CheckExpr) {
        self.walk_expr(&check_expr.test.node);
        walk_if!(self, walk_expr, check_expr.if_cond);
        walk_if!(self, walk_expr, check_expr.msg);
    }
    fn walk_lambda_expr(&mut self, lambda_expr: &ast::LambdaExpr) {
        walk_if!(self, walk_arguments, lambda_expr.args);
        walk_list!(self, walk_stmt, lambda_expr.body);
    }
    fn walk_keyword(&mut self, keyword: &ast::Keyword) {
        self.walk_identifier(&keyword.arg.node);
        if let Some(v) = &keyword.value {
            self.walk_expr(&v.node)
        }
    }
    fn walk_arguments(&mut self, arguments: &ast::Arguments) {
        walk_list!(self, walk_identifier, arguments.args);
        for default in arguments.defaults.iter().flatten() {
            self.walk_expr(&default.node)
        }
    }
    fn walk_compare(&mut self, compare: &ast::Compare) {
        self.walk_expr(&compare.left.node);
        walk_list!(self, walk_expr, compare.comparators);
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
        walk_list!(self, walk_expr, joined_string.values);
    }
    fn walk_formatted_value(&mut self, formatted_value: &ast::FormattedValue) {
        self.walk_expr(&formatted_value.value.node);
    }
    fn walk_comment(&mut self, comment: &ast::Comment) {
        // Nothing to do.
        let _ = comment;
    }
    fn walk_missing_expr(&mut self, missing_expr: &ast::MissingExpr) {
        // Nothing to do.
        let _ = missing_expr;
    }
    fn walk_module(&mut self, module: &ast::Module) {
        walk_list!(self, walk_stmt, module.body)
    }
    fn walk_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::TypeAlias(type_alias) => self.walk_type_alias_stmt(type_alias),
            ast::Stmt::Expr(expr_stmt) => self.walk_expr_stmt(expr_stmt),
            ast::Stmt::Unification(unification_stmt) => {
                self.walk_unification_stmt(unification_stmt)
            }
            ast::Stmt::Assign(assign_stmt) => self.walk_assign_stmt(assign_stmt),
            ast::Stmt::AugAssign(aug_assign_stmt) => self.walk_aug_assign_stmt(aug_assign_stmt),
            ast::Stmt::Assert(assert_stmt) => self.walk_assert_stmt(assert_stmt),
            ast::Stmt::If(if_stmt) => self.walk_if_stmt(if_stmt),
            ast::Stmt::Import(import_stmt) => self.walk_import_stmt(import_stmt),
            ast::Stmt::SchemaAttr(schema_attr) => self.walk_schema_attr(schema_attr),
            ast::Stmt::Schema(schema_stmt) => self.walk_schema_stmt(schema_stmt),
            ast::Stmt::Rule(rule_stmt) => self.walk_rule_stmt(rule_stmt),
        }
    }
    fn walk_expr(&mut self, expr: &ast::Expr) {
        match expr {
            ast::Expr::Identifier(identifier) => self.walk_identifier(identifier),
            ast::Expr::Unary(unary_expr) => self.walk_unary_expr(unary_expr),
            ast::Expr::Binary(binary_expr) => self.walk_binary_expr(binary_expr),
            ast::Expr::If(if_expr) => self.walk_if_expr(if_expr),
            ast::Expr::Selector(selector_expr) => self.walk_selector_expr(selector_expr),
            ast::Expr::Call(call_expr) => self.walk_call_expr(call_expr),
            ast::Expr::Paren(paren_expr) => self.walk_paren_expr(paren_expr),
            ast::Expr::Quant(quant_expr) => self.walk_quant_expr(quant_expr),
            ast::Expr::List(list_expr) => self.walk_list_expr(list_expr),
            ast::Expr::ListIfItem(list_if_item_expr) => {
                self.walk_list_if_item_expr(list_if_item_expr)
            }
            ast::Expr::ListComp(list_comp) => self.walk_list_comp(list_comp),
            ast::Expr::Starred(starred_expr) => self.walk_starred_expr(starred_expr),
            ast::Expr::DictComp(dict_comp) => self.walk_dict_comp(dict_comp),
            ast::Expr::ConfigIfEntry(config_if_entry_expr) => {
                self.walk_config_if_entry_expr(config_if_entry_expr)
            }
            ast::Expr::CompClause(comp_clause) => self.walk_comp_clause(comp_clause),
            ast::Expr::Schema(schema_expr) => self.walk_schema_expr(schema_expr),
            ast::Expr::Config(config_expr) => self.walk_config_expr(config_expr),
            ast::Expr::Check(check) => self.walk_check_expr(check),
            ast::Expr::Lambda(lambda) => self.walk_lambda_expr(lambda),
            ast::Expr::Subscript(subscript) => self.walk_subscript(subscript),
            ast::Expr::Keyword(keyword) => self.walk_keyword(keyword),
            ast::Expr::Arguments(arguments) => self.walk_arguments(arguments),
            ast::Expr::Compare(compare) => self.walk_compare(compare),
            ast::Expr::NumberLit(number_lit) => self.walk_number_lit(number_lit),
            ast::Expr::StringLit(string_lit) => self.walk_string_lit(string_lit),
            ast::Expr::NameConstantLit(name_constant_lit) => {
                self.walk_name_constant_lit(name_constant_lit)
            }
            ast::Expr::JoinedString(joined_string) => self.walk_joined_string(joined_string),
            ast::Expr::FormattedValue(formatted_value) => {
                self.walk_formatted_value(formatted_value)
            }
            ast::Expr::Missing(missing_expr) => self.walk_missing_expr(missing_expr),
        }
    }
}
