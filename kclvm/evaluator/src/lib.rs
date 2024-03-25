//! Copyright The KCL Authors. All rights reserved.

#[cfg(test)]
mod tests;

pub(crate) mod calculation;
pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod function;
pub(crate) mod module;
pub(crate) mod node;
pub(crate) mod scope;
pub(crate) mod value;

extern crate kclvm_error;

use context::EvaluatorContext;
use kclvm_ast::walker::TypedResultWalker;
use std::cell::RefCell;
use std::str;

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
}

impl<'ctx> Evaluator<'ctx> {
    /// New aa Evaluator using the LLVM Context and AST Program
    pub fn new(program: &'ctx ast::Program) -> Evaluator<'ctx> {
        Evaluator {
            ctx: RefCell::new(EvaluatorContext::default()),
            runtime_ctx: RefCell::new(Context::new()),
            program,
        }
    }

    /// Generate LLVM IR of ast module.
    pub fn run(self: &Evaluator<'ctx>) -> Result<(String, String)> {
        self.init_scope(kclvm_ast::MAIN_PKG);
        for module in self
            .program
            .pkgs
            .get(kclvm_ast::MAIN_PKG)
            .unwrap_or(&vec![])
        {
            self.walk_module(module)?;
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
