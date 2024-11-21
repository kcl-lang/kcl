//! Copyright The KCL Authors. All rights reserved.

pub mod entry;
pub mod file_graph;
mod lexer;
mod parser;
mod session;

#[cfg(test)]
mod tests;

extern crate kclvm_error;

use crate::entry::get_compile_entries_from_paths;
pub use crate::session::{ParseSession, ParseSessionRef};
use compiler_base_macros::bug;
use compiler_base_session::Session;
use compiler_base_span::span::new_byte_pos;
use file_graph::{toposort, Pkg, PkgFile, PkgFileGraph, PkgMap};
use indexmap::IndexMap;
use kclvm_ast::ast::Module;
use kclvm_ast::{ast, MAIN_PKG};
use kclvm_config::modfile::{get_vendor_home, KCL_FILE_EXTENSION, KCL_FILE_SUFFIX, KCL_MOD_FILE};
use kclvm_error::diagnostic::{Errors, Range};
use kclvm_error::{ErrorKind, Message, Position, Style};
use kclvm_sema::plugin::PLUGIN_MODULE_PREFIX;
use kclvm_utils::path::PathPrefix;
use kclvm_utils::pkgpath::parse_external_pkg_name;
use kclvm_utils::pkgpath::rm_external_pkg_name;

use anyhow::Result;
use lexer::parse_token_streams;
use parser::Parser;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use kclvm_span::create_session_globals_then;

#[derive(Default, Debug)]
/// [`PkgInfo`] is some basic information about a kcl package.
pub(crate) struct PkgInfo {
    /// the name of the kcl package.
    pkg_name: String,
    /// path to save the package locally. e.g. /usr/xxx
    pkg_root: String,
    /// package path. e.g. konfig.base.xxx
    pkg_path: String,
    /// The kcl files that need to be compiled in this package.
    k_files: Vec<String>,
}

impl PkgInfo {
    /// New a [`PkgInfo`].
    pub(crate) fn new(
        pkg_name: String,
        pkg_root: String,
        pkg_path: String,
        k_files: Vec<String>,
    ) -> Self {
        PkgInfo {
            pkg_name,
            pkg_root,
            pkg_path,
            k_files,
        }
    }
}

/// parser mode
#[derive(Debug, Clone)]
pub enum ParseMode {
    Null,
    ParseComments,
}

/// LoadProgramResult denotes the result of the whole program and a topological
/// ordering of all known files,
#[derive(Debug, Clone)]
pub struct LoadProgramResult {
    /// Program AST
    pub program: ast::Program,
    /// Parse errors
    pub errors: Errors,
    /// The topological ordering of all known files.
    pub paths: Vec<PathBuf>,
}

/// ParseFileResult denotes the result of a single file including AST,
/// errors and import dependencies.
#[derive(Debug, Clone)]
pub struct ParseFileResult {
    /// Module AST
    pub module: ast::Module,
    /// Parse errors
    pub errors: Errors,
    /// Dependency paths.
    pub deps: Vec<PkgFile>,
}

/// Parse a KCL file to the AST module with parse errors.
pub fn parse_single_file(filename: &str, code: Option<String>) -> Result<ParseFileResult> {
    let filename = filename.adjust_canonicalization();
    let sess = Arc::new(ParseSession::default());
    let mut loader = Loader::new(
        sess,
        &[&filename],
        Some(LoadProgramOptions {
            load_packages: false,
            k_code_list: if let Some(code) = code {
                vec![code]
            } else {
                vec![]
            },
            ..Default::default()
        }),
        None,
    );
    let result = loader.load_main()?;
    let module = match result.program.get_main_package_first_module() {
        Some(module) => module.clone(),
        None => ast::Module::default(),
    };
    let file_graph = match loader.file_graph.read() {
        Ok(file_graph) => file_graph,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to read KCL file graph. Because '{e}'"
            ))
        }
    };
    let file = PkgFile::new(PathBuf::from(filename), MAIN_PKG.to_string());
    let deps = if file_graph.contains_file(&file) {
        file_graph.dependencies_of(&file).into_iter().collect()
    } else {
        vec![]
    };
    Ok(ParseFileResult {
        module,
        errors: result.errors.clone(),
        deps,
    })
}

/// Parse a KCL file to the AST module and return errors when meets parse errors as result.
pub fn parse_file_force_errors(filename: &str, code: Option<String>) -> Result<ast::Module> {
    let sess = Arc::new(ParseSession::default());
    let result = parse_file_with_global_session(sess.clone(), filename, code);
    if sess.0.diag_handler.has_errors()? {
        let err = sess
            .0
            .emit_nth_diag_into_string(0)?
            .unwrap_or(Ok(ErrorKind::InvalidSyntax.name()))?;
        Err(anyhow::anyhow!(err))
    } else {
        result
    }
}

