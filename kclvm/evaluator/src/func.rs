use std::fmt::Debug;
use std::sync::Arc;

use generational_arena::Index;
use kclvm_ast::ast;
use kclvm_runtime::ValueRef;

use crate::error as kcl_error;
use crate::Evaluator;

pub type FunctionHandler =
    Arc<dyn Fn(&Evaluator, &ast::LambdaExpr, &ValueRef, &ValueRef) -> ValueRef>;

/// Proxy functions represent the saved functions of the runtime itself,
/// rather than executing KCL defined functions or plugin functions.
#[derive(Clone)]
pub struct FunctionProxy {
    lambda_expr: ast::LambdaExpr,
    inner: FunctionHandler,
}

impl Debug for FunctionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr_value = Arc::as_ptr(&self.inner);
        f.debug_struct("FunctionProxy")
            .field("inner", &format!("{ptr_value:p}"))
            .finish()
    }
}

impl FunctionProxy {
    #[inline]
    pub fn new(lambda_expr: ast::LambdaExpr, proxy: FunctionHandler) -> Self {
        Self {
            lambda_expr,
            inner: proxy,
        }
    }
}

impl<'ctx> Evaluator<'ctx> {
    #[inline]
    pub(crate) fn invoke_proxy_function(
        &self,
        proxy_index: Index,
        args: &ValueRef,
        kwargs: &ValueRef,
    ) -> ValueRef {
        let proxy = {
            let functions = self.functions.borrow();
            functions
                .get(proxy_index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        (proxy.inner)(self, &proxy.lambda_expr, args, kwargs)
    }
}
