#[cfg(test)]
mod tests;

pub mod option;
pub mod util;

use anyhow::Result;
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast::Program;
use kclvm_error::{diagnostic::Range, Diagnostic};
use kclvm_parser::{load_program, KCLModuleCache, LoadProgramOptions, ParseSessionRef};
use kclvm_sema::{
    advanced_resolver::AdvancedResolver,
    core::{
        global_state::GlobalState,
        scope::{LocalSymbolScopeKind, ScopeData, ScopeRef},
        symbol::{SymbolData, SymbolRef},
    },
    namer::Namer,
    resolver::{
        resolve_program_with_opts,
        scope::{KCLScopeCache, NodeKey},
    },
    ty::{Type, TypeRef},
};
use std::path::PathBuf;

type Errors = IndexSet<Diagnostic>;

#[derive(Debug, Clone)]
pub struct LoadPackageOptions {
    pub paths: Vec<String>,
    pub load_opts: Option<LoadProgramOptions>,
    pub resolve_ast: bool,
    pub load_builtin: bool,
}

impl Default for LoadPackageOptions {
    fn default() -> Self {
        Self {
            paths: Default::default(),
            load_opts: Default::default(),
            resolve_ast: true,
            load_builtin: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Packages {
    /// AST Program
    pub program: Program,
    /// All compiled files in the package
    pub paths: Vec<PathBuf>,
    /// All Parse errors
    pub parse_errors: Errors,
    /// Type errors
    pub type_errors: Errors,
    /// Symbol information
    pub symbols: IndexMap<SymbolRef, SymbolInfo>,
    /// Scope information
    pub scopes: IndexMap<ScopeRef, ScopeInfo>,
    /// AST Node-Symbol mapping
    pub node_symbol_map: IndexMap<NodeKey, SymbolRef>,
    /// <Package path>-<Root scope> mapping
    pub pkg_scope_map: IndexMap<String, ScopeRef>,
    /// Symbol-AST Node mapping
    pub symbol_node_map: IndexMap<SymbolRef, NodeKey>,
    /// Fully qualified name mapping
    pub fully_qualified_name_map: IndexMap<String, SymbolRef>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub ty: TypeRef,
    pub name: String,
    pub range: Range,
    pub owner: Option<SymbolRef>,
    pub def: Option<SymbolRef>,
    pub attrs: Vec<SymbolRef>,
    pub is_global: bool,
}

#[derive(Debug, Clone)]
pub struct ScopeInfo {
    /// Scope kind
    pub kind: ScopeKind,
    /// Scope parent
    pub parent: Option<ScopeRef>,
    /// Scope owner
    pub owner: Option<SymbolRef>,
    /// Children scopes
    pub children: Vec<ScopeRef>,
    /// Definitions
    pub defs: Vec<SymbolRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    Package,
    Module,
    List,
    Dict,
    Quant,
    Lambda,
    SchemaDef,
    SchemaConfig,
    Value,
    Check,
}

/// load_package provides users with the ability to parse kcl program and sematic model
/// information including symbols, types, definitions, etc.
pub fn load_packages(opts: &LoadPackageOptions) -> Result<Packages> {
    load_packages_with_cache(
        opts,
        KCLModuleCache::default(),
        KCLScopeCache::default(),
        GlobalState::default(),
    )
}

/// load_package_with_cache provides users with the ability to parse kcl program and sematic model
/// information including symbols, types, definitions, etc.
pub fn load_packages_with_cache(
    opts: &LoadPackageOptions,
    module_cache: KCLModuleCache,
    scope_cache: KCLScopeCache,
    gs: GlobalState,
) -> Result<Packages> {
    let sess = ParseSessionRef::default();
    let paths: Vec<&str> = opts.paths.iter().map(|s| s.as_str()).collect();
    let parse_result = load_program(
        sess.clone(),
        &paths,
        opts.load_opts.clone(),
        Some(module_cache),
    )?;
    let parse_errors = parse_result.errors;
    let (program, type_errors, gs) = if opts.resolve_ast {
        let mut program = parse_result.program;
        let prog_scope = resolve_program_with_opts(
            &mut program,
            kclvm_sema::resolver::Options {
                merge_program: false,
                type_erasure: false,
                ..Default::default()
            },
            Some(scope_cache),
        );
        let node_ty_map = prog_scope.node_ty_map;
        let gs = Namer::find_symbols(&program, gs);
        let gs = AdvancedResolver::resolve_program(&program, gs, node_ty_map.clone());
        (program, prog_scope.handler.diagnostics.clone(), gs)
    } else {
        (parse_result.program, IndexSet::default(), gs)
    };
    let mut packages = Packages {
        program,
        paths: parse_result.paths,
        parse_errors,
        type_errors,
        ..Default::default()
    };
    if !opts.resolve_ast {
        return Ok(packages);
    }
    let symbols = gs.get_symbols();
    if opts.load_builtin {
        for (_, symbol_ref) in symbols.get_builtin_symbols() {
            if let Some(symbol) = symbols.get_symbol(*symbol_ref) {
                let def_ty = match symbol.get_definition() {
                    Some(def) => symbols
                        .get_symbol(def)
                        .unwrap()
                        .get_sema_info()
                        .ty
                        .clone()
                        .unwrap_or(Type::any_ref()),
                    None => symbol.get_sema_info().ty.clone().unwrap_or(Type::any_ref()),
                };
                let info = SymbolInfo {
                    ty: def_ty,
                    range: symbol.get_range(),
                    name: symbol.get_name(),
                    owner: symbol.get_owner(),
                    def: symbol.get_definition(),
                    attrs: symbol.get_all_attributes(symbols, None),
                    is_global: symbol.is_global(),
                };
                packages.symbols.insert(*symbol_ref, info);
            }
        }
    }
    for path in &packages.paths {
        let path_str = path
            .to_str()
            .ok_or(anyhow::anyhow!("path {} to str failed", path.display()))?;
        if let Some(files) = gs.get_sema_db().get_file_sema(path_str) {
            for symbol_ref in files.get_symbols() {
                if let Some(symbol) = symbols.get_symbol(*symbol_ref) {
                    let def_ty = match symbol.get_definition() {
                        Some(def) => symbols
                            .get_symbol(def)
                            .unwrap()
                            .get_sema_info()
                            .ty
                            .clone()
                            .unwrap_or(Type::any_ref()),
                        None => symbol.get_sema_info().ty.clone().unwrap_or(Type::any_ref()),
                    };
                    let info = SymbolInfo {
                        ty: def_ty,
                        range: symbol.get_range(),
                        name: symbol.get_name(),
                        owner: symbol.get_owner(),
                        def: symbol.get_definition(),
                        attrs: symbol.get_all_attributes(symbols, None),
                        is_global: symbol.is_global(),
                    };
                    packages.symbols.insert(*symbol_ref, info);
                }
            }
        }
    }
    let scopes = gs.get_scopes();
    for (path, scope_ref) in scopes.get_root_scope_map() {
        packages.pkg_scope_map.insert(path.clone(), *scope_ref);
        // Root scopes
        if let Some(scope_ref) = scopes.get_root_scope(path.clone()) {
            collect_scope_info(
                &mut packages.scopes,
                &scope_ref,
                scopes,
                symbols,
                ScopeKind::Package,
            );
        }
    }
    // Update package semantic mappings
    packages.node_symbol_map = symbols.get_node_symbol_map().clone();
    packages.symbol_node_map = symbols.get_symbol_node_map().clone();
    packages.fully_qualified_name_map = symbols.get_fully_qualified_name_map().clone();
    Ok(packages)
}

impl From<LocalSymbolScopeKind> for ScopeKind {
    fn from(value: LocalSymbolScopeKind) -> Self {
        match value {
            LocalSymbolScopeKind::List => ScopeKind::List,
            LocalSymbolScopeKind::Dict => ScopeKind::Dict,
            LocalSymbolScopeKind::Quant => ScopeKind::Quant,
            LocalSymbolScopeKind::Lambda => ScopeKind::Lambda,
            LocalSymbolScopeKind::SchemaDef => ScopeKind::SchemaDef,
            LocalSymbolScopeKind::SchemaConfig => ScopeKind::SchemaConfig,
            LocalSymbolScopeKind::Value => ScopeKind::Value,
            LocalSymbolScopeKind::Check => ScopeKind::Check,
        }
    }
}

fn collect_scope_info(
    scopes: &mut IndexMap<ScopeRef, ScopeInfo>,
    scope_ref: &ScopeRef,
    scope_data: &ScopeData,
    symbol_data: &SymbolData,
    kind: ScopeKind,
) {
    if let Some(scope) = scope_data.get_scope(scope_ref) {
        let kind = if let Some(scope) = scope_data.try_get_local_scope(scope_ref) {
            scope.get_kind().clone().into()
        } else {
            kind
        };
        scopes.insert(
            *scope_ref,
            ScopeInfo {
                kind,
                parent: scope.get_parent(),
                owner: scope.get_owner(),
                children: scope.get_children(),
                defs: scope
                    .get_all_defs(scope_data, symbol_data, None, false)
                    .values()
                    .copied()
                    .collect::<Vec<_>>(),
            },
        );
        for s in scope.get_children() {
            collect_scope_info(scopes, &s, scope_data, symbol_data, ScopeKind::Module);
        }
    }
}
