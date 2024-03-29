use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::*;
use std::sync::Arc;

use crate::info::is_private_field;
use crate::ty::{
    sup, DictType, FunctionType, Parameter, Type, TypeInferMethods, TypeKind, TypeRef,
    RESERVED_TYPE_IDENTIFIERS,
};

use super::format::VALID_FORMAT_SPEC_SET;
use super::scope::{ScopeKind, ScopeObject, ScopeObjectKind};
use super::ty::ty_str_replace_pkgpath;
use super::Resolver;
/// ResolvedResult denotes the result, when the result is error,
/// put the message string into the diagnostic vector.
pub type ResolvedResult = TypeRef;

impl<'ctx> MutSelfTypedResultWalker<'ctx> for Resolver<'ctx> {
    type Result = ResolvedResult;

    fn walk_module(&mut self, module: &'ctx ast::Module) -> Self::Result {
        self.stmts(&module.body)
    }

    fn walk_expr_stmt(&mut self, expr_stmt: &'ctx ast::ExprStmt) -> Self::Result {
        let expr_types = self.exprs(&expr_stmt.exprs);
        if !expr_types.is_empty() {
            let ty = expr_types[expr_types.len() - 1].clone();
            if expr_types.len() > 1 {
                self.handler.add_compile_error(
                    "expression statement can only have one expression",
                    expr_stmt.exprs[1].get_span_pos(),
                );
            }
            ty
        } else {
            bug!("invalid expr statement exprs");
        }
    }

    fn walk_unification_stmt(
        &mut self,
        unification_stmt: &'ctx ast::UnificationStmt,
    ) -> Self::Result {
        let names = &unification_stmt.target.node.names;
        if names.len() > 1 {
            self.handler.add_compile_error(
                "unification identifier can not be selected",
                unification_stmt.target.get_span_pos(),
            );
        }
        let (start, end) = unification_stmt.value.get_span_pos();
        if names.is_empty() {
            self.handler.add_compile_error(
                "missing target in the unification statement",
                unification_stmt.value.get_span_pos(),
            );
            return self.any_ty();
        }
        self.ctx.l_value = true;
        let expected_ty = self.walk_identifier_expr(&unification_stmt.target);
        self.ctx.l_value = false;
        let obj =
            self.new_config_expr_context_item(&names[0].node, expected_ty.clone(), start, end);
        let init_stack_depth = self.switch_config_expr_context(Some(obj));
        let ty = self.walk_schema_expr(&unification_stmt.value.node);
        self.clear_config_expr_context(init_stack_depth as usize, false);
        self.must_assignable_to(
            ty.clone(),
            expected_ty.clone(),
            unification_stmt.target.get_span_pos(),
            None,
        );
        if !ty.is_any() && expected_ty.is_any() {
            self.set_type_to_scope(&names[0].node, ty, &names[0]);
        }
        expected_ty
    }

    fn walk_type_alias_stmt(&mut self, type_alias_stmt: &'ctx ast::TypeAliasStmt) -> Self::Result {
        let (start, end) = type_alias_stmt.type_name.get_span_pos();
        let mut ty = self
            .parse_ty_with_scope(Some(&type_alias_stmt.ty), (start.clone(), end.clone()))
            .as_ref()
            .clone();
        if let TypeKind::Schema(schema_ty) = &mut ty.kind {
            schema_ty.is_instance = false;
        }
        ty.is_type_alias = true;
        let ty = Arc::new(ty);
        let ty_str = ty.into_type_annotation_str();
        let name = type_alias_stmt.type_name.node.get_name();
        let mut mapping = IndexMap::default();
        mapping.insert(ty_str.clone(), "".to_string());
        self.ctx.import_names.insert(name.to_string(), mapping);
        self.add_type_alias(&name, &ty_str);
        if RESERVED_TYPE_IDENTIFIERS.contains(&name.as_str()) {
            self.handler.add_type_error(
                &format!(
                    "type alias '{}' cannot be the same as the built-in types ({:?})",
                    name, RESERVED_TYPE_IDENTIFIERS
                ),
                type_alias_stmt.type_name.get_span_pos(),
            );
        }
        self.insert_object(
            &name,
            ScopeObject {
                name: name.clone(),
                start,
                end,
                ty: ty.clone(),
                kind: ScopeObjectKind::TypeAlias,
                doc: None,
            },
        );
        self.node_ty_map.insert(
            self.get_node_key(type_alias_stmt.type_name.id.clone()),
            ty.clone(),
        );
        ty
    }

    fn walk_assign_stmt(&mut self, assign_stmt: &'ctx ast::AssignStmt) -> Self::Result {
        self.ctx.local_vars.clear();
        let mut value_ty = self.any_ty();
        let start = assign_stmt.targets[0].get_pos();
        let end = assign_stmt.value.get_pos();
        let is_config = matches!(assign_stmt.value.node, ast::Expr::Schema(_));
        for target in &assign_stmt.targets {
            // For invalid syntax assign statement, we just skip it
            // and show a syntax error only.
            if target.node.names.is_empty() {
                continue;
            }
            let name = &target.node.names[0].node;
            // Add global names.
            if (is_private_field(name) || is_config || !self.contains_global_name(name))
                && self.scope_level == 0
            {
                self.insert_global_name(name, &target.get_span_pos());
            }
            if target.node.names.len() == 1 {
                self.ctx.l_value = true;
                let expected_ty = self.walk_identifier_expr(target);
                self.ctx.l_value = false;
                match &expected_ty.kind {
                    TypeKind::Schema(ty) => {
                        let obj = self.new_config_expr_context_item(
                            &ty.name,
                            expected_ty.clone(),
                            start.clone(),
                            end.clone(),
                        );
                        let init_stack_depth = self.switch_config_expr_context(Some(obj));
                        value_ty = self.expr(&assign_stmt.value);
                        self.clear_config_expr_context(init_stack_depth as usize, false)
                    }
                    TypeKind::List(_) | TypeKind::Dict(_) | TypeKind::Union(_) => {
                        let obj = self.new_config_expr_context_item(
                            "[]",
                            expected_ty.clone(),
                            start.clone(),
                            end.clone(),
                        );
                        let init_stack_depth = self.switch_config_expr_context(Some(obj));
                        value_ty = self.expr(&assign_stmt.value);
                        self.check_assignment_type_annotation(assign_stmt, value_ty.clone());
                        self.clear_config_expr_context(init_stack_depth as usize, false)
                    }
                    _ => {
                        value_ty = self.expr(&assign_stmt.value);
                        // Check type annotation if exists.
                        self.check_assignment_type_annotation(assign_stmt, value_ty.clone());
                    }
                }
                self.must_assignable_to(
                    value_ty.clone(),
                    expected_ty.clone(),
                    target.get_span_pos(),
                    None,
                );
                if !value_ty.is_any() && expected_ty.is_any() && assign_stmt.ty.is_none() {
                    self.set_type_to_scope(name, value_ty.clone(), &target.node.names[0]);
                    if let Some(schema_ty) = &self.ctx.schema {
                        let mut schema_ty = schema_ty.borrow_mut();
                        schema_ty.set_type_of_attr(
                            name,
                            self.ctx.ty_ctx.infer_to_variable_type(value_ty.clone()),
                        );
                    }
                }
            } else {
                self.lookup_type_from_scope(name, target.get_span_pos());
                self.ctx.l_value = true;
                let expected_ty = self.walk_identifier_expr(target);
                self.ctx.l_value = false;
                value_ty = self.expr(&assign_stmt.value);
                // Check type annotation if exists.
                self.check_assignment_type_annotation(assign_stmt, value_ty.clone());
                self.must_assignable_to(value_ty.clone(), expected_ty, target.get_span_pos(), None)
            }
        }
        value_ty
    }

    fn walk_aug_assign_stmt(&mut self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        self.ctx.l_value = false;
        if !aug_assign_stmt.target.node.names.is_empty() {
            let is_config = matches!(aug_assign_stmt.value.node, ast::Expr::Schema(_));
            let name = &aug_assign_stmt.target.node.names[0].node;
            // Add global names.
            if is_private_field(name) || is_config || !self.contains_global_name(name) {
                if self.scope_level == 0 {
                    self.insert_global_name(name, &aug_assign_stmt.target.get_span_pos());
                }
            } else {
                let mut msgs = vec![Message {
                    range: aug_assign_stmt.target.get_span_pos(),
                    style: Style::LineAndColumn,
                    message: format!("Immutable variable '{}' is modified during compiling", name),
                    note: None,
                    suggested_replacement: None,
                }];
                if let Some(pos) = self.get_global_name_pos(name) {
                    msgs.push(Message {
                        range: pos.clone(),
                        style: Style::LineAndColumn,
                        message: format!("The variable '{}' is declared here firstly", name),
                        note: Some(format!(
                            "change the variable name to '_{}' to make it mutable",
                            name
                        )),
                        suggested_replacement: None,
                    })
                }
                self.handler.add_error(ErrorKind::ImmutableError, &msgs);
            }
        }
        let left_ty = self.walk_identifier_expr(&aug_assign_stmt.target);
        let right_ty = self.expr(&aug_assign_stmt.value);
        let op = match aug_assign_stmt.op.clone().try_into() {
            Ok(op) => op,
            Err(msg) => bug!("{}", msg),
        };
        let new_target_ty = self.binary(
            left_ty,
            right_ty,
            &op,
            aug_assign_stmt.target.get_span_pos(),
        );
        self.ctx.l_value = true;
        let expected_ty = self.walk_identifier_expr(&aug_assign_stmt.target);
        self.must_assignable_to(
            new_target_ty.clone(),
            expected_ty,
            aug_assign_stmt.target.get_span_pos(),
            None,
        );
        self.ctx.l_value = false;
        new_target_ty
    }

    fn walk_assert_stmt(&mut self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        self.expr(&assert_stmt.test);
        self.expr_or_any_type(&assert_stmt.if_cond);
        if let Some(msg) = &assert_stmt.msg {
            self.must_be_type(msg, self.str_ty());
        }
        self.any_ty()
    }

    fn walk_if_stmt(&mut self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        self.expr(&if_stmt.cond);
        self.stmts(&if_stmt.body);
        self.stmts(&if_stmt.orelse);
        self.any_ty()
    }

    fn walk_import_stmt(&mut self, _import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        // Nothing to do.
        self.any_ty()
    }

    fn walk_schema_stmt(&mut self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        self.resolve_schema_stmt(schema_stmt)
    }

    fn walk_rule_stmt(&mut self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        self.resolve_rule_stmt(rule_stmt)
    }

    fn walk_quant_expr(&mut self, quant_expr: &'ctx ast::QuantExpr) -> Self::Result {
        let iter_ty = self.expr(&quant_expr.target);
        let (start, mut end) = quant_expr.test.get_span_pos();
        if let Some(if_cond) = &quant_expr.if_cond {
            end = if_cond.get_end_pos();
        }
        self.enter_scope(start, end, ScopeKind::Loop);
        let (mut key_name, mut val_name) = (None, None);
        for (i, target) in quant_expr.variables.iter().enumerate() {
            if target.node.names.is_empty() {
                continue;
            }
            if target.node.names.len() > 1 {
                self.handler.add_compile_error(
                    "loop variables can only be ordinary identifiers",
                    target.get_span_pos(),
                );
            }
            let name = &target.node.names[0];
            if i == 0 {
                key_name = Some(name);
            } else if i == 1 {
                val_name = Some(name)
            } else {
                self.handler.add_compile_error(
                    &format!(
                        "the number of loop variables is {}, which can only be 1 or 2",
                        quant_expr.variables.len()
                    ),
                    target.get_span_pos(),
                );
                break;
            }
            self.ctx.local_vars.push(name.node.to_string());
            let (start, end) = target.get_span_pos();
            self.insert_object(
                &name.node,
                ScopeObject {
                    name: name.node.to_string(),
                    start,
                    end,
                    ty: self.any_ty(),
                    kind: ScopeObjectKind::Variable,
                    doc: None,
                },
            );
        }
        self.do_loop_type_check(
            key_name,
            val_name,
            iter_ty.clone(),
            quant_expr.target.get_span_pos(),
        );
        self.expr_or_any_type(&quant_expr.if_cond);
        let item_ty = self.expr(&quant_expr.test);
        self.leave_scope();
        match &quant_expr.op {
            ast::QuantOperation::All | ast::QuantOperation::Any => self.bool_ty(),
            ast::QuantOperation::Filter => iter_ty,
            ast::QuantOperation::Map => Arc::new(Type::list(item_ty)),
        }
    }

    fn walk_schema_attr(&mut self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        self.ctx.local_vars.clear();
        let (start, end) = schema_attr.name.get_span_pos();
        let name = if schema_attr.name.node.contains('.') {
            self.handler.add_compile_error(
                "schema attribute can not be selected",
                schema_attr.name.get_span_pos(),
            );
            schema_attr.name.node.split('.').collect::<Vec<&str>>()[0]
        } else {
            &schema_attr.name.node
        };
        let schema = self.ctx.schema.as_ref().unwrap();
        let expected_ty = schema
            .borrow()
            .get_type_of_attr(name)
            .map_or(self.any_ty(), |ty| ty);

        self.node_ty_map.insert(
            self.get_node_key(schema_attr.name.id.clone()),
            expected_ty.clone(),
        );

        let doc_str = schema
            .borrow()
            .attrs
            .get(name)
            .map(|attr| attr.doc.clone())
            .flatten();

        self.insert_object(
            name,
            ScopeObject {
                name: name.to_string(),
                start,
                end,
                ty: expected_ty.clone(),
                kind: ScopeObjectKind::Attribute,
                doc: doc_str,
            },
        );
        if let Some(value) = &schema_attr.value {
            let value_ty = if let TypeKind::Schema(ty) = &expected_ty.kind {
                let (start, end) = value.get_span_pos();
                let obj =
                    self.new_config_expr_context_item(&ty.name, expected_ty.clone(), start, end);
                let init_stack_depth = self.switch_config_expr_context(Some(obj));
                let value_ty = self.expr(value);
                self.clear_config_expr_context(init_stack_depth as usize, false);
                value_ty
            } else {
                self.expr(value)
            };
            match &schema_attr.op {
                Some(bin_or_aug) => match bin_or_aug {
                    // Union
                    ast::AugOp::BitOr => {
                        let op = ast::BinOp::BitOr;
                        let value_ty = self.binary(
                            value_ty,
                            expected_ty.clone(),
                            &op,
                            schema_attr.name.get_span_pos(),
                        );
                        self.must_assignable_to(
                            value_ty,
                            expected_ty,
                            schema_attr.name.get_span_pos(),
                            None,
                        );
                    }
                    // Assign
                    _ => self.must_assignable_to(
                        value_ty,
                        expected_ty,
                        schema_attr.name.get_span_pos(),
                        None,
                    ),
                },
                None => bug!("invalid ast schema attr op kind"),
            }
        }
        self.any_ty()
    }

    /// <body> if <cond> else <orelse> -> sup([body, orelse])
    fn walk_if_expr(&mut self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        self.expr(&if_expr.cond);
        let body_ty = self.expr(&if_expr.body);
        let orelse_ty = self.expr(&if_expr.orelse);
        sup(&[body_ty, orelse_ty])
    }

    fn walk_unary_expr(&mut self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        let operand_ty = self.expr(&unary_expr.operand);
        self.unary(
            operand_ty,
            &unary_expr.op,
            unary_expr.operand.get_span_pos(),
        )
    }

    fn walk_binary_expr(&mut self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        let left_ty = self.expr(&binary_expr.left);
        let mut right_ty = self.expr(&binary_expr.right);
        let range = (binary_expr.left.get_pos(), binary_expr.right.get_end_pos());
        match &binary_expr.op {
            ast::BinOp::As => {
                if let ast::Expr::Identifier(identifier) = &binary_expr.right.node {
                    right_ty = self.parse_ty_str_with_scope(
                        &identifier.get_name(),
                        binary_expr.right.get_span_pos(),
                    );
                    if right_ty.is_schema() {
                        let mut schema_ty = right_ty.into_schema_type();
                        schema_ty.is_instance = true;
                        right_ty = Arc::new(Type::schema(schema_ty));
                    }
                    let ty_annotation_str = right_ty.into_type_annotation_str();
                    self.add_type_alias(
                        &identifier.get_name(),
                        &ty_str_replace_pkgpath(&ty_annotation_str, &self.ctx.pkgpath),
                    );
                } else {
                    self.handler
                        .add_compile_error("keyword 'as' right operand must be a type", range);
                    return left_ty;
                }
                self.binary(left_ty, right_ty, &binary_expr.op, range)
            }
            _ => self.binary(left_ty, right_ty, &binary_expr.op, range),
        }
    }

    fn walk_selector_expr(&mut self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        let mut value_ty = self.expr(&selector_expr.value);
        if value_ty.is_module() && selector_expr.has_question {
            let attr = selector_expr.attr.node.get_name();
            self.handler.add_compile_error(
                &format!(
                    "For the module type, the use of '?.{}' is unnecessary and it can be modified as '.{}'",
                    attr,
                    attr
                ),
                selector_expr.value.get_span_pos(),
            );
        }
        for name in &selector_expr.attr.node.names {
            value_ty = self.load_attr(
                value_ty.clone(),
                &name.node,
                selector_expr.attr.get_span_pos(),
            );
            self.node_ty_map
                .insert(self.get_node_key(name.id.clone()), value_ty.clone());
        }

        if let TypeKind::Function(func) = &value_ty.kind {
            self.insert_object(
                &selector_expr.attr.node.get_name(),
                ScopeObject {
                    name: selector_expr.attr.node.get_name(),
                    start: selector_expr.attr.get_pos(),
                    end: selector_expr.attr.get_end_pos(),
                    ty: value_ty.clone(),
                    kind: ScopeObjectKind::FunctionCall,
                    doc: Some(func.doc.clone()),
                },
            )
        }

        value_ty
    }

    fn walk_call_expr(&mut self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        let call_ty = self.expr(&call_expr.func);
        let range = call_expr.func.get_span_pos();
        if call_ty.is_any() {
            self.do_arguments_type_check(
                &call_expr.func,
                &call_expr.args,
                &call_expr.keywords,
                &FunctionType::variadic_func(),
            );
            self.any_ty()
        } else if let TypeKind::Function(func_ty) = &call_ty.kind {
            self.do_arguments_type_check(
                &call_expr.func,
                &call_expr.args,
                &call_expr.keywords,
                &func_ty,
            );
            func_ty.return_ty.clone()
        } else if let TypeKind::Schema(schema_ty) = &call_ty.kind {
            if schema_ty.is_instance {
                self.handler.add_compile_error(
                    &format!("schema '{}' instance is not callable", call_ty.ty_str()),
                    range,
                );
                self.any_ty()
            } else {
                self.do_arguments_type_check(
                    &call_expr.func,
                    &call_expr.args,
                    &call_expr.keywords,
                    &schema_ty.func,
                );
                let mut return_ty = schema_ty.clone();
                return_ty.is_instance = true;
                Arc::new(Type::schema(return_ty))
            }
        } else {
            self.handler.add_compile_error(
                &format!("'{}' object is not callable", call_ty.ty_str()),
                range,
            );
            self.any_ty()
        }
    }

    fn walk_subscript(&mut self, subscript: &'ctx ast::Subscript) -> Self::Result {
        let value_ty = self.expr(&subscript.value);
        let range = subscript.value.get_span_pos();
        if value_ty.is_any() {
            value_ty
        } else {
            match &value_ty.kind {
                TypeKind::Str | TypeKind::StrLit(_) | TypeKind::List(_) => {
                    if let Some(index) = &subscript.index {
                        self.must_be_type(index, self.any_ty());
                        if value_ty.is_list() {
                            value_ty.list_item_ty()
                        } else {
                            self.str_ty()
                        }
                    } else {
                        for expr in [&subscript.lower, &subscript.upper, &subscript.step]
                            .iter()
                            .copied()
                            .flatten()
                        {
                            self.must_be_type(expr, self.int_ty());
                        }
                        if value_ty.is_list() {
                            value_ty
                        } else {
                            self.str_ty()
                        }
                    }
                }
                TypeKind::Dict(DictType {
                    key_ty: _, val_ty, ..
                }) => {
                    if let Some(index) = &subscript.index {
                        let index_key_ty = self.expr(index);
                        if index_key_ty.is_none_or_any() {
                            val_ty.clone()
                        } else if !index_key_ty.is_key() {
                            self.handler.add_compile_error(
                                &format!(
                                    "invalid dict/schema key type: '{}'",
                                    index_key_ty.ty_str()
                                ),
                                range,
                            );
                            self.any_ty()
                        } else if let TypeKind::StrLit(lit_value) = &index_key_ty.kind {
                            self.load_attr(value_ty, lit_value, range)
                        } else {
                            val_ty.clone()
                        }
                    } else {
                        self.handler
                            .add_compile_error("unhashable type: 'slice'", range);
                        self.any_ty()
                    }
                }
                TypeKind::Schema(schema_ty) => {
                    if let Some(index) = &subscript.index {
                        let index_key_ty = self.expr(index);
                        if index_key_ty.is_none_or_any() {
                            schema_ty.val_ty()
                        } else if !index_key_ty.is_key() {
                            self.handler.add_compile_error(
                                &format!(
                                    "invalid dict/schema key type: '{}'",
                                    index_key_ty.ty_str()
                                ),
                                range,
                            );
                            self.any_ty()
                        } else if let TypeKind::StrLit(lit_value) = &index_key_ty.kind {
                            self.load_attr(value_ty, lit_value, range)
                        } else {
                            schema_ty.val_ty()
                        }
                    } else {
                        self.handler
                            .add_compile_error("unhashable type: 'slice'", range);
                        self.any_ty()
                    }
                }
                _ => {
                    self.handler.add_compile_error(
                        &format!("'{}' object is not subscriptable", value_ty.ty_str()),
                        subscript.value.get_span_pos(),
                    );
                    self.any_ty()
                }
            }
        }
    }

    fn walk_paren_expr(&mut self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        self.expr(&paren_expr.expr)
    }

    fn walk_list_expr(&mut self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        let stack_depth = self.switch_list_expr_context();
        let item_type = sup(&self.exprs(&list_expr.elts).to_vec());
        self.clear_config_expr_context(stack_depth, false);
        Type::list_ref(item_type)
    }

    fn walk_list_comp(&mut self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        let start = list_comp.elt.get_pos();
        let stack_depth = self.switch_list_expr_context();
        let end = match list_comp.generators.last() {
            Some(last) => last.get_end_pos(),
            None => list_comp.elt.get_end_pos(),
        };
        self.enter_scope(start.clone(), end, ScopeKind::Loop);
        for comp_clause in &list_comp.generators {
            self.walk_comp_clause(&comp_clause.node);
        }
        if let ast::Expr::Starred(_) = list_comp.elt.node {
            self.handler.add_compile_error(
                "list unpacking cannot be used in list comprehension",
                list_comp.elt.get_span_pos(),
            );
        }
        let item_ty = self.expr(&list_comp.elt);
        self.leave_scope();
        self.clear_config_expr_context(stack_depth, false);
        Type::list_ref(item_ty)
    }

    fn walk_dict_comp(&mut self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        if dict_comp.entry.key.is_none() {
            self.handler.add_compile_error(
                "dict unpacking cannot be used in dict comprehension",
                dict_comp.entry.value.get_span_pos(),
            );
            let start = dict_comp.entry.value.get_pos();
            let end = match dict_comp.generators.last() {
                Some(last) => last.get_end_pos(),
                None => dict_comp.entry.value.get_end_pos(),
            };
            self.enter_scope(start.clone(), end, ScopeKind::Loop);
            for comp_clause in &dict_comp.generators {
                self.walk_comp_clause(&comp_clause.node);
            }
            let stack_depth = self.switch_config_expr_context_by_key(&dict_comp.entry.key);
            let val_ty = self.expr(&dict_comp.entry.value);
            let key_ty = match &val_ty.kind {
                TypeKind::None | TypeKind::Any => val_ty.clone(),
                TypeKind::Dict(DictType { key_ty, .. }) => key_ty.clone(),
                TypeKind::Schema(schema_ty) => schema_ty.key_ty().clone(),
                TypeKind::Union(types)
                    if self
                        .ctx
                        .ty_ctx
                        .is_config_type_or_config_union_type(val_ty.clone()) =>
                {
                    sup(&types
                        .iter()
                        .map(|ty| ty.config_key_ty())
                        .collect::<Vec<TypeRef>>())
                }
                _ => {
                    self.handler.add_compile_error(
                        &format!(
                            "only dict and schema can be used ** unpack, got '{}'",
                            val_ty.ty_str()
                        ),
                        dict_comp.entry.value.get_span_pos(),
                    );
                    self.any_ty()
                }
            };
            self.clear_config_expr_context(stack_depth, false);
            self.leave_scope();
            Type::dict_ref(key_ty, val_ty)
        } else {
            let key = dict_comp.entry.key.as_ref().unwrap();
            let end = match dict_comp.generators.last() {
                Some(last) => last.get_end_pos(),
                None => dict_comp.entry.value.get_end_pos(),
            };
            let start = key.get_pos();
            self.enter_scope(start.clone(), end, ScopeKind::Loop);
            for comp_clause in &dict_comp.generators {
                self.walk_comp_clause(&comp_clause.node);
            }
            let key_ty = self.expr(key);
            self.check_attr_ty(&key_ty, key.get_span_pos());
            let stack_depth = self.switch_config_expr_context_by_key(&dict_comp.entry.key);
            let val_ty = self.expr(&dict_comp.entry.value);
            self.clear_config_expr_context(stack_depth, false);
            self.leave_scope();
            Type::dict_ref(key_ty, val_ty)
        }
    }

    fn walk_list_if_item_expr(
        &mut self,
        list_if_item_expr: &'ctx ast::ListIfItemExpr,
    ) -> Self::Result {
        self.expr(&list_if_item_expr.if_cond);
        let mut or_else_ty = self.expr_or_any_type(&list_if_item_expr.orelse);
        // `orelse` node maybe a list unpack node, use its item type instead.
        if let TypeKind::List(item_ty) = &or_else_ty.kind {
            or_else_ty = item_ty.clone();
        }
        let exprs_ty = sup(&self.exprs(&list_if_item_expr.exprs).to_vec());
        sup(&[or_else_ty, exprs_ty])
    }

    fn walk_starred_expr(&mut self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        let value_ty = self.expr(&starred_expr.value);
        fn starred_ty_walk_fn(ty: &TypeRef) -> (TypeRef, bool) {
            match &ty.kind {
                TypeKind::None | TypeKind::Any => (ty.clone(), true),
                TypeKind::List(item_ty) => (item_ty.clone(), true),
                TypeKind::Dict(DictType { key_ty, .. }) => (key_ty.clone(), true),
                TypeKind::Schema(schema_ty) => (schema_ty.key_ty(), true),
                TypeKind::Union(types) => {
                    let results = types
                        .iter()
                        .map(starred_ty_walk_fn)
                        .collect::<Vec<(TypeRef, bool)>>();
                    (
                        sup(&results
                            .iter()
                            .map(|(ty, _)| ty)
                            .cloned()
                            .collect::<Vec<TypeRef>>()),
                        results.iter().all(|(_, r)| *r),
                    )
                }
                _ => (Arc::new(Type::ANY), false),
            }
        }
        let (ty, result) = starred_ty_walk_fn(&value_ty);
        if !result {
            self.handler.add_compile_error(
                &format!(
                    "only list, dict, schema object can be used * unpacked, got {}",
                    ty.ty_str()
                ),
                starred_expr.value.get_span_pos(),
            );
        }
        ty
    }

    fn walk_config_if_entry_expr(
        &mut self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        self.expr(&config_if_entry_expr.if_cond);
        let dict_ty = self.walk_config_entries(&config_if_entry_expr.items);
        if let Some(orelse) = &config_if_entry_expr.orelse {
            let or_else_ty = self.expr(orelse);
            sup(&[dict_ty, or_else_ty])
        } else {
            dict_ty
        }
    }

    fn walk_comp_clause(&mut self, comp_clause: &'ctx ast::CompClause) -> Self::Result {
        let iter_ty = self.expr(&comp_clause.iter);
        let (mut key_name, mut val_name) = (None, None);
        for (i, target) in comp_clause.targets.iter().enumerate() {
            if target.node.names.is_empty() {
                continue;
            }
            if target.node.names.len() > 1 {
                self.handler.add_compile_error(
                    "loop variables can only be ordinary identifiers",
                    target.get_span_pos(),
                );
            }
            let name = &target.node.names[0];
            if i == 0 {
                key_name = Some(name);
            } else if i == 1 {
                val_name = Some(name);
            } else {
                self.handler.add_compile_error(
                    &format!(
                        "the number of loop variables is {}, which can only be 1 or 2",
                        comp_clause.targets.len()
                    ),
                    target.get_span_pos(),
                );
                break;
            }
            self.ctx.local_vars.push(name.node.to_string());
            let (start, end) = target.get_span_pos();
            self.insert_object(
                &name.node,
                ScopeObject {
                    name: name.node.to_string(),
                    start,
                    end,
                    ty: self.any_ty(),
                    kind: ScopeObjectKind::Variable,
                    doc: None,
                },
            );
        }
        if iter_ty.is_any() {
            iter_ty
        } else {
            self.do_loop_type_check(key_name, val_name, iter_ty, comp_clause.iter.get_span_pos());
            self.exprs(&comp_clause.ifs);
            self.any_ty()
        }
    }

    fn walk_schema_expr(&mut self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        let def_ty = self.walk_identifier_expr(&schema_expr.name);
        if !matches!(&schema_expr.config.node, ast::Expr::Config(_)) {
            self.handler.add_compile_error(
                "Invalid schema config expr, expect config entries, e.g., {k1 = v1, k2 = v2}",
                schema_expr.config.get_span_pos(),
            );
        }
        let mut range = schema_expr.name.get_span_pos();
        let ret_ty = match &def_ty.kind {
            TypeKind::Dict(DictType { .. }) => {
                let obj = self.new_config_expr_context_item(
                    "",
                    def_ty.clone(),
                    Position::dummy_pos(),
                    Position::dummy_pos(),
                );
                let init_stack_depth = self.switch_config_expr_context(Some(obj));
                let config_ty = self.expr(&schema_expr.config);
                self.clear_config_expr_context(init_stack_depth as usize, false);
                self.binary(def_ty.clone(), config_ty, &ast::BinOp::BitOr, range)
            }
            TypeKind::Schema(schema_ty) => {
                if !schema_ty.is_instance {
                    let ty_annotation_str = ty_str_replace_pkgpath(
                        &def_ty.into_type_annotation_str(),
                        &self.ctx.pkgpath,
                    );
                    let name = schema_expr.name.node.get_name();
                    self.add_type_alias(&name, &ty_annotation_str);
                }
                let obj = self.new_config_expr_context_item(
                    &schema_ty.name,
                    def_ty.clone(),
                    Position::dummy_pos(),
                    Position::dummy_pos(),
                );
                let init_stack_depth = self.switch_config_expr_context(Some(obj));
                self.expr(&schema_expr.config);
                self.node_ty_map.insert(
                    self.get_node_key(schema_expr.config.id.clone()),
                    def_ty.clone(),
                );
                self.clear_config_expr_context(init_stack_depth as usize, false);
                if schema_ty.is_instance {
                    if !schema_expr.args.is_empty() || !schema_expr.kwargs.is_empty() {
                        self.handler.add_compile_error(
                            "Arguments cannot be used in the schema modification expression",
                            range,
                        );
                    }
                } else {
                    let func = Box::new(ast::Node::node_with_pos(
                        ast::Expr::Identifier(schema_expr.name.node.clone()),
                        schema_expr.name.pos(),
                    ));
                    self.do_arguments_type_check(
                        &func,
                        &schema_expr.args,
                        &schema_expr.kwargs,
                        &schema_ty.func,
                    );
                }
                self.any_ty()
            }
            _ => {
                range.0.filename = self.ctx.filename.clone();
                range.1.filename = self.ctx.filename.clone();
                self.handler.add_compile_error(
                    &format!("Invalid schema type '{}'", def_ty.ty_str()),
                    range,
                );
                return self.any_ty();
            }
        };
        let mut def_ty_clone = def_ty.as_ref().clone();
        if let TypeKind::Schema(schema_ty) = &mut def_ty_clone.kind {
            schema_ty.is_instance = true;
        }
        if def_ty_clone.is_schema() {
            Arc::new(def_ty_clone)
        } else {
            ret_ty
        }
    }

    fn walk_config_expr(&mut self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        self.walk_config_entries(&config_expr.items)
    }

    fn walk_check_expr(&mut self, check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        if let Some(msg) = &check_expr.msg {
            self.must_be_type(msg, self.str_ty());
        }
        // Check type in if_cond expression
        self.expr_or_any_type(&check_expr.if_cond);
        self.expr(&check_expr.test)
    }

    fn walk_lambda_expr(&mut self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        let mut ret_ty = self.any_ty();
        let mut params = vec![];
        self.do_parameters_check(&lambda_expr.args);
        if let Some(args) = &lambda_expr.args {
            for (i, arg) in args.node.args.iter().enumerate() {
                let name = arg.node.get_name();
                let arg_ty = args.node.get_arg_type_node(i);
                let range = match arg_ty {
                    Some(arg_type_node) => arg_type_node.get_span_pos(),
                    None => arg.get_span_pos(),
                };
                let ty = self.parse_ty_with_scope(arg_ty, range);

                // If the arguments type of a lambda is a schema type,
                // It should be marked as an schema instance type.
                let ty = if let TypeKind::Schema(sty) = &ty.kind {
                    let mut arg_ty = sty.clone();
                    arg_ty.is_instance = true;
                    Arc::new(Type::schema(arg_ty))
                } else {
                    ty.clone()
                };
                if let Some(name) = arg.node.names.last() {
                    self.node_ty_map
                        .insert(self.get_node_key(name.id.clone()), ty.clone());
                }

                let value = &args.node.defaults[i];
                params.push(Parameter {
                    name,
                    ty: ty.clone(),
                    has_default: value.is_some(),
                });
                self.expr_or_any_type(value);
            }
        }
        let (start, end) = (self.ctx.start_pos.clone(), self.ctx.end_pos.clone());
        if let Some(ret_annotation_ty) = &lambda_expr.return_ty {
            ret_ty =
                self.parse_ty_with_scope(Some(&ret_annotation_ty), (start.clone(), end.clone()));
        }
        self.enter_scope(start.clone(), end.clone(), ScopeKind::Lambda);
        self.ctx.in_lambda_expr.push(true);
        // Lambda parameters
        for param in &params {
            self.insert_object(
                &param.name,
                ScopeObject {
                    name: param.name.clone(),
                    start: start.clone(),
                    end: end.clone(),
                    ty: param.ty.clone(),
                    kind: ScopeObjectKind::Parameter,
                    doc: None,
                },
            )
        }
        if let Some(stmt) = lambda_expr.body.last() {
            if !matches!(
                stmt.node,
                ast::Stmt::Expr(_)
                    | ast::Stmt::Assign(_)
                    | ast::Stmt::AugAssign(_)
                    | ast::Stmt::Assert(_)
            ) {
                self.handler.add_compile_error(
                    "The last statement of the lambda body must be a expression e.g., x, 1, etc.",
                    stmt.get_span_pos(),
                );
            }
        }
        let real_ret_ty = self.stmts(&lambda_expr.body);
        self.leave_scope();
        self.ctx.in_lambda_expr.pop();
        self.must_assignable_to(real_ret_ty.clone(), ret_ty.clone(), (start, end), None);
        if !real_ret_ty.is_any() && ret_ty.is_any() && lambda_expr.return_ty.is_none() {
            ret_ty = real_ret_ty;
        }
        Arc::new(Type::function(None, ret_ty, &params, "", false, None))
    }

    fn walk_keyword(&mut self, keyword: &'ctx ast::Keyword) -> Self::Result {
        self.walk_identifier_expr(&keyword.arg);
        self.expr_or_any_type(&keyword.value)
    }

    fn walk_arguments(&mut self, arguments: &'ctx ast::Arguments) -> Self::Result {
        for (i, arg) in arguments.args.iter().enumerate() {
            let ty = arguments.get_arg_type_node(i);
            let ty = self.parse_ty_with_scope(ty, arg.get_span_pos());
            if let Some(name) = arg.node.names.last() {
                self.node_ty_map
                    .insert(self.get_node_key(name.id.clone()), ty.clone());
            }
            let value = &arguments.defaults[i];
            self.expr_or_any_type(value);
        }
        self.any_ty()
    }

    fn walk_compare(&mut self, compare: &'ctx ast::Compare) -> Self::Result {
        let t1 = self.expr(&compare.left);
        let t2 = self.expr(&compare.comparators[0]);
        self.compare(
            t1.clone(),
            t2.clone(),
            &compare.ops[0],
            (compare.left.get_pos(), compare.comparators[0].get_end_pos()),
        );
        for i in 1..compare.comparators.len() - 1 {
            let op = &compare.ops[i + 1];
            let t2 = self.expr(&compare.comparators[i]);
            self.compare(
                t1.clone(),
                t2.clone(),
                op,
                compare.comparators[i].get_span_pos(),
            );
        }
        self.bool_ty()
    }

    fn walk_identifier(&mut self, identifier: &'ctx ast::Identifier) -> Self::Result {
        let tys = self.resolve_var(
            &identifier.get_names(),
            &identifier.pkgpath,
            (self.ctx.start_pos.clone(), self.ctx.end_pos.clone()),
        );
        for (index, name) in identifier.names.iter().enumerate() {
            self.node_ty_map
                .insert(self.get_node_key(name.id.clone()), tys[index].clone());
        }
        tys.last().unwrap().clone()
    }

    fn walk_number_lit(&mut self, number_lit: &'ctx ast::NumberLit) -> Self::Result {
        match &number_lit.binary_suffix {
            Some(binary_suffix) => {
                let raw_value = match number_lit.value {
                    ast::NumberLitValue::Int(int_val) => int_val,
                    ast::NumberLitValue::Float(float_val) => {
                        self.handler.add_compile_error(
                            "float literal can not be followed the unit suffix",
                            (self.ctx.start_pos.clone(), self.ctx.end_pos.clone()),
                        );
                        float_val as i64
                    }
                };
                let binary_suffix_str: String = binary_suffix.value();
                let value = kclvm_runtime::units::cal_num(raw_value, &binary_suffix_str);
                Arc::new(Type::number_multiplier(
                    value,
                    raw_value,
                    &binary_suffix_str,
                ))
            }
            None => match number_lit.value {
                ast::NumberLitValue::Int(int_val) => Arc::new(Type::int_lit(int_val)),
                ast::NumberLitValue::Float(float_val) => Arc::new(Type::float_lit(float_val)),
            },
        }
    }

    fn walk_string_lit(&mut self, string_lit: &'ctx ast::StringLit) -> Self::Result {
        Arc::new(Type::str_lit(&string_lit.value))
    }

    fn walk_name_constant_lit(
        &mut self,
        name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        match &name_constant_lit.value {
            ast::NameConstant::True => Arc::new(Type::bool_lit(true)),
            ast::NameConstant::False => Arc::new(Type::bool_lit(false)),
            ast::NameConstant::None | ast::NameConstant::Undefined => self.none_ty(),
        }
    }

    fn walk_joined_string(&mut self, joined_string: &'ctx ast::JoinedString) -> Self::Result {
        self.ctx.l_value = false;
        self.exprs(&joined_string.values);
        self.str_ty()
    }

    fn walk_formatted_value(&mut self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result {
        if let Some(spec) = &formatted_value.format_spec {
            let spec_lower = spec.to_lowercase();
            if !VALID_FORMAT_SPEC_SET.contains(&spec_lower.as_str()) {
                self.handler.add_compile_error(
                    &format!("{} is a invalid format spec", spec),
                    formatted_value.value.get_span_pos(),
                );
            }
        }
        self.expr(&formatted_value.value)
    }

    fn walk_comment(&mut self, _comment: &'ctx ast::Comment) -> Self::Result {
        // Nothing to do.
        self.any_ty()
    }

    fn walk_missing_expr(&mut self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        // Nothing to do.
        self.any_ty()
    }
}

