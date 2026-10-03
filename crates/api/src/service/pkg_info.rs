//! Helpers for the `LoadPackage` info fields: the per-file direct import
//! graph (`LoadPackageResult.imports`), the parsed package manifest
//! (`LoadPackageResult.kcl_mod`) and the application directory scan
//! (`LoadPackageResult.apps`).
//!
//! These fields deliberately replace the removed `ListDepFiles` /
//! `ListUpStreamFiles` / `ListDownStreamFiles` RPCs: bindings derive
//! upstream/downstream file sets client-side from the direct import graph.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use kcl_ast::ast::{Program, Stmt};
use kcl_config::modfile::{self, Dependency, ModFile};
use kcl_config::vfs::fix_import_path;
use kcl_parser::KCLModuleCache;
use kcl_parser::file_graph::{Pkg, PkgFile};
use kcl_utils::path::PathPrefix;

use crate::gpyrpc::{
    AppInfo, FileImports, ImportInfo, KclMod, KclModDependency, KclModGitSource, KclModLocalSource,
    KclModOciSource, KclModPackage, KclModProfile,
};

/// Directory names pruned from the application directory scan.
const PRUNED_DIR_NAMES: [&str; 2] = ["vendor", "node_modules"];
/// Defensive cap on the number of directories visited by the application
/// directory scan, so a huge or degenerate tree can not stall the RPC.
const MAX_VISITED_DIRS: usize = 10_000;

/// Build the per-file direct import graph of the loaded program.
///
/// Reads `dep_cache` of the module cache that was handed to the loader: every
/// parsed file's direct dependencies are recorded there keyed by [`PkgFile`].
/// Only files that are part of the loaded program (`program_files`) are
/// included, so a shared or reused cache can not leak foreign entries in.
///
/// `dep_cache` stores resolved dep files only, so the import specifier as
/// written in the source is recovered from the parsed AST (`Program`) and
/// matched back through `fix_import_path` against each dep's package path —
/// the same transformation the parser applies in `get_deps`. When no
/// specifier matches, `path` falls back to the resolved dep path.
pub(crate) fn collect_imports(
    module_cache: &KCLModuleCache,
    program: &Program,
    program_files: &HashSet<String>,
) -> HashMap<String, FileImports> {
    let mut imports = HashMap::new();
    let specifiers = import_specifiers_per_file(program);
    let Ok(cache) = module_cache.read() else {
        return imports;
    };
    for (file, deps) in &cache.dep_cache {
        let file_path = file.get_path().adjust_canonicalization();
        if !program_files.contains(&file_path) {
            continue;
        }
        let mut infos: Vec<ImportInfo> = deps
            .iter()
            .map(|(dep_file, dep_pkg)| {
                let resolved = dep_file.get_path().adjust_canonicalization();
                // Match the as-written specifier of this import statement by
                // package path. `fix_import_path` returns non-relative
                // specifiers unchanged, so those match exactly; relative
                // specifiers resolve against the package root, which for a
                // relative import equals the dep's own root.
                let path = specifiers
                    .get(&file_path)
                    .and_then(|specs| match_specifier(specs, &file_path, dep_file, dep_pkg))
                    .unwrap_or_else(|| resolved.clone());
                ImportInfo { path, resolved }
            })
            .collect();
        infos.sort_by(|a, b| a.path.cmp(&b.path));
        imports.insert(file_path, FileImports { imports: infos });
    }
    imports
}

/// Collect the import specifiers as written in the source, per file, from
/// the parsed program AST.
fn import_specifiers_per_file(program: &Program) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    for (filename, module) in &program.modules {
        let Ok(module) = module.read() else {
            continue;
        };
        let specs: Vec<String> = module
            .body
            .iter()
            .filter_map(|stmt| match &stmt.node {
                // `rawpath` is the specifier as written in the source; the
                // resolver rewrites `path` during import resolution.
                Stmt::Import(import) => Some(import.rawpath.clone()),
                _ => None,
            })
            .collect();
        map.insert(filename.clone(), specs);
    }
    map
}

/// Find the specifier whose resolution equals the dep's package path.
///
/// Two forms of import statements resolve to the same recorded dep:
/// * directory form (`import pkg1` -> package `pkg1`): the resolved package
///   path equals `fix_import_path(spec)` exactly;
/// * single-file form (`import base.base` -> file `base/base.k`): the parser
///   canonicalizes the dep to its containing package, so the resolved package
///   path is `fix_import_path(spec)` with the file stem segment dropped.
///
/// `fix_import_path` returns non-relative specifiers unchanged, so those
/// match exactly; relative specifiers resolve against the package root,
/// which for a relative import equals the dep's own root.
fn match_specifier(
    specs: &[String],
    importer_path: &str,
    dep_file: &PkgFile,
    dep_pkg: &Pkg,
) -> Option<String> {
    let file_stem = dep_file
        .get_path()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let single_file_pkg_path = format!("{}.{}", dep_file.pkg_path, file_stem);
    specs
        .iter()
        .find(|spec| {
            let fixed = fix_import_path(&dep_pkg.pkg_root, importer_path, spec);
            fixed == dep_file.pkg_path || fixed == single_file_pkg_path
        })
        .cloned()
}

