use kclvm_runtime::ValueRef;

use crate::error as kcl_error;
use crate::func::FunctionCaller;
use crate::rule::RuleCaller;
use crate::schema::SchemaCaller;
use crate::Evaluator;

/// Caller proxy used by call_expr or schema_expr.
pub enum Proxy {
    Lambda(FunctionCaller),
    Schema(SchemaCaller),
    Rule(RuleCaller),
}

pub(crate) fn call_schema_body(
    s: &Evaluator,
    func: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    if let Some(index) = func.try_get_proxy() {
        let proxy = {
            let proxies = s.proxies.borrow();
            proxies
                .get(index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        if let Proxy::Schema(schema) = proxy.as_ref() {
            (schema.body)(s, &schema.ctx, args, kwargs)
        } else {
            s.undefined_value()
        }
    } else {
        s.undefined_value()
    }
}

pub(crate) fn call_schema_check(
    s: &Evaluator,
    func: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) {
    if let Some(index) = func.try_get_proxy() {
        let proxy = {
            let proxies = s.proxies.borrow();
            proxies
                .get(index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        if let Proxy::Schema(schema) = proxy.as_ref() {
            (schema.check)(s, &schema.ctx, args, kwargs);
        }
    }
}

pub(crate) fn call_rule_check(s: &Evaluator, func: &ValueRef, args: &ValueRef, kwargs: &ValueRef) {
    if let Some(index) = func.try_get_proxy() {
        let proxy = {
            let proxies = s.proxies.borrow();
            proxies
                .get(index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        if let Proxy::Rule(rule) = proxy.as_ref() {
            (rule.check)(s, &rule.ctx, args, kwargs);
        }
    }
}
