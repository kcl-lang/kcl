use std::path::PathBuf;

use anyhow::Result;
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast::Program;
use kclvm_error::{diagnostic::Range, Diagnostic};
use kclvm_parser::{load_program, KCLModuleCache, LoadProgramOptions, ParseSessionRef};
use kclvm_sema::{
    advanced_resolver::AdvancedResolver,
    core::{global_state::GlobalState, symbol::SymbolRef},
    namer::Namer,
    resolver::{resolve_program_with_opts, scope::NodeKey},
    ty::{Type, TypeRef},
};

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
            load_builtin: true,
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
    // Type errors
    pub type_errors: Errors,
    // Symbol-Type mapping
    pub symbols: IndexMap<SymbolRef, SymbolInfo>,
    // AST Node-Symbol mapping
    pub node_symbol_map: IndexMap<NodeKey, SymbolRef>,
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

/// load_package provides users with the ability to parse kcl program and sematic model
/// information including symbols, types, definitions, etc.
pub fn load_packages(opts: &LoadPackageOptions) -> Result<Packages> {
    let module_cache = KCLModuleCache::default();
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
            None,
        );
        let node_ty_map = prog_scope.node_ty_map;
        let gs = Namer::find_symbols(&program, GlobalState::default());
        let gs = AdvancedResolver::resolve_program(&program, gs, node_ty_map.clone());
        (program, prog_scope.handler.diagnostics.clone(), gs)
    } else {
        (
            parse_result.program,
            IndexSet::default(),
            GlobalState::default(),
        )
    };
    let mut packages = Packages {
        program,
        paths: parse_result.paths,
        parse_errors,
        type_errors,
        symbols: IndexMap::new(),
        node_symbol_map: IndexMap::new(),
    };
    if !opts.resolve_ast {
        return Ok(packages);
    }
    let symbols = gs.get_symbols();
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
                    let node_symbol_map = symbols.get_node_symbol_map();
                    for (k, s) in &node_symbol_map {
                        packages.node_symbol_map.insert(k.clone(), *s);
                    }
                }
            }
        }
    }
    Ok(packages)
}
