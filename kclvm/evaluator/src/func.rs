use std::fmt::Debug;
use std::sync::Arc;

use generational_arena::Index;
use kclvm_ast::ast;
use kclvm_primitives::IndexMap;
use kclvm_runtime::ValueRef;
use scopeguard::defer;

use crate::proxy::Proxy;
use crate::ty::type_pack_and_check;
use crate::Evaluator;
use crate::{error as kcl_error, EvalContext};

pub type FunctionHandler =
    Arc<dyn Fn(&Evaluator, &FunctionEvalContext, &ValueRef, &ValueRef) -> ValueRef>;

pub type ClosureMap = IndexMap<String, ValueRef>;

pub type FunctionEvalContextRef = Arc<FunctionEvalContext>;

#[derive(Clone)]
pub struct FunctionEvalContext {
    /// AST node.
    pub node: ast::LambdaExpr,
    /// Captured schema or rule eval context.
    pub this: Option<FunctionEvalThis>,
    /// Captured closure local variables.
    pub closure: ClosureMap,
    /// The scope level of the function definition.
    pub level: usize,
}

#[derive(Clone)]
pub struct FunctionEvalThis {
    pub ctx: EvalContext,
    pub value: ValueRef,
    pub config: ValueRef,
}

impl FunctionEvalThis {
    #[inline]
    pub fn eval_ctx(&self) -> EvalContext {
        match &self.ctx {
            EvalContext::Schema(schema_ctx) => EvalContext::Schema(
                schema_ctx
                    .borrow()
                    .new_with_value(&self.value, &self.config),
            ),
            EvalContext::Rule(rule_ctx) => {
                EvalContext::Rule(rule_ctx.borrow().new_with_value(&self.value, &self.config))
            }
        }
    }
}

/// Proxy functions represent the saved functions of the runtime itself,
/// rather than executing KCL defined functions or plugin functions.
#[derive(Clone)]
pub struct FunctionCaller {
    pub ctx: FunctionEvalContextRef,
    pub body: FunctionHandler,
}

impl Debug for FunctionCaller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr_value = Arc::as_ptr(&self.body);
        f.debug_struct("FunctionProxy")
            .field("inner", &format!("{ptr_value:p}"))
            .finish()
    }
}

impl FunctionCaller {
    #[inline]
    pub fn new(ctx: FunctionEvalContext, proxy: FunctionHandler) -> Self {
        Self {
            ctx: Arc::new(ctx),
            body: proxy,
        }
    }
}

impl<'ctx> Evaluator<'ctx> {
    #[inline]
    pub(crate) fn invoke_proxy_function(
        &'ctx self,
        proxy_index: Index,
        args: &ValueRef,
        kwargs: &ValueRef,
    ) -> ValueRef {
        let frame = {
            let frames = self.frames.borrow();
            frames
                .get(proxy_index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        // Change the package path scope.
        self.push_pkgpath(&frame.pkgpath);
        // Change the backtrace metadata: filename, line, etc.
        self.push_backtrace(&frame);
        defer! {
            // Recover the backtrace metadata: filename, line, etc.
            self.pop_backtrace();
            // Recover the package path scope.
            self.pop_pkgpath();
        }
        let value = match &frame.proxy {
            // Call a function and return the value
            Proxy::Lambda(lambda) => {
                // Push the current lambda scope level in the lambda stack.
                let pkgpath = self.current_pkgpath();
                let level = self.scope_level();
                self.push_lambda(lambda.ctx.clone(), &pkgpath, &frame.pkgpath, level);
                let value = (lambda.body)(self, &lambda.ctx, args, kwargs);
                self.pop_lambda(lambda.ctx.clone(), &pkgpath, &frame.pkgpath, level);
                value
            }
            // Call a schema and return the schema value.
            Proxy::Schema(schema) => (schema.body)(
                self,
                &schema
                    .ctx
                    .borrow()
                    .snapshot(self.dict_value(), self.dict_value()),
                args,
                kwargs,
            ),
            // Call a rule and return the rule value.
            Proxy::Rule(rule) => (rule.body)(self, &rule.ctx, args, kwargs),
            // The built-in lazy eval semantics prevent invoking
            Proxy::Global(_) => self.undefined_value(),
        };
        value
    }
}

/// Lambda function body
pub fn func_body(
    s: &Evaluator,
    ctx: &FunctionEvalContext,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    s.enter_scope();
    defer! {
        s.leave_scope();
    }
    // Evaluate arguments and keyword arguments and store values to local variables.
    s.walk_arguments(&ctx.node.args, args, kwargs);
    let mut result = s
        .walk_stmts(&ctx.node.body)
        .expect(kcl_error::RUNTIME_ERROR_MSG);
    if let Some(ty) = &ctx.node.return_ty {
        result = type_pack_and_check(s, &result, vec![&ty.node.to_string()], false);
    }
    result
}