/// Parse a KCL file to the AST module with the parse session .
pub fn parse_file_with_session(
    sess: ParseSessionRef,
    filename: &str,
    code: Option<String>,
) -> Result<ast::Module> {
    // Code source.
    let src = if let Some(s) = code {
        s
    } else {
        match std::fs::read_to_string(filename) {
            Ok(src) => src,
            Err(err) => {
                return Err(anyhow::anyhow!(
                    "Failed to load KCL file '{filename}'. Because '{err}'"
                ));
            }
        }
    };

    // Build a source map to store file sources.
    let sf = sess
        .0
        .sm
        .new_source_file(PathBuf::from(filename).into(), src);

    let src_from_sf = match sf.src.as_ref() {
        Some(src) => src,
        None => {
            return Err(anyhow::anyhow!(
                "Internal Bug: Failed to load KCL file '{filename}'."
            ));
        }
    };

    // Lexer
    let stream = lexer::parse_token_streams(&sess, src_from_sf.as_str(), sf.start_pos);
    // Parser
    let mut p = parser::Parser::new(&sess, stream);
    let mut m = p.parse_module();
    m.filename = filename.to_string().adjust_canonicalization();

    Ok(m)
}

/// Parse a KCL file to the AST module with the parse session and the global session
#[inline]
pub fn parse_file_with_global_session(
    sess: ParseSessionRef,
    filename: &str,
    code: Option<String>,
) -> Result<ast::Module> {
    create_session_globals_then(move || parse_file_with_session(sess, filename, code))
}

/// Parse a source string to a expression. When input empty string, it will return [None].
///
/// # Examples
/// ```
/// use kclvm_ast::ast;
/// use kclvm_parser::parse_expr;
///
/// let expr = parse_expr("'alice'").unwrap();
/// assert!(matches!(expr.node, ast::Expr::StringLit(_)));
/// let expr = parse_expr("");
/// assert!(matches!(expr, None));
/// ```
pub fn parse_expr(src: &str) -> Option<ast::NodeRef<ast::Expr>> {
    if src.is_empty() {
        None
    } else {
        let sess = Arc::new(Session::default());
        let sf = sess
            .sm
            .new_source_file(PathBuf::from("").into(), src.to_string());
        let src_from_sf = match sf.src.as_ref() {
            Some(src) => src,
            None => {
                bug!("Internal Bug: Failed to load KCL file.");
            }
        };

        let sess = &&ParseSession::with_session(sess);

        let expr: Option<ast::NodeRef<ast::Expr>> = Some(create_session_globals_then(|| {
            let stream = parse_token_streams(sess, src_from_sf.as_str(), new_byte_pos(0));
            let mut parser = Parser::new(sess, stream);
            parser.parse_expr()
        }));
        expr
    }
}

#[derive(Debug, Clone)]
pub struct LoadProgramOptions {
    pub work_dir: String,
    pub k_code_list: Vec<String>,
    pub vendor_dirs: Vec<String>,
    pub package_maps: HashMap<String, String>,
    /// The parser mode.
    pub mode: ParseMode,
    /// Whether to load packages.
    pub load_packages: bool,
    /// Whether to load plugins
    pub load_plugins: bool,
}

impl Default for LoadProgramOptions {
    fn default() -> Self {
        Self {
            work_dir: Default::default(),
            k_code_list: Default::default(),
            vendor_dirs: vec![get_vendor_home()],
            package_maps: Default::default(),
            mode: ParseMode::ParseComments,
            load_packages: true,
            load_plugins: false,
        }
    }
}

/// Load the KCL program by paths and options,
/// "module_cache" is used to cache parsed asts to support incremental parse,
/// if it is None, module caching will be disabled
///
/// # Examples
///
/// ```
/// use kclvm_parser::{load_program, ParseSession};
/// use kclvm_parser::KCLModuleCache;
/// use kclvm_ast::ast::Program;
/// use std::sync::Arc;
///
/// // Create sessions
/// let sess = Arc::new(ParseSession::default());
/// // Create module cache
/// let module_cache = KCLModuleCache::default();
///
/// // Parse kcl file
/// let kcl_path = "./testdata/import-01.k";
/// let prog = load_program(sess.clone(), &[kcl_path], None, Some(module_cache.clone())).unwrap();
///     
/// ```
pub fn load_program(
    sess: ParseSessionRef,
    paths: &[&str],
    opts: Option<LoadProgramOptions>,
    module_cache: Option<KCLModuleCache>,
) -> Result<LoadProgramResult> {
    Loader::new(sess, paths, opts, module_cache).load_main()
}

pub type KCLModuleCache = Arc<RwLock<ModuleCache>>;

#[derive(Default, Debug)]
pub struct ModuleCache {
    /// File ast cache
    pub ast_cache: IndexMap<PathBuf, Arc<RwLock<ast::Module>>>,
    /// Which pkgs the file belongs to. Sometimes a file is not only contained in the pkg in the file system directory, but may also be in the main package.
    pub file_pkg: IndexMap<PathBuf, HashSet<PkgFile>>,
    /// File dependency cache
    pub dep_cache: IndexMap<PkgFile, PkgMap>,
    /// File source code
    pub source_code: IndexMap<PathBuf, String>,
}

