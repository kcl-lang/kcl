use std::rc::Rc;

use generational_arena::Index;
use kclvm_ast::ast;
use kclvm_runtime::{BacktraceFrame, MAIN_PKG_PATH};

use crate::{
    error as kcl_error,
    func::FunctionCaller,
    lazy::{BacktrackMeta, Setter},
    proxy::{Frame, Proxy},
    rule::RuleCaller,
    schema::SchemaCaller,
    EvalContext, Evaluator, GLOBAL_LEVEL,
};

impl<'ctx> Evaluator<'ctx> {
    /// Current package path
    #[inline]
    pub(crate) fn current_pkgpath(&self) -> String {
        self.pkgpath_stack
            .borrow()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            .to_string()
    }

    /// Last package path
    #[inline]
    pub(crate) fn last_pkgpath(&self) -> String {
        let len = self.pkgpath_stack.borrow().len();
        self.pkgpath_stack
            .borrow()
            .get(if len > 2 { len - 2 } else { 2 - len })
            .unwrap_or(&MAIN_PKG_PATH.to_string())
            .to_string()
    }

    /// Current runtime context kcl line
    #[inline]
    pub(crate) fn update_ctx_panic_info<T>(&self, node: &'ctx ast::Node<T>) {
        let mut ctx = self.runtime_ctx.borrow_mut();
        ctx.panic_info.kcl_file = node.filename.clone();
        ctx.panic_info.kcl_line = node.line as i32;
    }

    /// Push a lambda definition scope into the lambda stack
    #[inline]
    pub fn push_lambda(&self, scope: usize) {
        self.lambda_stack.borrow_mut().push(scope);
    }

    /// Pop a lambda definition scope.
    #[inline]
    pub fn pop_lambda(&self) {
        self.lambda_stack.borrow_mut().pop();
    }

    #[inline]
    pub fn is_in_lambda(&self) -> bool {
        *self
            .lambda_stack
            .borrow()
            .last()
            .expect(kcl_error::INTERNAL_ERROR_MSG)
            > GLOBAL_LEVEL
    }

    #[inline]
    pub fn push_schema(&self, v: EvalContext) {
        self.schema_stack.borrow_mut().push(v);
    }

    #[inline]
    pub fn pop_schema(&self) {
        self.schema_stack.borrow_mut().pop();
    }

    #[inline]
    pub fn is_in_schema(&self) -> bool {
        !self.schema_stack.borrow().is_empty()
    }

    #[inline]
    pub fn push_schema_expr(&self) {
        self.schema_expr_stack.borrow_mut().push(());
    }

    #[inline]
    pub fn pop_schema_expr(&self) {
        self.schema_expr_stack.borrow_mut().pop();
    }

    #[inline]
    pub fn is_in_schema_expr(&self) -> bool {
        !self.schema_expr_stack.borrow().is_empty()
    }

    #[inline]
    pub fn add_local_var(&self, name: &str) {
        self.local_vars.borrow_mut().insert(name.to_string());
    }

    #[inline]
    pub fn remove_local_var(&self, name: &str) {
        self.local_vars.borrow_mut().remove(name);
    }

    #[inline]
    pub fn is_local_var(&self, name: &str) -> bool {
        self.local_vars.borrow().contains(name)
    }

    #[inline]
    pub(crate) fn clear_local_vars(&self) {
        self.local_vars.borrow_mut().clear();
    }

    #[inline]
    pub(crate) fn add_target_var(&self, name: &str) {
        self.target_vars.borrow_mut().push(name.to_string());
    }

    #[inline]
    pub(crate) fn pop_target_var(&self) {
        self.target_vars.borrow_mut().pop();
    }

    #[inline]
    pub(crate) fn get_target_var(&self) -> String {
        self.target_vars
            .borrow()
            .last()
            .cloned()
            .unwrap_or_default()
    }

    #[inline]
    pub(crate) fn check_imported(&self, pkgpath: &str) -> bool {
        let imported = &mut self.imported.borrow_mut();
        imported.contains(pkgpath)
    }

    #[inline]
    pub(crate) fn mark_imported(&self, pkgpath: &str) {
        let imported = &mut self.imported.borrow_mut();
        (*imported).insert(pkgpath.to_string());
    }

    #[inline]
    pub(crate) fn push_pkgpath(&self, pkgpath: &str) {
        self.pkgpath_stack.borrow_mut().push(pkgpath.to_string());
    }

    #[inline]
    pub(crate) fn pop_pkgpath(&self) {
        self.pkgpath_stack.borrow_mut().pop();
    }

    /// Append a global body into the scope.
    #[inline]
    pub(crate) fn add_global_body(&self, index: usize) -> Index {
        let pkgpath = self.current_pkgpath();
        self.frames.borrow_mut().insert(Rc::new(Frame {
            pkgpath,
            proxy: Proxy::Global(index),
        }))
    }

    /// Append a function into the scope.
    #[inline]
    pub(crate) fn add_function(&self, function: FunctionCaller) -> Index {
        let pkgpath = self.current_pkgpath();
        self.frames.borrow_mut().insert(Rc::new(Frame {
            pkgpath,
            proxy: Proxy::Lambda(function),
        }))
    }

    /// Append a schema into the scope.
    #[inline]
    pub(crate) fn add_schema(&self, schema: SchemaCaller) -> Index {
        let pkgpath = self.current_pkgpath();
        self.frames.borrow_mut().insert(Rc::new(Frame {
            pkgpath,
            proxy: Proxy::Schema(schema),
        }))
    }

    /// Append a rule into the scope.
    #[inline]
    pub(crate) fn add_rule(&self, rule: RuleCaller) -> Index {
        let pkgpath = self.current_pkgpath();
        self.frames.borrow_mut().insert(Rc::new(Frame {
            pkgpath,
            proxy: Proxy::Rule(rule),
        }))
    }

    pub(crate) fn push_backtrace(&self, frame: &Frame) {
        let ctx = &mut self.runtime_ctx.borrow_mut();
        if ctx.cfg.debug_mode {
            let backtrace_frame = BacktraceFrame::from_panic_info(&ctx.panic_info);
            ctx.backtrace.push(backtrace_frame);
            ctx.panic_info.kcl_func = frame.proxy.get_name();
        }
    }

    pub(crate) fn pop_backtrace(&self) {
        let ctx = &mut self.runtime_ctx.borrow_mut();
        if ctx.cfg.debug_mode {
            if let Some(backtrace_frame) = ctx.backtrace.pop() {
                ctx.panic_info.kcl_func = backtrace_frame.func;
                ctx.panic_info.kcl_line = backtrace_frame.line;
                ctx.panic_info.kcl_file = backtrace_frame.file;
            }
        }
    }

    pub(crate) fn push_backtrack_meta(&self, setter: &Setter) {
        let meta = &mut self.backtrack_meta.borrow_mut();
        meta.push(BacktrackMeta {
            stopped: setter.stopped.clone(),
            is_break: false,
        });
    }

    pub(crate) fn pop_backtrack_meta(&self) {
        let meta = &mut self.backtrack_meta.borrow_mut();
        meta.pop();
    }
}
