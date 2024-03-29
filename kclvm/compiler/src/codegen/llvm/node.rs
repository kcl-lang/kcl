// Copyright The KCL Authors. All rights reserved.

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;

use inkwell::basic_block::BasicBlock;
use inkwell::module::Linkage;
use inkwell::values::{BasicValueEnum, CallableValue, FunctionValue};
use inkwell::{AddressSpace, IntPredicate};
use kclvm_ast::ast::{self, CallExpr, ConfigEntry, NodeRef};
use kclvm_ast::walker::TypedResultWalker;
use kclvm_runtime::{ApiFunc, PKG_PATH_PREFIX};
use kclvm_sema::pkgpath_without_prefix;
use kclvm_sema::ty::{ANY_TYPE_STR, STR_TYPE_STR};

use crate::check_backtrack_stop;
use crate::codegen::error as kcl_error;
use crate::codegen::llvm::context::BacktrackMeta;
use crate::codegen::llvm::utils;
use crate::codegen::traits::*;
use crate::codegen::{ENTRY_NAME, GLOBAL_LEVEL, INNER_LEVEL, PKG_INIT_FUNCTION_SUFFIX};

use super::context::{CompileResult, LLVMCodeGenContext};
use crate::value;
use kclvm_sema::builtin;
use kclvm_sema::plugin;