impl ModuleCache {
    pub fn clear(&mut self, path: &PathBuf) {
        self.ast_cache.remove(path);
        self.source_code.remove(path);
        if let Some(pkgs) = self.file_pkg.remove(path) {
            for pkg in &pkgs {
                self.dep_cache.remove(pkg);
            }
        }
    }
}
struct Loader {
    sess: ParseSessionRef,
    paths: Vec<String>,
    opts: LoadProgramOptions,
    module_cache: KCLModuleCache,
    file_graph: FileGraphCache,
    pkgmap: PkgMap,
    parsed_file: HashSet<PkgFile>,
}

impl Loader {
    fn new(
        sess: ParseSessionRef,
        paths: &[&str],
        opts: Option<LoadProgramOptions>,
        module_cache: Option<KCLModuleCache>,
    ) -> Self {
        Self {
            sess,
            paths: paths
                .iter()
                .map(|s| kclvm_utils::path::convert_windows_drive_letter(s))
                .collect(),
            opts: opts.unwrap_or_default(),
            module_cache: module_cache.unwrap_or_default(),
            file_graph: FileGraphCache::default(),
            pkgmap: PkgMap::new(),
            parsed_file: HashSet::new(),
        }
    }

    #[inline]
    fn load_main(&mut self) -> Result<LoadProgramResult> {
        create_session_globals_then(move || self._load_main())
    }

    fn _load_main(&mut self) -> Result<LoadProgramResult> {
        parse_program(
            self.sess.clone(),
            self.paths.clone(),
            self.module_cache.clone(),
            self.file_graph.clone(),
            &mut self.pkgmap,
            &mut self.parsed_file,
            &self.opts,
        )
    }
}

fn fix_rel_import_path_with_file(
    pkgroot: &str,
    m: &mut ast::Module,
    file: &PkgFile,
    pkgmap: &PkgMap,
    opts: &LoadProgramOptions,
    sess: ParseSessionRef,
) {
    for stmt in &mut m.body {
        let pos = stmt.pos().clone();
        if let ast::Stmt::Import(ref mut import_spec) = &mut stmt.node {
            let fix_path = kclvm_config::vfs::fix_import_path(
                pkgroot,
                &m.filename,
                import_spec.path.node.as_str(),
            );
            import_spec.path.node = fix_path.clone();

            let pkg = pkgmap.get(file).expect("file not in pkgmap");
            import_spec.pkg_name = pkg.pkg_name.clone();
            // Load the import package source code and compile.
            let pkg_info = find_packages(
                pos.into(),
                &pkg.pkg_name,
                &pkg.pkg_root,
                &fix_path,
                opts,
                sess.clone(),
            )
            .unwrap_or(None);
            if let Some(pkg_info) = &pkg_info {
                // Add the external package name as prefix of the [`kclvm_ast::ImportStmt`]'s member [`path`].
                import_spec.path.node = pkg_info.pkg_path.to_string();
                import_spec.pkg_name = pkg_info.pkg_name.clone();
            }
        }
    }
}

fn is_plugin_pkg(pkgpath: &str) -> bool {
    pkgpath.starts_with(PLUGIN_MODULE_PREFIX)
}

fn is_builtin_pkg(pkgpath: &str) -> bool {
    let system_modules = kclvm_sema::builtin::system_module::STANDARD_SYSTEM_MODULES;
    system_modules.contains(&pkgpath)
}

fn find_packages(
    pos: ast::Pos,
    pkg_name: &str,
    pkg_root: &str,
    pkg_path: &str,
    opts: &LoadProgramOptions,
    sess: ParseSessionRef,
) -> Result<Option<PkgInfo>> {
    if pkg_path.is_empty() {
        return Ok(None);
    }

    // plugin pkgs
    if is_plugin_pkg(pkg_path) {
        if !opts.load_plugins {
            sess.1.write().add_error(
                ErrorKind::CannotFindModule,
                &[Message {
                    range: Into::<Range>::into(pos),
                    style: Style::Line,
                    message: format!("the plugin package `{}` is not found, please confirm if plugin mode is enabled", pkg_path),
                    note: None,
                    suggested_replacement: None,
                }],
            );
        }
        return Ok(None);
    }

    // builtin pkgs
    if is_builtin_pkg(pkg_path) {
        return Ok(None);
    }

    // 1. Look for in the current package's directory.
    let is_internal = is_internal_pkg(pkg_name, pkg_root, pkg_path)?;
    // 2. Look for in the vendor path.
    let is_external = is_external_pkg(pkg_path, opts)?;

    // 3. Internal and external packages cannot be duplicated
    if is_external.is_some() && is_internal.is_some() {
        sess.1.write().add_error(
            ErrorKind::CannotFindModule,
            &[Message {
                range: Into::<Range>::into(pos),
                style: Style::Line,
                message: format!(
                    "the `{}` is found multiple times in the current package and vendor package",
                    pkg_path
                ),
                note: None,
                suggested_replacement: None,
            }],
        );
        return Ok(None);
    }

    // 4. Get package information based on whether the package is internal or external.

    match is_internal.or(is_external) {
        Some(pkg_info) => Ok(Some(pkg_info)),
        None => {
            sess.1.write().add_error(
                ErrorKind::CannotFindModule,
                &[Message {
                    range: Into::<Range>::into(pos),
                    style: Style::Line,
                    message: format!("pkgpath {} not found in the program", pkg_path),
                    note: None,
                    suggested_replacement: None,
                }],
            );
            let mut suggestions = vec![format!("browse more packages at 'https://artifacthub.io'")];

            if let Ok(pkg_name) = parse_external_pkg_name(pkg_path) {
                suggestions.insert(
                    0,
                    format!(
                        "try 'kcl mod add {}' to download the missing package",
                        pkg_name
                    ),
                );
            }
            sess.1.write().add_suggestions(suggestions);
            Ok(None)
        }
    }
}

