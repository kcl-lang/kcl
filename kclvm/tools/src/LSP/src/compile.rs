use std::collections::HashSet;

use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_driver::{lookup_compile_workspace, toolchain};
use kclvm_error::Diagnostic;
use kclvm_parser::{
    entry::get_normalized_k_files_from_paths, load_program, KCLModuleCache, LoadProgramOptions,
    ParseSessionRef,
};
use kclvm_sema::{
    advanced_resolver::AdvancedResolver,
    core::global_state::GlobalState,
    namer::Namer,
    resolver::{resolve_program_with_opts, scope::KCLScopeCache},
};

use crate::{
    state::{KCLGlobalStateCache, KCLVfs},
    util::load_files_code_from_vfs,
};

pub struct Params {
    pub file: Option<String>,
    pub module_cache: Option<KCLModuleCache>,
    pub scope_cache: Option<KCLScopeCache>,
    pub vfs: Option<KCLVfs>,
    pub gs_cache: Option<KCLGlobalStateCache>,
}

pub fn compile(
    params: Params,
    files: &mut [String],
    opts: Option<LoadProgramOptions>,
) -> (IndexSet<Diagnostic>, anyhow::Result<(Program, GlobalState)>) {
    // Ignore the kcl plugin sematic check.
    let mut opts = opts.unwrap_or_default();
    opts.load_plugins = true;
    // Get input files code from vfs
    let normalized_files = match get_normalized_k_files_from_paths(files, &opts) {
        Ok(file_list) => file_list,
        Err(e) => {
            return (
                IndexSet::new(),
                Err(anyhow::anyhow!("Compile failed: {:?}", e)),
            )
        }
    };
    let normalized_files: Vec<&str> = normalized_files.iter().map(|s| s.as_str()).collect();
    // Update opt.k_code_list
    if let Some(vfs) = &params.vfs {
        let mut k_code_list = match load_files_code_from_vfs(&normalized_files, vfs) {
            Ok(code_list) => code_list,
            Err(e) => {
                return (
                    IndexSet::new(),
                    Err(anyhow::anyhow!("Compile failed: {:?}", e)),
                )
            }
        };
        opts.k_code_list.append(&mut k_code_list);
    }
    let mut diags = IndexSet::new();

    if let Some(module_cache) = params.module_cache.as_ref() {
        if let Some(file) = &params.file {
            let code = if let Some(vfs) = &params.vfs {
                match load_files_code_from_vfs(&[file.as_str()], vfs) {
                    Ok(code_list) => code_list.first().cloned(),
                    Err(_) => None,
                }
            } else {
                None
            };
            let mut module_cache_ref = module_cache.write().unwrap();
            module_cache_ref
                .invalidate_module
                .insert(file.clone(), code);
        }
    }

    let files: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    // Parser
    let sess = ParseSessionRef::default();
    let mut program = match load_program(sess.clone(), &files, Some(opts), params.module_cache) {
        Ok(r) => r.program,
        Err(e) => return (diags, Err(anyhow::anyhow!("Parse failed: {:?}", e))),
    };
    diags.extend(sess.1.read().diagnostics.clone());

    // Resolver
    if let Some(cached_scope) = params.scope_cache.as_ref() {
        if let Some(file) = &params.file {
            if let Some(mut cached_scope) = cached_scope.try_write() {
                let mut invalidate_pkg_modules = HashSet::new();
                invalidate_pkg_modules.insert(file.clone());
                cached_scope.invalidate_pkg_modules = Some(invalidate_pkg_modules);
            }
        }
    }

    let prog_scope = resolve_program_with_opts(
        &mut program,
        kclvm_sema::resolver::Options {
            merge_program: false,
            type_erasure: false,
            ..Default::default()
        },
        params.scope_cache.clone(),
    );
    diags.extend(prog_scope.handler.diagnostics);

    let mut default = GlobalState::default();
    let mut gs_ref;

    let gs = match &params.gs_cache {
        Some(cache) => match cache.try_lock() {
            Ok(locked_state) => {
                gs_ref = locked_state;
                &mut gs_ref
            }
            Err(_) => &mut default,
        },
        None => &mut default,
    };

    gs.new_or_invalidate_pkgs = match &params.scope_cache {
        Some(cache) => match cache.try_write() {
            Some(scope) => scope.invalidate_pkgs.clone(),
            None => HashSet::new(),
        },
        None => HashSet::new(),
    };
    gs.clear_cache();

    Namer::find_symbols(&program, gs);

    match AdvancedResolver::resolve_program(&program, gs, prog_scope.node_ty_map) {
        Ok(_) => (diags, Ok((program, gs.clone()))),
        Err(e) => (diags, Err(anyhow::anyhow!("Resolve failed: {:?}", e))),
    }
}

#[allow(unused)]
pub fn compile_with_params(
    params: Params,
) -> (
    IndexSet<kclvm_error::Diagnostic>,
    anyhow::Result<(Program, GlobalState)>,
) {
    let file = params.file.clone().unwrap();
    // Lookup compile workspace from the cursor file.
    let (mut files, opts, _) = lookup_compile_workspace(&toolchain::default(), &file, true);
    if !files.contains(&file) {
        files.push(file);
    }
    compile(params, &mut files, opts)
}