/// Compute the package root from the first parse path and load its kcl.mod
/// manifest. Returns the root (`None` when `paths` is empty) and the parsed
/// manifest (default/empty when the root has no kcl.mod or it fails to
/// parse — never an error).
pub(crate) fn pkg_root_and_mod(paths: &[String]) -> (Option<String>, KclMod) {
    let Some(root) = paths.first().and_then(|p| modfile::get_pkg_root(p)) else {
        return (None, KclMod::default());
    };
    let kcl_mod = modfile::load_mod_file(&root)
        .map(mod_file_to_pb)
        .unwrap_or_default();
    (Some(root), kcl_mod)
}

/// Convert a parsed `kcl.mod` manifest into the protobuf shape. `None`
/// strings become empty and `None` lists become empty vectors, matching the
/// proto3 defaults.
fn mod_file_to_pb(m: ModFile) -> KclMod {
    KclMod {
        package: m.package.map(|p| KclModPackage {
            name: p.name.unwrap_or_default(),
            edition: p.edition.unwrap_or_default(),
            version: p.version.unwrap_or_default(),
            description: p.description.unwrap_or_default(),
            include: p.include.unwrap_or_default(),
            exclude: p.exclude.unwrap_or_default(),
        }),
        profile: m.profile.map(|p| KclModProfile {
            entries: p.entries.unwrap_or_default(),
            disable_none: p.disable_none.unwrap_or_default(),
            sort_keys: p.sort_keys.unwrap_or_default(),
            selectors: p.selectors.unwrap_or_default(),
            overrides: p.overrides.unwrap_or_default(),
            options: p.options.unwrap_or_default(),
        }),
        dependencies: m
            .dependencies
            .unwrap_or_default()
            .into_iter()
            .map(|(name, dep)| (name, dep_to_pb(dep)))
            .collect(),
    }
}

/// Convert an untagged toml dependency into the protobuf shape with exactly
/// one of version/git/oci/local set.
fn dep_to_pb(dep: Dependency) -> KclModDependency {
    match dep {
        Dependency::Version(version) => KclModDependency {
            version,
            ..Default::default()
        },
        Dependency::Git(git) => KclModDependency {
            git: Some(KclModGitSource {
                git: git.git,
                branch: git.branch.unwrap_or_default(),
                commit: git.commit.unwrap_or_default(),
                tag: git.tag.unwrap_or_default(),
                version: git.version.unwrap_or_default(),
            }),
            ..Default::default()
        },
        Dependency::Oci(oci) => KclModDependency {
            oci: Some(KclModOciSource {
                oci: oci.oci,
                tag: oci.tag.unwrap_or_default(),
            }),
            ..Default::default()
        },
        Dependency::Local(local) => KclModDependency {
            local: Some(KclModLocalSource { path: local.path }),
            ..Default::default()
        },
    }
}

/// Discover application directories under the package root: every directory
/// that directly contains at least one `.k` file, including the root itself.
/// Directories whose name starts with `.`, plus `vendor` and `node_modules`,
/// are pruned. The result is sorted by path. The scan is defensive: unreadable
/// directories are skipped and a degenerate tree is cut off by
/// [`MAX_VISITED_DIRS`], so it never fails the RPC.
pub(crate) fn discover_apps(root: &str) -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let mut visited = 0usize;
    walk_apps(Path::new(root), &mut visited, &mut apps);
    apps.sort_by(|a, b| a.path.cmp(&b.path));
    apps
}

fn walk_apps(dir: &Path, visited: &mut usize, apps: &mut Vec<AppInfo>) {
    if *visited >= MAX_VISITED_DIRS {
        return;
    }
    *visited += 1;
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut has_k_file = false;
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if name.starts_with('.') || PRUNED_DIR_NAMES.contains(&name.as_str()) {
                continue;
            }
            subdirs.push(path);
        } else if file_type.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some(modfile::KCL_FILE_EXTENSION)
        {
            has_k_file = true;
        }
    }
    if has_k_file {
        apps.push(AppInfo {
            path: dir.adjust_canonicalization(),
            has_kcl_mod: dir.join(modfile::KCL_MOD_FILE).is_file(),
        });
    }
    for subdir in subdirs {
        walk_apps(&subdir, visited, apps);
    }
}