/// Search [`pkgpath`] among all the paths in [`pkgroots`].
///
/// # Notes
///
/// All paths in [`pkgpath`] must contain the kcl.mod file.
/// It returns the parent directory of kcl.mod if present, or none if not.
fn pkg_exists(pkgroots: &[String], pkgpath: &str) -> Option<String> {
    pkgroots
        .into_iter()
        .find(|root| pkg_exists_in_path(root, pkgpath))
        .cloned()
}

/// Search for [`pkgpath`] under [`path`].
/// It only returns [`true`] if [`path`]/[`pkgpath`] or [`path`]/[`pkgpath.k`] exists.
fn pkg_exists_in_path(path: &str, pkgpath: &str) -> bool {
    let mut pathbuf = PathBuf::from(path);
    pkgpath.split('.').for_each(|s| pathbuf.push(s));
    pathbuf.exists() || pathbuf.with_extension(KCL_FILE_EXTENSION).exists()
}

/// Look for [`pkgpath`] in the current package's [`pkgroot`].
/// If found, return to the [`PkgInfo`]， else return [`None`]
///
/// # Error
///
/// [`is_internal_pkg`] will return an error if the package's source files cannot be found.
fn is_internal_pkg(pkg_name: &str, pkg_root: &str, pkg_path: &str) -> Result<Option<PkgInfo>> {
    match pkg_exists(&[pkg_root.to_string()], pkg_path) {
        Some(internal_pkg_root) => {
            let fullpath = if pkg_name == kclvm_ast::MAIN_PKG {
                pkg_path.to_string()
            } else {
                format!("{}.{}", pkg_name, pkg_path)
            };
            let k_files = get_pkg_kfile_list(pkg_root, pkg_path)?;
            Ok(Some(PkgInfo::new(
                pkg_name.to_string(),
                internal_pkg_root,
                fullpath,
                k_files,
            )))
        }
        None => Ok(None),
    }
}

fn get_pkg_kfile_list(pkgroot: &str, pkgpath: &str) -> Result<Vec<String>> {
    // plugin pkgs
    if is_plugin_pkg(pkgpath) {
        return Ok(Vec::new());
    }

    // builtin pkgs
    if is_builtin_pkg(pkgpath) {
        return Ok(Vec::new());
    }

    if pkgroot.is_empty() {
        return Err(anyhow::anyhow!(format!("pkgroot not found: {:?}", pkgpath)));
    }

    let mut pathbuf = std::path::PathBuf::new();
    pathbuf.push(pkgroot);

    for s in pkgpath.split('.') {
        pathbuf.push(s);
    }

    let abspath = match pathbuf.canonicalize() {
        Ok(p) => p.to_str().unwrap().to_string(),
        Err(_) => pathbuf.as_path().to_str().unwrap().to_string(),
    };
    if std::path::Path::new(abspath.as_str()).exists() {
        return get_dir_files(abspath.as_str());
    }

    let as_k_path = abspath + KCL_FILE_SUFFIX;
    if std::path::Path::new((as_k_path).as_str()).exists() {
        return Ok(vec![as_k_path]);
    }

    Ok(Vec::new())
}

/// Get file list in the directory.
fn get_dir_files(dir: &str) -> Result<Vec<String>> {
    if !std::path::Path::new(dir).exists() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    for path in std::fs::read_dir(dir)? {
        let path = path?;
        if !path
            .file_name()
            .to_str()
            .unwrap()
            .ends_with(KCL_FILE_SUFFIX)
        {
            continue;
        }
        if path.file_name().to_str().unwrap().ends_with("_test.k") {
            continue;
        }
        if path.file_name().to_str().unwrap().starts_with('_') {
            continue;
        }

        let s = format!("{}", path.path().display());
        list.push(s);
    }

    list.sort();
    Ok(list)
}

