use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use generational_arena::Index;
use indexmap::IndexMap;
use kclvm_error::Handler;
use kclvm_runtime::{ValueRef, MAIN_PKG_PATH};

use crate::{
    error as kcl_error,
    func::FunctionCaller,
    proxy::{Frame, Proxy},
    rule::RuleCaller,
    schema::SchemaCaller,
    scope::Scope,
    Evaluator, GLOBAL_LEVEL,
};

pub struct EvaluatorContext {
    pub pkgpath_stack: Vec<String>,
    pub filename_stack: Vec<String>,
    /// Imported package path set to judge is there a duplicate import.
    pub imported: HashSet<String>,
    /// The lambda stack index denotes the scope level of the lambda function.
    pub lambda_stack: Vec<usize>,
    /// To judge is in the schema statement.
    pub schema_stack: Vec<()>,
    /// To judge is in the schema expression.
    pub schema_expr_stack: Vec<()>,
    /// Import names mapping
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    /// Package scope to store variable pointers.
    pub pkg_scopes: HashMap<String, Vec<Scope>>,
    /// Local variables in the loop.
    pub local_vars: HashSet<String>,
    /// The names of possible assignment objects for the current instruction.
    pub target_vars: Vec<String>,
    /// Global string caches
    pub global_strings: IndexMap<String, IndexMap<String, ValueRef>>,
    /// Global variable pointers cross different packages.
    pub global_vars: IndexMap<String, IndexMap<String, ValueRef>>,
    /// The filename of the source file corresponding to the current instruction
    pub current_filename: String,
    /// The line number of the source file corresponding to the current instruction
    pub current_line: u64,
    /// Error handler to store compile errors.
    pub handler: Handler,
    /// Debug mode
    pub debug: bool,
    /// Program work directory
    pub workdir: String,
}

impl Default for EvaluatorContext {
    fn default() -> Self {
        Self {
            imported: Default::default(),
            lambda_stack: vec![GLOBAL_LEVEL],
            schema_stack: Default::default(),
            schema_expr_stack: Default::default(),
            pkgpath_stack: vec![kclvm_ast::MAIN_PKG.to_string()],
            filename_stack: Default::default(),
            import_names: Default::default(),
            pkg_scopes: Default::default(),
            local_vars: Default::default(),
            target_vars: Default::default(),
            global_strings: Default::default(),
            global_vars: Default::default(),
            current_filename: Default::default(),
            current_line: Default::default(),
            handler: Default::default(),
            debug: Default::default(),
            workdir: Default::default(),
        }
    }
}

impl<'ctx> Evaluator<'ctx> {
    /// Current package path
    #[inline]
    pub(crate) fn current_pkgpath(&self) -> String {
        self.ctx
            .borrow()
            .pkgpath_stack
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .to_string()
    }

    /// Last package path
    #[inline]
    pub(crate) fn last_pkgpath(&self) -> String {
        let len = self.ctx.borrow().pkgpath_stack.len();
        self.ctx
            .borrow()
            .pkgpath_stack
            .get(if len > 2 { len - 2 } else { 2 - len })
            .unwrap_or(&MAIN_PKG_PATH.to_string())
            .to_string()
    }

    /// Current filename
    #[inline]
    pub(crate) fn current_filename(&self) -> String {
        self.ctx
            .borrow()
            .filename_stack
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .to_string()
    }

    /// Current line
    #[inline]
    pub(crate) fn current_line(&self) -> u64 {
        self.ctx.borrow().current_line
    }

    #[inline]
    pub fn push_filename(&self, filename: &str) {
        self.ctx
            .borrow_mut()
            .filename_stack
            .push(filename.to_string());
    }

    #[inline]
    pub fn pop_filename(&self) {
        self.ctx.borrow_mut().filename_stack.pop();
    }

    /// Push a lambda definition scope into the lambda stack
    #[inline]
    pub fn push_lambda(&self, scope: usize) {
        self.ctx.borrow_mut().lambda_stack.push(scope);
    }