/// Impl TypedResultWalker for LLVMCodeGenContext to visit AST nodes to emit LLVM IR.
impl<'ctx> TypedResultWalker<'ctx> for LLVMCodeGenContext<'ctx> {
    type Result = CompileResult<'ctx>;

    /*
     * Stmt
     */

    fn walk_stmt(&self, stmt: &'ctx ast::Node<ast::Stmt>) -> Self::Result {
        check_backtrack_stop!(self);
        utils::update_ctx_filename(self, stmt);
        utils::update_ctx_line_col(self, stmt);
        utils::reset_target_vars(self);
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
        check_backtrack_stop!(self);
        let mut result = self.ok_result();
        for expr in &expr_stmt.exprs {
            let scalar = self.walk_expr(expr)?;
            // Only non-call expressions are allowed to emit values bacause of the function void return type.
            if !matches!(expr.node, ast::Expr::Call(_)) {
                self.add_scalar(scalar, matches!(expr.node, ast::Expr::Schema(_)));
            }
            result = Ok(scalar);
        }
        result
    }

    fn walk_unification_stmt(&self, unification_stmt: &'ctx ast::UnificationStmt) -> Self::Result {
        check_backtrack_stop!(self);
        self.local_vars.borrow_mut().clear();
        let name = &unification_stmt.target.node.names[0].node;
        self.target_vars.borrow_mut().push(name.clone());
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
                let value = self.build_call(
                    &fn_name.name(),
                    &[self.current_runtime_ctx_ptr(), org_value, value],
                );
                // Store the identifier value
                self.walk_identifier_with_ctx(
                    &unification_stmt.target.node,
                    &ast::ExprContext::Store,
                    Some(value),
                )
                .expect(kcl_error::COMPILE_ERROR_MSG);
                return Ok(value);
            } else {
                self.walk_identifier_with_ctx(
                    &unification_stmt.target.node,
                    &unification_stmt.target.node.ctx,
                    Some(value),
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
            let value = self.build_call(
                &fn_name.name(),
                &[self.current_runtime_ctx_ptr(), org_value, value],
            );
            // Store the identifier value
            self.walk_identifier_with_ctx(
                &unification_stmt.target.node,
                &ast::ExprContext::Store,
                Some(value),
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
        check_backtrack_stop!(self);
        self.local_vars.borrow_mut().clear();
        // Set target vars.
        for name in &assign_stmt.targets {
            self.target_vars
                .borrow_mut()
                .push(name.node.names[0].node.clone());
        }
        // Load the right value
        let mut value = self
            .walk_expr(&assign_stmt.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        if let Some(ty) = &assign_stmt.ty {
            let type_annotation = self.native_global_string_value(&ty.node.to_string());
            let is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
            value = self.build_call(
                &ApiFunc::kclvm_convert_collection_value.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    value,
                    type_annotation,
                    self.bool_value(is_in_schema),
                ],
            );
        }
        if assign_stmt.targets.len() == 1 {
            // Store the single target
            let name = &assign_stmt.targets[0];
            self.walk_identifier_with_ctx(&name.node, &name.node.ctx, Some(value))
                .expect(kcl_error::COMPILE_ERROR_MSG);
        } else {
            // Store multiple targets
            for name in &assign_stmt.targets {
                let value = self.value_deep_copy(value);
                self.walk_identifier_with_ctx(&name.node, &name.node.ctx, Some(value))
                    .expect(kcl_error::COMPILE_ERROR_MSG);
            }
        }
        Ok(value)
    }

    fn walk_aug_assign_stmt(&self, aug_assign_stmt: &'ctx ast::AugAssignStmt) -> Self::Result {
        check_backtrack_stop!(self);
        self.target_vars
            .borrow_mut()
            .push(aug_assign_stmt.target.node.names[0].node.clone());
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
                return Err(kcl_error::KCLError::new(kcl_error::INVALID_OPERATOR_MSG));
            }
        };
        let value = self.build_call(
            &fn_name.name(),
            &[self.current_runtime_ctx_ptr(), org_value, right_value],
        );
        // Store the identifier value
        self.walk_identifier_with_ctx(
            &aug_assign_stmt.target.node,
            &ast::ExprContext::Store,
            Some(value),
        )
        .expect(kcl_error::COMPILE_ERROR_MSG);
        Ok(value)
    }

    fn walk_assert_stmt(&self, assert_stmt: &'ctx ast::AssertStmt) -> Self::Result {
        check_backtrack_stop!(self);
        let start_block = self.append_block("");
        let end_block = self.append_block("");
        if let Some(if_cond) = &assert_stmt.if_cond {
            let if_value = self.walk_expr(if_cond).expect(kcl_error::COMPILE_ERROR_MSG);
            let is_truth = self.value_is_truthy(if_value);
            self.cond_br(is_truth, start_block, end_block);
        } else {
            self.br(start_block);
        }
        self.builder.position_at_end(start_block);
        let assert_result = self
            .walk_expr(&assert_stmt.test)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        // Assert statement error message.
        let msg = {
            if let Some(msg) = &assert_stmt.msg {
                self.walk_expr(msg).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.string_value("")
            }
        };
        self.build_void_call(
            &ApiFunc::kclvm_assert.name(),
            &[self.current_runtime_ctx_ptr(), assert_result, msg],
        );
        self.br(end_block);
        self.builder.position_at_end(end_block);
        Ok(self.undefined_value())
    }

    fn walk_if_stmt(&self, if_stmt: &'ctx ast::IfStmt) -> Self::Result {
        check_backtrack_stop!(self);
        let cond = self
            .walk_expr(&if_stmt.cond)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let then_block = self.append_block("");
        let else_block = self.append_block("");
        let end_block = self.append_block("");
        let is_truth = self.value_is_truthy(cond);
        self.cond_br(is_truth, then_block, else_block);
        self.builder.position_at_end(then_block);
        self.walk_stmts(&if_stmt.body)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        self.br(end_block);
        self.builder.position_at_end(else_block);
        self.walk_stmts(&if_stmt.orelse)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        self.br(end_block);
        self.builder.position_at_end(end_block);
        Ok(self.none_value())
    }

    fn walk_import_stmt(&self, import_stmt: &'ctx ast::ImportStmt) -> Self::Result {
        check_backtrack_stop!(self);
        let pkgpath = import_stmt.path.node.as_str();
        // Check if it has already been generated, there is no need to generate code
        // for duplicate import statements.
        {
            let imported = self.imported.borrow_mut();
            if imported.contains(pkgpath) {
                return self.ok_result();
            }
            // Deref the borrow mut
        }
        // Standard or plugin modules.
        if builtin::STANDARD_SYSTEM_MODULES.contains(&pkgpath)
            || pkgpath.starts_with(plugin::PLUGIN_MODULE_PREFIX)
        {
            // Nothing to do on the builtin system module import because the check has been done.
            return self.ok_result();
        } else {
            let pkgpath = format!("{}{}", PKG_PATH_PREFIX, import_stmt.path.node);
            self.pkgpath_stack.borrow_mut().push(pkgpath.clone());
            let has_pkgpath = self.program.pkgs.contains_key(&import_stmt.path.node);
            let func_before_block = if self.no_link {
                if has_pkgpath {
                    let func_before_block = self.append_block("");
                    self.br(func_before_block);
                    let mut modules = self.modules.borrow_mut();
                    let name = pkgpath.clone();
                    let module = self.context.create_module(&name);
                    let module_name = format!(
                        "${}.{}",
                        pkgpath_without_prefix!(pkgpath),
                        PKG_INIT_FUNCTION_SUFFIX
                    );
                    let tpe = self.context.void_type();
                    let fn_type = tpe.fn_type(&[self.context_ptr_type().into()], false);
                    let function = module.add_function(
                        // Function name
                        &module_name,
                        // Function type
                        fn_type,
                        None,
                    );
                    // Add a block named entry into the function
                    let basic_block = self.context.append_basic_block(function, ENTRY_NAME);
                    self.builder.position_at_end(basic_block);
                    self.push_function(function);
                    modules.insert(name, RefCell::new(self.create_debug_module(module)));
                    Some(func_before_block)
                } else {
                    None
                }
            } else {
                None
            };
            if has_pkgpath {
                // Init all builtin functions.
                self.init_scope(pkgpath.as_str());
                self.compile_ast_modules(
                    self.program
                        .pkgs
                        .get(&import_stmt.path.node)
                        .expect(kcl_error::INTERNAL_ERROR_MSG),
                );
            }
            self.pkgpath_stack.borrow_mut().pop();
            if self.no_link {
                let name = format!(
                    "${}.{}",
                    pkgpath_without_prefix!(pkgpath),
                    PKG_INIT_FUNCTION_SUFFIX
                );
                let function = {
                    let pkgpath = self.current_pkgpath();
                    let modules = self.modules.borrow_mut();
                    let msg = format!("pkgpath {} is not found", pkgpath);
                    let module = &modules.get(&pkgpath).expect(&msg).borrow_mut().inner;
                    if has_pkgpath {
                        self.ret_void();
                        self.pop_function();
                        self.builder.position_at_end(
                            func_before_block.expect(kcl_error::INTERNAL_ERROR_MSG),
                        );
                    }
                    let tpe = self.context.void_type();
                    let fn_type = tpe.fn_type(&[self.context_ptr_type().into()], false);
                    module.add_function(&name, fn_type, Some(Linkage::External))
                };
                let ctx = self.current_runtime_ctx_ptr();
                let pkgpath_value = self.native_global_string_value(&name);
                let is_imported = self
                    .build_call(
                        &ApiFunc::kclvm_context_pkgpath_is_imported.name(),
                        &[self.current_runtime_ctx_ptr(), pkgpath_value],
                    )
                    .into_int_value();
                let is_not_imported = self.builder.build_int_compare(
                    IntPredicate::EQ,
                    is_imported,
                    self.native_i8_zero(),
                    "",
                );
                let then_block = self.append_block("");
                let else_block = self.append_block("");
                self.builder
                    .build_conditional_branch(is_not_imported, then_block, else_block);
                self.builder.position_at_end(then_block);
                self.builder.build_call(function, &[ctx.into()], "");
                self.br(else_block);
                self.builder.position_at_end(else_block);
            }
        };
        {
            let mut imported = self.imported.borrow_mut();
            (*imported).insert(pkgpath.to_string());
            // Deref the borrow mut
        }
        self.ok_result()
    }

    fn walk_schema_stmt(&self, schema_stmt: &'ctx ast::SchemaStmt) -> Self::Result {
        check_backtrack_stop!(self);
        let func_before_block = self.append_block("");
        self.br(func_before_block);
        let value_ptr_type = self.value_ptr_type();
        let schema_name = &schema_stmt.name.node;
        let schema_pkgpath = &self.current_pkgpath();
        let filename = &self.current_filename();
        let runtime_type = kclvm_runtime::schema_runtime_type(schema_name, schema_pkgpath);
        // Build schema body function
        let function = self.add_function(&format!(
            "{}.{}",
            value::SCHEMA_NAME,
            pkgpath_without_prefix!(runtime_type)
        ));
        // Build the schema check function.
        let check_function = self.add_function(&format!(
            "{}.{}",
            value::SCHEMA_CHECK_BLOCK_NAME,
            pkgpath_without_prefix!(runtime_type),
        ));
        let mut place_holder_map: HashMap<String, Vec<FunctionValue<'ctx>>> = HashMap::new();
        let mut body_map: HashMap<String, Vec<&ast::Node<ast::Stmt>>> = HashMap::new();
        // Enter the function
        self.push_function(function);
        // Lambda function body
        let block = self.append_block(ENTRY_NAME);
        self.builder.position_at_end(block);
        self.build_void_call(
            &ApiFunc::kclvm_context_set_kcl_filename.name(),
            &[
                self.current_runtime_ctx_ptr(),
                self.native_global_string_value(filename),
            ],
        );
        utils::update_ctx_pkgpath(self, schema_pkgpath);
        let args = function
            .get_nth_param(1)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let kwargs = function
            .get_nth_param(2)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        self.enter_scope();
        let add_variable = |name: &str, value: BasicValueEnum| {
            let var = self.builder.build_alloca(value_ptr_type, name);
            self.builder.build_store(var, value);
            self.add_variable(name, var);
        };
        // Schema function closures
        let instance_pkgpath = self.list_pop(args);
        let record_instance = self.list_pop(args);
        let backtrack_cache = self.list_pop(args);
        let backtrack_level_map = self.list_pop(args);
        let cal_map = self.list_pop(args);
        let attr_optional_mapping = self.list_pop(args);
        let schema_value = self.list_pop(args);
        let schema_config = self.list_pop(args);
        let schema_config_meta = self.list_pop(args);
        let is_sub_schema = self.list_pop(args);
        add_variable(value::BACKTRACK_CACHE, backtrack_cache);
        add_variable(value::BACKTRACK_LEVEL_MAP, backtrack_level_map);
        add_variable(value::SCHEMA_CAL_MAP, cal_map);
        add_variable(value::SCHEMA_CONFIG_NAME, schema_config);
        add_variable(value::SCHEMA_CONFIG_META_NAME, schema_config_meta);
        add_variable(value::SCHEMA_ARGS, args);
        add_variable(value::SCHEMA_KWARGS, kwargs);
        add_variable(value::SCHEMA_RUNTIME_TYPE, self.string_value(&runtime_type));
        self.walk_arguments(&schema_stmt.args, args, kwargs);
        let schema = value::SchemaType::new(
            schema_name,
            schema_pkgpath,
            &runtime_type,
            schema_stmt.is_mixin,
        );
        let schema_value = if let Some(parent_name) = &schema_stmt.parent_name {
            let base_constructor_func = self
                .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::COMPILE_ERROR_MSG);
            // Schema function closures
            let list_value = self.list_values(&[
                // is_sub_schema
                self.bool_value(false),
                schema_config_meta,
                schema_config,
                schema_value,
                attr_optional_mapping,
                cal_map,
                backtrack_level_map,
                backtrack_cache,
                record_instance,
                instance_pkgpath,
            ]);
            let dict_value = self.dict_value();
            let func_ptr = self.build_call(
                &ApiFunc::kclvm_value_function_ptr.name(),
                &[base_constructor_func],
            );
            let fn_ty = self.function_type().ptr_type(AddressSpace::default());
            let func_ptr_cast = self.builder.build_bitcast(func_ptr, fn_ty, "");
            self.builder
                .build_call(
                    CallableValue::try_from(func_ptr_cast.into_pointer_value())
                        .expect(kcl_error::INTERNAL_ERROR_MSG),
                    &[
                        self.current_runtime_ctx_ptr().into(),
                        list_value.into(),
                        dict_value.into(),
                    ],
                    "",
                )
                .try_as_basic_value()
                .left()
                .expect(kcl_error::FUNCTION_RETURN_VALUE_NOT_FOUND_MSG)
        } else {
            schema_value
        };
        if schema_stmt.parent_name.is_some() {
            self.build_void_call(
                &ApiFunc::kclvm_context_set_kcl_filename.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    self.native_global_string_value(filename),
                ],
            );
        }
        self.schema_stack.borrow_mut().push(schema);
        add_variable(value::SCHEMA_SELF_NAME, schema_value);
        self.emit_left_identifiers(
            &schema_stmt.body,
            &schema_stmt.index_signature,
            cal_map,
            &runtime_type,
            false,
            &mut place_holder_map,
            &mut body_map,
            &mut vec![],
        );
        let do_run_i1 = self.value_is_truthy(record_instance);
        let do_run_block = self.append_block("");
        let end_run_block = self.append_block("");
        self.cond_br(do_run_i1, do_run_block, end_run_block);
        self.builder.position_at_end(do_run_block);
        // Run schema compiled function
        for stmt in &schema_stmt.body {
            self.walk_stmt(stmt).expect(kcl_error::COMPILE_ERROR_MSG);
        }
        // Schema decorators check
        for decorator in &schema_stmt.decorators {
            self.walk_decorator_with_name(&decorator.node, Some(schema_name), true)
                .expect(kcl_error::COMPILE_ERROR_MSG);
        }
        // Append schema default settings, args, kwargs and runtime type.
        self.build_void_call(
            &ApiFunc::kclvm_schema_default_settings.name(),
            &[
                schema_value,
                schema_config,
                args,
                kwargs,
                self.native_global_string_value(&runtime_type),
            ],
        );
        self.br(end_run_block);
        self.builder.position_at_end(end_run_block);
        // Schema mixin
        for mixin in &schema_stmt.mixins {
            let mixin_func = self
                .walk_identifier_with_ctx(&mixin.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::COMPILE_ERROR_MSG);
            // Schema function closures
            let list_value = self.list_values(&[
                // is_sub_schema
                self.bool_value(false),
                schema_config_meta,
                schema_config,
                schema_value,
                attr_optional_mapping,
                cal_map,
                backtrack_level_map,
                backtrack_cache,
                record_instance,
                instance_pkgpath,
            ]);
            let dict_value = self.dict_value();
            let func_ptr =
                self.build_call(&ApiFunc::kclvm_value_function_ptr.name(), &[mixin_func]);
            let fn_ty = self.function_type().ptr_type(AddressSpace::default());
            let func_ptr_cast = self.builder.build_bitcast(func_ptr, fn_ty, "");
            self.builder.build_call(
                CallableValue::try_from(func_ptr_cast.into_pointer_value())
                    .expect(kcl_error::INTERNAL_ERROR_MSG),
                &[
                    self.current_runtime_ctx_ptr().into(),
                    list_value.into(),
                    dict_value.into(),
                ],
                "",
            );
            self.build_void_call(
                &ApiFunc::kclvm_context_set_kcl_filename.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    self.native_global_string_value(filename),
                ],
            );
        }
        // Schema Attribute optional check
        for stmt in &schema_stmt.body {
            if let ast::Stmt::SchemaAttr(schema_attr) = &stmt.node {
                self.dict_insert_override_item(
                    attr_optional_mapping,
                    schema_attr.name.node.as_str(),
                    self.bool_value(schema_attr.is_optional),
                )
            }
        }
        let is_sub_schema_i1 = self.value_is_truthy(is_sub_schema);
        let do_check_block = self.append_block("");
        let end_check_block = self.append_block("");
        self.cond_br(is_sub_schema_i1, do_check_block, end_check_block);
        self.builder.position_at_end(do_check_block);
        let schema_name_native_str = self.native_global_string_value(&schema_stmt.name.node);
        let schema_pkgpath_native_str = self.native_global_string_value(&self.current_pkgpath());
        {
            let index_sign_key_name = if let Some(index_signature) = &schema_stmt.index_signature {
                if let Some(key_name) = &index_signature.node.key_name {
                    key_name
                } else {
                    ""
                }
            } else {
                ""
            };
            let list_value = self.value_deep_copy(args);
            let dict_value = self.value_deep_copy(kwargs);
            // Schema check function closure
            self.list_append(list_value, schema_config_meta);
            self.list_append(list_value, schema_config);
            self.list_append(list_value, schema_value);
            self.list_append(list_value, cal_map);
            self.list_append(list_value, backtrack_level_map);
            self.list_append(list_value, backtrack_cache);
            if index_sign_key_name.is_empty() {
                // Call schema check block function
                self.builder.build_call(
                    check_function,
                    &[
                        self.current_runtime_ctx_ptr().into(),
                        list_value.into(),
                        dict_value.into(),
                    ],
                    "",
                );
            } else {
                // Call schema check block function with index sign attribute name loop set
                let check_lambda_fn_ptr = self.builder.build_bitcast(
                    check_function.as_global_value().as_pointer_value(),
                    self.context.i64_type().ptr_type(AddressSpace::default()),
                    "",
                );
                let attr_name = self.native_global_string_value(index_sign_key_name);
                self.build_void_call(
                    ApiFunc::kclvm_schema_do_check_with_index_sign_attr
                        .name()
                        .as_str(),
                    &[
                        self.current_runtime_ctx_ptr(),
                        list_value,
                        dict_value,
                        check_lambda_fn_ptr,
                        attr_name,
                    ],
                );
            }
        }
        self.br(end_check_block);
        self.builder.position_at_end(end_check_block);
        // Build a schema value and record instance
        let schema_value = self.build_call(
            &ApiFunc::kclvm_value_schema_with_config.name(),
            &[
                self.current_runtime_ctx_ptr(),
                schema_value,
                schema_config,
                schema_config_meta,
                schema_name_native_str,
                schema_pkgpath_native_str,
                is_sub_schema,
                record_instance,
                instance_pkgpath,
                attr_optional_mapping,
                args,
                kwargs,
            ],
        );
        // Schema constructor function returns a schema
        self.builder.build_return(Some(&schema_value));
        // Exist the function
        self.builder.position_at_end(func_before_block);
        // Build schema check function
        {
            self.push_function(check_function);
            let check_block = self.append_block(ENTRY_NAME);
            self.builder.position_at_end(check_block);
            let args = function
                .get_nth_param(1)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            // Schema check function closure
            let backtrack_cache = self.list_pop(args);
            let backtrack_level_map = self.list_pop(args);
            let cal_map = self.list_pop(args);
            let schema_value = self.list_pop(args);
            let schema_config = self.list_pop(args);
            let schema_config_meta = self.list_pop(args);
            add_variable(value::BACKTRACK_CACHE, backtrack_cache);
            add_variable(value::BACKTRACK_LEVEL_MAP, backtrack_level_map);
            add_variable(value::SCHEMA_CAL_MAP, cal_map);
            add_variable(value::SCHEMA_CONFIG_NAME, schema_config);
            add_variable(value::SCHEMA_CONFIG_META_NAME, schema_config_meta);
            add_variable(value::SCHEMA_SELF_NAME, schema_value);
            add_variable(value::SCHEMA_ARGS, args);
            add_variable(value::SCHEMA_KWARGS, kwargs);
            add_variable(value::SCHEMA_RUNTIME_TYPE, self.string_value(&runtime_type));
            let schema = self
                .schema_stack
                .borrow_mut()
                .pop()
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            self.walk_arguments(&schema_stmt.args, args, kwargs);
            self.schema_stack.borrow_mut().push(schema);
            // Schema runtime index signature and relaxed check
            if let Some(index_signature) = &schema_stmt.index_signature {
                let index_sign_value = if let Some(value) = &index_signature.node.value {
                    self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG)
                } else {
                    self.undefined_value()
                };
                let key_name_str_ptr = if let Some(key_name) = &index_signature.node.key_name {
                    self.native_global_string(key_name.as_str(), "")
                } else {
                    self.native_global_string("", "")
                };
                self.build_void_call(
                    &ApiFunc::kclvm_schema_value_check.name(),
                    &[
                        self.current_runtime_ctx_ptr(),
                        schema_value,
                        schema_config,
                        schema_config_meta,
                        schema_name_native_str,
                        index_sign_value,
                        key_name_str_ptr.into(),
                        self.native_global_string(
                            index_signature.node.key_ty.node.to_string().as_str(),
                            "",
                        )
                        .into(),
                        self.native_global_string(
                            index_signature.node.value_ty.node.to_string().as_str(),
                            "",
                        )
                        .into(),
                        self.native_i8(index_signature.node.any_other as i8).into(),
                    ],
                );
            } else {
                self.build_void_call(
                    &ApiFunc::kclvm_schema_value_check.name(),
                    &[
                        self.current_runtime_ctx_ptr(),
                        schema_value,
                        schema_config,
                        schema_config_meta,
                        schema_name_native_str,
                        self.none_value(),
                        self.native_global_string("", "").into(),
                        self.native_global_string("", "").into(),
                        self.native_global_string("", "").into(),
                        self.native_i8(0).into(),
                    ],
                );
            }
            // Call base check function
            if let Some(parent_name) = &schema_stmt.parent_name {
                let base_constructor_func = self
                    .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
                    .expect(kcl_error::COMPILE_ERROR_MSG);
                let func_ptr = self.build_call(
                    &ApiFunc::kclvm_value_check_function_ptr.name(),
                    &[base_constructor_func],
                );
                let fn_ty = self.function_type().ptr_type(AddressSpace::default());
                let func_ptr_cast = self.builder.build_bitcast(func_ptr, fn_ty, "");
                // Schema check function closure
                let list_value = self.list_values(&[
                    schema_config_meta,
                    schema_config,
                    schema_value,
                    cal_map,
                    backtrack_level_map,
                    backtrack_cache,
                ]);
                let dict_value = self.dict_value();
                self.builder.build_call(
                    CallableValue::try_from(func_ptr_cast.into_pointer_value())
                        .expect(kcl_error::INTERNAL_ERROR_MSG),
                    &[
                        self.current_runtime_ctx_ptr().into(),
                        list_value.into(),
                        dict_value.into(),
                    ],
                    "",
                );
                self.build_void_call(
                    &ApiFunc::kclvm_context_set_kcl_filename.name(),
                    &[
                        self.current_runtime_ctx_ptr(),
                        self.native_global_string_value(filename),
                    ],
                );
            }
            // Call self check function
            for check_expr in &schema_stmt.checks {
                self.walk_check_expr(&check_expr.node)
                    .expect(kcl_error::COMPILE_ERROR_MSG);
            }
            // Call mixin check functions
            for mixin in &schema_stmt.mixins {
                let mixin_func = self
                    .walk_identifier_with_ctx(&mixin.node, &ast::ExprContext::Load, None)
                    .expect(kcl_error::COMPILE_ERROR_MSG);
                let func_ptr = self.build_call(
                    &ApiFunc::kclvm_value_check_function_ptr.name(),
                    &[mixin_func],
                );
                let fn_ty = self.function_type().ptr_type(AddressSpace::default());
                let func_ptr_cast = self.builder.build_bitcast(func_ptr, fn_ty, "");
                // Schema check function closure
                let list_value = self.list_values(&[
                    schema_config_meta,
                    schema_config,
                    schema_value,
                    cal_map,
                    backtrack_level_map,
                    backtrack_cache,
                ]);
                let dict_value = self.dict_value();
                self.builder.build_call(
                    CallableValue::try_from(func_ptr_cast.into_pointer_value())
                        .expect(kcl_error::INTERNAL_ERROR_MSG),
                    &[
                        self.current_runtime_ctx_ptr().into(),
                        list_value.into(),
                        dict_value.into(),
                    ],
                    "",
                );
                self.build_void_call(
                    &ApiFunc::kclvm_context_set_kcl_filename.name(),
                    &[
                        self.current_runtime_ctx_ptr(),
                        self.native_global_string_value(filename),
                    ],
                );
            }
            self.builder.build_return(Some(&schema_value));
            self.builder.position_at_end(func_before_block);
            self.pop_function();
        }
        // Build schema attr backtrack functions
        {
            for (k, functions) in &place_holder_map {
                if k == kclvm_runtime::CAL_MAP_INDEX_SIGNATURE {
                    continue;
                }
                let stmt_list = body_map.get(k).expect(kcl_error::INTERNAL_ERROR_MSG);
                let mut if_level = 0;
                for (attr_func, stmt) in functions.iter().zip(stmt_list) {
                    let function = *attr_func;
                    let name = function
                        .get_name()
                        .to_str()
                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                    // Get schema attr function from the module
                    let function = self.lookup_function(name);
                    self.push_function(function);
                    self.enter_scope();
                    let attr_block = self.append_block(ENTRY_NAME);
                    self.builder.position_at_end(attr_block);
                    let args = function
                        .get_nth_param(1)
                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                    let kwargs = function
                        .get_nth_param(2)
                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                    // Schema attr function closure
                    let backtrack_cache = self.list_pop(args);
                    let backtrack_level_map = self.list_pop(args);
                    let cal_map = self.list_pop(args);
                    let schema_value = self.list_pop(args);
                    let schema_config = self.list_pop(args);
                    let schema_config_meta = self.list_pop(args);
                    // Store magic variable
                    add_variable(value::BACKTRACK_CACHE, backtrack_cache);
                    add_variable(value::BACKTRACK_LEVEL_MAP, backtrack_level_map);
                    add_variable(value::SCHEMA_CAL_MAP, cal_map);
                    add_variable(value::SCHEMA_CONFIG_NAME, schema_config);
                    add_variable(value::SCHEMA_CONFIG_META_NAME, schema_config_meta);
                    add_variable(value::SCHEMA_SELF_NAME, schema_value);
                    add_variable(value::SCHEMA_ARGS, args);
                    add_variable(value::SCHEMA_KWARGS, kwargs);
                    add_variable(value::SCHEMA_RUNTIME_TYPE, self.string_value(&runtime_type));
                    self.build_void_call(
                        &ApiFunc::kclvm_context_set_kcl_filename.name(),
                        &[
                            self.current_runtime_ctx_ptr(),
                            self.native_global_string_value(filename),
                        ],
                    );
                    let schema = self
                        .schema_stack
                        .borrow_mut()
                        .pop()
                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                    self.walk_arguments(&schema_stmt.args, args, kwargs);
                    self.schema_stack.borrow_mut().push(schema);
                    // Backtrack meta begin
                    if matches!(&stmt.node, ast::Stmt::If(..)) {
                        if_level += 1;
                        *self.backtrack_meta.borrow_mut() = Some(BacktrackMeta {
                            target: k.clone(),
                            level: if_level,
                            count: 0,
                            stop: false,
                        });
                    } else {
                        if_level = 0;
                    }
                    self.walk_stmt(stmt).expect(kcl_error::COMPILE_ERROR_MSG);
                    // Backtrack meta end
                    if matches!(&stmt.node, ast::Stmt::If(..)) {
                        *self.backtrack_meta.borrow_mut() = None
                    }
                    // Build return
                    self.builder.build_return(Some(&schema_value));
                    // Position at global main function block
                    self.builder.position_at_end(func_before_block);
                    self.leave_scope();
                    self.pop_function();
                }
            }
        }
        let function = self.struct_function_value(
            &[function, check_function],
            &place_holder_map,
            &runtime_type,
        );
        self.leave_scope();
        self.pop_function();
        self.schema_stack.borrow_mut().pop();
        // Store or add the variable in the scope
        if !self.store_variable(schema_name, function) {
            let global_var_ptr = self.new_global_kcl_value_ptr("");
            self.builder.build_store(global_var_ptr, function);
            self.add_variable(schema_name, global_var_ptr);
        }
        Ok(function)
    }

    fn walk_rule_stmt(&self, rule_stmt: &'ctx ast::RuleStmt) -> Self::Result {
        check_backtrack_stop!(self);
        let func_before_block = self.append_block("");
        self.br(func_before_block);
        let value_ptr_type = self.value_ptr_type();
        let name = &rule_stmt.name.node;
        let pkgpath = &self.current_pkgpath();
        let filename = &self.current_filename();
        let runtime_type = kclvm_runtime::schema_runtime_type(name, pkgpath);
        // Build schema body function
        let function = self.add_function(&format!(
            "{}.{}",
            value::SCHEMA_NAME,
            pkgpath_without_prefix!(runtime_type)
        ));
        // Build the schema check function.
        let check_function = self.add_function(&format!(
            "{}.{}",
            value::SCHEMA_CHECK_BLOCK_NAME,
            pkgpath_without_prefix!(runtime_type),
        ));
        // Enter the function
        self.push_function(function);
        // Lambda function body
        let block = self.append_block(ENTRY_NAME);
        self.builder.position_at_end(block);
        self.build_void_call(
            &ApiFunc::kclvm_context_set_kcl_filename.name(),
            &[
                self.current_runtime_ctx_ptr(),
                self.native_global_string_value(filename),
            ],
        );
        let args = function
            .get_nth_param(1)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let kwargs = function
            .get_nth_param(2)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        self.enter_scope();
        // Schema function closures
        let instance_pkgpath = self.list_pop(args);
        let record_instance = self.list_pop(args);
        let backtrack_cache = self.list_pop(args);
        let backtrack_level_map = self.list_pop(args);
        let cal_map = self.list_pop(args);
        let attr_optional_mapping = self.list_pop(args);
        let schema_value = self.list_pop(args);
        let schema_config = self.list_pop(args);
        let schema_config_meta = self.list_pop(args);
        let is_sub_schema = self.list_pop(args);
        let add_variable = |name: &str, value: BasicValueEnum| {
            let var = self.builder.build_alloca(value_ptr_type, name);
            self.builder.build_store(var, value);
            self.add_variable(name, var);
        };
        add_variable(value::BACKTRACK_CACHE, backtrack_cache);
        add_variable(value::BACKTRACK_LEVEL_MAP, backtrack_level_map);
        add_variable(value::SCHEMA_CAL_MAP, cal_map);
        add_variable(value::SCHEMA_CONFIG_NAME, schema_config);
        add_variable(value::SCHEMA_CONFIG_META_NAME, schema_config_meta);
        add_variable(value::SCHEMA_ARGS, args);
        add_variable(value::SCHEMA_KWARGS, kwargs);
        add_variable(value::SCHEMA_RUNTIME_TYPE, self.string_value(&runtime_type));
        self.walk_arguments(&rule_stmt.args, args, kwargs);
        let schema = value::SchemaType::new(name, pkgpath, &runtime_type, false);
        self.schema_stack.borrow_mut().push(schema);
        add_variable(value::SCHEMA_SELF_NAME, schema_value);
        // construct for protocol
        let schema_value = if let Some(for_host_name) = &rule_stmt.for_host_name {
            let base_constructor_func = self
                .walk_identifier_with_ctx(&for_host_name.node, &ast::ExprContext::Load, None)
                .expect(kcl_error::COMPILE_ERROR_MSG);
            // Schema function closures
            let list_value = self.list_values(&[
                // is_sub_schema
                self.bool_value(false),
                schema_config_meta,
                schema_config,
                schema_value,
                attr_optional_mapping,
                cal_map,
                backtrack_level_map,
                backtrack_cache,
                record_instance,
                instance_pkgpath,
            ]);
            let dict_value = self.dict_value();
            let func_ptr = self.build_call(
                &ApiFunc::kclvm_value_function_ptr.name(),
                &[base_constructor_func],
            );
            let fn_ty = self.function_type().ptr_type(AddressSpace::default());
            let func_ptr_cast = self.builder.build_bitcast(func_ptr, fn_ty, "");
            let schema_value = self
                .builder
                .build_call(
                    CallableValue::try_from(func_ptr_cast.into_pointer_value())
                        .expect(kcl_error::INTERNAL_ERROR_MSG),
                    &[
                        self.current_runtime_ctx_ptr().into(),
                        list_value.into(),
                        dict_value.into(),
                    ],
                    "",
                )
                .try_as_basic_value()
                .left()
                .expect(kcl_error::FUNCTION_RETURN_VALUE_NOT_FOUND_MSG);
            let protocol_name_native_str =
                self.native_global_string_value(&for_host_name.node.get_name());
            self.build_void_call(
                &ApiFunc::kclvm_schema_value_check.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    schema_value,
                    schema_config,
                    schema_config_meta,
                    protocol_name_native_str,
                    self.undefined_value(),
                    self.native_global_string("", "").into(),
                    self.native_global_string(STR_TYPE_STR, "").into(),
                    self.native_global_string(ANY_TYPE_STR, "").into(),
                    self.native_i8(1).into(),
                ],
            );
            schema_value
        } else {
            schema_value
        };
        let do_run_i1 = self.value_is_truthy(record_instance);
        let do_run_block = self.append_block("");
        let end_run_block = self.append_block("");
        self.cond_br(do_run_i1, do_run_block, end_run_block);
        self.builder.position_at_end(do_run_block);
        // Rule decorators check
        for decorator in &rule_stmt.decorators {
            self.walk_decorator_with_name(&decorator.node, Some(name), true)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
        }
        self.br(end_run_block);
        self.builder.position_at_end(end_run_block);
        let is_sub_schema_i1 = self.value_is_truthy(is_sub_schema);
        let do_check_block = self.append_block("");
        let end_check_block = self.append_block("");
        self.cond_br(is_sub_schema_i1, do_check_block, end_check_block);
        self.builder.position_at_end(do_check_block);
        {
            // Schema check function closure
            let list_value = self.list_values(&[
                schema_config_meta,
                schema_config,
                schema_value,
                cal_map,
                backtrack_level_map,
                backtrack_cache,
            ]);
            let dict_value = self.dict_value();
            // Call schema check block function
            self.builder.build_call(
                check_function,
                &[
                    self.current_runtime_ctx_ptr().into(),
                    list_value.into(),
                    dict_value.into(),
                ],
                "",
            );
        }
        self.br(end_check_block);
        self.builder.position_at_end(end_check_block);
        // Rule constructor function returns a rule
        self.builder.build_return(Some(&schema_value));
        // Exist the function
        self.builder.position_at_end(func_before_block);
        // Build rule check function
        {
            self.push_function(check_function);
            let check_block = self.append_block(ENTRY_NAME);
            self.builder.position_at_end(check_block);
            let args = function
                .get_nth_param(1)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            // Schema check function closure
            let backtrack_cache = self.list_pop(args);
            let backtrack_level_map = self.list_pop(args);
            let cal_map = self.list_pop(args);
            let schema_value = self.list_pop(args);
            let schema_config = self.list_pop(args);
            let schema_config_meta = self.list_pop(args);
            add_variable(value::BACKTRACK_CACHE, backtrack_cache);
            add_variable(value::BACKTRACK_LEVEL_MAP, backtrack_level_map);
            add_variable(value::SCHEMA_CAL_MAP, cal_map);
            add_variable(value::SCHEMA_CONFIG_NAME, schema_config);
            add_variable(value::SCHEMA_CONFIG_META_NAME, schema_config_meta);
            add_variable(value::SCHEMA_SELF_NAME, schema_value);
            add_variable(value::SCHEMA_ARGS, args);
            add_variable(value::SCHEMA_KWARGS, kwargs);
            add_variable(value::SCHEMA_RUNTIME_TYPE, self.string_value(&runtime_type));
            let schema = self
                .schema_stack
                .borrow_mut()
                .pop()
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            self.walk_arguments(&rule_stmt.args, args, kwargs);
            self.schema_stack.borrow_mut().push(schema);
            // Call base check function
            for parent_name in &rule_stmt.parent_rules {
                let base_constructor_func = self
                    .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
                    .expect(kcl_error::COMPILE_ERROR_MSG);
                let func_ptr = self.build_call(
                    &ApiFunc::kclvm_value_check_function_ptr.name(),
                    &[base_constructor_func],
                );
                let fn_ty = self.function_type().ptr_type(AddressSpace::default());
                let func_ptr_cast = self.builder.build_bitcast(func_ptr, fn_ty, "");
                // Schema check function closure
                let list_value = self.list_values(&[
                    schema_config_meta,
                    schema_config,
                    schema_value,
                    cal_map,
                    backtrack_level_map,
                    backtrack_cache,
                ]);
                let dict_value = self.dict_value();
                self.builder.build_call(
                    CallableValue::try_from(func_ptr_cast.into_pointer_value())
                        .expect(kcl_error::INTERNAL_ERROR_MSG),
                    &[
                        self.current_runtime_ctx_ptr().into(),
                        list_value.into(),
                        dict_value.into(),
                    ],
                    "",
                );
            }
            // Call self rule check expressions
            for check_expr in &rule_stmt.checks {
                self.walk_check_expr(&check_expr.node)
                    .expect(kcl_error::COMPILE_ERROR_MSG);
            }
            self.builder.build_return(Some(&schema_value));
            self.builder.position_at_end(func_before_block);
            self.pop_function();
        }
        let function =
            self.struct_function_value(&[function, check_function], &HashMap::new(), &runtime_type);
        self.leave_scope();
        self.pop_function();
        self.schema_stack.borrow_mut().pop();
        // Store or add the variable in the scope
        if !self.store_variable(name, function) {
            let global_var_ptr = self.new_global_kcl_value_ptr(&runtime_type);
            self.builder.build_store(global_var_ptr, function);
            self.add_variable(name, global_var_ptr);
        }
        Ok(function)
    }

    /*
     * Expr
     */

    fn walk_expr(&self, expr: &'ctx ast::Node<ast::Expr>) -> Self::Result {
        check_backtrack_stop!(self);
        utils::update_ctx_filename(self, expr);
        utils::update_ctx_line_col(self, expr);
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
        check_backtrack_stop!(self);
        let result = match quant_expr.op {
            ast::QuantOperation::All => self.bool_value(true),
            ast::QuantOperation::Any => self.bool_value(false),
            ast::QuantOperation::Map => self.list_value(),
            ast::QuantOperation::Filter => self.value_deep_copy(
                self.walk_expr(&quant_expr.target)
                    .expect(kcl_error::COMPILE_ERROR_MSG),
            ),
        };
        // Blocks
        let start_block = self.append_block("");
        let next_value_block = self.append_block("");
        let continue_block = self.append_block("");
        let end_for_block = self.append_block("");
        let all_break_block = self.append_block("");
        let any_break_block = self.append_block("");
        let result_block = self.append_block("");
        // Iterator
        let iter_host_value = if let ast::QuantOperation::Filter = quant_expr.op {
            self.value_deep_copy(result)
        } else {
            self.walk_expr(&quant_expr.target)
                .expect(kcl_error::COMPILE_ERROR_MSG)
        };
        let iter_value = self.build_call(&ApiFunc::kclvm_value_iter.name(), &[iter_host_value]);
        self.br(start_block);
        self.builder.position_at_end(start_block);
        self.enter_scope();
        let is_end = self
            .build_call(&ApiFunc::kclvm_iterator_is_end.name(), &[iter_value])
            .into_int_value();
        let is_end =
            self.builder
                .build_int_compare(IntPredicate::NE, is_end, self.native_i8_zero(), "");
        self.builder
            .build_conditional_branch(is_end, end_for_block, next_value_block);
        self.builder.position_at_end(next_value_block);
        let next_value = self.build_call(
            &ApiFunc::kclvm_iterator_next_value.name(),
            &[iter_value, iter_host_value],
        );
        let key = self.build_call(&ApiFunc::kclvm_iterator_cur_key.name(), &[iter_value]);
        let variables = &quant_expr.variables;
        {
            let mut local_vars = self.local_vars.borrow_mut();
            for v in variables {
                let name = &v.node.names[0].node;
                local_vars.insert(name.clone());
            }
        }
        if variables.len() == 1 {
            // Store the target
            self.walk_identifier_with_ctx(
                &variables.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                &ast::ExprContext::Store,
                Some(next_value),
            )
            .expect(kcl_error::COMPILE_ERROR_MSG);
        } else if variables.len() == 2 {
            let value = self.build_call(&ApiFunc::kclvm_iterator_cur_value.name(), &[iter_value]);
            // Store the target
            self.walk_identifier_with_ctx(
                &variables.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                &ast::ExprContext::Store,
                Some(key),
            )
            .expect(kcl_error::COMPILE_ERROR_MSG);
            self.walk_identifier_with_ctx(
                &variables.get(1).expect(kcl_error::INTERNAL_ERROR_MSG).node,
                &ast::ExprContext::Store,
                Some(value),
            )
            .expect(kcl_error::COMPILE_ERROR_MSG);
        } else {
            panic!(
                "the number of loop variables is {}, which can only be 1 or 2",
                variables.len()
            )
        }
        if let Some(if_expr) = &quant_expr.if_cond {
            let if_truth = self.walk_expr(if_expr).expect(kcl_error::COMPILE_ERROR_MSG);
            let is_truth = self.value_is_truthy(if_truth);
            self.cond_br(is_truth, continue_block, start_block);
        } else {
            self.br(continue_block);
        }
        self.builder.position_at_end(continue_block);
        // Body block
        let test = &quant_expr.test;
        let value = self.walk_expr(test).expect(kcl_error::COMPILE_ERROR_MSG);
        let is_truth = self.value_is_truthy(value);
        match quant_expr.op {
            ast::QuantOperation::All => {
                self.cond_br(is_truth, start_block, all_break_block);
            }
            ast::QuantOperation::Any => {
                self.cond_br(is_truth, any_break_block, start_block);
            }
            ast::QuantOperation::Filter => {
                let then_block = self.append_block("");
                self.cond_br(is_truth, start_block, then_block);
                self.builder.position_at_end(then_block);
                self.build_void_call(
                    &ApiFunc::kclvm_value_remove_item.name(),
                    &[result, next_value],
                );
                self.br(start_block);
            }
            ast::QuantOperation::Map => {
                self.list_append(result, value);
                self.br(start_block);
            }
        }
        self.builder.position_at_end(all_break_block);
        let all_false_value = self.bool_value(false);
        self.br(result_block);
        self.builder.position_at_end(any_break_block);
        let any_true_value = self.bool_value(true);
        self.br(result_block);
        self.builder.position_at_end(end_for_block);
        let tpe = self.value_ptr_type();
        let ptr = self.builder.build_alloca(tpe, "");
        self.builder.build_store(ptr, result);
        let value = self.builder.build_load(ptr, "");
        self.br(result_block);
        self.builder.position_at_end(result_block);
        let phi = self.builder.build_phi(tpe, "");
        phi.add_incoming(&[
            (&all_false_value, all_break_block),
            (&any_true_value, any_break_block),
            (&value, end_for_block),
        ]);
        self.leave_scope();
        self.local_vars.borrow_mut().clear();
        self.build_void_call(&ApiFunc::kclvm_iterator_delete.name(), &[iter_value]);
        Ok(phi.as_basic_value())
    }

    fn walk_schema_attr(&self, schema_attr: &'ctx ast::SchemaAttr) -> Self::Result {
        check_backtrack_stop!(self);
        self.local_vars.borrow_mut().clear();
        let name = schema_attr.name.node.as_str();
        self.target_vars.borrow_mut().push(name.to_string());
        for decorator in &schema_attr.decorators {
            self.walk_decorator_with_name(&decorator.node, Some(name), false)
                .expect(kcl_error::COMPILE_ERROR_MSG);
        }
        let config_value = self
            .get_variable(value::SCHEMA_CONFIG_NAME)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let schema_value = self
            .get_variable(value::SCHEMA_SELF_NAME)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let string_ptr_value = self.native_global_string(name, "").into();
        let type_str_ptr_value = self
            .native_global_string(&schema_attr.ty.node.to_string(), "")
            .into();
        self.build_void_call(
            &ApiFunc::kclvm_config_attr_map.name(),
            &[schema_value, string_ptr_value, type_str_ptr_value],
        );
        let has_key = self
            .build_call(
                &ApiFunc::kclvm_dict_has_value.name(),
                &[config_value, string_ptr_value],
            )
            .into_int_value();
        let has_key =
            self.builder
                .build_int_compare(IntPredicate::NE, has_key, self.native_i8_zero(), "");
        let then_block = self.append_block("");
        let else_block = self.append_block("");
        let end_block = self.append_block("");
        self.builder
            .build_conditional_branch(has_key, then_block, else_block);
        self.builder.position_at_end(then_block);
        let config_attr_value = self.build_call(
            &ApiFunc::kclvm_dict_get_entry.name(),
            &[
                self.current_runtime_ctx_ptr(),
                config_value,
                string_ptr_value,
            ],
        );
        // If the attribute operator is not `=`, eval the schema attribute value.
        // if is_not_override:
        let is_override_attr = self
            .build_call(
                &ApiFunc::kclvm_dict_is_override_attr.name(),
                &[config_value, string_ptr_value],
            )
            .into_int_value();
        let is_not_override_attr = self.builder.build_int_compare(
            IntPredicate::EQ,
            is_override_attr,
            self.native_i8_zero(),
            "",
        );
        let is_not_override_then_block = self.append_block("");
        let is_not_override_else_block = self.append_block("");
        self.builder.build_conditional_branch(
            is_not_override_attr,
            is_not_override_then_block,
            is_not_override_else_block,
        );
        self.builder.position_at_end(is_not_override_then_block);
        let value = match &schema_attr.value {
            Some(value) => self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG),
            None => self.undefined_value(),
        };
        if let Some(op) = &schema_attr.op {
            match op {
                // Union
                ast::AugOp::BitOr => {
                    let org_value = self.build_call(
                        &ApiFunc::kclvm_dict_get_value.name(),
                        &[
                            self.current_runtime_ctx_ptr(),
                            schema_value,
                            string_ptr_value,
                        ],
                    );
                    let fn_name = ApiFunc::kclvm_value_op_bit_or;
                    let value = self.build_call(
                        &fn_name.name(),
                        &[self.current_runtime_ctx_ptr(), org_value, value],
                    );
                    self.dict_merge(schema_value, name, value, 1, -1);
                }
                // Assign
                _ => self.dict_merge(schema_value, name, value, 1, -1),
            }
        }
        self.br(is_not_override_else_block);
        self.builder.position_at_end(is_not_override_else_block);
        self.value_union(schema_value, config_attr_value);
        let cal_map = self
            .get_variable(value::SCHEMA_CAL_MAP)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let backtrack_cache = self
            .get_variable(value::BACKTRACK_CACHE)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let runtime_type = self
            .get_variable(value::SCHEMA_RUNTIME_TYPE)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        self.build_void_call(
            &ApiFunc::kclvm_schema_backtrack_cache.name(),
            &[
                self.current_runtime_ctx_ptr(),
                schema_value,
                backtrack_cache,
                cal_map,
                string_ptr_value,
                runtime_type,
            ],
        );
        // Update backtrack meta
        if self.update_backtrack_meta(name, schema_value) {
            return Ok(schema_value);
        }
        self.br(end_block);
        self.builder.position_at_end(else_block);
        // Lazy eval for the schema attribute.
        let value = match &schema_attr.value {
            Some(value) => self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG),
            None => self.undefined_value(),
        };
        if let Some(op) = &schema_attr.op {
            match op {
                // Union
                ast::AugOp::BitOr => {
                    let org_value = self.build_call(
                        &ApiFunc::kclvm_dict_get_value.name(),
                        &[
                            self.current_runtime_ctx_ptr(),
                            schema_value,
                            string_ptr_value,
                        ],
                    );
                    let fn_name = ApiFunc::kclvm_value_op_bit_or;
                    let value = self.build_call(
                        &fn_name.name(),
                        &[self.current_runtime_ctx_ptr(), org_value, value],
                    );
                    self.dict_merge(schema_value, name, value, 1, -1);
                }
                // Assign
                _ => self.dict_merge(schema_value, name, value, 1, -1),
            }
        }
        self.br(end_block);
        self.builder.position_at_end(end_block);
        Ok(schema_value)
    }

    fn walk_if_expr(&self, if_expr: &'ctx ast::IfExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let cond = self
            .walk_expr(&if_expr.cond)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let then_block = self.append_block("");
        let else_block = self.append_block("");
        let end_block = self.append_block("");
        let is_truth = self.value_is_truthy(cond);
        let tpe = self.value_ptr_type();
        self.cond_br(is_truth, then_block, else_block);
        self.builder.position_at_end(then_block);
        let then_value = self
            .walk_expr(&if_expr.body)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let then_block = self.append_block("");
        self.br(then_block);
        self.builder.position_at_end(then_block);
        let ptr = self.builder.build_alloca(tpe, "");
        self.builder.build_store(ptr, then_value);
        let then_value = self.builder.build_load(ptr, "");
        self.br(end_block);
        self.builder.position_at_end(else_block);
        let else_value = self
            .walk_expr(&if_expr.orelse)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let else_block = self.append_block("");
        self.br(else_block);
        self.builder.position_at_end(else_block);
        let ptr = self.alloca(tpe, "", None).into_pointer_value();
        self.builder.build_store(ptr, else_value);
        let else_value = self.builder.build_load(ptr, "");
        self.br(end_block);
        self.builder.position_at_end(end_block);
        let phi = self.builder.build_phi(tpe, "");
        phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
        Ok(phi.as_basic_value())
    }

    fn walk_unary_expr(&self, unary_expr: &'ctx ast::UnaryExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let value = self
            .walk_expr(&unary_expr.operand)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let fn_name = match unary_expr.op {
            ast::UnaryOp::UAdd => ApiFunc::kclvm_value_unary_plus,
            ast::UnaryOp::USub => ApiFunc::kclvm_value_unary_minus,
            ast::UnaryOp::Invert => ApiFunc::kclvm_value_unary_not,
            ast::UnaryOp::Not => ApiFunc::kclvm_value_unary_l_not,
        };
        Ok(self.build_call(&fn_name.name(), &[self.current_runtime_ctx_ptr(), value]))
    }

    fn walk_binary_expr(&self, binary_expr: &'ctx ast::BinaryExpr) -> Self::Result {
        check_backtrack_stop!(self);
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
            let jump_if_false = matches!(binary_expr.op, ast::BinOp::And);
            let start_block = self.append_block("");
            let value_block = self.append_block("");
            let end_block = self.append_block("");
            let left_value = self
                .walk_expr(&binary_expr.left)
                .expect(kcl_error::COMPILE_ERROR_MSG);
            self.br(start_block);
            self.builder.position_at_end(start_block);
            let is_truth = self.value_is_truthy(left_value);
            let tpe = self.value_ptr_type();
            if jump_if_false {
                // Jump if false on logic and
                self.cond_br(is_truth, value_block, end_block);
            } else {
                // Jump if true on logic or
                self.cond_br(is_truth, end_block, value_block);
            };
            self.builder.position_at_end(value_block);
            let right_value = self
                .walk_expr(&binary_expr.right)
                .expect(kcl_error::COMPILE_ERROR_MSG);
            let value_block = self.append_block("");
            self.br(value_block);
            self.builder.position_at_end(value_block);
            let ptr = self.builder.build_alloca(tpe, "");
            self.builder.build_store(ptr, right_value);
            let right_value = self.builder.build_load(ptr, "");
            self.br(end_block);
            self.builder.position_at_end(end_block);
            let phi = self.builder.build_phi(tpe, "");
            phi.add_incoming(&[(&left_value, start_block), (&right_value, value_block)]);
            Ok(phi.as_basic_value())
        }
    }

    fn walk_selector_expr(&self, selector_expr: &'ctx ast::SelectorExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let mut value = self
            .walk_expr(&selector_expr.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let string_ptr_value = self
            .native_global_string(selector_expr.attr.node.names[0].node.as_str(), "")
            .into();
        let fn_name = if selector_expr.has_question {
            &ApiFunc::kclvm_value_load_attr_option
        } else {
            &ApiFunc::kclvm_value_load_attr
        };
        value = self.build_call(
            &fn_name.name(),
            &[self.current_runtime_ctx_ptr(), value, string_ptr_value],
        );
        for name in &selector_expr.attr.node.names[1..] {
            let string_ptr_value = self.native_global_string(&name.node, "").into();
            value = self.build_call(
                &ApiFunc::kclvm_value_load_attr.name(),
                &[self.current_runtime_ctx_ptr(), value, string_ptr_value],
            );
        }
        Ok(value)
    }

    fn walk_call_expr(&self, call_expr: &'ctx ast::CallExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let func = self
            .walk_expr(&call_expr.func)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        // args
        let list_value = self.list_value();
        for arg in &call_expr.args {
            let value = self.walk_expr(arg).expect(kcl_error::COMPILE_ERROR_MSG);
            self.list_append(list_value, value);
        }
        let dict_value = self.dict_value();
        // kwargs
        for keyword in &call_expr.keywords {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert(dict_value, name.node.as_str(), value, 0, -1);
        }
        let pkgpath = self.native_global_string_value(&self.current_pkgpath());
        let is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
        Ok(self.build_call(
            &ApiFunc::kclvm_value_function_invoke.name(),
            &[
                func,
                self.current_runtime_ctx_ptr(),
                list_value,
                dict_value,
                pkgpath,
                self.bool_value(is_in_schema),
            ],
        ))
    }

    fn walk_subscript(&self, subscript: &'ctx ast::Subscript) -> Self::Result {
        check_backtrack_stop!(self);
        let mut value = self
            .walk_expr(&subscript.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        if let Some(index) = &subscript.index {
            // index
            let index = self.walk_expr(index).expect(kcl_error::COMPILE_ERROR_MSG);
            let fn_name = if subscript.has_question {
                &ApiFunc::kclvm_value_subscr_option
            } else {
                &ApiFunc::kclvm_value_subscr
            };
            value = self.build_call(
                &fn_name.name(),
                &[self.current_runtime_ctx_ptr(), value, index],
            );
        } else {
            let lower = {
                if let Some(lower) = &subscript.lower {
                    self.walk_expr(lower).expect(kcl_error::COMPILE_ERROR_MSG)
                } else {
                    self.none_value()
                }
            };
            let upper = {
                if let Some(upper) = &subscript.upper {
                    self.walk_expr(upper).expect(kcl_error::COMPILE_ERROR_MSG)
                } else {
                    self.none_value()
                }
            };
            let step = {
                if let Some(step) = &subscript.step {
                    self.walk_expr(step).expect(kcl_error::COMPILE_ERROR_MSG)
                } else {
                    self.none_value()
                }
            };
            let fn_name = if subscript.has_question {
                &ApiFunc::kclvm_value_slice_option
            } else {
                &ApiFunc::kclvm_value_slice
            };
            value = self.build_call(
                &fn_name.name(),
                &[self.current_runtime_ctx_ptr(), value, lower, upper, step],
            );
        }
        Ok(value)
    }

    fn walk_paren_expr(&self, paren_expr: &'ctx ast::ParenExpr) -> Self::Result {
        check_backtrack_stop!(self);
        self.walk_expr(&paren_expr.expr)
    }

    fn walk_list_expr(&self, list_expr: &'ctx ast::ListExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let list_value = self.list_value();
        for item in &list_expr.elts {
            let value = self.walk_expr(item).expect(kcl_error::COMPILE_ERROR_MSG);
            let fn_name = match &item.node {
                ast::Expr::Starred(_) | ast::Expr::ListIfItem(_) => {
                    ApiFunc::kclvm_list_append_unpack
                }
                _ => ApiFunc::kclvm_list_append,
            };
            self.build_void_call(&fn_name.name(), &[list_value, value]);
        }
        Ok(list_value)
    }

    fn walk_list_if_item_expr(&self, list_if_item_expr: &'ctx ast::ListIfItemExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let cond = self
            .walk_expr(&list_if_item_expr.if_cond)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let then_block = self.append_block("");
        let else_block = self.append_block("");
        let end_block = self.append_block("");
        let is_truth = self.value_is_truthy(cond);
        let tpe = self.value_ptr_type();
        self.cond_br(is_truth, then_block, else_block);
        self.builder.position_at_end(then_block);
        let then_value = self.list_value();
        for expr in &list_if_item_expr.exprs {
            let value = self.walk_expr(expr).expect(kcl_error::COMPILE_ERROR_MSG);
            match &expr.node {
                ast::Expr::Starred(_) | ast::Expr::ListIfItem(_) => {
                    self.list_append_unpack(then_value, value)
                }
                _ => self.list_append(then_value, value),
            };
        }
        let then_block = self.append_block("");
        self.br(then_block);
        self.builder.position_at_end(then_block);
        let ptr = self.builder.build_alloca(tpe, "");
        self.builder.build_store(ptr, then_value);
        let then_value = self.builder.build_load(ptr, "");
        self.br(end_block);
        self.builder.position_at_end(else_block);
        let else_value = if let Some(orelse) = &list_if_item_expr.orelse {
            self.walk_expr(orelse).expect(kcl_error::COMPILE_ERROR_MSG)
        } else {
            self.none_value()
        };
        let else_block = self.append_block("");
        self.br(else_block);
        self.builder.position_at_end(else_block);
        let ptr = self.builder.build_alloca(tpe, "");
        self.builder.build_store(ptr, else_value);
        let else_value = self.builder.build_load(ptr, "");
        self.br(end_block);
        self.builder.position_at_end(end_block);
        let phi = self.builder.build_phi(tpe, "");
        phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
        Ok(phi.as_basic_value())
    }

    fn walk_starred_expr(&self, starred_expr: &'ctx ast::StarredExpr) -> Self::Result {
        check_backtrack_stop!(self);
        self.walk_expr(&starred_expr.value)
    }

    fn walk_list_comp(&self, list_comp: &'ctx ast::ListComp) -> Self::Result {
        check_backtrack_stop!(self);
        let collection_value = self.list_value();
        self.enter_scope();
        self.walk_generator(
            &list_comp.generators,
            &list_comp.elt,
            None,
            None,
            0,
            collection_value,
            ast::CompType::List,
        );
        self.leave_scope();
        let tpe = self.value_ptr_type();
        let ptr = self.builder.build_alloca(tpe, "");
        self.builder.build_store(ptr, collection_value);
        let value = self.builder.build_load(ptr, "");
        Ok(value)
    }

    fn walk_dict_comp(&self, dict_comp: &'ctx ast::DictComp) -> Self::Result {
        check_backtrack_stop!(self);
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
            collection_value,
            ast::CompType::Dict,
        );
        self.leave_scope();
        Ok(collection_value)
    }

    fn walk_config_if_entry_expr(
        &self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) -> Self::Result {
        check_backtrack_stop!(self);
        let cond = self
            .walk_expr(&config_if_entry_expr.if_cond)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let then_block = self.append_block("");
        let else_block = self.append_block("");
        let end_block = self.append_block("");
        let is_truth = self.value_is_truthy(cond);
        let tpe = self.value_ptr_type();
        self.cond_br(is_truth, then_block, else_block);
        self.builder.position_at_end(then_block);
        let then_value = self.walk_config_entries(&config_if_entry_expr.items)?;
        let then_block = self.append_block("");
        self.br(then_block);
        self.builder.position_at_end(then_block);
        let ptr = self.builder.build_alloca(tpe, "");
        self.builder.build_store(ptr, then_value);
        let then_value = self.builder.build_load(ptr, "");
        self.br(end_block);
        self.builder.position_at_end(else_block);
        let else_value = if let Some(orelse) = &config_if_entry_expr.orelse {
            self.walk_expr(orelse).expect(kcl_error::COMPILE_ERROR_MSG)
        } else {
            self.none_value()
        };
        let else_block = self.append_block("");
        self.br(else_block);
        self.builder.position_at_end(else_block);
        let ptr = self.builder.build_alloca(tpe, "");
        self.builder.build_store(ptr, else_value);
        let else_value = self.builder.build_load(ptr, "");
        self.br(end_block);
        self.builder.position_at_end(end_block);
        let phi = self.builder.build_phi(tpe, "");
        phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
        Ok(phi.as_basic_value())
    }

    fn walk_comp_clause(&self, _comp_clause: &'ctx ast::CompClause) -> Self::Result {
        // Nothing to do on this AST node
        self.ok_result()
    }

    fn walk_schema_expr(&self, schema_expr: &'ctx ast::SchemaExpr) -> Self::Result {
        check_backtrack_stop!(self);
        // Check the required attributes only when the values of all attributes
        // in the final schema are solved.
        let is_in_schema = self.is_in_schema() || self.is_in_schema_expr();
        {
            self.schema_expr_stack.borrow_mut().push(());
        }
        let config_value = self
            .walk_expr(&schema_expr.config)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let schema_type = self
            .walk_identifier_with_ctx(&schema_expr.name.node, &schema_expr.name.node.ctx, None)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let config_expr = match &schema_expr.config.node {
            ast::Expr::Config(config_expr) => config_expr,
            _ => panic!("invalid schema config expr"),
        };
        let config_meta = self.get_schema_config_meta(Some(&schema_expr.name), config_expr);
        let list_value = self.list_value();
        for arg in &schema_expr.args {
            let value = self.walk_expr(arg).expect(kcl_error::COMPILE_ERROR_MSG);
            self.list_append(list_value, value);
        }
        let dict_value = self.dict_value();
        for keyword in &schema_expr.kwargs {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert(dict_value, name.node.as_str(), value, 0, -1);
        }
        let pkgpath = self.native_global_string_value(&self.current_pkgpath());
        let schema = self.build_call(
            &ApiFunc::kclvm_schema_value_new.name(),
            &[
                self.current_runtime_ctx_ptr(),
                list_value,
                dict_value,
                schema_type,
                config_value,
                config_meta,
                pkgpath,
            ],
        );
        if !is_in_schema {
            self.build_void_call(
                &ApiFunc::kclvm_schema_optional_check.name(),
                &[self.current_runtime_ctx_ptr(), schema],
            );
        }
        utils::update_ctx_filename(self, &schema_expr.config);
        {
            self.schema_expr_stack.borrow_mut().pop();
        }
        Ok(schema)
    }

    fn walk_config_expr(&self, config_expr: &'ctx ast::ConfigExpr) -> Self::Result {
        check_backtrack_stop!(self);
        self.walk_config_entries(&config_expr.items)
    }

    fn walk_check_expr(&self, check_expr: &'ctx ast::CheckExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let start_block = self.append_block("");
        let end_block = self.append_block("");
        if let Some(if_cond) = &check_expr.if_cond {
            let if_value = self.walk_expr(if_cond).expect(kcl_error::COMPILE_ERROR_MSG);
            let is_truth = self.value_is_truthy(if_value);
            self.cond_br(is_truth, start_block, end_block);
        } else {
            self.br(start_block);
        }
        self.builder.position_at_end(start_block);
        let check_result = self
            .walk_expr(&check_expr.test)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let msg = {
            if let Some(msg) = &check_expr.msg {
                self.walk_expr(msg).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.string_value("")
            }
        };
        let schema_config_meta = self
            .get_variable(value::SCHEMA_CONFIG_META_NAME)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        utils::update_ctx_current_line(self);
        self.build_void_call(
            &ApiFunc::kclvm_schema_assert.name(),
            &[
                self.current_runtime_ctx_ptr(),
                check_result,
                msg,
                schema_config_meta,
            ],
        );
        self.br(end_block);
        self.builder.position_at_end(end_block);
        self.ok_result()
    }

    fn walk_lambda_expr(&self, lambda_expr: &'ctx ast::LambdaExpr) -> Self::Result {
        check_backtrack_stop!(self);
        let pkgpath = &self.current_pkgpath();
        // Higher-order lambda requires capturing the current lambda closure variable
        // as well as the closure of a more external scope.
        let last_closure_map = self.get_current_inner_scope_variable_map();
        let func_before_block = self.append_block("");
        self.br(func_before_block);
        // Use "pkgpath"+"kclvm_lambda" to name 'function' to prevent conflicts between lambdas with the same name in different packages
        let function = self.add_function(&format!(
            "{}.{}",
            pkgpath_without_prefix!(pkgpath),
            value::LAMBDA_NAME
        ));
        // Enter the function
        self.push_function(function);
        // Push the current lambda scope level in the lambda stack.
        self.push_lambda(self.scope_level() + 1);
        // Lambda function body
        let block = self.context.append_basic_block(function, ENTRY_NAME);
        self.builder.position_at_end(block);
        let args = function
            .get_nth_param(1)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let kwargs = function
            .get_nth_param(2)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        self.enter_scope();
        let closure_map = self.list_pop_first(args);
        let tpe = self.value_ptr_type();
        let var = self.builder.build_alloca(tpe, value::LAMBDA_CLOSURE);
        self.builder.build_store(var, closure_map);
        self.add_variable(value::LAMBDA_CLOSURE, var);
        if self.is_in_schema() {
            for schema_closure_name in value::SCHEMA_VARIABLE_LIST {
                let string_ptr_value = self.native_global_string(schema_closure_name, "").into();
                let schema_value = self.build_call(
                    &ApiFunc::kclvm_dict_get_value.name(),
                    &[
                        self.current_runtime_ctx_ptr(),
                        closure_map,
                        string_ptr_value,
                    ],
                );
                let value_ptr_type = self.value_ptr_type();
                let var = self
                    .builder
                    .build_alloca(value_ptr_type, schema_closure_name);
                self.builder.build_store(var, schema_value);
                self.add_variable(schema_closure_name, var);
            }
        }
        self.walk_arguments(&lambda_expr.args, args, kwargs);
        let val = self
            .walk_stmts(&lambda_expr.body)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        self.builder.build_return(Some(&val));
        // Exist the function
        self.builder.position_at_end(func_before_block);
        let closure = self.list_value();
        // Use closure map in the last scope to construct current closure map.
        // The default value of the closure map is `{}`.
        self.list_append(closure, last_closure_map);
        let function = self.closure_value(function, closure);
        self.leave_scope();
        self.pop_function();
        self.pop_lambda();
        Ok(function)
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
        check_backtrack_stop!(self);
        let mut left_value = self
            .walk_expr(&compare.left)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        if compare.comparators.len() > 1 {
            let tpe = self.value_ptr_type();
            let mut next_block = self.append_block("");
            let end_block = self.append_block("");
            self.br(next_block);
            self.builder.position_at_end(next_block);
            let mut values_blocks: Vec<(BasicValueEnum, BasicBlock)> = vec![];
            for (i, op) in compare.ops.iter().enumerate() {
                let has_next = i < (compare.ops.len() - 1);
                let right_value = self
                    .walk_expr(&compare.comparators[i])
                    .expect(kcl_error::COMPILE_ERROR_MSG);
                let fn_name = match op {
                    ast::CmpOp::Eq => ApiFunc::kclvm_value_cmp_equal_to,
                    ast::CmpOp::NotEq => ApiFunc::kclvm_value_cmp_not_equal_to,
                    ast::CmpOp::Gt => ApiFunc::kclvm_value_cmp_greater_than,
                    ast::CmpOp::GtE => ApiFunc::kclvm_value_cmp_greater_than_or_equal,
                    ast::CmpOp::Lt => ApiFunc::kclvm_value_cmp_less_than,
                    ast::CmpOp::LtE => ApiFunc::kclvm_value_cmp_less_than_or_equal,
                    ast::CmpOp::Is => ApiFunc::kclvm_value_is,
                    ast::CmpOp::IsNot => ApiFunc::kclvm_value_is_not,
                    ast::CmpOp::Not => ApiFunc::kclvm_value_is_not,
                    ast::CmpOp::NotIn => ApiFunc::kclvm_value_not_in,
                    ast::CmpOp::In => ApiFunc::kclvm_value_in,
                };
                let result_value = self.build_call(
                    &fn_name.name(),
                    &[self.current_runtime_ctx_ptr(), left_value, right_value],
                );
                let is_truth = self.value_is_truthy(result_value);
                left_value = right_value;
                // Get next value using a store/load temp block
                let next_value_block = self.append_block("");
                self.br(next_value_block);
                self.builder.position_at_end(next_value_block);
                let ptr = self.builder.build_alloca(tpe, "");
                self.builder.build_store(ptr, result_value);
                let result_value = self.builder.build_load(ptr, "");
                // Append a value-block pair in the vec
                values_blocks.push((result_value, next_value_block));
                if has_next {
                    next_block = self.append_block("");
                    self.cond_br(is_truth, next_block, end_block);
                    self.builder.position_at_end(next_block);
                } else {
                    self.br(end_block);
                }
            }
            self.builder.position_at_end(end_block);
            let phi = self.builder.build_phi(tpe, "");
            for (value, block) in values_blocks {
                phi.add_incoming(&[(&value, block)]);
            }
            Ok(phi.as_basic_value())
        } else {
            let right_value = self
                .walk_expr(&compare.comparators[0])
                .expect(kcl_error::COMPILE_ERROR_MSG);
            let fn_name = match &compare.ops[0] {
                ast::CmpOp::Eq => ApiFunc::kclvm_value_cmp_equal_to,
                ast::CmpOp::NotEq => ApiFunc::kclvm_value_cmp_not_equal_to,
                ast::CmpOp::Gt => ApiFunc::kclvm_value_cmp_greater_than,
                ast::CmpOp::GtE => ApiFunc::kclvm_value_cmp_greater_than_or_equal,
                ast::CmpOp::Lt => ApiFunc::kclvm_value_cmp_less_than,
                ast::CmpOp::LtE => ApiFunc::kclvm_value_cmp_less_than_or_equal,
                ast::CmpOp::Is => ApiFunc::kclvm_value_is,
                ast::CmpOp::IsNot => ApiFunc::kclvm_value_is_not,
                ast::CmpOp::Not => ApiFunc::kclvm_value_is_not,
                ast::CmpOp::NotIn => ApiFunc::kclvm_value_not_in,
                ast::CmpOp::In => ApiFunc::kclvm_value_in,
            };
            left_value = self.build_call(
                &fn_name.name(),
                &[self.current_runtime_ctx_ptr(), left_value, right_value],
            );
            Ok(left_value)
        }
    }

    fn walk_identifier(&self, identifier: &'ctx ast::Identifier) -> Self::Result {
        check_backtrack_stop!(self);
        self.walk_identifier_with_ctx(identifier, &identifier.ctx, None)
    }

    fn walk_number_lit(&self, number_lit: &'ctx ast::NumberLit) -> Self::Result {
        check_backtrack_stop!(self);
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

    fn walk_string_lit(&self, string_lit: &'ctx ast::StringLit) -> Self::Result {
        check_backtrack_stop!(self);
        let string_ptr_value = self
            .native_global_string(string_lit.value.as_str(), "")
            .into();
        Ok(self.build_call(
            &ApiFunc::kclvm_value_Str.name(),
            &[self.current_runtime_ctx_ptr(), string_ptr_value],
        ))
    }

    fn walk_name_constant_lit(
        &self,
        name_constant_lit: &'ctx ast::NameConstantLit,
    ) -> Self::Result {
        check_backtrack_stop!(self);
        match name_constant_lit.value {
            ast::NameConstant::True => Ok(self.bool_value(true)),
            ast::NameConstant::False => Ok(self.bool_value(false)),
            ast::NameConstant::None => Ok(self.none_value()),
            ast::NameConstant::Undefined => Ok(self.undefined_value()),
        }
    }

    fn walk_joined_string(&self, joined_string: &'ctx ast::JoinedString) -> Self::Result {
        check_backtrack_stop!(self);
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
            result_value = self.build_call(
                &ApiFunc::kclvm_value_op_add.name(),
                &[self.current_runtime_ctx_ptr(), result_value, value],
            );
        }
        Ok(result_value)
    }

    fn walk_formatted_value(&self, formatted_value: &'ctx ast::FormattedValue) -> Self::Result {
        check_backtrack_stop!(self);
        let formatted_expr_value = self
            .walk_expr(&formatted_value.value)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let mut fn_name = ApiFunc::kclvm_value_to_str_value;
        if let Some(spec) = &formatted_value.format_spec {
            fn_name = match spec.to_lowercase().as_str() {
                "#json" => ApiFunc::kclvm_value_to_json_value,
                "#yaml" => ApiFunc::kclvm_value_to_yaml_value,
                _ => panic!("{}", kcl_error::INVALID_STR_INTERPOLATION_SPEC_MSG),
            };
        }
        Ok(self.build_call(
            &fn_name.name(),
            &[self.current_runtime_ctx_ptr(), formatted_expr_value],
        ))
    }

    fn walk_comment(&self, _comment: &'ctx ast::Comment) -> Self::Result {
        // Nothing to do
        self.ok_result()
    }

    fn walk_missing_expr(&self, _missing_expr: &'ctx ast::MissingExpr) -> Self::Result {
        Err(kcl_error::KCLError::new(
            "compile error: missing expression",
        ))
    }

    fn walk_module(&self, module: &'ctx ast::Module) -> Self::Result {
        check_backtrack_stop!(self);
        // Compile all statements of the module except all import statements
        self.walk_stmts_except_import(&module.body)
    }
}