/// Look for [`pkgpath`] in the external package's home.
/// If found, return to the [`PkgInfo`]， else return [`None`]
///
/// # Error
///
/// - [`is_external_pkg`] will return an error if the package's source files cannot be found.
/// - The name of the external package could not be resolved from [`pkg_path`].
fn is_external_pkg(pkg_path: &str, opts: &LoadProgramOptions) -> Result<Option<PkgInfo>> {
    let pkg_name = parse_external_pkg_name(pkg_path)?;
    let external_pkg_root = if let Some(root) = opts.package_maps.get(&pkg_name) {
        PathBuf::from(root).join(KCL_MOD_FILE)
    } else {
        match pkg_exists(&opts.vendor_dirs, pkg_path) {
            Some(path) => PathBuf::from(path).join(&pkg_name).join(KCL_MOD_FILE),
            None => return Ok(None),
        }
    };

    if external_pkg_root.exists() {
        return Ok(Some(match external_pkg_root.parent() {
            Some(root) => {
                let abs_root: String = match root.canonicalize() {
                    Ok(p) => p.to_str().unwrap().to_string(),
                    Err(_) => root.display().to_string(),
                };
                let k_files = get_pkg_kfile_list(&abs_root, &rm_external_pkg_name(pkg_path)?)?;
                PkgInfo::new(
                    pkg_name.to_string(),
                    abs_root,
                    pkg_path.to_string(),
                    k_files,
                )
            }
            None => return Ok(None),
        }));
    } else {
        Ok(None)
    }
}

pub type ASTCache = Arc<RwLock<IndexMap<PathBuf, Arc<ast::Module>>>>;
pub type FileGraphCache = Arc<RwLock<PkgFileGraph>>;

pub fn parse_file(
    sess: ParseSessionRef,
    file: PkgFile,
    src: Option<String>,
    module_cache: KCLModuleCache,
    pkgs: &mut HashMap<String, Vec<String>>,
    pkgmap: &mut PkgMap,
    file_graph: FileGraphCache,
    opts: &LoadProgramOptions,
) -> Result<Vec<PkgFile>> {
    let src = match src {
        Some(src) => Some(src),
        None => match &module_cache.read() {
            Ok(cache) => cache.source_code.get(file.get_path()),
            Err(_) => None,
        }
        .cloned(),
    };
    let m = parse_file_with_session(sess.clone(), file.get_path().to_str().unwrap(), src)?;
    let deps = get_deps(&file, &m, pkgs, pkgmap, opts, sess)?;
    let dep_files = deps.keys().map(|f| f.clone()).collect();
    pkgmap.extend(deps.clone());
    match &mut module_cache.write() {
        Ok(module_cache) => {
            module_cache
                .ast_cache
                .insert(file.get_path().clone(), Arc::new(RwLock::new(m)));
            match module_cache.file_pkg.get_mut(&file.get_path().clone()) {
                Some(s) => {
                    s.insert(file.clone());
                }
                None => {
                    let mut s = HashSet::new();
                    s.insert(file.clone());
                    module_cache.file_pkg.insert(file.get_path().clone(), s);
                }
            }
            module_cache.dep_cache.insert(file.clone(), deps);
        }
        Err(e) => return Err(anyhow::anyhow!("Parse file failed: {e}")),
    }

    match &mut file_graph.write() {
        Ok(file_graph) => {
            file_graph.update_file(&file, &dep_files);
        }
        Err(e) => return Err(anyhow::anyhow!("Parse file failed: {e}")),
    }

    Ok(dep_files)
}

pub fn get_deps(
    file: &PkgFile,
    m: &Module,
    pkgs: &mut HashMap<String, Vec<String>>,
    pkgmap: &PkgMap,
    opts: &LoadProgramOptions,
    sess: ParseSessionRef,
) -> Result<PkgMap> {
    let mut deps = PkgMap::default();
    for stmt in &m.body {
        let pos = stmt.pos().clone();
        let pkg = pkgmap.get(file).expect("file not in pkgmap").clone();
        if let ast::Stmt::Import(import_spec) = &stmt.node {
            let fix_path = kclvm_config::vfs::fix_import_path(
                &pkg.pkg_root,
                &m.filename,
                import_spec.path.node.as_str(),
            );
            let pkg_info = find_packages(
                pos.into(),
                &pkg.pkg_name,
                &pkg.pkg_root,
                &fix_path,
                opts,
                sess.clone(),
            )?;
            if let Some(pkg_info) = &pkg_info {
                // If k_files is empty, the pkg information will not be found in the file graph.
                // Record the empty pkg to prevent loss. After the parse file is completed, fill in the modules
                if pkg_info.k_files.is_empty() {
                    pkgs.insert(pkg_info.pkg_path.clone(), vec![]);
                }

                pkg_info.k_files.iter().for_each(|p| {
                    let file = PkgFile::new(p.into(), pkg_info.pkg_path.clone());
                    deps.insert(
                        file.clone(),
                        file_graph::Pkg {
                            pkg_name: pkg_info.pkg_name.clone(),
                            pkg_root: pkg_info.pkg_root.clone().into(),
                        },
                    );
                });
            }
        }
    }
    Ok(deps)
}