    /// Pop a lambda definition scope.
    #[inline]
    pub fn pop_lambda(&self) {
        self.ctx.borrow_mut().lambda_stack.pop();
    }

    #[inline]
    pub fn is_in_lambda(&self) -> bool {
        *self
            .ctx
            .borrow()
            .lambda_stack
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            > GLOBAL_LEVEL
    }

    #[inline]
    pub fn push_schema(&self) {
        self.ctx.borrow_mut().schema_stack.push(());
    }

    #[inline]
    pub fn pop_schema(&self) {
        self.ctx.borrow_mut().schema_stack.pop();
    }

    #[inline]
    pub fn is_in_schema(&self) -> bool {
        !self.ctx.borrow().schema_stack.is_empty()
    }

    #[inline]
    pub fn push_schema_expr(&self) {
        self.ctx.borrow_mut().schema_expr_stack.push(());
    }

    #[inline]
    pub fn pop_schema_expr(&self) {
        self.ctx.borrow_mut().schema_expr_stack.pop();
    }

    #[inline]
    pub fn is_in_schema_expr(&self) -> bool {
        !self.ctx.borrow().schema_expr_stack.is_empty()
    }

    #[inline]
    pub fn add_local_var(&self, name: &str) {
        self.ctx.borrow_mut().local_vars.insert(name.to_string());
    }

    #[inline]
    pub fn remove_local_var(&self, name: &str) {
        self.ctx.borrow_mut().local_vars.remove(name);
    }

    #[inline]
    pub fn is_local_var(&self, name: &str) -> bool {
        self.ctx.borrow().local_vars.contains(name)
    }

    #[inline]
    pub(crate) fn clear_local_vars(&self) {
        self.ctx.borrow_mut().local_vars.clear();
    }

    /// Reset target vars.
    #[inline]
    pub(crate) fn reset_target_vars(&self) {
        let target_vars = &mut self.ctx.borrow_mut().target_vars;
        target_vars.clear();
        target_vars.push("".to_string());
    }

    #[inline]
    pub(crate) fn add_target_var(&self, name: &str) {
        self.ctx.borrow_mut().target_vars.push(name.to_string());
    }

    #[inline]
    pub(crate) fn check_imported(&self, pkgpath: &str) -> bool {
        let imported = &mut self.ctx.borrow_mut().imported;
        imported.contains(pkgpath)
    }

    #[inline]
    pub(crate) fn mark_imported(&self, pkgpath: &str) {
        let imported = &mut self.ctx.borrow_mut().imported;
        (*imported).insert(pkgpath.to_string());
    }

    #[inline]
    pub(crate) fn push_pkgpath(&self, pkgpath: &str) {
        self.ctx
            .borrow_mut()
            .pkgpath_stack
            .push(pkgpath.to_string());
    }

    #[inline]
    pub(crate) fn pop_pkgpath(&self) {
        self.ctx.borrow_mut().pkgpath_stack.pop();
    }

    /// Append a function into the scope
    #[inline]
    pub(crate) fn add_function(&self, function: FunctionCaller) -> Index {
        let pkgpath = self.current_pkgpath();
        self.frames.borrow_mut().insert(Rc::new(Frame {
            pkgpath,
            proxy: Proxy::Lambda(function),
        }))
    }

    /// Append a schema into the scope
    #[inline]
    pub(crate) fn add_schema(&self, schema: SchemaCaller) -> Index {
        let pkgpath = self.current_pkgpath();
        self.frames.borrow_mut().insert(Rc::new(Frame {
            pkgpath,
            proxy: Proxy::Schema(schema),
        }))
    }

    /// Append a rule into the scope
    #[inline]
    pub(crate) fn add_rule(&self, rule: RuleCaller) -> Index {
        let pkgpath = self.current_pkgpath();
        self.frames.borrow_mut().insert(Rc::new(Frame {
            pkgpath,
            proxy: Proxy::Rule(rule),
        }))
    }
}
