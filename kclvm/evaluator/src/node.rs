// Copyright The KCL Authors. All rights reserved.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Ok;
use kclvm_ast::ast::{self, CallExpr, ConfigEntry, NodeRef};
use kclvm_ast::walker::TypedResultWalker;
use kclvm_runtime::val_func::invoke_function;
use kclvm_runtime::walker::walk_value_mut;
use kclvm_runtime::{
    schema_assert, schema_runtime_type, type_pack_and_check, ApiFunc, ConfigEntryOperationKind,
    DecoratorValue, RuntimeErrorType, UnionOptions, ValueRef, PKG_PATH_PREFIX,
};
use kclvm_sema::{builtin, plugin};

use crate::func::{func_body, FunctionCaller, FunctionEvalContext};
use crate::lazy::EvalBodyRange;
use crate::proxy::Proxy;
use crate::rule::{rule_body, rule_check, RuleCaller, RuleEvalContext};
use crate::schema::{schema_body, schema_check, SchemaCaller, SchemaEvalContext};
use crate::{error as kcl_error, GLOBAL_LEVEL, INNER_LEVEL};
use crate::{EvalResult, Evaluator};

/// Impl TypedResultWalker for Evaluator to visit AST nodes to evaluate the result.
impl<'ctx> TypedResultWalker<'ctx> for Evaluator<'ctx> {
    type Result = EvalResult;

    /*
     * Stmt
     */

    fn walk_stmt(&self, stmt: &'ctx ast::Node<ast::Stmt>) -> Self::Result {
        self.reset_target_vars();
        match &stmt.node {
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

    fn walk_expr_stmt(&self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        let mut result = self.ok_result();
        for expr in &expr_stmt.exprs {
            let scalar = self.walk_expr(expr)?;
            // Only non-call expressions are allowed to emit values bacause of the function void return type.
            if !matches!(expr.node, ast::Expr::Call(_)) {
                self.add_scalar(scalar.clone(), matches!(expr.node, ast::Expr::Schema(_)));
            }
            result = Ok(scalar);
        }
        result
    }

    fn walk_unification_stmt(&self, unification_stmt: &'ctx ast::UnificationStmt) -> Self::Result {
        self.clear_local_vars();
        let name = &unification_stmt.target.node.names[0].node;
        self.add_target_var(name);
        // The right value of the unification_stmt is a schema_expr.
        let value = self
            .walk_schema_expr(&unification_stmt.value.node)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        if self.scope_level() == GLOBAL_LEVEL || self.is_in_lambda() {
            if self.resolve_variable(name) {
                let mut org_value = self
                    .walk_identifier_with_ctx(
                        &unification_stmt.target.node,
                        &ast::ExprContext::Load,
                        None,
                    )
                    .expect(kcl_error::RUNTIME_ERROR_MSG);
                let value = org_value.bin_aug_bit_or(&mut self.runtime_ctx.borrow_mut(), &value);
                // Store the identifier value
                self.walk_identifier_with_ctx(
                    &unification_stmt.target.node,
                    &ast::ExprContext::Store,
                    Some(value.clone()),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
                return Ok(value.clone());
            } else {
                self.walk_identifier_with_ctx(
                    &unification_stmt.target.node,
                    &unification_stmt.target.node.ctx,
                    Some(value.clone()),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
                return Ok(value);
            }
        // Local variables including schema/rule/lambda
        } else if self.is_in_schema() {
            // Load the identifier value
            let org_value = self
                .walk_identifier_with_ctx(
                    &unification_stmt.target.node,
                    &ast::ExprContext::Load,
                    None,
                )
                .unwrap_or(self.undefined_value());
            let value = org_value.bin_bit_or(&mut self.runtime_ctx.borrow_mut(), &value);
            // Store the identifier value
            self.walk_identifier_with_ctx(
                &unification_stmt.target.node,
                &ast::ExprContext::Store,
                Some(value.clone()),
            )
            .expect(kcl_error::RUNTIME_ERROR_MSG);
            return Ok(value);
        }
        Ok(value)
    }

    fn walk_type_alias_stmt(&self, _type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        // Nothing to do, because all type aliases have been replaced at compile time
        self.ok_result()
    }

    fn walk_assign_stmt(&self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        self.clear_local_vars();
        // Set target vars.
        for name in &assign_stmt.targets {
            self.add_target_var(&name.node.names[0].node)
        }
        // Load the right value
        let mut value = self
            .walk_expr(&assign_stmt.value)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        // Runtime type cast if exists the type annotation.
        if let Some(ty) = &assign_stmt.ty {
            let is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
            value = type_pack_and_check(
                &mut self.runtime_ctx.borrow_mut(),
                &value,
                vec![&ty.node.to_string()],
            );
            // Schema required attribute validating.
            if !is_in_schema {
                walk_value_mut(&value, &mut |value: &ValueRef| {
                    if value.is_schema() {
                        value.schema_check_attr_optional(&mut self.runtime_ctx.borrow_mut(), true);
                    }
                })
            }
        }
        if assign_stmt.targets.len() == 1 {
            // Store the single target
            let name = &assign_stmt.targets[0];
            self.walk_identifier_with_ctx(&name.node, &name.node.ctx, Some(value.clone()))
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        } else {
            // Store multiple targets
            for name in &assign_stmt.targets {
                let value = self.value_deep_copy(&value);
                self.walk_identifier_with_ctx(&name.node, &name.node.ctx, Some(value.clone()))
                    .expect(kcl_error::RUNTIME_ERROR_MSG);
            }
        }
        Ok(value)
    }

    fn walk_aug_assign_stmt(&self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        self.add_target_var(&aug_assign_stmt.target.node.names[0].node);
        // Load the right value
        let right_value = self
            .walk_expr(&aug_assign_stmt.value)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        // Load the identifier value
        let org_value = self
            .walk_identifier_with_ctx(&aug_assign_stmt.target.node, &ast::ExprContext::Load, None)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let value = match aug_assign_stmt.op {
            ast::AugOp::Add => self.add(org_value, right_value),
            ast::AugOp::Sub => self.sub(org_value, right_value),
            ast::AugOp::Mul => self.mul(org_value, right_value),
            ast::AugOp::Div => self.div(org_value, right_value),
            ast::AugOp::Mod => self.r#mod(org_value, right_value),
            ast::AugOp::Pow => self.pow(org_value, right_value),
            ast::AugOp::LShift => self.bit_lshift(org_value, right_value),
            ast::AugOp::RShift => self.bit_rshift(org_value, right_value),
            ast::AugOp::BitOr => self.bit_or(org_value, right_value),
            ast::AugOp::BitXor => self.bit_xor(org_value, right_value),
            ast::AugOp::BitAnd => self.bit_and(org_value, right_value),
            ast::AugOp::FloorDiv => self.floor_div(org_value, right_value),
            ast::AugOp::Assign => {
                return Err(anyhow::anyhow!(kcl_error::INVALID_OPERATOR_MSG));
            }
        };
        // Store the identifier value
        self.walk_identifier_with_ctx(
            &aug_assign_stmt.target.node,
            &ast::ExprContext::Store,
            Some(value.clone()),
        )
        .expect(kcl_error::RUNTIME_ERROR_MSG);
        Ok(value)
    }

    fn walk_assert_stmt(&self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        let do_assert = || {
            let assert_result = self
                .walk_expr(&assert_stmt.test)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            // Assert statement error message.
            let msg = {
                if let Some(msg) = &assert_stmt.msg {
                    self.walk_expr(msg).expect(kcl_error::RUNTIME_ERROR_MSG)
                } else {
                    self.string_value("")
                }
            };
            if !assert_result.is_truthy() {
                let mut ctx = self.runtime_ctx.borrow_mut();
                ctx.set_err_type(&RuntimeErrorType::AssertionError);
                let msg = msg.as_str();
                panic!("{}", msg);
            }
        };
        if let Some(if_cond) = &assert_stmt.if_cond {
            let if_value = self.walk_expr(if_cond).expect(kcl_error::RUNTIME_ERROR_MSG);
            let is_truth = self.value_is_truthy(&if_value);
            if is_truth {
                do_assert()
            }
        } else {
            do_assert()
        }
        self.ok_result()
    }

    fn walk_if_stmt(&self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        let cond = self
            .walk_expr(&if_stmt.cond)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let is_truth = self.value_is_truthy(&cond);
        if is_truth {
            self.walk_stmts(&if_stmt.body)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        } else {
            self.walk_stmts(&if_stmt.orelse)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        }
        self.ok_result()
    }

    fn walk_import_stmt(&self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        let pkgpath = import_stmt.path.node.as_str();
        // Check if it has already been generated, there is no need to generate code
        // for duplicate import statements.
        if self.check_imported(pkgpath) {
            return self.ok_result();
        }
        // Standard or plugin modules.
        if builtin::STANDARD_SYSTEM_MODULES.contains(&pkgpath)
            || pkgpath.starts_with(plugin::PLUGIN_MODULE_PREFIX)
        {
            // Nothing to do on the builtin system module import because the check has been done.
            return self.ok_result();
        } else {
            let pkgpath = format!("{}{}", PKG_PATH_PREFIX, import_stmt.path.node);
            if let Some(modules) = self.program.pkgs.get(&import_stmt.path.node) {
                self.push_pkgpath(&pkgpath);
                self.init_scope(&pkgpath);
                self.compile_ast_modules(modules);
                self.pop_pkgpath();
            }
        }
        self.mark_imported(pkgpath);
        self.ok_result()
    }

    fn walk_schema_stmt(&self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        let body = Arc::new(schema_body);
        let check = Arc::new(schema_check);
        let proxy = SchemaCaller {
            ctx: Rc::new(RefCell::new(SchemaEvalContext::new_with_node(
                schema_stmt.clone(),
            ))),
            body,
            check,
        };
        // Add function to the global state
        let index = self.add_schema(proxy);
        let runtime_type = schema_runtime_type(&schema_stmt.name.node, &self.current_pkgpath());
        let function = self.proxy_function_value_with_type(index, &runtime_type);
        // Store or add the variable in the scope
        let name = &schema_stmt.name.node;
        if !self.store_variable(name, function.clone()) {
            self.add_variable(name, function.clone());
        }
        Ok(function)
    }

    fn walk_rule_stmt(&self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        let body = Arc::new(rule_body);
        let check = Arc::new(rule_check);
        let proxy = RuleCaller {
            ctx: Rc::new(RefCell::new(RuleEvalContext::new_with_node(
                rule_stmt.clone(),
            ))),
            body,
            check,
        };
        // Add function to the global state
        let index = self.add_rule(proxy);
        let function = self.proxy_function_value(index);
        // Store or add the variable in the scope
        let name = &rule_stmt.name.node;
        if !self.store_variable(name, function.clone()) {
            self.add_variable(name, function.clone());
        }
        Ok(function)
    }

    /*
     * Expr
     */

    fn walk_expr(&self, expr: &'ctx ast::Node<ast::Expr>) -> Self::Result {
        match &expr.node {
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
            ast::Expr::Arguments(..) => self.ok_result(),
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

    fn walk_quant_expr(&self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        let mut result = match quant_expr.op {
            ast::QuantOperation::All => self.bool_value(true),
            ast::QuantOperation::Any => self.bool_value(false),
            ast::QuantOperation::Map => self.list_value(),
            ast::QuantOperation::Filter => self.value_deep_copy(
                &self
                    .walk_expr(&quant_expr.target)
                    .expect(kcl_error::RUNTIME_ERROR_MSG),
            ),
        };
        // Iterator
        let iter_host_value = if let ast::QuantOperation::Filter = quant_expr.op {
            self.value_deep_copy(&result)
        } else {
            self.walk_expr(&quant_expr.target)
                .expect(kcl_error::RUNTIME_ERROR_MSG)
        };
        let mut iter_value = iter_host_value.iter();
        // Start iteration and enter the loop scope for the loop variable.
        self.enter_scope();
        // Start block
        while let Some((next_value, key, value)) = iter_value.next_with_key_value(&iter_host_value)
        {
            // Next value block
            let variables = &quant_expr.variables;
            for v in variables {
                self.add_local_var(&v.node.names[0].node);
            }
            if variables.len() == 1 {
                // Store the target
                self.walk_identifier_with_ctx(
                    &variables.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                    &ast::ExprContext::Store,
                    Some(next_value.clone()),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            } else if variables.len() == 2 {
                // Store the target
                self.walk_identifier_with_ctx(
                    &variables.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                    &ast::ExprContext::Store,
                    Some(key.clone()),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
                self.walk_identifier_with_ctx(
                    &variables.get(1).expect(kcl_error::INTERNAL_ERROR_MSG).node,
                    &ast::ExprContext::Store,
                    Some(value.clone()),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            } else {
                panic!(
                    "the number of loop variables is {}, which can only be 1 or 2",
                    variables.len()
                )
            }
            // Check the if filter condition
            if let Some(if_expr) = &quant_expr.if_cond {
                let if_truth = self.walk_expr(if_expr).expect(kcl_error::RUNTIME_ERROR_MSG);
                let is_truth = self.value_is_truthy(&if_truth);
                if is_truth {
                    // Skip this iteration and goto the start block
                    continue;
                }
            }
            // Loop var generation body block
            let test = &quant_expr.test;
            let value = self.walk_expr(test).expect(kcl_error::RUNTIME_ERROR_MSG);
            let is_truth = self.value_is_truthy(&value);
            match quant_expr.op {
                ast::QuantOperation::All => {
                    if !is_truth {
                        self.leave_scope();
                        return Ok(self.bool_value(false));
                    }
                }
                ast::QuantOperation::Any => {
                    if is_truth {
                        self.leave_scope();
                        return Ok(self.bool_value(true));
                    }
                }
                ast::QuantOperation::Filter => {
                    if !is_truth {
                        if result.is_dict() {
                            result.dict_remove(&next_value.as_str());
                        } else if result.is_list() {
                            result.list_remove(&next_value);
                        } else {
                            panic!("only list, dict and schema can be removed item");
                        }
                    }
                }
                ast::QuantOperation::Map => {
                    self.list_append(&mut result, &value);
                }
            }
        }
        self.leave_scope();
        // End for block.
        Ok(result)
    }

    fn walk_schema_attr(&self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        let name = schema_attr.name.node.as_str();
        self.add_target_var(name);
        for decorator in &schema_attr.decorators {
            self.walk_decorator_with_name(&decorator.node, Some(name), false)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        }
        let (mut schema_value, config_value) = self
            .get_schema_and_config()
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        if let Some(entry) = config_value.dict_get_entry(name) {
            let is_override_attr = {
                let is_override_op = matches!(
                    config_value.dict_get_attr_operator(name),
                    Some(ConfigEntryOperationKind::Override)
                );
                let without_index =
                    matches!(config_value.dict_get_insert_index(name), Some(-1) | None);
                is_override_op && without_index
            };
            if !is_override_attr {
                let value = match &schema_attr.value {
                    Some(value) => self.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG),
                    None => self.undefined_value(),
                };
                if let Some(op) = &schema_attr.op {
                    match op {
                        // Union
                        ast::AugOp::BitOr => {
                            let org_value = schema_value
                                .dict_get_value(name)
                                .unwrap_or(self.undefined_value());
                            let value = self.bit_or(org_value, value);
                            self.schema_dict_merge(
                                &mut schema_value,
                                name,
                                &value,
                                &ast::ConfigEntryOperation::Override,
                                -1,
                            );
                        }
                        // Assign
                        _ => self.schema_dict_merge(
                            &mut schema_value,
                            name,
                            &value,
                            &ast::ConfigEntryOperation::Override,
                            -1,
                        ),
                    }
                }
            }
            self.value_union(&mut schema_value, &entry);
        } else {
            // Lazy eval for the schema attribute.
            let value = match &schema_attr.value {
                Some(value) => self.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG),
                None => {
                    // When the schema has no default value and config value,
                    // set it with a undefined value.
                    let value = self.undefined_value();
                    self.dict_insert_value(&mut schema_value, name, &value);
                    value
                }
            };
            if let Some(op) = &schema_attr.op {
                match op {
                    // Union
                    ast::AugOp::BitOr => {
                        let org_value = schema_value
                            .dict_get_value(name)
                            .unwrap_or(self.undefined_value());
                        let value = self.bit_or(org_value, value);
                        self.schema_dict_merge(
                            &mut schema_value,
                            name,
                            &value,
                            &ast::ConfigEntryOperation::Override,
                            -1,
                        );
                    }
                    // Assign
                    _ => self.schema_dict_merge(
                        &mut schema_value,
                        name,
                        &value,
                        &ast::ConfigEntryOperation::Override,
                        -1,
                    ),
                }
            }
        }
        Ok(schema_value)
    }

    fn walk_if_expr(&self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        let cond = self
            .walk_expr(&if_expr.cond)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let is_truth = self.value_is_truthy(&cond);
        if is_truth {
            self.walk_expr(&if_expr.body)
        } else {
            self.walk_expr(&if_expr.orelse)
        }
    }

    fn walk_unary_expr(&self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        let value = self
            .walk_expr(&unary_expr.operand)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        Ok(match unary_expr.op {
            ast::UnaryOp::UAdd => value.unary_plus(),
            ast::UnaryOp::USub => value.unary_minus(),
            ast::UnaryOp::Invert => value.unary_not(),
            ast::UnaryOp::Not => value.unary_l_not(),
        })
    }

    fn walk_binary_expr(&self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        let is_logic_op = matches!(binary_expr.op, ast::BinOp::And | ast::BinOp::Or);
        let is_membership_as_op = matches!(binary_expr.op, ast::BinOp::As);
        if !is_logic_op {
            let left_value = self
                .walk_expr(&binary_expr.left)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            let right_value = if is_membership_as_op {
                match &binary_expr.right.node {
                    ast::Expr::Identifier(id) => {
                        let name = id.get_names().join(".");
                        self.string_value(&name)
                    }
                    _ => self.none_value(),
                }
            } else {
                self.walk_expr(&binary_expr.right)
                    .expect(kcl_error::RUNTIME_ERROR_MSG)
            };
            let value = match binary_expr.op {
                ast::BinOp::Add => self.add(left_value, right_value),
                ast::BinOp::Sub => self.sub(left_value, right_value),
                ast::BinOp::Mul => self.mul(left_value, right_value),
                ast::BinOp::Div => self.div(left_value, right_value),
                ast::BinOp::FloorDiv => self.floor_div(left_value, right_value),
                ast::BinOp::Mod => self.r#mod(left_value, right_value),
                ast::BinOp::Pow => self.pow(left_value, right_value),
                ast::BinOp::LShift => self.bit_lshift(left_value, right_value),
                ast::BinOp::RShift => self.bit_rshift(left_value, right_value),
                ast::BinOp::BitAnd => self.bit_and(left_value, right_value),
                ast::BinOp::BitOr => self.bit_or(left_value, right_value),
                ast::BinOp::BitXor => self.bit_xor(left_value, right_value),
                ast::BinOp::And => self.logic_and(left_value, right_value),
                ast::BinOp::Or => self.logic_or(left_value, right_value),
                ast::BinOp::As => self.r#as(left_value, right_value),
            };
            Ok(value)
        } else {
            // Short circuit operation of logical operators
            let jump_if_false = matches!(binary_expr.op, ast::BinOp::And);
            let left_value = self
                .walk_expr(&binary_expr.left)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            let is_truth = self.value_is_truthy(&left_value);
            if jump_if_false {
                // Jump if false on logic and
                if is_truth {
                    let right_value = self
                        .walk_expr(&binary_expr.right)
                        .expect(kcl_error::RUNTIME_ERROR_MSG);
                    return Ok(right_value);
                }
            } else {
                // Jump if true on logic or
                if !is_truth {
                    let right_value = self
                        .walk_expr(&binary_expr.right)
                        .expect(kcl_error::RUNTIME_ERROR_MSG);
                    return Ok(right_value);
                }
            };
            Ok(left_value)
        }
    }

    fn walk_selector_expr(&self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        let value = self
            .walk_expr(&selector_expr.value)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let key = selector_expr.attr.node.names[0].node.as_str();
        let mut value = if selector_expr.has_question {
            if value.is_truthy() {
                value.load_attr(key)
            } else {
                self.none_value()
            }
        } else {
            value.load_attr(key)
        };
        for name in &selector_expr.attr.node.names[1..] {
            value = value.load_attr(&name.node)
        }
        Ok(value)
    }

    fn walk_call_expr(&self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        let func = self
            .walk_expr(&call_expr.func)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        // args
        let mut list_value = self.list_value();
        for arg in &call_expr.args {
            let value = self.walk_expr(arg).expect(kcl_error::RUNTIME_ERROR_MSG);
            self.list_append(&mut list_value, &value);
        }
        let mut dict_value = self.dict_value();
        // keyword arguments
        for keyword in &call_expr.keywords {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert_value(&mut dict_value, name.node.as_str(), &value);
        }
        if let Some(proxy) = func.try_get_proxy() {
            Ok(self.invoke_proxy_function(proxy, &list_value, &dict_value))
        } else {
            let pkgpath = self.current_pkgpath();
            let is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
            Ok(invoke_function(
                &func,
                &mut list_value,
                &dict_value,
                &mut self.runtime_ctx.borrow_mut(),
                &pkgpath,
                is_in_schema,
            ))
        }
    }

    fn walk_subscript(&self, subscript: &'ctx ast::Subscript) -> Self::Result {
        let mut value = self
            .walk_expr(&subscript.value)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        if let Some(index) = &subscript.index {
            // index
            let index = self.walk_expr(index).expect(kcl_error::RUNTIME_ERROR_MSG);
            value = if subscript.has_question {
                value.bin_subscr_option(&index)
            } else {
                value.bin_subscr(&index)
            };
        } else {
            let lower = {
                if let Some(lower) = &subscript.lower {
                    self.walk_expr(lower).expect(kcl_error::RUNTIME_ERROR_MSG)
                } else {
                    self.none_value()
                }
            };
            let upper = {
                if let Some(upper) = &subscript.upper {
                    self.walk_expr(upper).expect(kcl_error::RUNTIME_ERROR_MSG)
                } else {
                    self.none_value()
                }
            };
            let step = {
                if let Some(step) = &subscript.step {
                    self.walk_expr(step).expect(kcl_error::RUNTIME_ERROR_MSG)
                } else {
                    self.none_value()
                }
            };
            value = if subscript.has_question {
                if value.is_truthy() {
                    value.list_slice(&lower, &upper, &step)
                } else {
                    self.none_value()
                }
            } else {
                value.list_slice(&lower, &upper, &step)
            };
        }
        Ok(value)
    }

    fn walk_paren_expr(&self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        self.walk_expr(&paren_expr.expr)
    }

    fn walk_list_expr(&self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        let mut list_value = self.list_value();
        for item in &list_expr.elts {
            let value = self.walk_expr(item).expect(kcl_error::RUNTIME_ERROR_MSG);
            match &item.node {
                ast::Expr::Starred(_) | ast::Expr::ListIfItem(_) => {
                    self.list_append_unpack(&mut list_value, &value);
                }
                _ => self.list_append(&mut list_value, &value),
            };
        }
        Ok(list_value)
    }

    fn walk_list_if_item_expr(&self, list_if_item_expr: &'ctx ast::ListIfItemExpr) -> Self::Result {
        let cond = self
            .walk_expr(&list_if_item_expr.if_cond)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let is_truth = self.value_is_truthy(&cond);
        Ok(if is_truth {
            let mut then_value = self.list_value();
            for expr in &list_if_item_expr.exprs {
                let value = self.walk_expr(expr).expect(kcl_error::RUNTIME_ERROR_MSG);
                match &expr.node {
                    ast::Expr::Starred(_) | ast::Expr::ListIfItem(_) => {
                        self.list_append_unpack(&mut then_value, &value)
                    }
                    _ => self.list_append(&mut then_value, &value),
                };
            }
            then_value
        } else if let Some(orelse) = &list_if_item_expr.orelse {
            self.walk_expr(orelse).expect(kcl_error::RUNTIME_ERROR_MSG)
        } else {
            self.none_value()
        })
    }

    fn walk_starred_expr(&self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        self.walk_expr(&starred_expr.value)
    }

    fn walk_list_comp(&self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        let mut collection_value = self.list_value();
        self.enter_scope();
        self.walk_generator(
            &list_comp.generators,
            &list_comp.elt,
            None,
            None,
            0,
            &mut collection_value,
            &ast::CompType::List,
        );
        self.leave_scope();
        Ok(collection_value)
    }

    fn walk_dict_comp(&self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        let mut collection_value = self.dict_value();
        self.enter_scope();
        let key = dict_comp
            .entry
            .key
            .as_ref()
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        self.walk_generator(
            &dict_comp.generators,
            key,
            Some(&dict_comp.entry.value),
            Some(&dict_comp.entry.operation),
            0,
            &mut collection_value,
            &ast::CompType::Dict,
        );
        self.leave_scope();
        Ok(collection_value)
    }

    fn walk_config_if_entry_expr(
        &self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        let cond = self
            .walk_expr(&config_if_entry_expr.if_cond)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let is_truth = self.value_is_truthy(&cond);
        Ok(if is_truth {
            self.walk_config_entries(&config_if_entry_expr.items)?
        } else if let Some(orelse) = &config_if_entry_expr.orelse {
            self.walk_expr(orelse).expect(kcl_error::RUNTIME_ERROR_MSG)
        } else {
            self.none_value()
        })
    }

    fn walk_comp_clause(&self, _comp_clause: &'ctx ast::CompClause) -> Self::Result {
        // Nothing to do on this AST node
        self.ok_result()
    }

    fn walk_schema_expr(&self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        // Check the required attributes only when the values of all attributes
        // in the final schema are solved.
        let is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
        self.push_schema_expr();
        let config_value = self
            .walk_expr(&schema_expr.config)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let schema_type = self
            .walk_identifier_with_ctx(&schema_expr.name.node, &schema_expr.name.node.ctx, None)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let config_expr = match &schema_expr.config.node {
            ast::Expr::Config(config_expr) => config_expr,
            _ => panic!("invalid schema config expr"),
        };
        let config_meta = self.construct_schema_config_meta(Some(&schema_expr.name), config_expr);
        let mut list_value = self.list_value();
        for arg in &schema_expr.args {
            let value = self.walk_expr(arg).expect(kcl_error::RUNTIME_ERROR_MSG);
            self.list_append(&mut list_value, &value);
        }
        let mut dict_value = self.dict_value();
        for keyword in &schema_expr.kwargs {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert_merge_value(&mut dict_value, name.node.as_str(), &value);
        }
        let schema = if let Some(index) = schema_type.try_get_proxy() {
            let frame = {
                let frames = self.frames.borrow();
                frames
                    .get(index)
                    .expect(kcl_error::INTERNAL_ERROR_MSG)
                    .clone()
            };
            if let Proxy::Schema(schema) = &frame.proxy {
                self.push_pkgpath(&frame.pkgpath);
                // Set new schema and config
                {
                    let mut ctx = schema.ctx.borrow_mut();
                    ctx.reset_with_config(config_value, config_meta);
                }
                let value = (schema.body)(self, &schema.ctx, &list_value, &dict_value);
                self.pop_pkgpath();
                value
            } else {
                self.undefined_value()
            }
        } else {
            schema_type.deep_copy().union_entry(
                &mut self.runtime_ctx.borrow_mut(),
                &config_value,
                true,
                &UnionOptions::default(),
            )
        };
        if !is_in_schema {
            schema.schema_check_attr_optional(&mut self.runtime_ctx.borrow_mut(), true)
        }
        self.pop_schema_expr();
        Ok(schema)
    }

    #[inline]
    fn walk_config_expr(&self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        self.walk_config_entries(&config_expr.items)
    }

    fn walk_check_expr(&self, check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        if let Some(if_cond) = &check_expr.if_cond {
            let if_value = self.walk_expr(if_cond).expect(kcl_error::RUNTIME_ERROR_MSG);
            let is_truth = self.value_is_truthy(&if_value);
            if !is_truth {
                return self.ok_result();
            }
        }
        let check_result = self
            .walk_expr(&check_expr.test)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let msg = {
            if let Some(msg) = &check_expr.msg {
                self.walk_expr(msg).expect(kcl_error::RUNTIME_ERROR_MSG)
            } else {
                self.string_value("")
            }
        }
        .as_str();
        let config_meta = self
            .get_schema_config_meta()
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        schema_assert(
            &mut self.runtime_ctx.borrow_mut(),
            &check_result,
            &msg,
            &config_meta,
        );
        self.ok_result()
    }

    fn walk_lambda_expr(&self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        let func = Arc::new(func_body);
        let proxy = FunctionCaller::new(
            FunctionEvalContext {
                node: lambda_expr.clone(),
            },
            func,
        );
        // Add function to the global state
        let index = self.add_function(proxy);
        Ok(self.proxy_function_value(index))
    }

    fn walk_keyword(&self, _keyword: &'ctx ast::Keyword) -> Self::Result {
        // Nothing to do
        self.ok_result()
    }

    fn walk_arguments(&self, _arguments: &'ctx ast::Arguments) -> Self::Result {
        // Nothing to do
        self.ok_result()
    }

    fn walk_compare(&self, compare: &'ctx ast::Compare) -> Self::Result {
        let mut left_value = self
            .walk_expr(&compare.left)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        if compare.comparators.len() > 1 {
            let mut result_value = self.undefined_value();
            for (i, op) in compare.ops.iter().enumerate() {
                let has_next = i < (compare.ops.len() - 1);
                let right_value = self
                    .walk_expr(&compare.comparators[i])
                    .expect(kcl_error::RUNTIME_ERROR_MSG);
                result_value = match op {
                    ast::CmpOp::Eq => self.cmp_equal_to(left_value, right_value.clone()),
                    ast::CmpOp::NotEq => self.cmp_not_equal_to(left_value, right_value.clone()),
                    ast::CmpOp::Gt => self.cmp_greater_than(left_value, right_value.clone()),
                    ast::CmpOp::GtE => {
                        self.cmp_greater_than_or_equal(left_value, right_value.clone())
                    }
                    ast::CmpOp::Lt => self.cmp_less_than(left_value, right_value.clone()),
                    ast::CmpOp::LtE => self.cmp_less_than_or_equal(left_value, right_value.clone()),
                    ast::CmpOp::Is => self.is(left_value, right_value.clone()),
                    ast::CmpOp::IsNot => self.is_not(left_value, right_value.clone()),
                    ast::CmpOp::Not => self.is_not(left_value, right_value.clone()),
                    ast::CmpOp::NotIn => self.not_in(left_value, right_value.clone()),
                    ast::CmpOp::In => self.r#in(left_value, right_value.clone()),
                };
                left_value = right_value;
                let is_truth = self.value_is_truthy(&result_value);
                if has_next {
                    if !is_truth {
                        break;
                    }
                } else {
                    break;
                }
            }
            Ok(result_value)
        } else {
            let right_value = self
                .walk_expr(&compare.comparators[0])
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            Ok(match &compare.ops[0] {
                ast::CmpOp::Eq => self.cmp_equal_to(left_value, right_value),
                ast::CmpOp::NotEq => self.cmp_not_equal_to(left_value, right_value),
                ast::CmpOp::Gt => self.cmp_greater_than(left_value, right_value),
                ast::CmpOp::GtE => self.cmp_greater_than_or_equal(left_value, right_value),
                ast::CmpOp::Lt => self.cmp_less_than(left_value, right_value),
                ast::CmpOp::LtE => self.cmp_less_than_or_equal(left_value, right_value),
                ast::CmpOp::Is => self.is(left_value, right_value),
                ast::CmpOp::IsNot => self.is_not(left_value, right_value),
                ast::CmpOp::Not => self.is_not(left_value, right_value),
                ast::CmpOp::NotIn => self.not_in(left_value, right_value),
                ast::CmpOp::In => self.r#in(left_value, right_value),
            })
        }
    }

    fn walk_identifier(&self, identifier: &'ctx ast::Identifier) -> Self::Result {
        self.walk_identifier_with_ctx(identifier, &identifier.ctx, None)
    }

    fn walk_number_lit(&self, number_lit: &'ctx ast::NumberLit) -> Self::Result {
        match number_lit.value {
            ast::NumberLitValue::Int(int_value) => match &number_lit.binary_suffix {
                Some(binary_suffix) => {
                    let unit = binary_suffix.value();
                    let value = kclvm_runtime::cal_num(int_value, unit.as_str());
                    Ok(self.unit_value(value, int_value, &unit))
                }
                None => Ok(self.int_value(int_value)),
            },
            ast::NumberLitValue::Float(float_value) => Ok(self.float_value(float_value)),
        }
    }

    #[inline]
    fn walk_string_lit(&self, string_lit: &'ctx ast::StringLit) -> Self::Result {
        Ok(ValueRef::str(string_lit.value.as_str()))
    }

    #[inline]
    fn walk_name_constant_lit(
        &self,
        name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        match name_constant_lit.value {
            ast::NameConstant::True => Ok(self.bool_value(true)),
            ast::NameConstant::False => Ok(self.bool_value(false)),
            ast::NameConstant::None => Ok(self.none_value()),
            ast::NameConstant::Undefined => Ok(self.undefined_value()),
        }
    }

    fn walk_joined_string(&self, joined_string: &'ctx ast::JoinedString) -> Self::Result {
        let mut result_value = self.string_value("");
        for value in &joined_string.values {
            let value = &value.node;
            let value = match value {
                ast::Expr::FormattedValue(formatted_value) => self
                    .walk_formatted_value(formatted_value)
                    .expect(kcl_error::INTERNAL_ERROR_MSG),
                ast::Expr::StringLit(string_lit) => self
                    .walk_string_lit(string_lit)
                    .expect(kcl_error::INTERNAL_ERROR_MSG),
                _ => panic!("{}", kcl_error::INVALID_JOINED_STR_MSG),
            };
            result_value = self.add(result_value, value)
        }
        Ok(result_value)
    }

    fn walk_formatted_value(&self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result {
        let formatted_expr_value = self
            .walk_expr(&formatted_value.value)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let _fn_name = ApiFunc::kclvm_value_to_str_value;
        let value = if let Some(spec) = &formatted_value.format_spec {
            match spec.to_lowercase().as_str() {
                "#json" => formatted_expr_value.to_json_string(),
                "#yaml" => formatted_expr_value.to_yaml_string(),
                _ => panic!("{}", kcl_error::INVALID_STR_INTERPOLATION_SPEC_MSG),
            }
        } else {
            formatted_expr_value.to_string()
        };
        Ok(ValueRef::str(&value))
    }

    fn walk_comment(&self, _comment: &'ctx ast::Comment) -> Self::Result {
        // Nothing to do
        self.ok_result()
    }

    fn walk_missing_expr(&self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        Err(anyhow::anyhow!("compile error: missing expression",))
    }

    fn walk_module(&self, module: &'ctx ast::Module) -> Self::Result {
        // Compile all statements of the module except all import statements
        self.walk_stmts_except_import(&module.body)
    }
}

impl<'ctx> Evaluator<'ctx> {
    pub fn walk_stmts_except_import(&self, stmts: &'ctx [Box<ast::Node<ast::Stmt>>]) -> EvalResult {
        let mut result = self.ok_result();
        for stmt in stmts {
            if !matches!(&stmt.node, ast::Stmt::Import(..)) {
                result = self.walk_stmt(stmt);
            }
        }
        result
    }

    pub fn walk_stmts(&self, stmts: &'ctx [Box<ast::Node<ast::Stmt>>]) -> EvalResult {
        // Empty statements return None value
        let mut result = self.ok_result();
        for stmt in stmts {
            result = self.walk_stmt(stmt);
        }
        result
    }

    pub fn walk_stmts_with_range(
        &self,
        stmts: &'ctx [Box<ast::Node<ast::Stmt>>],
        range: &EvalBodyRange,
    ) -> EvalResult {
        let mut result = self.ok_result();
        if let Some(stmts) = stmts.get(range.clone()) {
            result = self.walk_stmts(stmts);
        }
        result
    }

    pub fn walk_identifier_with_ctx(
        &self,
        identifier: &'ctx ast::Identifier,
        identifier_ctx: &ast::ExprContext,
        right_value: Option<ValueRef>,
    ) -> EvalResult {
        let is_in_schema = self.is_in_schema();
        match identifier_ctx {
            // Store a.b.c = 1
            ast::ExprContext::Store => {
                if identifier.names.len() == 1 {
                    let name = identifier.names[0].node.as_str();
                    // Global variables
                    if self.scope_level() == GLOBAL_LEVEL {
                        self.add_or_update_global_variable(
                            name,
                            right_value.clone().expect(kcl_error::INTERNAL_ERROR_MSG),
                        );
                    // Lambda local variables.
                    } else if self.is_in_lambda() {
                        let value = right_value.clone().expect(kcl_error::INTERNAL_ERROR_MSG);
                        // If variable exists in the scope and update it, if not, add it to the scope.
                        if !self.store_variable_in_current_scope(name, value.clone()) {
                            self.add_variable(name, self.undefined_value());
                            self.store_variable(name, value);
                        }
                    } else {
                        let is_local_var = self.is_local_var(name);
                        let value = right_value.clone().expect(kcl_error::INTERNAL_ERROR_MSG);
                        match (is_local_var, is_in_schema) {
                            (false, true) => self.update_schema_scope_value(name, Some(&value)),
                            _ => self.add_variable(name, value),
                        }
                    }
                } else {
                    let names = &identifier.names;
                    let name = names[0].node.as_str();
                    // In KCL, we cannot modify global variables in other packages,
                    // so pkgpath is empty here.
                    let mut value = self
                        .load_value("", &[name])
                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                    // Convert `store a.b.c = 1` -> `%t = load &a; %t = load_attr %t %b; store_attr %t %c with 1`
                    for i in 0..names.len() - 1 {
                        let attr = names[i + 1].node.as_str();
                        let ctx = if matches!(identifier_ctx, ast::ExprContext::Store)
                            && i != names.len() - 2
                            && names.len() > 2
                        {
                            &ast::ExprContext::Load
                        } else {
                            identifier_ctx
                        };
                        match ctx {
                            ast::ExprContext::Load => {
                                value = value.load_attr(attr);
                            }
                            ast::ExprContext::Store => {
                                value.dict_set_value(
                                    &mut self.runtime_ctx.borrow_mut(),
                                    attr,
                                    &right_value.clone().expect(kcl_error::INTERNAL_ERROR_MSG),
                                );
                                let is_local_var = self.is_local_var(name);
                                let is_in_lambda = self.is_in_lambda();
                                // Set config value for the schema attribute if the attribute is in the schema and
                                // it is not a local variable in the lambda function.
                                if self.scope_level() >= INNER_LEVEL
                                    && is_in_schema
                                    && !is_in_lambda
                                    && !is_local_var
                                {
                                    self.update_schema_scope_value(name, None);
                                }
                            }
                        }
                    }
                }
                Ok(right_value.expect(kcl_error::INTERNAL_ERROR_MSG))
            }
            // Load <pkg>.a.b.c
            ast::ExprContext::Load => self.load_value(
                &identifier.pkgpath,
                &identifier
                    .names
                    .iter()
                    .map(|n| n.node.as_str())
                    .collect::<Vec<&str>>(),
            ),
        }
    }

    pub fn walk_decorator_with_name(
        &self,
        decorator: &'ctx CallExpr,
        attr_name: Option<&str>,
        is_schema_target: bool,
    ) -> EvalResult {
        let mut list_value = self.list_value();
        let mut dict_value = self.dict_value();
        let (_, config_value) = self
            .get_schema_and_config()
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let config_meta = self
            .get_schema_config_meta()
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        for arg in &decorator.args {
            let value = self.walk_expr(arg).expect(kcl_error::RUNTIME_ERROR_MSG);
            self.list_append(&mut list_value, &value);
        }
        for keyword in &decorator.keywords {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert_value(&mut dict_value, name.node.as_str(), &value);
        }
        let name = match &decorator.func.node {
            ast::Expr::Identifier(ident) if ident.names.len() == 1 => ident.names[0].clone(),
            _ => panic!("invalid decorator name, expect single identifier"),
        };
        let attr_name = if let Some(v) = attr_name { v } else { "" };
        DecoratorValue::new(&name.node, &list_value, &dict_value).run(
            &mut self.runtime_ctx.borrow_mut(),
            attr_name,
            is_schema_target,
            &config_value,
            &config_meta,
        );
        self.ok_result()
    }

    pub fn walk_arguments(
        &self,
        arguments: &'ctx Option<ast::NodeRef<ast::Arguments>>,
        args: &ValueRef,
        kwargs: &ValueRef,
    ) {
        // Arguments names and defaults
        let (arg_names, arg_defaults) = if let Some(args) = &arguments {
            let names = &args.node.args;
            let defaults = &args.node.defaults;
            (
                names.iter().map(|identifier| &identifier.node).collect(),
                defaults.iter().collect(),
            )
        } else {
            (vec![], vec![])
        };
        // Default parameter values
        for (arg_name, value) in arg_names.iter().zip(arg_defaults.iter()) {
            let arg_value = if let Some(value) = value {
                self.walk_expr(value).expect(kcl_error::RUNTIME_ERROR_MSG)
            } else {
                self.none_value()
            };
            // Arguments are immutable, so we place them in different scopes.
            let name = arg_name.get_name();
            self.store_argument_in_current_scope(&name);
            // Argument is a local variable instead of a global variable or schema attribute.
            self.add_local_var(&name);
            self.walk_identifier_with_ctx(arg_name, &ast::ExprContext::Store, Some(arg_value))
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            self.remove_local_var(&name);
        }
        // Positional arguments
        let argument_len = args.len();
        for (i, arg_name) in arg_names.iter().enumerate() {
            // Positional arguments
            let is_in_range = i < argument_len;
            if is_in_range {
                let arg_value = match args.list_get_option(i as isize) {
                    Some(v) => v,
                    None => self.undefined_value(),
                };
                self.store_variable(&arg_name.names[0].node, arg_value);
            } else {
                break;
            }
        }
        // Keyword arguments
        for arg_name in arg_names.iter() {
            let name = &arg_name.names[0].node;
            if let Some(arg) = kwargs.dict_get_value(name) {
                // Find argument name in the scope
                self.store_variable(&arg_name.names[0].node, arg);
            }
        }
    }

    pub fn walk_generator(
        &self,
        generators: &'ctx [Box<ast::Node<ast::CompClause>>],
        elt: &'ctx ast::Node<ast::Expr>,
        val: Option<&'ctx ast::Node<ast::Expr>>,
        op: Option<&'ctx ast::ConfigEntryOperation>,
        gen_index: usize,
        collection_value: &mut ValueRef,
        comp_type: &ast::CompType,
    ) {
        // Start block
        let generator = &generators[gen_index];
        let iter_host_value = self
            .walk_expr(&generator.node.iter)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        let mut iter_value = iter_host_value.iter();
        let targets = &generator.node.targets;

        while let Some((next_value, key, value)) = iter_value.next_with_key_value(&iter_host_value)
        {
            for v in targets {
                self.add_local_var(&v.node.names[0].node)
            }
            if targets.len() == 1 {
                // Store the target
                self.walk_identifier_with_ctx(
                    &targets.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                    &ast::ExprContext::Store,
                    Some(next_value),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            } else if targets.len() == 2 {
                // Store the target
                self.walk_identifier_with_ctx(
                    &targets.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                    &ast::ExprContext::Store,
                    Some(key),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
                self.walk_identifier_with_ctx(
                    &targets.get(1).expect(kcl_error::INTERNAL_ERROR_MSG).node,
                    &ast::ExprContext::Store,
                    Some(value),
                )
                .expect(kcl_error::RUNTIME_ERROR_MSG);
            } else {
                panic!(
                    "the number of loop variables is {}, which can only be 1 or 2",
                    generator.node.targets.len()
                )
            }
            // Check the if filter
            let mut skip = false;
            for if_expr in &generator.node.ifs {
                let value = self.walk_expr(if_expr).expect(kcl_error::RUNTIME_ERROR_MSG);
                // Skip the iteration
                if !value.is_truthy() {
                    skip = true;
                }
            }
            if skip {
                continue;
            }
            let next_gen_index = gen_index + 1;
            if next_gen_index >= generators.len() {
                match comp_type {
                    ast::CompType::List => {
                        let item = self.walk_expr(elt).expect(kcl_error::RUNTIME_ERROR_MSG);
                        self.list_append(collection_value, &item);
                    }
                    ast::CompType::Dict => {
                        let value = self
                            .walk_expr(val.expect(kcl_error::INTERNAL_ERROR_MSG))
                            .expect(kcl_error::RUNTIME_ERROR_MSG);
                        let key = self.walk_expr(elt).expect(kcl_error::RUNTIME_ERROR_MSG);
                        let op = op.expect(kcl_error::INTERNAL_ERROR_MSG);
                        self.dict_insert(
                            collection_value,
                            &key.as_str(),
                            &value.deep_copy(),
                            op,
                            -1,
                        );
                    }
                }
            } else {
                self.walk_generator(
                    generators,
                    elt,
                    val,
                    op,
                    next_gen_index,
                    collection_value,
                    comp_type,
                );
            }
        }
        for v in targets {
            self.remove_local_var(&v.node.names[0].node)
        }
    }

    pub(crate) fn walk_config_entries(&self, items: &'ctx [NodeRef<ConfigEntry>]) -> EvalResult {
        let mut config_value = self.dict_value();
        self.enter_scope();
        for item in items {
            let value = self.walk_expr(&item.node.value)?;
            if let Some(key) = &item.node.key {
                let mut insert_index = -1;
                let optional_name = match &key.node {
                    ast::Expr::Identifier(identifier) => Some(identifier.names[0].node.clone()),
                    ast::Expr::StringLit(string_lit) => Some(string_lit.value.clone()),
                    ast::Expr::Subscript(subscript) => {
                        let mut name = None;
                        if let ast::Expr::Identifier(identifier) = &subscript.value.node {
                            if let Some(index_node) = &subscript.index {
                                if let ast::Expr::NumberLit(number) = &index_node.node {
                                    if let ast::NumberLitValue::Int(v) = number.value {
                                        insert_index = v;
                                        name = Some(identifier.names[0].node.clone())
                                    }
                                }
                            }
                        }
                        name
                    }
                    _ => None,
                };
                // Store a local variable for every entry key.
                let key = match &optional_name {
                    Some(name) if !self.is_local_var(name) => self.string_value(name),
                    _ => self.walk_expr(key)?,
                };
                self.dict_insert(
                    &mut config_value,
                    &key.as_str(),
                    &value,
                    &item.node.operation,
                    insert_index as i32,
                );
                if let Some(name) = &optional_name {
                    let value = self.dict_get_value(&config_value, name);
                    self.add_or_update_local_variable(name, value);
                }
            } else {
                // If the key does not exist, execute the logic of unpacking expression `**expr` here.
                config_value.dict_insert_unpack(&mut self.runtime_ctx.borrow_mut(), &value)
            }
        }
        self.leave_scope();
        Ok(config_value)
    }
}