impl<'ctx> LLVMCodeGenContext<'ctx> {
    pub fn walk_stmts_except_import(
        &self,
        stmts: &'ctx [Box<ast::Node<ast::Stmt>>],
    ) -> CompileResult<'ctx> {
        check_backtrack_stop!(self);
        let mut result = self.ok_result();
        for stmt in stmts {
            if !matches!(&stmt.node, ast::Stmt::Import(..)) {
                result = self.walk_stmt(stmt);
            }
        }
        result
    }

    pub fn walk_stmts(&self, stmts: &'ctx [Box<ast::Node<ast::Stmt>>]) -> CompileResult<'ctx> {
        check_backtrack_stop!(self);
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
        right_value: Option<BasicValueEnum<'ctx>>,
    ) -> CompileResult<'ctx> {
        check_backtrack_stop!(self);
        let is_in_schema = self.is_in_schema();
        match identifier_ctx {
            // Store a.b.c = 1
            ast::ExprContext::Store => {
                if identifier.names.len() == 1 {
                    let name = identifier.names[0].node.as_str();
                    let tpe = self.value_ptr_type();
                    // Global variables
                    if self.scope_level() == GLOBAL_LEVEL {
                        self.add_or_update_global_variable(
                            name,
                            right_value.expect(kcl_error::INTERNAL_ERROR_MSG),
                        );
                    // Lambda local variables.
                    } else if self.is_in_lambda() {
                        let value = right_value.expect(kcl_error::INTERNAL_ERROR_MSG);
                        // If variable exists in the scope and update it, if not, add it to the scope.
                        if !self.store_variable_in_current_scope(name, value) {
                            let cur_bb = self.builder.get_insert_block().unwrap();
                            let lambda_func = cur_bb.get_parent().unwrap();
                            let entry_bb = lambda_func.get_first_basic_block().unwrap();
                            match entry_bb.get_first_instruction() {
                                Some(inst) => self.builder.position_before(&inst),
                                None => self.builder.position_at_end(entry_bb),
                            };
                            let var = self.builder.build_alloca(tpe, name);
                            let undefined_val = self.undefined_value();
                            self.builder.build_store(var, undefined_val);
                            self.add_variable(name, var);
                            self.builder.position_at_end(cur_bb);
                            self.store_variable(name, value);
                        }
                    } else {
                        let is_local_var = self.is_local_var(name);
                        let value = right_value.expect(kcl_error::INTERNAL_ERROR_MSG);
                        // Store schema attribute
                        if is_in_schema {
                            let schema_value = self
                                .get_variable(value::SCHEMA_SELF_NAME)
                                .expect(kcl_error::INTERNAL_ERROR_MSG);
                            // Schema config
                            let config_value = self
                                .get_variable(value::SCHEMA_CONFIG_NAME)
                                .expect(kcl_error::INTERNAL_ERROR_MSG);
                            // If is in the backtrack, return the schema value.
                            if self.update_schema_scope_value(
                                schema_value,
                                config_value,
                                name,
                                Some(value),
                            ) {
                                return Ok(schema_value);
                            }
                        }
                        // Store loop variable
                        if is_local_var || !is_in_schema {
                            let var = self.builder.build_alloca(tpe, name);
                            self.builder.build_store(var, value);
                            self.add_variable(name, var);
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
                                let attr = self.native_global_string(attr, "").into();
                                value = self.build_call(
                                    &ApiFunc::kclvm_value_load_attr.name(),
                                    &[self.current_runtime_ctx_ptr(), value, attr],
                                );
                            }
                            ast::ExprContext::Store => {
                                let attr = self.native_global_string(attr, "").into();
                                self.build_void_call(
                                    &ApiFunc::kclvm_dict_set_value.name(),
                                    &[
                                        self.current_runtime_ctx_ptr(),
                                        value,
                                        attr,
                                        right_value.expect(kcl_error::INTERNAL_ERROR_MSG),
                                    ],
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
                                    let schema_value = self
                                        .get_variable(value::SCHEMA_SELF_NAME)
                                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                                    let config_value = self
                                        .get_variable(value::SCHEMA_CONFIG_NAME)
                                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                                    if self.update_schema_scope_value(
                                        schema_value,
                                        config_value,
                                        name,
                                        None,
                                    ) {
                                        return Ok(schema_value);
                                    }
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
    ) -> CompileResult<'ctx> {
        check_backtrack_stop!(self);
        let list_value = self.list_value();
        let dict_value = self.dict_value();
        let schema_config_meta = self
            .get_variable(value::SCHEMA_CONFIG_META_NAME)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let schema_config_value = self
            .get_variable(value::SCHEMA_CONFIG_NAME)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        for arg in &decorator.args {
            let value = self.walk_expr(arg).expect(kcl_error::COMPILE_ERROR_MSG);
            self.list_append(list_value, value);
        }
        for keyword in &decorator.keywords {
            let name = &keyword.node.arg.node.names[0];
            let value = if let Some(value) = &keyword.node.value {
                self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.none_value()
            };
            self.dict_insert(dict_value, name.node.as_str(), value, 0, -1);
        }
        let name = match &decorator.func.node {
            ast::Expr::Identifier(ident) if ident.names.len() == 1 => ident.names[0].clone(),
            _ => panic!("invalid decorator name, expect single identifier"),
        };
        let attr_name = if let Some(v) = attr_name { v } else { "" };
        let attr_name = self.native_global_string_value(attr_name);
        Ok(self.build_call(
            &ApiFunc::kclvm_value_Decorator.name(),
            &[
                self.current_runtime_ctx_ptr(),
                self.native_global_string_value(name.node.as_str()),
                list_value,
                dict_value,
                schema_config_meta,
                attr_name,
                schema_config_value,
                self.bool_value(is_schema_target),
            ],
        ))
    }

    pub fn walk_arguments(
        &self,
        arguments: &'ctx Option<ast::NodeRef<ast::Arguments>>,
        args: BasicValueEnum<'ctx>,
        kwargs: BasicValueEnum<'ctx>,
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
                self.walk_expr(value).expect(kcl_error::COMPILE_ERROR_MSG)
            } else {
                self.none_value()
            };
            // Arguments are immutable, so we place them in different scopes.
            self.store_argument_in_current_scope(&arg_name.get_name());
            self.walk_identifier_with_ctx(arg_name, &ast::ExprContext::Store, Some(arg_value))
                .expect(kcl_error::COMPILE_ERROR_MSG);
        }
        // for loop in 0..argument_len in LLVM begin
        let argument_len = self.build_call(&ApiFunc::kclvm_list_len.name(), &[args]);
        let end_block = self.append_block("");
        for (i, arg_name) in arg_names.iter().enumerate() {
            // Positional arguments
            let is_in_range = self.builder.build_int_compare(
                IntPredicate::ULT,
                self.native_int_value(i as i32).into_int_value(),
                argument_len.into_int_value(),
                "",
            );
            let next_block = self.append_block("");
            self.builder
                .build_conditional_branch(is_in_range, next_block, end_block);
            self.builder.position_at_end(next_block);
            let arg_value = self.build_call(
                &ApiFunc::kclvm_list_get_option.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    args,
                    self.native_int_value(i as i32),
                ],
            );
            self.store_variable(&arg_name.names[0].node, arg_value);
        }
        // for loop in 0..argument_len in LLVM end
        self.br(end_block);
        self.builder.position_at_end(end_block);
        // Keyword arguments
        for arg_name in arg_names.iter() {
            let name = &arg_name.names[0].node;
            let string_ptr_value = self.native_global_string(name.as_str(), "").into();
            let has_key = self
                .build_call(
                    &ApiFunc::kclvm_dict_has_value.name(),
                    &[kwargs, string_ptr_value],
                )
                .into_int_value();
            let has_key = self.builder.build_int_compare(
                IntPredicate::NE,
                has_key,
                self.native_i8_zero(),
                "",
            );
            let then_block = self.append_block("");
            let else_block = self.append_block("");
            self.builder
                .build_conditional_branch(has_key, then_block, else_block);
            self.builder.position_at_end(then_block);
            let arg = self.build_call(
                &ApiFunc::kclvm_dict_get_value.name(),
                &[self.current_runtime_ctx_ptr(), kwargs, string_ptr_value],
            );
            // Find argument name in the scope
            self.store_variable(&arg_name.names[0].node, arg);
            self.br(else_block);
            self.builder.position_at_end(else_block);
        }
    }

    pub fn walk_generator(
        &self,
        generators: &'ctx [Box<ast::Node<ast::CompClause>>],
        elt: &'ctx ast::Node<ast::Expr>,
        val: Option<&'ctx ast::Node<ast::Expr>>,
        op: Option<&'ctx ast::ConfigEntryOperation>,
        gen_index: usize,
        collection_value: BasicValueEnum<'ctx>,
        comp_type: ast::CompType,
    ) {
        let start_block = self.append_block("");
        let next_value_block = self.append_block("");
        let continue_block = self.append_block("");
        let end_for_block = self.append_block("");
        let generator = &generators[gen_index];
        let iter_host_value = self
            .walk_expr(&generator.node.iter)
            .expect(kcl_error::COMPILE_ERROR_MSG);
        let iter_value = self.build_call(&ApiFunc::kclvm_value_iter.name(), &[iter_host_value]);
        self.br(start_block);
        self.builder.position_at_end(start_block);
        let is_end = self
            .build_call(&ApiFunc::kclvm_iterator_is_end.name(), &[iter_value])
            .into_int_value();
        let is_end =
            self.builder
                .build_int_compare(IntPredicate::NE, is_end, self.native_i8_zero(), "");
        self.builder
            .build_conditional_branch(is_end, end_for_block, next_value_block);
        self.builder.position_at_end(next_value_block);
        let next_value = self.build_call(
            &ApiFunc::kclvm_iterator_next_value.name(),
            &[iter_value, iter_host_value],
        );
        let targets = &generator.node.targets;
        {
            let mut local_vars = self.local_vars.borrow_mut();
            for v in targets {
                let name = &v.node.names[0].node;
                local_vars.insert(name.clone());
            }
        }
        if targets.len() == 1 {
            // Store the target
            self.walk_identifier_with_ctx(
                &targets.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                &ast::ExprContext::Store,
                Some(next_value),
            )
            .expect(kcl_error::COMPILE_ERROR_MSG);
        } else if targets.len() == 2 {
            let key = self.build_call(&ApiFunc::kclvm_iterator_cur_key.name(), &[iter_value]);
            let value = self.build_call(&ApiFunc::kclvm_iterator_cur_value.name(), &[iter_value]);
            // Store the target
            self.walk_identifier_with_ctx(
                &targets.first().expect(kcl_error::INTERNAL_ERROR_MSG).node,
                &ast::ExprContext::Store,
                Some(key),
            )
            .expect(kcl_error::COMPILE_ERROR_MSG);
            self.walk_identifier_with_ctx(
                &targets.get(1).expect(kcl_error::INTERNAL_ERROR_MSG).node,
                &ast::ExprContext::Store,
                Some(value),
            )
            .expect(kcl_error::COMPILE_ERROR_MSG);
        } else {
            panic!(
                "the number of loop variables is {}, which can only be 1 or 2",
                generator.node.targets.len()
            )
        }
        for if_expr in &generator.node.ifs {
            let is_truth = self.walk_expr(if_expr).expect(kcl_error::COMPILE_ERROR_MSG);
            let is_truth = self.value_is_truthy(is_truth);
            self.cond_br(is_truth, continue_block, start_block);
        }
        if generator.node.ifs.is_empty() {
            self.br(continue_block);
        }
        self.builder.position_at_end(continue_block);
        let next_gen_index = gen_index + 1;
        if next_gen_index >= generators.len() {
            match comp_type {
                ast::CompType::List => {
                    let item = self.walk_expr(elt).expect(kcl_error::COMPILE_ERROR_MSG);
                    self.list_append(collection_value, item);
                }
                ast::CompType::Dict => {
                    let value = self
                        .walk_expr(val.expect(kcl_error::INTERNAL_ERROR_MSG))
                        .expect(kcl_error::COMPILE_ERROR_MSG);
                    let key = self.walk_expr(elt).expect(kcl_error::COMPILE_ERROR_MSG);
                    let op = op.expect(kcl_error::INTERNAL_ERROR_MSG);
                    self.dict_insert_with_key_value(
                        collection_value,
                        key,
                        self.value_deep_copy(value),
                        op.value(),
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
        self.br(start_block);
        self.builder.position_at_end(end_for_block);
        self.build_void_call(&ApiFunc::kclvm_iterator_delete.name(), &[iter_value]);
        {
            let mut local_vars = self.local_vars.borrow_mut();
            for v in targets {
                let name = &v.node.names[0].node;
                local_vars.remove(name);
            }
        }
    }

    pub(crate) fn walk_config_entries(
        &self,
        items: &'ctx [NodeRef<ConfigEntry>],
    ) -> CompileResult<'ctx> {
        let config_value = self.dict_value();
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
                    Some(name) if !self.local_vars.borrow().contains(name) => {
                        self.string_value(name)
                    }
                    _ => self.walk_expr(key)?,
                };
                self.dict_insert_with_key_value(
                    config_value,
                    key,
                    value,
                    item.node.operation.value(),
                    insert_index as i32,
                );
                if let Some(name) = &optional_name {
                    let value =
                        self.dict_get(config_value, self.native_global_string(name, "").into());
                    self.add_or_update_local_variable(name, value);
                }
            } else {
                // If the key does not exist, execute the logic of unpacking expression `**expr` here.
                self.build_void_call(
                    &ApiFunc::kclvm_dict_insert_unpack.name(),
                    &[self.current_runtime_ctx_ptr(), config_value, value],
                );
            }
        }
        self.leave_scope();
        Ok(config_value)
    }
}
