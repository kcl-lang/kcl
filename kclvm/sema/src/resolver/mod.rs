mod arg;
mod attr;
mod calculation;
mod config;
pub mod doc;
mod format;
pub mod global;
mod import;
mod r#loop;
mod node;
mod para;
mod schema;
pub mod scope;
pub(crate) mod ty;
mod ty_alias;
mod ty_erasure;
mod var;

#[cfg(test)]
mod tests;

use indexmap::IndexMap;
use kclvm_error::diagnostic::Range;
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use crate::lint::{CombinedLintPass, Linter};
use crate::pre_process::pre_process_program;
use crate::resolver::scope::ScopeObject;
use crate::resolver::ty_alias::type_alias_pass;
use crate::resolver::ty_erasure::type_func_erasure_pass;
use crate::ty::TypeContext;
use crate::{resolver::scope::Scope, ty::SchemaType};
use kclvm_ast::ast::Program;
use kclvm_error::*;

use self::scope::{builtin_scope, KCLScopeCache, NodeTyMap, ProgramScope};

/// Resolver is responsible for program semantic checking, mainly
/// including type checking and contract model checking.
pub struct Resolver<'ctx> {
    pub program: &'ctx Program,
    pub scope_map: IndexMap<String, Rc<RefCell<Scope>>>,
    pub scope: Rc<RefCell<Scope>>,
    pub scope_level: usize,
    pub node_ty_map: NodeTyMap,
    pub builtin_scope: Rc<RefCell<Scope>>,
    pub ctx: Context,
    pub options: Options,
    pub handler: Handler,
    pub linter: Linter<CombinedLintPass>,
}

impl<'ctx> Resolver<'ctx> {
    pub fn new(program: &'ctx Program, options: Options) -> Self {
        let builtin_scope = Rc::new(RefCell::new(builtin_scope()));
        let scope = Rc::clone(&builtin_scope);
        Resolver {
            program,
            scope_map: IndexMap::default(),
            builtin_scope,
            scope,
            scope_level: 0,
            node_ty_map: IndexMap::default(),
            ctx: Context::default(),
            options,
            handler: Handler::default(),
            linter: Linter::<CombinedLintPass>::new(),
        }
    }

    /// The check main function.
    pub(crate) fn check(&mut self, pkgpath: &str) {
        self.check_import(pkgpath);
        self.init_global_types();
        match self.program.pkgs.get(pkgpath) {
            Some(modules) => {
                for module in modules {
                    self.ctx.filename = module.filename.to_string();
                    if let scope::ScopeKind::Package(files) = &mut self.scope.borrow_mut().kind {
                        files.insert(module.filename.to_string());
                    }
                    for stmt in &module.body {
                        self.stmt(&stmt);
                    }
                    if self.options.lint_check {
                        self.lint_check_module(module);
                    }
                }
            }
            None => {}
        }
    }

    pub(crate) fn check_and_lint(&mut self, pkgpath: &str) -> ProgramScope {
        self.check(pkgpath);
        let mut scope = ProgramScope {
            scope_map: self.scope_map.clone(),
            import_names: self.ctx.import_names.clone(),
            node_ty_map: self.node_ty_map.clone(),
            handler: self.handler.clone(),
        };
        self.lint_check_scope_map();
        for diag in &self.linter.handler.diagnostics {
            scope.handler.diagnostics.insert(diag.clone());
        }
        scope
    }
}

/// Resolve context
#[derive(Debug, Default)]
pub struct Context {
    /// What source file are we in.
    pub filename: String,
    /// What package path are we in.
    pub pkgpath: String,
    /// What schema are we in.
    pub schema: Option<Rc<RefCell<SchemaType>>>,
    /// What schema are we in.
    pub schema_mapping: IndexMap<String, Arc<RefCell<SchemaType>>>,
    /// For loop local vars.
    pub local_vars: Vec<String>,
    /// Import pkgpath and name
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    /// Global names at top level of the program.
    pub global_names: IndexMap<String, IndexMap<String, Range>>,
    /// Are we resolving the left value.
    pub l_value: bool,
    /// Are we resolving the statement start position.
    pub start_pos: Position,
    /// Are we resolving the statement end position.
    pub end_pos: Position,
    /// Is in lambda expression.
    pub in_lambda_expr: Vec<bool>,
    /// Current schema expr type stack
    pub config_expr_context: Vec<Option<ScopeObject>>,
    /// Type context.
    pub ty_ctx: TypeContext,
    /// Type alias mapping
    pub type_alias_mapping: IndexMap<String, IndexMap<String, String>>,
}

/// Resolve options.
/// - lint_check: whether to run lint passes
/// - resolve_val: whether to resolve and print their AST to value for some nodes.
#[derive(Clone, Debug)]
pub struct Options {
    pub lint_check: bool,
    pub resolve_val: bool,
    pub merge_program: bool,
    pub type_erasure: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            lint_check: true,
            resolve_val: false,
            merge_program: true,
            type_erasure: true,
        }
    }
}

/// Resolve program with default options.
#[inline]
pub fn resolve_program(program: &mut Program) -> ProgramScope {
    resolve_program_with_opts(program, Options::default(), None)
}

/// Resolve program with options. See [Options]
pub fn resolve_program_with_opts(
    program: &mut Program,
    opts: Options,
    cached_scope: Option<KCLScopeCache>,
) -> ProgramScope {
    pre_process_program(program, &opts);
    let mut resolver = Resolver::new(program, opts.clone());
    resolver.resolve_import();
    if let Some(cached_scope) = cached_scope.as_ref() {
        if let Ok(mut cached_scope) = cached_scope.try_lock() {
            cached_scope.update(program);
            resolver.scope_map = cached_scope.scope_map.clone();
            resolver.scope_map.remove(kclvm_ast::MAIN_PKG);
            resolver.node_ty_map = cached_scope.node_ty_map.clone()
        }
    }
    let scope = resolver.check_and_lint(kclvm_ast::MAIN_PKG);
    if let Some(cached_scope) = cached_scope.as_ref() {
        if let Ok(mut cached_scope) = cached_scope.try_lock() {
            cached_scope.update(program);
            cached_scope.scope_map = scope.scope_map.clone();
            cached_scope.node_ty_map = scope.node_ty_map.clone();
            cached_scope.scope_map.remove(kclvm_ast::MAIN_PKG);
        }
    }

    if opts.type_erasure {
        let type_alias_mapping = resolver.ctx.type_alias_mapping.clone();
        // Erase all the function type to a named type "function"
        type_func_erasure_pass(program);
        // Erase types with their type alias
        type_alias_pass(program, type_alias_mapping);
    }
    scope
}