impl<'ctx> Resolver<'ctx> {
    #[inline]
    pub fn stmts(&mut self, stmts: &'ctx [ast::NodeRef<ast::Stmt>]) -> ResolvedResult {
        let stmt_types: Vec<TypeRef> = stmts.iter().map(|stmt| self.stmt(&stmt)).collect();
        match stmt_types.last() {
            Some(ty) => ty.clone(),
            _ => self.any_ty(),
        }
    }

    #[inline]
    pub fn exprs(&mut self, exprs: &'ctx [ast::NodeRef<ast::Expr>]) -> Vec<ResolvedResult> {
        exprs.iter().map(|expr| self.expr(&expr)).collect()
    }

    #[inline]
    pub fn expr(&mut self, expr: &'ctx ast::NodeRef<ast::Expr>) -> ResolvedResult {
        if let ast::Expr::Identifier(_) = &expr.node {
            let (start, end) = expr.get_span_pos();
            self.ctx.start_pos = start;
            self.ctx.end_pos = end;
        }
        let ty = self.walk_expr(&expr.node);
        self.node_ty_map
            .insert(self.get_node_key(expr.id.clone()), ty.clone());
        ty
    }

    #[inline]
    pub fn stmt(&mut self, stmt: &'ctx ast::NodeRef<ast::Stmt>) -> ResolvedResult {
        let (start, end) = stmt.get_span_pos();
        self.ctx.start_pos = start;
        self.ctx.end_pos = end;
        let ty = self.walk_stmt(&stmt.node);
        self.node_ty_map
            .insert(self.get_node_key(stmt.id.clone()), ty.clone());
        ty
    }

    #[inline]
    pub fn expr_or_any_type(
        &mut self,
        expr: &'ctx Option<ast::NodeRef<ast::Expr>>,
    ) -> ResolvedResult {
        match expr {
            Some(expr) => {
                let ty = self.walk_expr(&expr.node);
                self.node_ty_map
                    .insert(self.get_node_key(expr.id.clone()), ty.clone());
                ty
            }
            None => self.any_ty(),
        }
    }

    #[inline]
    pub fn walk_identifier_expr(
        &mut self,
        identifier: &'ctx ast::NodeRef<ast::Identifier>,
    ) -> ResolvedResult {
        let tys = self.resolve_var(
            &identifier.node.get_names(),
            &identifier.node.pkgpath,
            identifier.get_span_pos(),
        );
        for (index, name) in identifier.node.names.iter().enumerate() {
            self.node_ty_map
                .insert(self.get_node_key(name.id.clone()), tys[index].clone());
        }
        let ident_ty = tys.last().unwrap().clone();
        self.node_ty_map
            .insert(self.get_node_key(identifier.id.clone()), ident_ty.clone());

        ident_ty
    }
}
