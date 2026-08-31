// Copyright The KCL Authors. All rights reserved.

use std::sync::{Arc, RwLock};

use kcl_ast::ast;
use kcl_ast::walker::TypedResultWalker;
use kcl_runtime::ValueRef;

use super::Evaluator;
use crate::error as kcl_error;

impl<'ctx> Evaluator<'_> {
    pub fn compile_module_import_and_types(&self, module: &'ctx ast::Module) {
        for stmt in &module.body {
            match &stmt.node {
                ast::Stmt::Import(import_stmt) => {
                    self.walk_import_stmt(import_stmt)
                        .expect(kcl_error::RUNTIME_ERROR_MSG);
                }
                ast::Stmt::Schema(schema_stmt) => {
                    // Pre define global types with undefined values
                    self.predefine_global_types(&schema_stmt.name.node);
                    self.walk_schema_stmt(schema_stmt)
                        .expect(kcl_error::RUNTIME_ERROR_MSG);
                }
                ast::Stmt::Rule(rule_stmt) => {
                    // Pre define global types with undefined values
                    self.predefine_global_types(&rule_stmt.name.node);
                    self.walk_rule_stmt(rule_stmt)
                        .expect(kcl_error::RUNTIME_ERROR_MSG);
                }
                _ => {}
            };
        }
    }

    pub fn predefine_global_types(&self, name: &str) {
        // Store or add the variable in the scope
        let function = self.undefined_value();
        if !self.store_variable(name, function.clone()) {
            self.add_variable(name, function);
        }
    }

    /// Predefine all global variables.
    #[inline]
    pub(crate) fn predefine_global_vars(&self, module: &'ctx ast::Module) {
        self.emit_global_vars(&module.body);
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

    /// Compile AST Modules, which requires traversing three times.
    /// 1. scan all possible global variables and allocate undefined values to global pointers.
    /// 2. build all user-defined schema/rule types.
    /// 3. evaluate all codes for the third time.
    pub(crate) fn compile_ast_modules(&self, modules: &[Arc<RwLock<ast::Module>>]) -> ValueRef {
        self.compile_ast_modules_with_skip(modules, false)
    }

    /// Like [`compile_ast_modules`], but allows the caller to skip
    /// pass-3 (statement evaluation). Passes 1 (predefine global
    /// variables) and 2 (build schema/rule types) always run so that
    /// undefined-name errors and type lookups remain correct for
    /// referenced names; only the heavy statement bodies are skipped
    /// when `skip_body` is `true`.
    pub(crate) fn compile_ast_modules_with_skip(
        &self,
        modules: &[Arc<RwLock<ast::Module>>],
        skip_body: bool,
    ) -> ValueRef {
        // Scan global variables
        for ast_module in modules {
            let ast_module = ast_module.read().expect("Failed to acquire module lock");
            // Pre define global variables with undefined values
            self.predefine_global_vars(&ast_module);
        }
        // Scan global types
        for ast_module in modules {
            let ast_module = ast_module.read().expect("Failed to acquire module lock");
            self.compile_module_import_and_types(&ast_module);
        }
        // Gate pass-3 (statement evaluation). When skipping, we still
        // need the schemas declared in this module to be visible to
        // *referenced* packages; pass-2 already populates that graph.
        if skip_body {
            return ValueRef::undefined();
        }
        let mut result = ValueRef::undefined();
        // Compile the ast module in the pkgpath.
        for ast_module in modules {
            let ast_module = ast_module.read().expect("Failed to acquire module lock");
            result = self
                .walk_module(&ast_module)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        }
        result
    }
}
