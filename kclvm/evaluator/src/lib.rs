//! Copyright The KCL Authors. All rights reserved.

#[cfg(test)]
mod tests;

mod calculation;
mod context;
mod error;
mod func;
mod lazy;
mod module;
mod node;
mod proxy;
mod rule;
mod schema;
mod scope;
mod value;

extern crate kclvm_error;

use context::EvaluatorContext;
use generational_arena::Arena;
use proxy::{Frame, Proxy};
use std::panic::RefUnwindSafe;
use std::rc::Rc;
use std::str;
use std::{cell::RefCell, panic::UnwindSafe};

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
    pub frames: RefCell<Arena<Rc<Frame>>>,
    pub runtime_ctx: Rc<RefCell<Context>>,
}

impl<'ctx> Evaluator<'ctx> {
    /// New aa Evaluator using the AST program
    #[inline]
    pub fn new(program: &'ctx ast::Program) -> Evaluator<'ctx> {
        Self::new_with_runtime_ctx(program, Rc::new(RefCell::new(Context::new())))
    }

    /// New aa Evaluator using the AST program and runtime context
    #[inline]
    pub fn new_with_runtime_ctx(
        program: &'ctx ast::Program,
        runtime_ctx: Rc<RefCell<Context>>,
    ) -> Evaluator<'ctx> {
        Evaluator {
            ctx: RefCell::new(EvaluatorContext::default()),
            runtime_ctx,
            program,
            frames: RefCell::new(Arena::new()),
        }
    }

    /// Evaluate the program and return the JSON and YAML result.
    pub fn run(self: &Evaluator<'ctx>) -> Result<(String, String)> {
        if let Some(modules) = self.program.pkgs.get(kclvm_ast::MAIN_PKG) {
            self.init_scope(kclvm_ast::MAIN_PKG);
            self.compile_ast_modules(modules)
        }
        Ok(self.plan_globals_to_string())
    }

    /// Plan globals to a planed json and yaml string.
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
            return self.plan_value(&global_dict);
        }
        // Deal scalars
        for scalar in scalars.iter() {
            self.dict_insert_merge_value(&mut global_dict, SCALAR_KEY, scalar);
        }
        // Deal global variables
        for (name, value) in globals.iter() {
            let mut value_dict = self.dict_value();
            self.dict_insert_merge_value(&mut value_dict, name.as_str(), value);
            self.dict_insert_merge_value(&mut global_dict, SCALAR_KEY, &value_dict);
        }
        // Plan result to JSON and YAML string.
        match global_dict.dict_get_value(SCALAR_KEY) {
            Some(value) => self.plan_value(&value),
            None => self.plan_value(&self.dict_value()),
        }
    }

    /// Get evaluator default ok result
    #[inline]
    pub fn ok_result(&self) -> EvalResult {
        Ok(self.undefined_value())
    }

    fn plan_value(&self, p: &ValueRef) -> (String, String) {
        let mut ctx = self.runtime_ctx.borrow_mut();
        let value = match ctx.buffer.custom_manifests_output.clone() {
            Some(output) => ValueRef::from_yaml_stream(&mut ctx, &output).unwrap(),
            None => p.clone(),
        };
        let (json_string, yaml_string) = value.plan(&ctx);
        ctx.json_result = json_string.clone();
        ctx.yaml_result = yaml_string.clone();
        (ctx.json_result.clone(), ctx.yaml_result.clone())
    }
}

impl UnwindSafe for Evaluator<'_> {}
impl RefUnwindSafe for Evaluator<'_> {}
