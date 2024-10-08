use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use kclvm_ast::ast;
use kclvm_ast::walker::TypedResultWalker;
use kclvm_runtime::ValueRef;
use scopeguard::defer;

use crate::error as kcl_error;

use crate::proxy::{call_rule_check, call_schema_body_from_rule};
use crate::Evaluator;

pub type RuleBodyHandler =
    Arc<dyn Fn(&Evaluator, &RuleEvalContextRef, &ValueRef, &ValueRef) -> ValueRef>;

pub type RuleEvalContextRef = Rc<RefCell<RuleEvalContext>>;

/// Proxy functions represent the saved functions of the runtime its,
/// rather than executing KCL defined functions or plugin functions.
#[derive(Clone, Debug)]
pub struct RuleEvalContext {
    pub node: Rc<ast::RuleStmt>,
    pub value: ValueRef,
    pub config: ValueRef,
    pub config_meta: ValueRef,
    pub optional_mapping: ValueRef,
    pub is_sub_schema: bool,
}

impl RuleEvalContext {
    #[inline]
    pub fn new_with_node(node: ast::RuleStmt) -> Self {
        RuleEvalContext {
            node: Rc::new(node),
            value: ValueRef::dict(None),
            config: ValueRef::dict(None),
            config_meta: ValueRef::dict(None),
            optional_mapping: ValueRef::dict(None),
            is_sub_schema: true,
        }
    }

    /// New a rule evaluation context with schema value and config.
    #[inline]
    pub fn new_with_value(&self, value: &ValueRef, config: &ValueRef) -> RuleEvalContextRef {
        Rc::new(RefCell::new(Self {
            node: self.node.clone(),
            value: value.clone(),
            config: config.clone(),
            config_meta: ValueRef::dict(None),
            optional_mapping: ValueRef::dict(None),
            is_sub_schema: true,
        }))
    }

    /// Reset rule evaluation context state.
    pub fn reset(&mut self) {
        self.value = ValueRef::dict(None);
        self.config = ValueRef::dict(None);
        self.config_meta = ValueRef::dict(None);
        self.optional_mapping = ValueRef::dict(None);
        self.is_sub_schema = true;
    }

    /// Reset schema evaluation context state.
    #[inline]
    pub fn snapshot(&self, config: ValueRef, config_meta: ValueRef) -> RuleEvalContextRef {
        Rc::new(RefCell::new(Self {
            node: self.node.clone(),
            value: ValueRef::dict(None),
            config,
            config_meta,
            optional_mapping: ValueRef::dict(None),
            is_sub_schema: true,
        }))
    }
}

#[derive(Clone)]
pub struct RuleCaller {
    pub ctx: RuleEvalContextRef,
    pub body: RuleBodyHandler,
    pub check: RuleBodyHandler,
}

/// Rule function body
pub fn rule_body(
    s: &Evaluator,
    ctx: &RuleEvalContextRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    // Schema Value
    let rule_value = if let Some(for_host_name) = &ctx.borrow().node.for_host_name {
        let base_constructor_func = s
            .walk_identifier_with_ctx(&for_host_name.node, &ast::ExprContext::Load, None)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        // Call base schema function
        call_schema_body_from_rule(s, &base_constructor_func, args, kwargs, ctx)
    } else {
        ctx.borrow().value.clone()
    };
    let rule_name = &ctx.borrow().node.name.node;
    s.push_schema(crate::EvalContext::Rule(ctx.clone()));
    s.enter_scope();
    defer! {
        s.leave_scope();
        s.pop_schema();
    }
    // Evaluate arguments and keyword arguments and store values to local variables.
    s.walk_arguments(&ctx.borrow().node.args, args, kwargs);
    // Eval schema body and record schema instances.
    {
        // Rule decorators check
        for decorator in &ctx.borrow().node.decorators {
            s.walk_decorator_with_name(&decorator.node, Some(rule_name), true)
                .expect(kcl_error::RUNTIME_ERROR_MSG);
        }
    }
    // Do rule check for the sub rule.
    if ctx.borrow().is_sub_schema {
        // Call rule check block function
        rule_check(s, ctx, args, kwargs);
    }
    rule_value
}

pub fn rule_check(
    s: &Evaluator,
    ctx: &RuleEvalContextRef,
    args: &ValueRef,
    kwargs: &ValueRef,
) -> ValueRef {
    // Call base check function
    for parent_name in &ctx.borrow().node.parent_rules {
        let base_constructor_func = s
            .walk_identifier_with_ctx(&parent_name.node, &ast::ExprContext::Load, None)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
        call_rule_check(s, &base_constructor_func, args, kwargs)
    }
    // Call self check function
    for check_expr in &ctx.borrow().node.checks {
        s.walk_check_expr(&check_expr.node)
            .expect(kcl_error::RUNTIME_ERROR_MSG);
    }
    ctx.borrow().value.clone()
}
