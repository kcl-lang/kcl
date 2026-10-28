//! Copyright The KCL Authors. All rights reserved.
//!
//! Every schema instantiation records the new instance in the runtime context
//! so that `Schema.instances()` can return it. The records keep every instance
//! alive until the context is dropped, even when nothing ever reads them.
//!
//! `instances` is the only way to read the records, and it is only reachable
//! by loading the attribute of that name from a schema type. KCL has no
//! computed attribute access, so the attribute name always appears in the
//! source as a dotted identifier, selector or assignment path. Native code can
//! read the records outside the AST being scanned: plugins and host-registered
//! schema constructors receive the runtime context itself, and a host can pass
//! in a function value, such as an already loaded `A.instances`, as an option
//! or by placing it in the evaluator's scopes before the run.

use std::rc::Rc;

use kcl_ast::ast;
use kcl_ast::walker::MutSelfWalker;
use kcl_runtime::{Context, Value, ValueRef};

const INSTANCES: &str = "instances";

/// Nesting depth beyond which an option value is not inspected further.
const MAX_OPTION_DEPTH: usize = 256;

/// Returns false only if no code in the program can read recorded instances.
pub(crate) fn program_may_read_instances(program: &ast::Program) -> bool {
    // Fail closed rather than blocking on an AST writer (evaluation may
    // already hold a read lock).
    let modules = program
        .modules
        .values()
        .chain(program.modules_not_imported.values())
        .map(|module| module.try_read())
        .collect::<Result<Vec<_>, _>>();
    let Ok(modules) = modules else {
        return true;
    };
    let mut scan = InstancesScan { found: false };
    for module in &modules {
        scan.walk_module(module);
        if scan.found {
            return true;
        }
    }
    false
}

/// Returns false only if the context holds no host code that the program can
/// reach: no registered schema constructors, and options that are plain data
/// held only by the context. A value the host still holds can be changed
/// during the run, for example by a panic hook. Check this before the program
/// runs, since it can copy options elsewhere.
pub(crate) fn context_may_read_instances(ctx: &Context) -> bool {
    !ctx.all_schemas.is_empty()
        || !ctx
            .option_values
            .values()
            .all(|value| is_unshared_data(value, 0))
}

/// Whether the value is plain data held only by its parent. A function value
/// can be called and may read the records. Schema values, values being mutated
/// and deeply nested values fail closed. Shared and cyclic values fail at
/// their first repeat, so each value is visited at most once.
fn is_unshared_data(value: &ValueRef, depth: usize) -> bool {
    if depth > MAX_OPTION_DEPTH || Rc::strong_count(&value.rc) != 1 {
        return false;
    }
    let Ok(value) = value.rc.try_borrow() else {
        return false;
    };
    match &*value {
        Value::list_value(list) => list.values.iter().all(|v| is_unshared_data(v, depth + 1)),
        Value::dict_value(dict) if dict.schema_args.is_none() => {
            dict.values.values().all(|v| is_unshared_data(v, depth + 1))
        }
        Value::dict_value(_) | Value::schema_value(_) | Value::func_value(_) => false,
        Value::undefined
        | Value::none
        | Value::bool_value(_)
        | Value::int_value(_)
        | Value::float_value(_)
        | Value::str_value(_)
        | Value::unit_value(..) => true,
    }
}

struct InstancesScan {
    found: bool,
}

impl MutSelfWalker for InstancesScan {
    fn walk_import_stmt(&mut self, stmt: &ast::ImportStmt) {
        if stmt.path.node.starts_with("kcl_plugin") {
            self.found = true;
        }
    }

    /// `A.instances` and `pkg.A.instances`. A leading `instances` names a
    /// variable rather than loading an attribute.
    fn walk_identifier(&mut self, identifier: &ast::Identifier) {
        if identifier.names.iter().skip(1).any(|n| n.node == INSTANCES) {
            self.found = true;
        }
    }

    /// `f().instances` and `a?.instances`: every name after the dot is an
    /// attribute.
    fn walk_selector_expr(&mut self, selector: &ast::SelectorExpr) {
        if selector.attr.node.names.iter().any(|n| n.node == INSTANCES) {
            self.found = true;
        }
        self.walk_expr(&selector.value.node);
    }

    /// Intermediate members of an assignment path are loaded.
    fn walk_target(&mut self, target: &ast::Target) {
        for path in &target.paths {
            match path {
                ast::MemberOrIndex::Member(member) if member.node == INSTANCES => {
                    self.found = true;
                }
                ast::MemberOrIndex::Index(index) => self.walk_expr(&index.node),
                ast::MemberOrIndex::Member(_) => {}
            }
        }
    }
}
