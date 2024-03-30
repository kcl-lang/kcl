use std::fmt::Debug;
use std::sync::Arc;

use generational_arena::Index;
use kclvm_ast::ast;
use kclvm_runtime::ValueRef;

use crate::error as kcl_error;
use crate::proxy::Proxy;
use crate::Evaluator;

pub type FunctionHandler =
    Arc<dyn Fn(&Evaluator, &FunctionEvalContext, &ValueRef, &ValueRef) -> ValueRef>;

#[derive(Clone)]
pub struct FunctionEvalContext {
    pub node: ast::LambdaExpr,
}

/// Proxy functions represent the saved functions of the runtime itself,
/// rather than executing KCL defined functions or plugin functions.
#[derive(Clone)]
pub struct FunctionCaller {
    pub ctx: FunctionEvalContext,
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
        Self { ctx, body: proxy }
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
        self.push_pkgpath(&frame.pkgpath);
        let value = match &frame.proxy {
            Proxy::Lambda(lambda) => (lambda.body)(self, &lambda.ctx, args, kwargs),
            Proxy::Schema(schema) => {
                {
                    let ctx = &mut schema.ctx.borrow_mut();
                    ctx.reset_with_config(self.dict_value(), self.dict_value());
                }
                (schema.body)(self, &schema.ctx, args, kwargs)
            }
            Proxy::Rule(rule) => (rule.body)(self, &rule.ctx, args, kwargs),
        };
        self.pop_pkgpath();
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
    // Push the current lambda scope level in the lambda stack.
    s.push_lambda(s.scope_level() + 1);
    s.enter_scope();
    // Evaluate arguments and keyword arguments and store values to local variables.
    s.walk_arguments(&ctx.node.args, args, kwargs);
    let result = s
        .walk_stmts(&ctx.node.body)
        .expect(kcl_error::RUNTIME_ERROR_MSG);
    s.leave_scope();
    s.pop_lambda();
    result
}
