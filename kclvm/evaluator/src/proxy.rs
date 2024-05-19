use kclvm_runtime::ValueRef;

use crate::error as kcl_error;
use crate::func::FunctionCaller;
use crate::rule::{RuleCaller, RuleEvalContextRef};
use crate::schema::{SchemaCaller, SchemaEvalContextRef};
use crate::Evaluator;

/// Caller frame with the package path. When a caller occurs,
/// it is necessary to switch the frame to ensure that a suitable
/// scope is found.
pub struct Frame {
    pub pkgpath: String,
    pub proxy: Proxy,
}

/// Caller proxy used by call_expr or schema_expr.
pub enum Proxy {
    Lambda(FunctionCaller),
    Schema(SchemaCaller),
    Rule(RuleCaller),
    Global(usize),
}

impl Proxy {
    /// Get the name of the proxy, if it is an anonymous function, returns "lambda"
    /// if it is a schema or rule, returns its name.
    #[inline]
    pub fn get_name(&self) -> String {
        match self {
            Proxy::Lambda(_) => "lambda".to_string(),
            Proxy::Schema(s) => s.ctx.borrow().node.name.node.to_string(),
            Proxy::Rule(r) => r.ctx.borrow().node.name.node.to_string(),
            Proxy::Global(index) => index.to_string(),
        }
    }
}

/// Call the associated schemas including parent schema and mixin schema
pub(crate) fn call_schema_body(
    s: &Evaluator,
    func: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
    ctx: &SchemaEvalContextRef,
) -> ValueRef {
    if let Some(index) = func.try_get_proxy() {
        let frame = {
            let frames = s.frames.borrow();
            frames
                .get(index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        if let Proxy::Schema(schema) = &frame.proxy {
            s.push_pkgpath(&frame.pkgpath);
            s.push_backtrace(&frame);
            {
                schema.ctx.borrow_mut().set_info_with_schema(&ctx.borrow())
            }
            let value = (schema.body)(s, &schema.ctx, args, kwargs);
            s.pop_backtrace();
            s.pop_pkgpath();
            value
        } else {
            ctx.borrow().value.clone()
        }
    } else {
        ctx.borrow().value.clone()
    }
}

/// Call the associated schemas including parent schema and mixin schema
pub(crate) fn call_schema_body_from_rule(
    s: &Evaluator,
    func: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
    ctx: &RuleEvalContextRef,
) -> ValueRef {
    if let Some(index) = func.try_get_proxy() {
        let frame = {
            let frames = s.frames.borrow();
            frames
                .get(index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        if let Proxy::Schema(schema) = &frame.proxy {
            s.push_pkgpath(&frame.pkgpath);
            s.push_backtrace(&frame);
            {
                schema.ctx.borrow_mut().set_info_with_rule(&ctx.borrow())
            }
            let value = (schema.body)(s, &schema.ctx, args, kwargs);
            s.pop_backtrace();
            s.pop_pkgpath();
            value
        } else {
            ctx.borrow().value.clone()
        }
    } else {
        ctx.borrow().value.clone()
    }
}

pub(crate) fn call_schema_check(
    s: &Evaluator,
    func: &ValueRef,
    args: &ValueRef,
    kwargs: &ValueRef,
    ctx: Option<&SchemaEvalContextRef>,
) {
    if let Some(index) = func.try_get_proxy() {
        let frame = {
            let frames = s.frames.borrow();
            frames
                .get(index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        if let Proxy::Schema(schema) = &frame.proxy {
            s.push_pkgpath(&frame.pkgpath);
            s.push_backtrace(&frame);
            if let Some(ctx) = ctx {
                schema.ctx.borrow_mut().set_info_with_schema(&ctx.borrow())
            }
            (schema.check)(s, &schema.ctx, args, kwargs);
            s.pop_backtrace();
            s.pop_pkgpath();
        }
    }
}

pub(crate) fn call_rule_check(s: &Evaluator, func: &ValueRef, args: &ValueRef, kwargs: &ValueRef) {
    if let Some(index) = func.try_get_proxy() {
        let frame = {
            let frames = s.frames.borrow();
            frames
                .get(index)
                .expect(kcl_error::INTERNAL_ERROR_MSG)
                .clone()
        };
        if let Proxy::Rule(rule) = &frame.proxy {
            s.push_pkgpath(&frame.pkgpath);
            s.push_backtrace(&frame);
            (rule.check)(s, &rule.ctx, args, kwargs);
            s.pop_backtrace();
            s.pop_pkgpath();
        }
    }
}
