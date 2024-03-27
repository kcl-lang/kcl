//! Copyright The KCL Authors. All rights reserved.

#[cfg(test)]
mod tests;

pub(crate) mod calculation;
pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod func;
pub(crate) mod module;
pub(crate) mod node;
pub(crate) mod scope;
pub(crate) mod value;

extern crate kclvm_error;

use context::EvaluatorContext;
use func::FunctionProxy;
use generational_arena::Arena;
use std::str;
use std::{cell::RefCell, sync::Arc};

use crate::error as kcl_error;
use anyhow::Result;
use kclvm_ast::ast;
use kclvm_runtime::{Context, ValueRef};

/// SCALAR_KEY denotes the temp scalar key for the global variable json plan process.
const SCALAR_KEY: &str = "";
/// Global level
const GLOBAL_LEVEL: usize = 1;
/// Inner level
const INNER_LEVEL: usize = 2;

/// The evaluator function result
pub type EvalResult = Result<ValueRef>;

/// The evaluator for the program
pub struct Evaluator<'ctx> {
    pub program: &'ctx ast::Program,
    pub ctx: RefCell<EvaluatorContext>,
    pub runtime_ctx: RefCell<Context>,
    pub functions: RefCell<Arena<Arc<FunctionProxy>>>,
}

impl<'ctx> Evaluator<'ctx> {
    /// New aa Evaluator using the AST program
    #[inline]
    pub fn new(program: &'ctx ast::Program) -> Evaluator<'ctx> {
        Self::new_with_runtime_ctx(program, Context::new())
    }

    /// New aa Evaluator using the AST program and runtime context
    #[inline]
    pub fn new_with_runtime_ctx(
        program: &'ctx ast::Program,
        runtime_ctx: Context,
    ) -> Evaluator<'ctx> {
        Evaluator {
            ctx: RefCell::new(EvaluatorContext::default()),
            runtime_ctx: RefCell::new(runtime_ctx),
            program,
            functions: RefCell::new(Arena::new()),
        }
    }

    /// Evaluate the program
    pub fn run(self: &Evaluator<'ctx>) -> Result<(String, String)> {
        if let Some(modules) = self.program.pkgs.get(kclvm_ast::MAIN_PKG) {
            self.init_scope(kclvm_ast::MAIN_PKG);
            self.compile_ast_modules(modules)
        }
        Ok(self.plan_globals_to_string())
    }

    /// Plan globals to a planed json and yaml string
    pub fn plan_globals_to_string(&self) -> (String, String) {
        let current_pkgpath = self.current_pkgpath();
        let ctx = self.ctx.borrow();
        let pkg_scopes = &ctx.pkg_scopes;
        let scopes = pkg_scopes
            .get(&current_pkgpath)
            .unwrap_or_else(|| panic!("pkgpath {} is not found", current_pkgpath));
        // The global scope.
        let scope = scopes.last().expect(kcl_error::INTERNAL_ERROR_MSG);
        let scalars = &scope.scalars;
        let globals = &scope.variables;
        // Construct a plan object.
        let mut global_dict = self.dict_value();
        // Plan empty dict result.
        if scalars.is_empty() && globals.is_empty() {
            return global_dict.plan(&self.runtime_ctx.borrow());
        }
        // Deal scalars
        for scalar in scalars.iter() {
            self.dict_insert_value(&mut global_dict, SCALAR_KEY, scalar);
        }
        // Deal global variables
        for (name, value) in globals.iter() {
            let mut value_dict = self.dict_value();
            self.dict_insert_value(&mut value_dict, name.as_str(), value);
            self.dict_insert_value(&mut global_dict, SCALAR_KEY, &value_dict);
        }
        // Plan result to JSON and YAML string.
        match global_dict.dict_get_value(SCALAR_KEY) {
            Some(value) => value.plan(&self.runtime_ctx.borrow()),
            None => self.dict_value().plan(&self.runtime_ctx.borrow()),
        }
    }

    /// Get evaluator default ok result
    #[inline]
    pub fn ok_result(&self) -> EvalResult {
        Ok(self.undefined_value())
    }
}
