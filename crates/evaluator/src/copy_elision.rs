//! Copyright The KCL Authors. All rights reserved.
//!
//! A plain assignment deep-copies its right-hand side so that a later in-place
//! mutation cannot leak into its source (kcl-lang/kcl#2080). Lambda locals can
//! share values when the entire program contains no potentially observable
//! in-place mutation.
//!
//! Deliberately do not try to prove that a mutated value is fresh. A literal's
//! entries can escape through sibling names, closures, lazy globals and schema
//! construction before a later entry mutates them. Instead, reject all path
//! writes, composite schema attributes, schema-body augmented assignments and
//! unifications, plugins, and config entries that could merge into an existing slot.
//! This sacrifices precision to keep the proof independent of alias analysis.
//!
//! Ordinary binary/augmented operators build new values or copy their left
//! operand. Dict comprehensions copy each value before merging it. Config
//! unpacking and conditional entries copy their input, but may still mutate an
//! earlier slot, so they are only allowed at the start of a config.

use std::collections::HashSet;

use kcl_ast::ast;
use kcl_ast::walker::MutSelfWalker;

/// Returns true only if no modelled mutation can reach an aliased value.
pub(crate) fn program_allows_copy_elision(program: &ast::Program) -> bool {
    // Keep a consistent view across both passes, and fail closed rather than
    // blocking on an AST writer (evaluation may already hold a read lock).
    let modules = program
        .modules
        .values()
        .chain(program.modules_not_imported.values())
        .map(|module| module.try_read())
        .collect::<Result<Vec<_>, _>>();
    let Ok(modules) = modules else {
        return false;
    };
    let mut loops = LoopVariables::default();
    for module in &modules {
        loops.walk_module(module);
    }
    let mut checker = MutationChecker {
        loop_vars: &loops.0,
        in_schema_body: false,
        safe: true,
    };
    for module in &modules {
        checker.walk_module(module);
        if !checker.safe {
            return false;
        }
    }
    true
}

/// A config key spelled as a loop variable is evaluated, not a literal name.
#[derive(Default)]
struct LoopVariables(HashSet<String>);

impl MutSelfWalker for LoopVariables {
    fn walk_comp_clause(&mut self, clause: &ast::CompClause) {
        for target in &clause.targets {
            self.0.insert(target.node.names[0].node.clone());
        }
        self.walk_expr(&clause.iter.node);
        for cond in &clause.ifs {
            self.walk_expr(&cond.node);
        }
    }

    fn walk_quant_expr(&mut self, quant: &ast::QuantExpr) {
        for variable in &quant.variables {
            self.0.insert(variable.node.names[0].node.clone());
        }
        self.walk_expr(&quant.target.node);
        self.walk_expr(&quant.test.node);
        if let Some(cond) = &quant.if_cond {
            self.walk_expr(&cond.node);
        }
    }
}

struct MutationChecker<'a> {
    loop_vars: &'a HashSet<String>,
    in_schema_body: bool,
    safe: bool,
}

impl MutationChecker<'_> {
    fn walk_entries(&mut self, entries: &[ast::NodeRef<ast::ConfigEntry>]) {
        let mut keys = HashSet::new();
        let mut unknown_key = false;
        for entry in entries {
            let key = entry.node.key.as_ref().and_then(|key| match &key.node {
                ast::Expr::Identifier(identifier)
                    if identifier.names.len() == 1
                        && !self.loop_vars.contains(&identifier.names[0].node) =>
                {
                    Some(identifier.names[0].node.as_str())
                }
                ast::Expr::StringLit(literal) => Some(literal.value.as_str()),
                _ => None,
            });
            let has_prior_slot = unknown_key || !keys.is_empty();
            let collides = unknown_key || key.is_none_or(|key| keys.contains(key));
            let indexed = entry
                .node
                .key
                .as_ref()
                .is_some_and(|key| matches!(key.node, ast::Expr::Subscript(_)));
            if indexed
                || (has_prior_slot
                    && (entry.node.key.is_none()
                        || (!matches!(entry.node.operation, ast::ConfigEntryOperation::Override)
                            && collides)))
            {
                self.safe = false;
            }
            if let Some(key) = key {
                keys.insert(key);
            } else {
                unknown_key = true;
            }
            if let Some(key) = &entry.node.key {
                self.walk_expr(&key.node);
            }
            self.walk_expr(&entry.node.value.node);
        }
    }
}