pub fn parse_pkg(
    sess: ParseSessionRef,
    files: Vec<(PkgFile, Option<String>)>,
    module_cache: KCLModuleCache,
    pkgs: &mut HashMap<String, Vec<String>>,
    pkgmap: &mut PkgMap,
    file_graph: FileGraphCache,
    opts: &LoadProgramOptions,
) -> Result<Vec<PkgFile>> {
    let mut dependent = vec![];
    for (file, src) in files {
        let deps = parse_file(
            sess.clone(),
            file.clone(),
            src,
            module_cache.clone(),
            pkgs,
            pkgmap,
            file_graph.clone(),
            opts,
        )?;
        dependent.extend(deps);
    }
    Ok(dependent)
}

pub fn parse_entry(
    sess: ParseSessionRef,
    entry: &entry::Entry,
    module_cache: KCLModuleCache,
    pkgs: &mut HashMap<String, Vec<String>>,
    pkgmap: &mut PkgMap,
    file_graph: FileGraphCache,
    opts: &LoadProgramOptions,
    parsed_file: &mut HashSet<PkgFile>,
) -> Result<HashSet<PkgFile>> {
    let k_files = entry.get_k_files();
    let maybe_k_codes = entry.get_k_codes();
    let mut files = vec![];
    let mut new_files = HashSet::new();
    for (i, f) in k_files.iter().enumerate() {
        let file = PkgFile::new(f.adjust_canonicalization().into(), MAIN_PKG.to_string());
        files.push((file.clone(), maybe_k_codes.get(i).unwrap_or(&None).clone()));
        new_files.insert(file.clone());
        pkgmap.insert(
            file,
            Pkg {
                pkg_name: entry.name().clone(),
                pkg_root: entry.path().into(),
            },
        );
    }
    let dependent_paths = parse_pkg(
        sess.clone(),
        files,
        module_cache.clone(),
        pkgs,
        pkgmap,
        file_graph.clone(),
        opts,
    )?;
    let mut unparsed_file: VecDeque<PkgFile> = dependent_paths.into();

    // Bfs unparsed and import files
    while let Some(file) = unparsed_file.pop_front() {
        match &mut module_cache.write() {
            Ok(m_cache) => match m_cache.file_pkg.get_mut(file.get_path()) {
                Some(s) => {
                    // The module ast has been parsed, but does not belong to the same package
                    if s.insert(file.clone()) {
                        new_files.insert(file.clone());
                    }
                }
                None => {
                    let mut s = HashSet::new();
                    s.insert(file.clone());
                    m_cache.file_pkg.insert(file.get_path().clone(), s);
                    new_files.insert(file.clone());
                }
            },
            Err(e) => return Err(anyhow::anyhow!("Parse file failed: {e}")),
        }

        let module_cache_read = module_cache.read();
        match &module_cache_read {
            Ok(m_cache) => match m_cache.ast_cache.get(file.get_path()) {
                Some(m) => {
                    let deps = m_cache.dep_cache.get(&file).cloned().unwrap_or_else(|| {
                        get_deps(&file, &m.read().unwrap(), pkgs, pkgmap, opts, sess.clone())
                            .unwrap()
                    });
                    let dep_files: Vec<PkgFile> = deps.keys().map(|f| f.clone()).collect();
                    pkgmap.extend(deps.clone());

                    match &mut file_graph.write() {
                        Ok(file_graph) => {
                            file_graph.update_file(&file, &dep_files);

                            for dep in dep_files {
                                if parsed_file.insert(dep.clone()) {
                                    unparsed_file.push_back(dep.clone());
                                }
                            }

                            continue;
                        }
                        Err(e) => return Err(anyhow::anyhow!("Parse entry failed: {e}")),
                    }
                }
                None => {
                    new_files.insert(file.clone());
                    drop(module_cache_read);
                    let deps = parse_file(
                        sess.clone(),
                        file,
                        None,
                        module_cache.clone(),
                        pkgs,
                        pkgmap,
                        file_graph.clone(),
                        &opts,
                    )?;
                    for dep in deps {
                        if parsed_file.insert(dep.clone()) {
                            unparsed_file.push_back(dep.clone());
                        }
                    }
                }
            },
            Err(e) => return Err(anyhow::anyhow!("Parse entry failed: {e}")),
        };
    }
    Ok(new_files)
}

