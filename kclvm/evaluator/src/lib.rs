//! Copyright The KCL Authors. All rights reserved.

#[cfg(test)]
mod tests;

mod calculation;
mod context;
mod error;
mod func;
#[macro_use]
mod lazy;
mod module;
mod node;
mod proxy;
mod rule;
mod schema;
mod scope;
mod ty;
mod union;
mod value;

extern crate kclvm_error;

use func::FunctionEvalContextRef;
use generational_arena::{Arena, Index};
use indexmap::IndexMap;
use lazy::{BacktrackMeta, LazyEvalScope};
use proxy::{Frame, Proxy};
use rule::RuleEvalContextRef;
use schema::SchemaEvalContextRef;
use scope::Scope;
use std::collections::{HashMap, HashSet};
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
    /// All frames including functions, schemas and rules
    pub frames: RefCell<Arena<Rc<Frame>>>,
    /// All schema index in the package path, we can find the frame through the index.
    pub schemas: RefCell<IndexMap<String, Index>>,
    /// Runtime evaluation context.
    pub runtime_ctx: Rc<RefCell<Context>>,
    /// Package path stack.
    pub pkgpath_stack: RefCell<Vec<String>>,
    /// Filename stack.
    pub filename_stack: RefCell<Vec<String>>,
    /// The names of possible assignment objects for the current instruction.
    pub target_vars: RefCell<Vec<String>>,
    /// Imported package path set to judge is there a duplicate import.
    pub imported: RefCell<HashSet<String>>,
    /// The lambda stack index denotes the scope level of the lambda function.
    pub lambda_stack: RefCell<Vec<FunctionEvalContextRef>>,
    /// To judge is in the schema statement.
    pub schema_stack: RefCell<Vec<EvalContext>>,
    /// To judge is in the schema expression.
    pub schema_expr_stack: RefCell<Vec<()>>,
    /// Import names mapping
    pub import_names: RefCell<IndexMap<String, IndexMap<String, String>>>,
    /// Package scope to store variable values.
    pub pkg_scopes: RefCell<HashMap<String, Vec<Scope>>>,
    /// Package lazy scope to store variable cached values.
    pub lazy_scopes: RefCell<HashMap<String, LazyEvalScope>>,
    /// Scope cover to block the acquisition of certain scopes.
    pub scope_covers: RefCell<Vec<(usize, usize)>>,
    /// Local variables in the loop.
    pub local_vars: RefCell<HashSet<String>>,
    /// Schema attr backtrack meta
    pub backtrack_meta: RefCell<Vec<BacktrackMeta>>,
}

#[derive(Clone)]
pub enum EvalContext {
    Schema(SchemaEvalContextRef),
    Rule(RuleEvalContextRef),
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
            runtime_ctx,
            program,
            frames: RefCell::new(Arena::new()),
            schemas: RefCell::new(IndexMap::new()),
            target_vars: RefCell::new(vec![]),
            lambda_stack: RefCell::new(vec![]),
            imported: RefCell::new(Default::default()),
            schema_stack: RefCell::new(Default::default()),
            schema_expr_stack: RefCell::new(Default::default()),
            pkgpath_stack: RefCell::new(vec![kclvm_ast::MAIN_PKG.to_string()]),
            filename_stack: RefCell::new(Default::default()),
            import_names: RefCell::new(Default::default()),
            pkg_scopes: RefCell::new(Default::default()),
            lazy_scopes: RefCell::new(Default::default()),
            scope_covers: RefCell::new(Default::default()),
            local_vars: RefCell::new(Default::default()),
            backtrack_meta: RefCell::new(Default::default()),
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
        let pkg_scopes = &self.pkg_scopes.borrow();
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

    fn plan_value(&self, value: &ValueRef) -> (String, String) {
        let mut ctx = self.runtime_ctx.borrow_mut();
        let value = match ctx.buffer.custom_manifests_output.clone() {
            Some(output) => ValueRef::from_yaml_stream(&mut ctx, &output).unwrap(),
            None => value.clone(),
        };
        let (json_string, yaml_string) = value.plan(&ctx);
        ctx.json_result = json_string.clone();
        ctx.yaml_result = yaml_string.clone();
        (json_string, yaml_string)
    }
}

impl UnwindSafe for Evaluator<'_> {}
impl RefUnwindSafe for Evaluator<'_> {}
