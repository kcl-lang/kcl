mod arg;
mod attr;
mod calculation;
mod config;
mod doc;
mod format;
pub mod global;
mod import;
mod r#loop;
mod node;
mod para;
mod schema;
pub mod scope;
mod ty;
mod ty_alias;
mod var;

#[cfg(test)]
mod tests;

use indexmap::IndexMap;
use std::{cell::RefCell, rc::Rc};

use crate::lint::{CombinedLintPass, Linter};
use crate::pre_process::pre_process_program;
use crate::resolver::scope::ScopeObject;
use crate::resolver::ty_alias::process_program_type_alias;
use crate::{resolver::scope::Scope, ty::SchemaType};
use kclvm_ast::ast::Program;
use kclvm_error::*;

use crate::ty::TypeContext;

use self::scope::{builtin_scope, ProgramScope};

/// Resolver is responsible for program semantic checking, mainly
/// including type checking and contract model checking.
pub struct Resolver<'ctx> {
    pub program: &'ctx Program,
    pub scope_map: IndexMap<String, Rc<RefCell<Scope>>>,
    pub scope: Rc<RefCell<Scope>>,
    pub scope_level: usize,
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
            ctx: Context::default(),
            options,
            handler: Handler::default(),
            linter: Linter::<CombinedLintPass>::new(),
        }
    }

    /// The check main function.
    pub(crate) fn check(&mut self, pkgpath: &str) -> ProgramScope {
        self.check_import(pkgpath);
        self.init_global_types();
        match self.program.pkgs.get(pkgpath) {
            Some(modules) => {
                for module in modules {
                    self.ctx.filename = module.filename.to_string();
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
        ProgramScope {
            scope_map: self.scope_map.clone(),
            import_names: self.ctx.import_names.clone(),
            handler: self.handler.clone(),
        }
    }

    pub(crate) fn check_and_lint(&mut self, pkgpath: &str) -> ProgramScope {
        let mut scope = self.check(pkgpath);
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
    pub schema_mapping: IndexMap<String, Rc<RefCell<SchemaType>>>,
    /// For loop local vars.
    pub local_vars: Vec<String>,
    /// Import pkgpath and name
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    /// Global names at top level of the program.
    pub global_names: IndexMap<String, IndexMap<String, Position>>,
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

/// Resolve options
#[derive(Clone, Debug, Default)]
pub struct Options {
    pub raise_err: bool,
    pub config_auto_fix: bool,
    pub lint_check: bool,
}

/// Resolve program
pub fn resolve_program(program: &mut Program) -> ProgramScope {
    pre_process_program(program);
    let mut resolver = Resolver::new(
        program,
        Options {
            raise_err: true,
            config_auto_fix: false,
            lint_check: true,
        },
    );
    resolver.resolve_import();
    let scope = resolver.check_and_lint(kclvm_ast::MAIN_PKG);
    let type_alias_mapping = resolver.ctx.type_alias_mapping.clone();
    process_program_type_alias(program, type_alias_mapping);
    scope
}
