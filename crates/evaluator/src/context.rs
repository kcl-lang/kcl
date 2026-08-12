use std::{collections::HashSet, rc::Rc};

use generational_arena::Index;
use kcl_ast::ast;
use kcl_runtime::{BacktraceFrame, MAIN_PKG_PATH};

use crate::{
    EvalContext, Evaluator, LambdaOrSchemaEvalContext, error as kcl_error,
    func::{FunctionCaller, FunctionEvalContextRef},
    lazy::{BacktrackMeta, Setter, SetterKind},
    proxy::{Frame, Proxy},
    rule::RuleCaller,
    schema::SchemaCaller,
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
            .get(len.abs_diff(2))
            .unwrap_or(&MAIN_PKG_PATH.to_string())
            .to_string()
    }

    /// Update current runtime context kcl filename and line
    #[inline]
    pub(crate) fn update_ctx_panic_info<T>(&self, node: &'ctx ast::Node<T>) {
        let mut ctx = self.runtime_ctx.borrow_mut();
        ctx.panic_info.kcl_file = node.filename.clone();
        ctx.panic_info.kcl_line = node.line as i32;
    }

    /// Update current AST index.
    #[inline]
    pub(crate) fn update_ast_id<T>(&self, node: &'ctx ast::Node<T>) {
        *self.ast_id.borrow_mut() = node.id.clone();
    }

    /// Push a lambda definition scope into the lambda stack
    #[inline]
    pub fn push_lambda(
        &self,
        lambda_ctx: FunctionEvalContextRef,
        current_pkgpath: &str,
        frame_pkgpath: &str,
        level: usize,
    ) {
        // Capture function schema this reference.
        if let Some(this) = &lambda_ctx.this {
            self.push_schema(this.eval_ctx());
        }
        // Inner scope function calling.
        // Note the minimum lambda.ctx.level is 2 for the top level lambda definitions.
        if frame_pkgpath == current_pkgpath && level >= lambda_ctx.level {
            // The scope cover is [lambda.ctx.level, self.scope_level()]
            self.push_scope_cover(lambda_ctx.level, level);
        }
        self.lambda_stack.borrow_mut().push(lambda_ctx.clone());
        self.ctx_stack
            .borrow_mut()
            .push(LambdaOrSchemaEvalContext::Lambda(lambda_ctx));
    }

    /// Pop a lambda definition scope.
    #[inline]
    pub fn pop_lambda(
        &self,
        lambda_ctx: FunctionEvalContextRef,
        current_pkgpath: &str,
        frame_pkgpath: &str,
        level: usize,
    ) {
        self.lambda_stack.borrow_mut().pop();
        self.ctx_stack.borrow_mut().pop();
        // Inner scope function calling.
        if frame_pkgpath == current_pkgpath && level >= lambda_ctx.level {
            self.pop_scope_cover();
        }
        // Release function schema this reference.
        if lambda_ctx.this.is_some() {
            self.pop_schema()
        }
    }

    #[inline]
    pub fn is_in_lambda(&self) -> bool {
        !self.lambda_stack.borrow().is_empty()
    }

    #[inline]
    pub fn last_lambda_ctx(&self) -> Option<FunctionEvalContextRef> {
        self.lambda_stack.borrow().last().cloned()
    }

    #[inline]
    pub fn push_schema(&self, v: EvalContext) {
        self.schema_stack.borrow_mut().push(v.clone());
        self.ctx_stack
            .borrow_mut()
            .push(LambdaOrSchemaEvalContext::Schema(v));
    }

    #[inline]
    pub fn pop_schema(&self) {
        self.schema_stack.borrow_mut().pop();
        self.ctx_stack.borrow_mut().pop();
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
    pub(crate) fn add_loop_var(&self, name: &str) {
        self.local_vars.borrow_mut().insert(name.to_string());
        self.loop_vars.borrow_mut().insert(name.to_string());
    }

    #[inline]
    pub(crate) fn remove_loop_var(&self, name: &str) {
        self.local_vars.borrow_mut().remove(name);
        self.loop_vars.borrow_mut().remove(name);
    }

    #[inline]
    pub(crate) fn is_loop_var(&self, name: &str) -> bool {
        self.loop_vars.borrow().contains(name)
    }

    /// Enter a config expression scope for config entry variable marks.
    #[inline]
    pub(crate) fn enter_config_entry_scope(&self) {
        self.config_entry_vars.borrow_mut().push(vec![]);
    }

    /// Mark a config entry name as a local variable, so that the following
    /// sibling entries resolve it lexically in the current config scope
    /// instead of falling back to an enclosing schema attribute of the same
    /// name. Names that are already local (loop variables, lambda arguments,
    /// entries of an outer config, ...) are left untouched so that leaving
    /// this config scope does not drop a mark it does not own.
    #[inline]
    pub(crate) fn mark_config_entry_var(&self, name: &str) {
        if self.is_local_var(name) {
            return;
        }
        match self.config_entry_vars.borrow_mut().last_mut() {
            Some(frame) => frame.push(name.to_string()),
            // No enclosing config scope to undo the mark, so do not add one.
            None => return,
        }
        self.add_local_var(name);
    }

    /// Leave a config expression scope and undo the marks it added.
    #[inline]
    pub(crate) fn leave_config_entry_scope(&self) {
        let frame = self.config_entry_vars.borrow_mut().pop();
        if let Some(frame) = frame {
            for name in frame {
                self.remove_local_var(&name);
            }
        }
    }

    /// Re-assert the marks of the current config scope.
    ///
    /// Evaluating a nested schema body runs `clear_local_vars` for every schema
    /// attribute, which drops the marks of the config expression we are still
    /// walking. Re-asserting them before each entry keeps the preceding sibling
    /// entries visible to the following ones.
    #[inline]
    pub(crate) fn restore_config_entry_vars(&self) {
        if let Some(frame) = self.config_entry_vars.borrow().last() {
            let mut local_vars = self.local_vars.borrow_mut();
            for name in frame {
                local_vars.insert(name.clone());
            }
        }
    }

    #[inline]
    pub(crate) fn clear_local_vars(&self) {
        self.local_vars.borrow_mut().clear();
        self.loop_vars.borrow_mut().clear();
    }

    #[inline]
    pub(crate) fn clean_and_cloned_local_vars(&self) -> (HashSet<String>, HashSet<String>) {
        let mut local_vars = self.local_vars.borrow_mut();
        let r = local_vars.clone();
        local_vars.clear();
        let mut loop_vars = self.loop_vars.borrow_mut();
        let lr = loop_vars.clone();
        loop_vars.clear();
        (r, lr)
    }

    #[inline]
    pub(crate) fn set_local_vars(&self, vars: (HashSet<String>, HashSet<String>)) {
        // Replace (not extend) local_vars and loop_vars. The call site
        // (walk_call_expr) uses `clean_and_cloned_local_vars` to save the
        // pre-call state and clear the current state before invoking the
        // function. Extending after the call would leave any local vars
        // introduced by the called function (e.g. lambda argument names
        // added by `walk_arguments`) behind, polluting the surrounding
        // scope and breaking subsequent identifier lookups via
        // `is_local_var`/`load_name`.
        *self.local_vars.borrow_mut() = vars.0;
        *self.loop_vars.borrow_mut() = vars.1;
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
        self.runtime_ctx.borrow_mut().set_kcl_pkgpath(pkgpath);
    }

    #[inline]
    pub(crate) fn pop_pkgpath(&self) {
        if let Some(pkgpath) = self.pkgpath_stack.borrow_mut().pop() {
            self.runtime_ctx.borrow_mut().set_kcl_pkgpath(&pkgpath);
        }
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
        if ctx.cfg.debug_mode
            && let Some(backtrace_frame) = ctx.backtrace.pop()
        {
            ctx.panic_info.kcl_func = backtrace_frame.func;
            ctx.panic_info.kcl_line = backtrace_frame.line;
            ctx.panic_info.kcl_file = backtrace_frame.file;
        }
    }

    #[inline]
    pub(crate) fn push_backtrack_meta(&self, setter: &Setter) {
        let meta = &mut self.backtrack_meta.borrow_mut();
        meta.push(BacktrackMeta {
            stmt_id: Some(setter.stmt_id.clone()),
            stopped: setter.stopped.clone(),
            is_break: false,
            kind: setter.kind.clone(),
        });
    }

    #[inline]
    pub(crate) fn pop_backtrack_meta(&self) {
        let meta = &mut self.backtrack_meta.borrow_mut();
        meta.pop();
    }

    #[inline]
    pub(crate) fn is_backtrack_only_if(&self) -> bool {
        let meta = self.backtrack_meta.borrow_mut();
        let current_ast_id = self.ast_id.borrow().clone();
        meta.last()
            .map(|m| {
                let stmt_id_matches = m.stmt_id.as_ref().is_none_or(|id| current_ast_id == *id);
                matches!(m.kind, SetterKind::If) && stmt_id_matches
            })
            .unwrap_or_default()
    }

    #[inline]
    pub(crate) fn is_backtrack_only_or_else(&self) -> bool {
        let meta = self.backtrack_meta.borrow_mut();
        let current_ast_id = self.ast_id.borrow().clone();
        meta.last()
            .map(|m| {
                let stmt_id_matches = m.stmt_id.as_ref().is_none_or(|id| current_ast_id == *id);
                matches!(m.kind, SetterKind::OrElse) && stmt_id_matches
            })
            .unwrap_or_default()
    }

    pub(crate) fn push_scope_cover(&self, start: usize, stop: usize) {
        self.scope_covers.borrow_mut().push((start, stop));
    }

    pub(crate) fn pop_scope_cover(&self) {
        self.scope_covers.borrow_mut().pop();
    }
}