pub fn parse_program(
    sess: ParseSessionRef,
    paths: Vec<String>,
    module_cache: KCLModuleCache,
    file_graph: FileGraphCache,
    pkgmap: &mut PkgMap,
    parsed_file: &mut HashSet<PkgFile>,
    opts: &LoadProgramOptions,
) -> Result<LoadProgramResult> {
    let compile_entries = get_compile_entries_from_paths(&paths, &opts)?;
    let workdir = compile_entries.get_root_path().to_string();
    let mut pkgs: HashMap<String, Vec<String>> = HashMap::new();
    let mut new_files = HashSet::new();
    for entry in compile_entries.iter() {
        new_files.extend(parse_entry(
            sess.clone(),
            entry,
            module_cache.clone(),
            &mut pkgs,
            pkgmap,
            file_graph.clone(),
            &opts,
            parsed_file,
        )?);
    }

    let files = match file_graph.read() {
        Ok(file_graph) => {
            let files = match file_graph.toposort() {
                Ok(files) => files,
                Err(_) => file_graph.paths(),
            };

            let file_path_graph = file_graph.file_path_graph().0;
            if let Err(cycle) = toposort(&file_path_graph) {
                let formatted_cycle = cycle
                    .iter()
                    .map(|file| format!("- {}\n", file.to_string_lossy()))
                    .collect::<String>();

                sess.1.write().add_error(
                    ErrorKind::RecursiveLoad,
                    &[Message {
                        range: (Position::dummy_pos(), Position::dummy_pos()),
                        style: Style::Line,
                        message: format!(
                            "Could not compiles due to cyclic import statements\n{}",
                            formatted_cycle.trim_end()
                        ),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
            }
            files
        }
        Err(e) => return Err(anyhow::anyhow!("Parse program failed: {e}")),
    };

    let mut modules: HashMap<String, Arc<RwLock<Module>>> = HashMap::new();
    for file in files.iter() {
        let filename = file.get_path().to_str().unwrap().to_string();
        let m_ref = match module_cache.read() {
            Ok(module_cache) => module_cache
                .ast_cache
                .get(file.get_path())
                .expect(&format!(
                    "Module not found in module: {:?}",
                    file.get_path()
                ))
                .clone(),
            Err(e) => return Err(anyhow::anyhow!("Parse program failed: {e}")),
        };
        if new_files.contains(file) {
            let pkg = pkgmap.get(file).expect("file not in pkgmap");
            let mut m = m_ref.write().unwrap();
            fix_rel_import_path_with_file(&pkg.pkg_root, &mut m, file, &pkgmap, opts, sess.clone());
        }
        modules.insert(filename.clone(), m_ref);
        match pkgs.get_mut(&file.pkg_path) {
            Some(pkg_modules) => {
                pkg_modules.push(filename.clone());
            }
            None => {
                pkgs.insert(file.pkg_path.clone(), vec![filename]);
            }
        }
    }
    let program = ast::Program {
        root: workdir,
        pkgs,
        pkgs_not_imported: HashMap::new(),
        modules,
        modules_not_imported: HashMap::new(),
    };

    Ok(LoadProgramResult {
        program,
        errors: sess.1.read().diagnostics.clone(),
        paths: files.iter().map(|file| file.get_path().clone()).collect(),
    })
}

/// Parse all kcl files under path and dependencies from opts.
/// Different from `load_program`, this function will compile files that are not imported.
pub fn load_all_files_under_paths(
    sess: ParseSessionRef,
    paths: &[&str],
    opts: Option<LoadProgramOptions>,
    module_cache: Option<KCLModuleCache>,
) -> Result<LoadProgramResult> {
    let mut loader = Loader::new(sess.clone(), paths, opts.clone(), module_cache.clone());
    create_session_globals_then(move || {
        match parse_program(
            loader.sess.clone(),
            loader.paths.clone(),
            loader.module_cache.clone(),
            loader.file_graph.clone(),
            &mut loader.pkgmap,
            &mut loader.parsed_file,
            &loader.opts,
        ) {
            Ok(res) => {
                let diag = sess.1.read().diagnostics.clone();
                let mut res = res.clone();
                let k_files_from_import = res.paths.clone();
                let mut paths = paths.to_vec();
                paths.push(&res.program.root);
                let (k_files_under_path, pkgmap) =
                    get_files_from_path(&res.program.root, &paths, opts)?;
                loader.pkgmap.extend(pkgmap);

                // Filter unparsed file
                let mut unparsed_file: VecDeque<PkgFile> = VecDeque::new();
                for (pkg, paths) in &k_files_under_path {
                    for p in paths {
                        if !k_files_from_import.contains(p) {
                            let pkgfile = PkgFile::new(p.clone(), pkg.clone());
                            unparsed_file.push_back(pkgfile);
                        }
                    }
                }

                let module_cache = module_cache.unwrap_or_default();
                let pkgs_not_imported = &mut res.program.pkgs_not_imported;

                let mut new_files = HashSet::new();

                // Bfs unparsed and import files
                while let Some(file) = unparsed_file.pop_front() {
                    new_files.insert(file.clone());

                    let module_cache_read = module_cache.read();
                    match &module_cache_read {
                        Ok(m_cache) => match m_cache.ast_cache.get(file.get_path()) {
                            Some(_) => continue,
                            None => {
                                drop(module_cache_read);
                                let deps = parse_file(
                                    sess.clone(),
                                    file.clone(),
                                    None,
                                    module_cache.clone(),
                                    pkgs_not_imported,
                                    &mut loader.pkgmap,
                                    loader.file_graph.clone(),
                                    &loader.opts,
                                )?;

                                let m_ref = match module_cache.read() {
                                    Ok(module_cache) => module_cache
                                        .ast_cache
                                        .get(file.get_path())
                                        .expect(&format!(
                                            "Module not found in module: {:?}",
                                            file.get_path()
                                        ))
                                        .clone(),
                                    Err(e) => {
                                        return Err(anyhow::anyhow!("Parse program failed: {e}"))
                                    }
                                };

                                let pkg = loader.pkgmap.get(&file).expect("file not in pkgmap");
                                let mut m = m_ref.write().unwrap();
                                fix_rel_import_path_with_file(
                                    &pkg.pkg_root,
                                    &mut m,
                                    &file,
                                    &loader.pkgmap,
                                    &loader.opts,
                                    sess.clone(),
                                );

                                for dep in deps {
                                    if loader.parsed_file.insert(dep.clone()) {
                                        unparsed_file.push_back(dep.clone());
                                    }
                                }
                            }
                        },
                        Err(e) => return Err(anyhow::anyhow!("Parse entry failed: {e}")),
                    }
                }

                // Merge unparsed module into res
                let modules_not_imported = &mut res.program.modules_not_imported;
                for file in &new_files {
                    let filename = file.get_path().to_str().unwrap().to_string();
                    let m_ref = match module_cache.read() {
                        Ok(module_cache) => module_cache
                            .ast_cache
                            .get(file.get_path())
                            .expect(&format!(
                                "Module not found in module: {:?}",
                                file.get_path()
                            ))
                            .clone(),
                        Err(e) => return Err(anyhow::anyhow!("Parse program failed: {e}")),
                    };
                    modules_not_imported.insert(filename.clone(), m_ref);
                    match pkgs_not_imported.get_mut(&file.pkg_path) {
                        Some(pkg_modules) => {
                            pkg_modules.push(filename.clone());
                        }
                        None => {
                            pkgs_not_imported.insert(file.pkg_path.clone(), vec![filename]);
                        }
                    }
                }
                sess.1.write().diagnostics = diag;
                return Ok(res);
            }
            e => return e,
        }
    })
}

/// Get all kcl files under path and dependencies from opts, regardless of whether they are imported or not
pub fn get_files_from_path(
    root: &str,
    paths: &[&str],
    opts: Option<LoadProgramOptions>,
) -> Result<(HashMap<String, Vec<PathBuf>>, HashMap<PkgFile, Pkg>)> {
    let mut k_files_under_path = HashMap::new();
    let mut pkgmap = HashMap::new();

    // get files from config
    if let Some(opt) = &opts {
        for (name, path) in &opt.package_maps {
            let path_buf = PathBuf::from(path.clone());
            if path_buf.is_dir() {
                let all_k_files_under_path = get_kcl_files(path.clone(), true)?;
                for f in &all_k_files_under_path {
                    let p = PathBuf::from(f);
                    let fix_path = {
                        match p.parent().unwrap().strip_prefix(Path::new(&path)) {
                            Ok(p) => Path::new(&name).join(p),
                            Err(_) => match p.parent().unwrap().strip_prefix(Path::new(&path)) {
                                Ok(p) => Path::new(&name).join(p),
                                Err(_) => Path::new(&name).to_path_buf(),
                            },
                        }
                    }
                    .to_str()
                    .unwrap()
                    .to_string();
                    let mut fix_path = fix_path
                        .replace(['/', '\\'], ".")
                        .trim_end_matches('.')
                        .to_string();

                    if fix_path.is_empty() {
                        fix_path = MAIN_PKG.to_string();
                    }

                    let pkgfile = PkgFile::new(p.clone(), fix_path.clone());
                    pkgmap.insert(
                        pkgfile,
                        Pkg {
                            pkg_name: name.clone(),
                            pkg_root: path.clone(),
                        },
                    );
                    k_files_under_path
                        .entry(fix_path)
                        .or_insert(Vec::new())
                        .push(p);
                }
            }
        }
    }

    // get files from input paths
    for path in paths {
        let path_buf = PathBuf::from(path);
        if path_buf.is_dir() {
            let all_k_files_under_path = get_kcl_files(path, true)?;
            for f in &all_k_files_under_path {
                let p = PathBuf::from(f);

                let fix_path = p
                    .parent()
                    .unwrap()
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                let fix_path = fix_path
                    .replace(['/', '\\'], ".")
                    .trim_end_matches('.')
                    .to_string();

                let pkgfile = PkgFile::new(p.clone(), fix_path.clone());
                pkgmap.insert(
                    pkgfile,
                    Pkg {
                        pkg_name: MAIN_PKG.to_owned(),
                        pkg_root: path.to_string(),
                    },
                );
                k_files_under_path
                    .entry(fix_path)
                    .or_insert(Vec::new())
                    .push(p);
            }
        }
    }

    Ok((k_files_under_path, pkgmap))
}

/// Get kcl files from path.
pub fn get_kcl_files<P: AsRef<std::path::Path>>(path: P, recursively: bool) -> Result<Vec<String>> {
    let mut files = vec![];
    let walkdir = if recursively {
        walkdir::WalkDir::new(path)
    } else {
        walkdir::WalkDir::new(path).max_depth(1)
    };
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let file = path.to_str().unwrap();
            if file.ends_with(KCL_FILE_SUFFIX) {
                files.push(file.to_string())
            }
        }
    }
    files.sort();
    Ok(files)
}