impl MutSelfWalker for MutationChecker<'_> {
    fn walk_assign_stmt(&mut self, stmt: &ast::AssignStmt) {
        if stmt
            .targets
            .iter()
            .any(|target| !target.node.paths.is_empty())
        {
            self.safe = false;
        }
        self.walk_expr(&stmt.value.node);
    }

    fn walk_aug_assign_stmt(&mut self, stmt: &ast::AugAssignStmt) {
        if self.in_schema_body || !stmt.target.node.paths.is_empty() {
            self.safe = false;
        }
        self.walk_expr(&stmt.value.node);
    }

    fn walk_unification_stmt(&mut self, stmt: &ast::UnificationStmt) {
        if self.in_schema_body || stmt.target.node.names.len() > 1 {
            self.safe = false;
        }
        self.walk_schema_expr(&stmt.value.node);
    }

    fn walk_identifier(&mut self, identifier: &ast::Identifier) {
        if matches!(identifier.ctx, ast::ExprContext::Store) && identifier.names.len() > 1 {
            self.safe = false;
        }
        // References, not just calls: a builtin may be passed or returned as
        // a function value. dict() merges entries before copying its result.
        if identifier
            .names
            .first()
            .is_some_and(|name| name.node == "dict")
        {
            self.safe = false;
        }
    }

    fn walk_import_stmt(&mut self, stmt: &ast::ImportStmt) {
        // union_all() copies its first input, but later inputs can add aliases
        // that subsequent inputs merge into. Import aliases must reject too.
        // Native plugins receive ValueRefs and can mutate their arguments.
        // Their implementations are outside the AST being checked.
        if stmt.path.node == "collection" || stmt.path.node.starts_with("kcl_plugin") {
            self.safe = false;
        }
    }

    fn walk_schema_attr(&mut self, attr: &ast::SchemaAttr) {
        if !is_scalar_type(&attr.ty.node) {
            self.safe = false;
        }
        for decorator in &attr.decorators {
            self.walk_call_expr(&decorator.node);
        }
        if let Some(value) = &attr.value {
            self.walk_expr(&value.node);
        }
    }

    fn walk_schema_stmt(&mut self, stmt: &ast::SchemaStmt) {
        let previous = std::mem::replace(&mut self.in_schema_body, true);
        if let Some(args) = &stmt.args {
            self.walk_arguments(&args.node);
        }
        if let Some(signature) = &stmt.index_signature
            && let Some(value) = &signature.node.value
        {
            self.walk_expr(&value.node);
        }
        for decorator in &stmt.decorators {
            self.walk_call_expr(&decorator.node);
        }
        for check in &stmt.checks {
            self.walk_check_expr(&check.node);
        }
        for stmt in &stmt.body {
            self.walk_stmt(&stmt.node);
        }
        self.in_schema_body = previous;
    }

    fn walk_lambda_expr(&mut self, lambda: &ast::LambdaExpr) {
        let previous = std::mem::replace(&mut self.in_schema_body, false);
        if let Some(args) = &lambda.args {
            self.walk_arguments(&args.node);
        }
        for stmt in &lambda.body {
            self.walk_stmt(&stmt.node);
        }
        self.in_schema_body = previous;
    }

    fn walk_config_expr(&mut self, config: &ast::ConfigExpr) {
        self.walk_entries(&config.items);
    }

    fn walk_config_if_entry_expr(&mut self, entry: &ast::ConfigIfEntryExpr) {
        self.walk_expr(&entry.if_cond.node);
        self.walk_entries(&entry.items);
        if let Some(orelse) = &entry.orelse {
            self.walk_expr(&orelse.node);
        }
    }
}

fn is_scalar_type(ty: &ast::Type) -> bool {
    match ty {
        ast::Type::Basic(_) | ast::Type::Literal(_) => true,
        ast::Type::Union(union) => union
            .type_elements
            .iter()
            .all(|ty| is_scalar_type(&ty.node)),
        _ => false,
    }
}
