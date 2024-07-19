// Copyright The KCL Authors. All rights reserved.

use indexmap::IndexMap;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;
use kclvm_ast::ast;
use kclvm_ast::walker::TypedResultWalker;
use kclvm_runtime::ApiFunc;
use kclvm_sema::pkgpath_without_prefix;

use super::context::{BacktrackMeta, LLVMCodeGenContext};
use crate::codegen::llvm::context::BacktrackKind;
use crate::codegen::traits::{BuilderMethods, ProgramCodeGen, ValueMethods};
use crate::codegen::{error as kcl_error, ENTRY_NAME};
use crate::value;
use std::str;

impl<'ctx> LLVMCodeGenContext<'ctx> {
    pub fn compile_module_import_and_types(&self, module: &'ctx ast::Module) {
        for stmt in &module.body {
            match &stmt.node {
                ast::Stmt::Import(import_stmt) => {
                    self.walk_import_stmt(import_stmt)
                        .expect(kcl_error::COMPILE_ERROR_MSG);
                }
                ast::Stmt::Schema(schema_stmt) => {
                    // Pre define global types with undefined values
                    self.predefine_global_types(&schema_stmt.name.node);
                    self.walk_schema_stmt(schema_stmt)
                        .expect(kcl_error::COMPILE_ERROR_MSG);
                }
                ast::Stmt::Rule(rule_stmt) => {
                    // Pre define global types with undefined values
                    self.predefine_global_types(&rule_stmt.name.node);
                    self.walk_rule_stmt(rule_stmt)
                        .expect(kcl_error::COMPILE_ERROR_MSG);
                }
                _ => {}
            };
        }
        // Pre define global variables with setter functions.
        self.predefine_global_setters(module);
    }

    pub fn predefine_global_types(&self, name: &str) {
        // Store or add the variable in the scope
        let function = self.undefined_value();
        if !self.store_variable(name, function) {
            let global_var_ptr = self.new_global_kcl_value_ptr("");
            self.builder.build_store(global_var_ptr, function);
            self.add_variable(name, global_var_ptr);
        }
    }

    /// Predefine all global variables.
    #[inline]
    pub(crate) fn predefine_global_vars(&self, module: &'ctx ast::Module) {
        self.emit_global_vars(&module.body);
    }

    /// Predefine all global variables.
    pub fn predefine_global_setters(&self, module: &'ctx ast::Module) {
        // New a function block to the global setter construction process.
        let global_setter_block = self.append_block("");
        self.br(global_setter_block);
        self.builder.position_at_end(global_setter_block);
        let mut place_holder_map: IndexMap<String, Vec<FunctionValue<'ctx>>> = IndexMap::new();
        let mut body_map: IndexMap<String, Vec<(&ast::Node<ast::Stmt>, BacktrackKind)>> =
            IndexMap::new();
        let pkgpath = &self.current_pkgpath();
        // Setter function name format: "$set.<pkg_path>.$<var_name>"
        self.emit_global_setters(
            &module.body,
            &pkgpath,
            false,
            &mut place_holder_map,
            &mut body_map,
            &mut vec![],
        );
        // Build global attribute backtrack functions.
        {
            for (k, functions) in &place_holder_map {
                if k == kclvm_runtime::CAL_MAP_INDEX_SIGNATURE {
                    continue;
                }
                let stmt_list = body_map.get(k).expect(kcl_error::INTERNAL_ERROR_MSG);
                let mut if_level = 0;
                for (attr_func, (stmt, kind)) in functions.iter().zip(stmt_list) {
                    let function = *attr_func;
                    let name = function
                        .get_name()
                        .to_str()
                        .expect(kcl_error::INTERNAL_ERROR_MSG);
                    // Get attribute function from the module.
                    let function = self.lookup_function(name);
                    self.push_function(function);
                    let attr_block = self.append_block(ENTRY_NAME);
                    self.builder.position_at_end(attr_block);
                    // Backtrack meta begin
                    if matches!(&stmt.node, ast::Stmt::If(..)) {
                        if_level += 1;
                        *self.backtrack_meta.borrow_mut() = Some(BacktrackMeta {
                            target: k.clone(),
                            level: if_level,
                            count: 0,
                            stop: false,
                            kind: kind.clone(),
                        });
                    } else {
                        if_level = 0;
                    }
                    let result = self.walk_stmt(stmt).expect(kcl_error::COMPILE_ERROR_MSG);
                    // Backtrack meta end
                    if matches!(&stmt.node, ast::Stmt::If(..)) {
                        *self.backtrack_meta.borrow_mut() = None
                    }
                    // Build return
                    self.builder.build_return(Some(&result));
                    // Position at global main function block
                    self.builder.position_at_end(global_setter_block);
                    self.pop_function();
                }
            }
        }
    }

