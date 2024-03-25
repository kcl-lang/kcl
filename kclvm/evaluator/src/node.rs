// Copyright The KCL Authors. All rights reserved.

use anyhow::Ok;
use kclvm_ast::ast::{self, CallExpr, ConfigEntry, NodeRef};
use kclvm_ast::walker::TypedResultWalker;
use kclvm_runtime::{ApiFunc, ValueRef};

use crate::{error as kcl_error, GLOBAL_LEVEL, INNER_LEVEL};
use crate::{EvalResult, Evaluator};

/// Impl TypedResultWalker for Evaluator to visit AST nodes to emit LLVM IR.
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
        {
            self.ctx.borrow_mut().target_vars.push(name.clone());
        }
        // The right value of the unification_stmt is a schema_expr.
        let value = self
            .walk_schema_expr(&unification_stmt.value.node)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        if self.scope_level() == GLOBAL_LEVEL || self.is_in_lambda() {
            if self.resolve_variable(name) {
                let org_value = self
                    .walk_identifier_with_ctx(
                        &unification_stmt.target.node,
                        &ast::ExprContext::Load,
                        None,
                    )
                    .expect(kcl_error::COMPILE_ERROR_MSG);
                let fn_name = ApiFunc::kclvm_value_op_aug_bit_or;
                let value = self.build_call(&fn_name.name(), &[org_value, value]);
                // Store the identifier value
                self.walk_identifier_with_ctx(
                    &unification_stmt.target.node,
                    &ast::ExprContext::Store,
                    Some(value.clone()),
                )
                .expect(kcl_error::COMPILE_ERROR_MSG);
                return Ok(value);
            } else {
                self.walk_identifier_with_ctx(
                    &unification_stmt.target.node,
                    &unification_stmt.target.node.ctx,
                    Some(value.clone()),
                )
                .expect(kcl_error::COMPILE_ERROR_MSG);
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
                .expect(kcl_error::COMPILE_ERROR_MSG);
            let fn_name = ApiFunc::kclvm_value_op_bit_or;
            let value = self.build_call(&fn_name.name(), &[org_value, value]);
            // Store the identifier value
            self.walk_identifier_with_ctx(
                &unification_stmt.target.node,
                &ast::ExprContext::Store,
                Some(value.clone()),
            )
            .expect(kcl_error::COMPILE_ERROR_MSG);
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
            self.ctx
                .borrow_mut()
                .target_vars
                .push(name.node.names[0].node.clone());
        }
        // Load the right value
        let value = self
            .walk_expr(&assign_stmt.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        if let Some(_ty) = &assign_stmt.ty {
            // todo
        }
        if assign_stmt.targets.len() == 1 {
            // Store the single target
            let name = &assign_stmt.targets[0];
            self.walk_identifier_with_ctx(&name.node, &name.node.ctx, Some(value.clone()))
                .expect(kcl_error::COMPILE_ERROR_MSG);
        } else {
            // Store multiple targets
            for name in &assign_stmt.targets {
                let value = self.value_deep_copy(&value);
                self.walk_identifier_with_ctx(&name.node, &name.node.ctx, Some(value.clone()))
                    .expect(kcl_error::COMPILE_ERROR_MSG);
            }
        }
        Ok(value)
    }

    fn walk_aug_assign_stmt(&self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        {
            self.ctx
                .borrow_mut()
                .target_vars
                .push(aug_assign_stmt.target.node.names[0].node.clone());
        }
        // Load the right value
        let right_value = self
            .walk_expr(&aug_assign_stmt.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        // Load the identifier value
        let org_value = self
            .walk_identifier_with_ctx(&aug_assign_stmt.target.node, &ast::ExprContext::Load, None)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let fn_name = match aug_assign_stmt.op {
            ast::AugOp::Add => ApiFunc::kclvm_value_op_aug_add,
            ast::AugOp::Sub => ApiFunc::kclvm_value_op_aug_sub,
            ast::AugOp::Mul => ApiFunc::kclvm_value_op_aug_mul,
            ast::AugOp::Div => ApiFunc::kclvm_value_op_aug_div,
            ast::AugOp::Mod => ApiFunc::kclvm_value_op_aug_mod,
            ast::AugOp::Pow => ApiFunc::kclvm_value_op_aug_pow,
            ast::AugOp::LShift => ApiFunc::kclvm_value_op_aug_bit_lshift,
            ast::AugOp::RShift => ApiFunc::kclvm_value_op_aug_bit_rshift,
            ast::AugOp::BitOr => ApiFunc::kclvm_value_op_bit_or,
            ast::AugOp::BitXor => ApiFunc::kclvm_value_op_aug_bit_xor,
            ast::AugOp::BitAnd => ApiFunc::kclvm_value_op_aug_bit_and,
            ast::AugOp::FloorDiv => ApiFunc::kclvm_value_op_aug_floor_div,
            ast::AugOp::Assign => {
                return Err(anyhow::anyhow!(kcl_error::INVALID_OPERATOR_MSG));
            }
        };
        let value = self.build_call(&fn_name.name(), &[org_value, right_value]);
        // Store the identifier value
        self.walk_identifier_with_ctx(
            &aug_assign_stmt.target.node,
            &ast::ExprContext::Store,
            Some(value.clone()),
        )
        .expect(kcl_error::COMPILE_ERROR_MSG);
        Ok(value)
    }

    fn walk_assert_stmt(&self, _assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        todo!()
    }

    fn walk_if_stmt(&self, _if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        todo!()
    }
    fn walk_import_stmt(&self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        let pkgpath = import_stmt.path.node.as_str();
        // Check if it has already been generated, there is no need to generate code
        // for duplicate import statements.
        {
            let imported = &mut self.ctx.borrow_mut().imported;
            if imported.contains(pkgpath) {
                return self.ok_result();
            }
            // Deref the borrow mut
        }
        {
            let imported = &mut self.ctx.borrow_mut().imported;
            (*imported).insert(pkgpath.to_string());
            // Deref the borrow mut
        }
        self.ok_result()
    }

    fn walk_schema_stmt(&self, _schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        todo!()
    }

    fn walk_rule_stmt(&self, _rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        todo!()
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

    fn walk_quant_expr(&self, _quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        todo!()
    }

    fn walk_schema_attr(&self, _schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        todo!()
    }

    fn walk_if_expr(&self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        let cond = self
            .walk_expr(&if_expr.cond)
            .expect(kcl_error::COMPILE_ERROR_MSG);
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
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let fn_name = match unary_expr.op {
            ast::UnaryOp::UAdd => ApiFunc::kclvm_value_unary_plus,
            ast::UnaryOp::USub => ApiFunc::kclvm_value_unary_minus,
            ast::UnaryOp::Invert => ApiFunc::kclvm_value_unary_not,
            ast::UnaryOp::Not => ApiFunc::kclvm_value_unary_l_not,
        };
        Ok(self.build_call(&fn_name.name(), &[value]))
    }

    fn walk_binary_expr(&self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        let is_logic_op = matches!(binary_expr.op, ast::BinOp::And | ast::BinOp::Or);
        let is_membership_as_op = matches!(binary_expr.op, ast::BinOp::As);
        if !is_logic_op {
            let left_value = self
                .walk_expr(&binary_expr.left)
                .expect(kcl_error::COMPILE_ERROR_MSG);
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
                    .expect(kcl_error::COMPILE_ERROR_MSG)
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
            todo!()
        }
    }

    fn walk_selector_expr(&self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        let mut value = self
            .walk_expr(&selector_expr.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let string_ptr_value = self.string_value(selector_expr.attr.node.names[0].node.as_str());
        let fn_name = if selector_expr.has_question {
            &ApiFunc::kclvm_value_load_attr_option
        } else {
            &ApiFunc::kclvm_value_load_attr
        };
        value = self.build_call(&fn_name.name(), &[value, string_ptr_value]);
        for name in &selector_expr.attr.node.names[1..] {
            value = value.load_attr(&name.node)
        }
        Ok(value)
    }

    fn walk_call_expr(&self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        let _func = self
            .walk_expr(&call_expr.func)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        // args
        let mut list_value = self.list_value();
        for arg in &call_expr.args {
            let value = self.walk_expr(arg).expect(kcl_error::COMPILE_ERROR_MSG);
            self.list_append(&mut list_value, &value);
        }
        let mut dict_value = self.dict_value();
        // kwargs
        for keyword in &call_expr.keywords {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert(
                &mut dict_value,
                name.node.as_str(),
                &value,
                &ast::ConfigEntryOperation::Union,
                -1,
            );
        }
        let _pkgpath = self.current_pkgpath();
        let _is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
        todo!();
    }

    fn walk_subscript(&self, subscript: &'ctx ast::Subscript) -> Self::Result {
        let _value = self
            .walk_expr(&subscript.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        todo!();
    }

    fn walk_paren_expr(&self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        self.walk_expr(&paren_expr.expr)
    }

    fn walk_list_expr(&self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        let mut list_value = self.list_value();
        for item in &list_expr.elts {
            let value = self.walk_expr(item).expect(kcl_error::COMPILE_ERROR_MSG);
            match &item.node {
                ast::Expr::Starred(_) | ast::Expr::ListIfItem(_) => {
                    self.list_append_unpack(&mut list_value, &value);
                }
                _ => self.list_append(&mut list_value, &value),
            };
        }
        Ok(list_value)
    }

    fn walk_list_if_item_expr(
        &self,
        _list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        todo!()
    }

    fn walk_starred_expr(&self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        self.walk_expr(&starred_expr.value)
    }

    fn walk_list_comp(&self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        let collection_value = self.list_value();
        self.enter_scope();
        self.walk_generator(
            &list_comp.generators,
            &list_comp.elt,
            None,
            None,
            0,
            collection_value.clone(),
            ast::CompType::List,
        );
        self.leave_scope();
        Ok(collection_value)
    }

    fn walk_dict_comp(&self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        let collection_value = self.dict_value();
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
            collection_value.clone(),
            ast::CompType::Dict,
        );
        self.leave_scope();
        Ok(collection_value)
    }

    fn walk_config_if_entry_expr(
        &self,
        _config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        todo!()
    }

    fn walk_comp_clause(&self, _comp_clause: &'ctx ast::CompClause) -> Self::Result {
        // Nothing to do on this AST node
        self.ok_result()
    }

    fn walk_schema_expr(&self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        // Check the required attributes only when the values of all attributes
        // in the final schema are solved.
        let _is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
        {
            self.ctx.borrow_mut().schema_expr_stack.push(());
        }
        let _config_value = self
            .walk_expr(&schema_expr.config)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let _schema_type = self
            .walk_identifier_with_ctx(&schema_expr.name.node, &schema_expr.name.node.ctx, None)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let _config_expr = match &schema_expr.config.node {
            ast::Expr::Config(config_expr) => config_expr,
            _ => panic!("invalid schema config expr"),
        };
        let mut list_value = self.list_value();
        for arg in &schema_expr.args {
            let value = self.walk_expr(arg).expect(kcl_error::COMPILE_ERROR_MSG);
            self.list_append(&mut list_value, &value);
        }
        let mut dict_value = self.dict_value();
        for keyword in &schema_expr.kwargs {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert(
                &mut dict_value,
                name.node.as_str(),
                &value,
                &ast::ConfigEntryOperation::Union,
                -1,
            );
        }
        {
            self.ctx.borrow_mut().schema_expr_stack.pop();
        }
        self.ok_result()
    }

    fn walk_config_expr(&self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        self.walk_config_entries(&config_expr.items)
    }

    fn walk_check_expr(&self, _check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        todo!()
    }

    fn walk_lambda_expr(&self, _lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        let _pkgpath = &self.current_pkgpath();
        // Higher-order lambda requires capturing the current lambda closure variable
        // as well as the closure of a more external scope.
        let _last_closure_map = self.get_current_inner_scope_variable_map();
        todo!()
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
        let left_value = self
            .walk_expr(&compare.left)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        if compare.comparators.len() > 1 {
            for (i, op) in compare.ops.iter().enumerate() {
                let _has_next = i < (compare.ops.len() - 1);
                let right_value = self
                    .walk_expr(&compare.comparators[i])
                    .expect(kcl_error::COMPILE_ERROR_MSG);
                let _result_value = match op {
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
                };
                // Get next value using a store/load temp block
                todo!()
            }
            Ok(left_value)
        } else {
            let right_value = self
                .walk_expr(&compare.comparators[0])
                .expect(kcl_error::COMPILE_ERROR_MSG);
            let value = match &compare.ops[0] {
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
            };
            Ok(value)
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
            .expect(kcl_error::COMPILE_ERROR_MSG);
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
        let mut result = Ok(self.none_value());
        for stmt in stmts {
            result = self.walk_stmt(stmt);
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
                        if !self.store_variable_in_current_scope(name, value) {
                            todo!()
                        }
                    } else {
                        let is_local_var = self.is_local_var(name);
                        let value = right_value.clone().expect(kcl_error::INTERNAL_ERROR_MSG);
                        // Store schema attribute
                        if is_in_schema {
                            // If is in the backtrack, return the schema value.
                            todo!()
                        }
                        // Store loop variable
                        if is_local_var || !is_in_schema {
                            self.add_variable(name, value);
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
                                    todo!()
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
        _decorator: &'ctx CallExpr,
        _attr_name: Option<&str>,
        _is_schema_target: bool,
    ) -> EvalResult {
        todo!()
    }

    pub fn walk_arguments(
        &self,
        _arguments: &'ctx Option<ast::NodeRef<ast::Arguments>>,
        _args: ValueRef,
        _kwargs: ValueRef,
    ) {
        todo!()
    }

    pub fn walk_generator(
        &self,
        _generators: &'ctx [Box<ast::Node<ast::CompClause>>],
        _elt: &'ctx ast::Node<ast::Expr>,
        _val: Option<&'ctx ast::Node<ast::Expr>>,
        _op: Option<&'ctx ast::ConfigEntryOperation>,
        _gen_index: usize,
        _collection_value: ValueRef,
        _comp_type: ast::CompType,
    ) {
        todo!()
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
