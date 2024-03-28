// Copyright The KCL Authors. All rights reserved.

use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::{AddressSpace, IntPredicate};
use kclvm_ast::ast;
use kclvm_runtime::ApiFunc;
use kclvm_sema::pkgpath_without_prefix;
use std::collections::HashMap;
use std::str;

use super::context::LLVMCodeGenContext;
use crate::codegen::traits::{
    BuilderMethods, DerivedTypeMethods, DerivedValueCalculationMethods, ProgramCodeGen,
    ValueMethods,
};
use crate::codegen::{error as kcl_error, INNER_LEVEL};
use crate::value;

impl<'ctx> LLVMCodeGenContext<'ctx> {
    /// Emit all left identifiers because all the attribute can be forward referenced.
    pub(crate) fn emit_left_identifiers(
        &self,
        body: &'ctx [Box<ast::Node<ast::Stmt>>],
        index_signature: &'ctx Option<ast::NodeRef<ast::SchemaIndexSignature>>,
        cal_map: BasicValueEnum<'ctx>,
        runtime_type: &str,
        is_in_if: bool,
        place_holder_map: &mut HashMap<String, Vec<FunctionValue<'ctx>>>,
        body_map: &mut HashMap<String, Vec<&'ctx ast::Node<ast::Stmt>>>,
        in_if_names: &mut Vec<String>,
    ) {
        let schema_value = self
            .get_variable(value::SCHEMA_SELF_NAME)
            .expect(kcl_error::INTERNAL_ERROR_MSG);
        let value = self.undefined_value();
        let add_stmt =
            |name: &str,
             stmt: &'ctx ast::Node<ast::Stmt>,
             place_holder_map: &mut HashMap<String, Vec<FunctionValue<'ctx>>>,
             body_map: &mut HashMap<String, Vec<&'ctx ast::Node<ast::Stmt>>>| {
                let function = self.add_function(&format!(
                    "{}.{}.{}",
                    value::SCHEMA_ATTR_NAME,
                    pkgpath_without_prefix!(runtime_type),
                    name
                ));
                let lambda_fn_ptr = self.builder.build_bitcast(
                    function.as_global_value().as_pointer_value(),
                    self.context.i64_type().ptr_type(AddressSpace::default()),
                    "",
                );
                if !place_holder_map.contains_key(name) {
                    place_holder_map.insert(name.to_string(), vec![]);
                }
                let name_vec = place_holder_map
                    .get_mut(name)
                    .expect(kcl_error::INTERNAL_ERROR_MSG);
                name_vec.push(function);
                self.default_collection_insert_int_pointer(cal_map, name, lambda_fn_ptr);
                self.default_collection_insert_value(
                    cal_map,
                    &format!("{}_{}", name, kclvm_runtime::CAL_MAP_RUNTIME_TYPE),
                    self.string_value(runtime_type),
                );
                self.default_collection_insert_value(
                    cal_map,
                    &format!("{}_{}", name, kclvm_runtime::CAL_MAP_META_LINE),
                    self.int_value(stmt.line as i64),
                );
                if !body_map.contains_key(name) {
                    body_map.insert(name.to_string(), vec![]);
                }
                let body_vec = body_map.get_mut(name).expect(kcl_error::INTERNAL_ERROR_MSG);
                body_vec.push(stmt);
            };
        if let Some(index_signature) = index_signature {
            self.default_collection_insert_value(
                cal_map,
                kclvm_runtime::CAL_MAP_INDEX_SIGNATURE,
                self.int_value(index_signature.line as i64),
            );
            place_holder_map.insert(kclvm_runtime::CAL_MAP_INDEX_SIGNATURE.to_string(), vec![]);
        }
        for stmt in body {
            match &stmt.node {
                ast::Stmt::Unification(unification_stmt) => {
                    let name = &unification_stmt.target.node.names[0].node;
                    self.dict_merge(schema_value, name, value, 0, -1);
                    if is_in_if {
                        in_if_names.push(name.to_string());
                    } else {
                        add_stmt(name, stmt, place_holder_map, body_map);
                    }
                }
                ast::Stmt::Assign(assign_stmt) => {
                    for target in &assign_stmt.targets {
                        let name = &target.node.names[0].node;
                        self.dict_merge(schema_value, name, value, 0, -1);
                        if is_in_if {
                            in_if_names.push(name.to_string());
                        } else {
                            add_stmt(name, stmt, place_holder_map, body_map);
                        }
                    }
                }
                ast::Stmt::AugAssign(aug_assign_stmt) => {
                    let target = &aug_assign_stmt.target;
                    let name = &target.node.names[0].node;
                    self.dict_merge(schema_value, name, value, 0, -1);
                    if is_in_if {
                        in_if_names.push(name.to_string());
                    } else {
                        add_stmt(name, stmt, place_holder_map, body_map);
                    }
                }
                ast::Stmt::If(if_stmt) => {
                    let mut names: Vec<String> = vec![];
                    self.emit_left_identifiers(
                        &if_stmt.body,
                        &None,
                        cal_map,
                        runtime_type,
                        true,
                        place_holder_map,
                        body_map,
                        &mut names,
                    );
                    if is_in_if {
                        for name in &names {
                            in_if_names.push(name.to_string());
                        }
                    } else {
                        for name in &names {
                            add_stmt(name, stmt, place_holder_map, body_map);
                        }
                        names.clear();
                    }
                    self.emit_left_identifiers(
                        &if_stmt.orelse,
                        &None,
                        cal_map,
                        runtime_type,
                        true,
                        place_holder_map,
                        body_map,
                        &mut names,
                    );
                    if is_in_if {
                        for name in &names {
                            in_if_names.push(name.to_string());
                        }
                    } else {
                        for name in &names {
                            add_stmt(name, stmt, place_holder_map, body_map);
                        }
                        names.clear();
                    }
                }
                ast::Stmt::SchemaAttr(schema_attr) => {
                    let name = schema_attr.name.node.as_str();
                    self.dict_merge(schema_value, name, value, 0, -1);
                    if is_in_if {
                        in_if_names.push(name.to_string());
                    } else {
                        add_stmt(name, stmt, place_holder_map, body_map);
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn get_schema_config_meta(
        &self,
        n: Option<&'ctx ast::Node<ast::Identifier>>,
        t: &'ctx ast::ConfigExpr,
    ) -> BasicValueEnum<'ctx> {
        let config_meta = self.dict_value();
        if let Some(n) = n {
            let value = self.string_value(&n.filename);
            self.dict_insert_override_item(config_meta, kclvm_runtime::CONFIG_META_FILENAME, value);
            let value = self.int_value(n.line as i64);
            self.dict_insert_override_item(config_meta, kclvm_runtime::CONFIG_META_LINE, value);
            let value = self.int_value(n.column as i64);
            self.dict_insert_override_item(config_meta, kclvm_runtime::CONFIG_META_COLUMN, value);
        }
        for item in &t.items {
            if let Some(key) = &item.node.key {
                let name = match &key.node {
                    ast::Expr::Identifier(t) => t.names[0].node.clone(),
                    ast::Expr::NumberLit(t) => match t.value {
                        ast::NumberLitValue::Int(i) => i.to_string(),
                        ast::NumberLitValue::Float(f) => f.to_string(),
                    },
                    ast::Expr::StringLit(t) => t.value.clone(),
                    ast::Expr::NameConstantLit(t) => match t.value {
                        ast::NameConstant::True => {
                            kclvm_runtime::KCL_NAME_CONSTANT_TRUE.to_string()
                        }
                        ast::NameConstant::False => {
                            kclvm_runtime::KCL_NAME_CONSTANT_FALSE.to_string()
                        }
                        ast::NameConstant::None => {
                            kclvm_runtime::KCL_NAME_CONSTANT_NONE.to_string()
                        }
                        ast::NameConstant::Undefined => {
                            kclvm_runtime::KCL_NAME_CONSTANT_UNDEFINED.to_string()
                        }
                    },
                    _ => format!("{:?}", key.node),
                };
                let config_item_meta = self.dict_value();
                let value = self.string_value(&key.filename);
                self.dict_insert_override_item(
                    config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META_FILENAME,
                    value,
                );
                let value = self.int_value(key.line as i64);
                self.dict_insert_override_item(
                    config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META_LINE,
                    value,
                );
                let value = self.int_value(key.column as i64);
                self.dict_insert_override_item(
                    config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META_COLUMN,
                    value,
                );
                let value = match &item.node.value.node {
                    ast::Expr::Config(config_expr) => {
                        self.get_schema_config_meta(None, config_expr)
                    }
                    _ => self.dict_value(),
                };
                self.dict_insert_override_item(
                    config_item_meta,
                    kclvm_runtime::CONFIG_ITEM_META,
                    value,
                );
                self.dict_insert_override_item(config_meta, &name, config_item_meta)
            }
        }
        config_meta
    }

    pub(crate) fn update_schema_scope_value(
        &self,
        schema_value: BasicValueEnum<'ctx>,  // Schema self value
        config_value: BasicValueEnum<'ctx>,  // Schema config value
        name: &str,                          // Schema attribute name
        value: Option<BasicValueEnum<'ctx>>, // Optional right override value
    ) -> bool {
        // Attribute name
        let string_ptr_value = self.native_global_string(name, "").into();
        // i8 has_key
        let has_key = self
            .build_call(
                &ApiFunc::kclvm_dict_has_value.name(),
                &[config_value, string_ptr_value],
            )
            .into_int_value();
        // i1 has_key
        let has_key =
            self.builder
                .build_int_compare(IntPredicate::NE, has_key, self.native_i8_zero(), "");
        let last_block = self.append_block("");
        let then_block = self.append_block("");
        let else_block = self.append_block("");
        self.br(last_block);
        self.builder.position_at_end(last_block);
        let none_value = self.none_value();
        self.builder
            .build_conditional_branch(has_key, then_block, else_block);
        self.builder.position_at_end(then_block);
        let config_entry = self.build_call(
            &ApiFunc::kclvm_dict_get_entry.name(),
            &[
                self.current_runtime_ctx_ptr(),
                config_value,
                string_ptr_value,
            ],
        );
        self.br(else_block);
        self.builder.position_at_end(else_block);
        let tpe = self.value_ptr_type();
        let phi = self.builder.build_phi(tpe, "");
        phi.add_incoming(&[(&none_value, last_block), (&config_entry, then_block)]);
        let config_value = phi.as_basic_value();
        if self.scope_level() >= INNER_LEVEL && !self.local_vars.borrow().contains(name) {
            if let Some(value) = value {
                self.dict_merge(schema_value, name, value, 1, -1);
            }
            self.value_union(schema_value, config_value);
            let cal_map = self
                .get_variable(value::SCHEMA_CAL_MAP)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let backtrack_cache = self
                .get_variable(value::BACKTRACK_CACHE)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let runtime_type = self
                .get_variable(value::SCHEMA_RUNTIME_TYPE)
                .expect(kcl_error::INTERNAL_ERROR_MSG);
            let name_native_str = self.native_global_string_value(name);
            self.build_void_call(
                &ApiFunc::kclvm_schema_backtrack_cache.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    schema_value,
                    backtrack_cache,
                    cal_map,
                    name_native_str,
                    runtime_type,
                ],
            );
            // Update backtrack meta
            if self.update_backtrack_meta(name, schema_value) {
                return true;
            }
        }
        false
    }
}