    fn emit_global_vars(&self, body: &'ctx [Box<ast::Node<ast::Stmt>>]) {
        for stmt in body {
            match &stmt.node {
                ast::Stmt::Unification(unification_stmt) => {
                    let names = &unification_stmt.target.node.names;
                    if names.len() == 1 {
                        self.add_or_update_global_variable(
                            &names[0].node,
                            self.undefined_value(),
                            false,
                        );
                    }
                }
                ast::Stmt::Assign(assign_stmt) => {
                    for target in &assign_stmt.targets {
                        self.add_or_update_global_variable(
                            target.node.get_name(),
                            self.undefined_value(),
                            false,
                        );
                    }
                }
                ast::Stmt::If(if_stmt) => {
                    self.emit_global_vars(&if_stmt.body);
                    self.emit_global_vars(&if_stmt.orelse);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn emit_config_if_entry_expr_vars(
        &self,
        config_if_entry_expr: &'ctx ast::ConfigIfEntryExpr,
    ) {
        self.emit_config_entries_vars(&config_if_entry_expr.items);
        if let Some(orelse) = &config_if_entry_expr.orelse {
            // Config expr or config if entry expr.
            if let ast::Expr::Config(config_expr) = &orelse.node {
                self.emit_config_entries_vars(&config_expr.items);
            } else if let ast::Expr::ConfigIfEntry(config_if_entry_expr) = &orelse.node {
                self.emit_config_if_entry_expr_vars(config_if_entry_expr);
            }
        }
    }

    pub(crate) fn emit_config_entries_vars(&self, items: &'ctx [ast::NodeRef<ast::ConfigEntry>]) {
        for item in items {
            if let ast::Expr::ConfigIfEntry(config_if_entry_expr) = &item.node.value.node {
                self.emit_config_if_entry_expr_vars(config_if_entry_expr);
            }
            if let Some(key) = &item.node.key {
                let optional_name = match &key.node {
                    ast::Expr::Identifier(identifier) => Some(identifier.names[0].node.clone()),
                    ast::Expr::StringLit(string_lit) => Some(string_lit.value.clone()),
                    ast::Expr::Subscript(subscript) => {
                        let mut name = None;
                        if let ast::Expr::Identifier(identifier) = &subscript.value.node {
                            if let Some(index_node) = &subscript.index {
                                if let ast::Expr::NumberLit(number) = &index_node.node {
                                    if let ast::NumberLitValue::Int(_) = number.value {
                                        name = Some(identifier.names[0].node.clone())
                                    }
                                }
                            }
                        }
                        name
                    }
                    _ => None,
                };
                if let Some(name) = &optional_name {
                    self.add_or_update_local_variable_within_scope(name, None);
                }
            }
        }
    }

    /// Compile AST Modules, which requires traversing three times.
    /// 1. scan all possible global variables and allocate undefined values to global pointers.
    /// 2. build all user-defined schema/rule types.
    /// 3. generate all LLVM IR codes for the third time.
    pub(crate) fn compile_ast_modules(&self, modules: &'ctx [ast::Module]) {
        // Scan global variables
        for ast_module in modules {
            {
                self.filename_stack
                    .borrow_mut()
                    .push(ast_module.filename.clone());
            }
            // Pre define global variables with undefined values
            self.predefine_global_vars(ast_module);
            {
                self.filename_stack.borrow_mut().pop();
            }
        }
        // Scan global types
        for ast_module in modules {
            {
                self.filename_stack
                    .borrow_mut()
                    .push(ast_module.filename.clone());
            }
            self.compile_module_import_and_types(ast_module);
            {
                self.filename_stack.borrow_mut().pop();
            }
        }
        // Compile the ast module in the pkgpath.
        for ast_module in modules {
            {
                self.filename_stack
                    .borrow_mut()
                    .push(ast_module.filename.clone());
            }
            self.walk_module(ast_module)
                .expect(kcl_error::COMPILE_ERROR_MSG);
            {
                self.filename_stack.borrow_mut().pop();
            }
        }
    }

    /// Emit setter functions for global variables.
    pub(crate) fn emit_global_setters(
        &self,
        body: &'ctx [Box<ast::Node<ast::Stmt>>],
        pkgpath: &str,
        is_in_if: bool,
        place_holder_map: &mut IndexMap<String, Vec<FunctionValue<'ctx>>>,
        body_map: &mut IndexMap<String, Vec<(&'ctx ast::Node<ast::Stmt>, BacktrackKind)>>,
        in_if_names: &mut Vec<String>,
    ) {
        let add_stmt = |name: &str,
                        stmt: &'ctx ast::Node<ast::Stmt>,
                        kind: BacktrackKind,
                        place_holder_map: &mut IndexMap<String, Vec<FunctionValue<'ctx>>>,
                        body_map: &mut IndexMap<
            String,
            Vec<(&'ctx ast::Node<ast::Stmt>, BacktrackKind)>,
        >| {
            // The function form e.g., $set.__main__.a(&Context, &LazyScope, &ValueRef, &ValueRef)
            let var_key = format!("{}.{name}", pkgpath_without_prefix!(pkgpath));
            let function =
                self.add_setter_function(&format!("{}.{}", value::GLOBAL_SETTER, var_key));
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
            self.build_void_call(
                &ApiFunc::kclvm_scope_add_setter.name(),
                &[
                    self.current_runtime_ctx_ptr(),
                    self.current_scope_ptr(),
                    self.native_global_string(pkgpath, "").into(),
                    self.native_global_string(name, "").into(),
                    lambda_fn_ptr,
                ],
            );
            let key = format!("{}.{name}", pkgpath_without_prefix!(pkgpath));
            self.setter_keys.borrow_mut().insert(key);
            if !body_map.contains_key(name) {
                body_map.insert(name.to_string(), vec![]);
            }
            let body_vec = body_map.get_mut(name).expect(kcl_error::INTERNAL_ERROR_MSG);
            body_vec.push((stmt, kind));
        };
        for stmt in body {
            match &stmt.node {
                ast::Stmt::Unification(unification_stmt) => {
                    let name = &unification_stmt.target.node.names[0].node;
                    if is_in_if {
                        in_if_names.push(name.to_string());
                    } else {
                        add_stmt(
                            name,
                            stmt,
                            BacktrackKind::Normal,
                            place_holder_map,
                            body_map,
                        );
                    }
                }
                ast::Stmt::Assign(assign_stmt) => {
                    for target in &assign_stmt.targets {
                        let name = &target.node.name.node;
                        if is_in_if {
                            in_if_names.push(name.to_string());
                        } else {
                            add_stmt(
                                name,
                                stmt,
                                BacktrackKind::Normal,
                                place_holder_map,
                                body_map,
                            );
                        }
                    }
                }
                ast::Stmt::AugAssign(aug_assign_stmt) => {
                    let target = &aug_assign_stmt.target;
                    let name = &target.node.name.node;
                    if is_in_if {
                        in_if_names.push(name.to_string());
                    } else {
                        add_stmt(
                            name,
                            stmt,
                            BacktrackKind::Normal,
                            place_holder_map,
                            body_map,
                        );
                    }
                }
                ast::Stmt::If(if_stmt) => {
                    let mut names: Vec<String> = vec![];
                    self.emit_global_setters(
                        &if_stmt.body,
                        pkgpath,
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
                            add_stmt(name, stmt, BacktrackKind::If, place_holder_map, body_map);
                        }
                    }
                    names.clear();
                    self.emit_global_setters(
                        &if_stmt.orelse,
                        pkgpath,
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
                            add_stmt(
                                name,
                                stmt,
                                BacktrackKind::OrElse,
                                place_holder_map,
                                body_map,
                            );
                        }
                    }
                    names.clear();
                }
                ast::Stmt::SchemaAttr(schema_attr) => {
                    let name = schema_attr.name.node.as_str();
                    if is_in_if {
                        in_if_names.push(name.to_string());
                    } else {
                        add_stmt(
                            name,
                            stmt,
                            BacktrackKind::Normal,
                            place_holder_map,
                            body_map,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
