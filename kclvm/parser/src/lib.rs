// Copyright 2021 The KCL Authors. All rights reserved.

pub mod entry;
mod lexer;
mod parser;
mod session;

#[cfg(test)]
mod tests;

extern crate kclvm_error;

use crate::entry::get_compile_entries_from_paths;
pub use crate::session::ParseSession;
use compiler_base_macros::bug;
use compiler_base_session::Session;
use compiler_base_span::span::new_byte_pos;
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_config::modfile::{get_vendor_home, KCL_FILE_EXTENSION, KCL_FILE_SUFFIX, KCL_MOD_FILE};
use kclvm_error::diagnostic::Range;
use kclvm_error::{ErrorKind, Message, Style};
use kclvm_sema::plugin::PLUGIN_MODULE_PREFIX;
use kclvm_utils::path::PathPrefix;
use kclvm_utils::pkgpath::parse_external_pkg_name;
use kclvm_utils::pkgpath::rm_external_pkg_name;

use lexer::parse_token_streams;
use parser::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
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

/// Parse a KCL file to the AST Program.
pub fn parse_program(filename: &str) -> Result<ast::Program, String> {
    let abspath = std::fs::canonicalize(std::path::PathBuf::from(filename)).unwrap();

    let mut prog = ast::Program {
        root: abspath.parent().unwrap().adjust_canonicalization(),
        main: kclvm_ast::MAIN_PKG.to_string(),
        pkgs: HashMap::new(),
    };

    let mut module = parse_file(abspath.to_str().unwrap(), None)?;
    module.filename = filename.to_string();
    module.pkg = kclvm_ast::MAIN_PKG.to_string();
    module.name = kclvm_ast::MAIN_PKG.to_string();

    prog.pkgs
        .insert(kclvm_ast::MAIN_PKG.to_string(), vec![module]);

    Ok(prog)
}

/// Parse a KCL file to the AST module.
#[inline]
pub fn parse_file(filename: &str, code: Option<String>) -> Result<ast::Module, String> {
    let sess = Arc::new(ParseSession::default());
    let result = parse_file_with_global_session(sess.clone(), filename, code);
    if sess
        .0
        .diag_handler
        .has_errors()
        .map_err(|e| e.to_string())?
    {
        let err = sess
            .0
            .emit_nth_diag_into_string(0)
            .map_err(|e| e.to_string())?
            .unwrap_or(Ok(ErrorKind::InvalidSyntax.name()))
            .map_err(|e| e.to_string())?;
        Err(err)
    } else {
        result
    }
}

/// Parse a KCL file with all errors.
///
/// Note that an optional AST structure is returned here because there may be other error reasons
/// that may result in no AST being returned, such as file not being found, etc.
pub fn parse_file_with_errors(
    filename: &str,
    code: Option<String>,
) -> (Option<ast::Module>, String) {
    let sess = Arc::new(ParseSession::default());
    match parse_file_with_global_session(sess.clone(), filename, code) {
        Ok(module) => match sess.0.diag_handler.has_errors() {
            Ok(has_error) => {
                let get_err = || -> anyhow::Result<String> {
                    let err = sess
                        .0
                        .emit_nth_diag_into_string(0)?
                        .unwrap_or(Ok(ErrorKind::InvalidSyntax.name()))?;
                    Ok(err)
                };
                if has_error {
                    match get_err() {
                        Ok(err) => (Some(module), err),
                        Err(err) => (Some(module), err.to_string()),
                    }
                } else {
                    (Some(module), "".to_string())
                }
            }
            Err(err) => (Some(module), err.to_string()),
        },
        Err(err) => (None, err),
    }
}

/// Parse a KCL file to the AST module with the parse session .
pub fn parse_file_with_session(
    sess: Arc<ParseSession>,
    filename: &str,
    code: Option<String>,
) -> Result<ast::Module, String> {
    // Code source.
    let src = if let Some(s) = code {
        s
    } else {
        match std::fs::read_to_string(filename) {
            Ok(src) => src,
            Err(err) => {
                return Err(format!(
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
            return Err(format!(
                "Internal Bug: Failed to load KCL file '{filename}'."
            ));
        }
    };

    // Lexer
    let stream = lexer::parse_token_streams(&sess, src_from_sf.as_str(), sf.start_pos);
    // Parser
    let mut p = parser::Parser::new(&sess, stream);
    let mut m = p.parse_module();
    m.filename = filename.to_string();
    m.pkg = kclvm_ast::MAIN_PKG.to_string();
    m.name = kclvm_ast::MAIN_PKG.to_string();

    Ok(m)
}

/// Parse a KCL file to the AST module with the parse session and the global session
#[inline]
pub fn parse_file_with_global_session(
    sess: Arc<ParseSession>,
    filename: &str,
    code: Option<String>,
) -> Result<ast::Module, String> {
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
    pub recursive: bool,
    pub package_maps: HashMap<String, String>,

    pub cmd_args: Vec<ast::CmdArgSpec>,
    pub cmd_overrides: Vec<ast::OverrideSpec>,
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
            cmd_args: Default::default(),
            cmd_overrides: Default::default(),
            mode: ParseMode::ParseComments,
            load_packages: true,
            load_plugins: false,
            recursive: false,
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
    sess: Arc<ParseSession>,
    paths: &[&str],
    opts: Option<LoadProgramOptions>,
    module_cache: Option<KCLModuleCache>,
) -> Result<ast::Program, String> {
    Loader::new(sess, paths, opts, module_cache).load_main()
}

pub type KCLModuleCache = Arc<RwLock<IndexMap<String, ast::Module>>>;
struct Loader {
    sess: Arc<ParseSession>,
    paths: Vec<String>,
    opts: LoadProgramOptions,
    missing_pkgs: Vec<String>,
    module_cache: Option<KCLModuleCache>,
}

impl Loader {
    fn new(
        sess: Arc<ParseSession>,
        paths: &[&str],
        opts: Option<LoadProgramOptions>,
        module_cache: Option<Arc<RwLock<IndexMap<String, ast::Module>>>>,
    ) -> Self {
        Self {
            sess,
            paths: paths.iter().map(|s| s.to_string()).collect(),
            opts: opts.unwrap_or_default(),
            missing_pkgs: Default::default(),
            module_cache,
        }
    }

    #[inline]
    fn load_main(&mut self) -> Result<ast::Program, String> {
        create_session_globals_then(move || self._load_main())
    }

    fn _load_main(&mut self) -> Result<ast::Program, String> {
        let compile_entries = get_compile_entries_from_paths(&self.paths, &self.opts)?;
        let mut pkgs = HashMap::new();
        let workdir = compile_entries.get_root_path().to_string();

        // debug_assert_eq!(compile_entries.len(), self.paths.len());
        // todo: need to be removed after support vfs

        let mut pkg_files = Vec::new();
        for entry in compile_entries.iter() {
            // Get files from options with root.
            // let k_files = self.get_main_files_from_pkg(entry.path(), entry.name())?;
            let k_files = entry.get_k_files();
            let maybe_k_codes = entry.get_k_codes();

            // load module

            for (i, filename) in k_files.iter().enumerate() {
                let mut m = if let Some(module_cache) = self.module_cache.as_ref() {
                    let m = parse_file_with_session(
                        self.sess.clone(),
                        filename,
                        maybe_k_codes[i].clone(),
                    )?;
                    let mut module_cache_ref = module_cache.write().unwrap();
                    module_cache_ref.insert(filename.clone(), m.clone());
                    m
                } else {
                    parse_file_with_session(self.sess.clone(), filename, maybe_k_codes[i].clone())?
                };
                self.fix_rel_import_path(entry.path(), &mut m);
                pkg_files.push(m);
            }

            // Insert an empty vec to determine whether there is a circular import.
            pkgs.insert(kclvm_ast::MAIN_PKG.to_string(), vec![]);
            self.load_import_package(
                &entry.path(),
                entry.name().to_string(),
                &mut pkg_files,
                &mut pkgs,
            )?;
        }
        // Insert the complete ast to replace the empty list.
        pkgs.insert(kclvm_ast::MAIN_PKG.to_string(), pkg_files);
        Ok(ast::Program {
            root: workdir,
            main: kclvm_ast::MAIN_PKG.to_string(),
            pkgs,
        })
    }

    /// [`find_packages`] will find the kcl package.
    /// If the package is found, the basic information of the package [`PkgInfo`] will be returned.
    ///
    /// # Errors
    ///
    ///  This method will return an error in the following two cases:
    ///
    /// 1. The package not found.
    /// 2. The package was found both internal and external the current package.
    fn find_packages(
        &self,
        pos: ast::Pos,
        pkg_name: &str,
        pkg_root: &str,
        pkg_path: &str,
    ) -> Result<Option<PkgInfo>, String> {
        // 1. Look for in the current package's directory.
        let is_internal = self.is_internal_pkg(pkg_name, pkg_root, pkg_path)?;

        // 2. Look for in the vendor path.
        let is_external = self.is_external_pkg(pkg_path)?;

        // 3. Internal and external packages cannot be duplicated
        if is_external.is_some() && is_internal.is_some() {
            self.sess.1.borrow_mut().add_error(
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
            Some(pkg_info) => return Ok(Some(pkg_info)),
            None => {
                self.sess.1.borrow_mut().add_error(
                    ErrorKind::CannotFindModule,
                    &[Message {
                        range: Into::<Range>::into(pos),
                        style: Style::Line,
                        message: format!("pkgpath {} not found in the program", pkg_path),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
                let mut suggestions =
                    vec![format!("find more package on 'https://artifacthub.io'")];

                if let Ok(pkg_name) = parse_external_pkg_name(pkg_path) {
                    suggestions.insert(
                        0,
                        format!(
                            "try 'kcl mod add {}' to download the package not found",
                            pkg_name
                        ),
                    );
                }
                self.sess.1.borrow_mut().add_suggestions(suggestions);
                return Ok(None);
            }
        };
    }

    /// [`load_import_package`] will traverse all the [`kclvm_ast::ImportStmt`] on the input AST nodes [`pkg`],
    ///  load the source code and parse the code to corresponding AST.
    ///
    /// And store the result of parse in [`pkgs`].
    ///
    /// # Note
    /// [`load_import_package`] will add the external package name as prefix of the [`kclvm_ast::ImportStmt`]'s member [`path`].
    fn load_import_package(
        &mut self,
        pkgroot: &str,
        pkg_name: String,
        pkg: &mut [ast::Module],
        pkgs: &mut HashMap<String, Vec<ast::Module>>,
    ) -> Result<(), String> {
        for m in pkg {
            for stmt in &mut m.body {
                let pos = stmt.pos().clone();
                if let ast::Stmt::Import(ref mut import_spec) = &mut stmt.node {
                    import_spec.path = kclvm_config::vfs::fix_import_path(
                        pkgroot,
                        &m.filename,
                        import_spec.path.as_str(),
                    );
                    import_spec.pkg_name = pkg_name.to_string();
                    // Load the import package source code and compile.
                    if let Some(pkg_info) = self.load_package(
                        &pkgroot,
                        pkg_name.to_string(),
                        import_spec.path.to_string(),
                        pos.into(),
                        pkgs,
                    )? {
                        // Add the external package name as prefix of the [`kclvm_ast::ImportStmt`]'s member [`path`].
                        import_spec.path = pkg_info.pkg_path.to_string();
                        import_spec.pkg_name = pkg_info.pkg_name
                    }
                }
            }
        }
        return Ok(());
    }

    fn fix_rel_import_path(&mut self, pkgroot: &str, m: &mut ast::Module) {
        for stmt in &mut m.body {
            if let ast::Stmt::Import(ref mut import_spec) = &mut stmt.node {
                import_spec.path = kclvm_config::vfs::fix_import_path(
                    pkgroot,
                    &m.filename,
                    import_spec.path.as_str(),
                );
            }
        }
    }

    /// [`load_package`] will return some basic information about the package
    /// according to whether the package is internal or external.
    fn load_package(
        &mut self,
        pkgroot: &str,
        pkgname: String,
        pkgpath: String,
        pos: ast::Pos,
        pkgs: &mut HashMap<String, Vec<ast::Module>>,
    ) -> Result<Option<PkgInfo>, String> {
        if !self.opts.load_packages {
            return Ok(None);
        }

        if pkgpath.is_empty() {
            return Ok(None);
        }

        if pkgs.contains_key(&pkgpath) {
            return Ok(None);
        }
        if self.missing_pkgs.contains(&pkgpath) {
            return Ok(None);
        }

        // plugin pkgs
        if self.is_plugin_pkg(pkgpath.as_str()) {
            if !self.opts.load_plugins {
                self.sess.1.borrow_mut().add_error(
                    ErrorKind::CannotFindModule,
                    &[Message {
                        range: Into::<Range>::into(pos),
                        style: Style::Line,
                        message: format!("the plugin package `{}` is not found, please confirm if plugin mode is enabled", pkgpath),
                        note: None,
                        suggested_replacement: None,
                    }],
                );
            }
            return Ok(None);
        }

        // builtin pkgs
        if self.is_builtin_pkg(pkgpath.as_str()) {
            return Ok(None);
        }

        // find the package.
        let pkg_info = match self.find_packages(pos.clone(), &pkgname, pkgroot, &pkgpath)? {
            Some(info) => info,
            None => return Ok(None),
        };

        // If there is a circular import, return the information of the found package.
        if pkgs.contains_key(&pkg_info.pkg_path) {
            return Ok(Some(pkg_info));
        }

        if pkg_info.k_files.is_empty() {
            self.missing_pkgs.push(pkgpath);
            return Ok(Some(pkg_info));
        }

        let mut pkg_files = Vec::new();
        let k_files = pkg_info.k_files.clone();
        for filename in k_files {
            let mut m = if let Some(module_cache) = self.module_cache.as_ref() {
                let module_cache_ref = module_cache.read().unwrap();
                if let Some(module) = module_cache_ref.get(&filename) {
                    module.clone()
                } else {
                    let m = parse_file_with_session(self.sess.clone(), &filename, None)?;
                    drop(module_cache_ref);
                    let mut module_cache_ref = module_cache.write().unwrap();
                    module_cache_ref.insert(filename.clone(), m.clone());
                    m
                }
            } else {
                parse_file_with_session(self.sess.clone(), &filename, None)?
            };

            m.pkg = pkg_info.pkg_path.clone();
            m.name = "".to_string();
            self.fix_rel_import_path(&pkg_info.pkg_root, &mut m);

            pkg_files.push(m);
        }

        // Insert an empty vec to determine whether there is a circular import.
        pkgs.insert(pkg_info.pkg_path.clone(), vec![]);

        self.load_import_package(
            &pkg_info.pkg_root.to_string(),
            pkg_info.pkg_name.to_string(),
            &mut pkg_files,
            pkgs,
        )?;

        // Insert the complete ast to replace the empty list.
        pkgs.insert(pkg_info.pkg_path.clone(), pkg_files);

        Ok(Some(pkg_info))
    }

    fn get_pkg_kfile_list(&self, pkgroot: &str, pkgpath: &str) -> Result<Vec<String>, String> {
        // plugin pkgs
        if self.is_plugin_pkg(pkgpath) {
            return Ok(Vec::new());
        }

        // builtin pkgs
        if self.is_builtin_pkg(pkgpath) {
            return Ok(Vec::new());
        }

        if pkgroot.is_empty() {
            return Err("pkgroot not found".to_string());
        }

        let mut pathbuf = std::path::PathBuf::new();
        pathbuf.push(pkgroot);

        for s in pkgpath.split('.') {
            pathbuf.push(s);
        }

        let abspath: String = pathbuf.as_path().to_str().unwrap().to_string();

        if std::path::Path::new(abspath.as_str()).exists() {
            return self.get_dir_files(abspath.as_str());
        }

        let as_k_path = abspath + KCL_FILE_SUFFIX;
        if std::path::Path::new((as_k_path).as_str()).exists() {
            return Ok(vec![as_k_path]);
        }

        Ok(Vec::new())
    }

    /// Get file list in the directory.
    fn get_dir_files(&self, dir: &str) -> Result<Vec<String>, String> {
        if !std::path::Path::new(dir).exists() {
            return Ok(Vec::new());
        }

        let mut list = Vec::new();

        for path in std::fs::read_dir(dir).unwrap() {
            let path = path.unwrap();
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

    fn is_builtin_pkg(&self, pkgpath: &str) -> bool {
        let system_modules = kclvm_sema::builtin::system_module::STANDARD_SYSTEM_MODULES;
        system_modules.contains(&pkgpath)
    }

    fn is_plugin_pkg(&self, pkgpath: &str) -> bool {
        pkgpath.starts_with(PLUGIN_MODULE_PREFIX)
    }

    /// Look for [`pkgpath`] in the current package's [`pkgroot`].
    /// If found, return to the [`PkgInfo`]， else return [`None`]
    ///
    /// # Error
    ///
    /// [`is_internal_pkg`] will return an error if the package's source files cannot be found.
    fn is_internal_pkg(
        &self,
        pkg_name: &str,
        pkg_root: &str,
        pkg_path: &str,
    ) -> Result<Option<PkgInfo>, String> {
        match self.pkg_exists(vec![pkg_root.to_string()], pkg_path) {
            Some(internal_pkg_root) => {
                let fullpath = if pkg_name == kclvm_ast::MAIN_PKG {
                    pkg_path.to_string()
                } else {
                    format!("{}.{}", pkg_name, pkg_path)
                };
                let k_files = self.get_pkg_kfile_list(pkg_root, pkg_path)?;
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

    /// Look for [`pkgpath`] in the external package's home.
    /// If found, return to the [`PkgInfo`]， else return [`None`]
    ///
    /// # Error
    ///
    /// - [`is_external_pkg`] will return an error if the package's source files cannot be found.
    /// - The name of the external package could not be resolved from [`pkg_path`].
    fn is_external_pkg(&self, pkg_path: &str) -> Result<Option<PkgInfo>, String> {
        let pkg_name = parse_external_pkg_name(pkg_path)?;
        let external_pkg_root = if let Some(root) = self.opts.package_maps.get(&pkg_name) {
            PathBuf::from(root).join(KCL_MOD_FILE)
        } else {
            match self.pkg_exists(self.opts.vendor_dirs.clone(), pkg_path) {
                Some(path) => PathBuf::from(path)
                    .join(pkg_name.to_string())
                    .join(KCL_MOD_FILE),
                None => return Ok(None),
            }
        };

        if external_pkg_root.exists() {
            return Ok(Some(match external_pkg_root.parent() {
                Some(root) => {
                    let k_files = self.get_pkg_kfile_list(
                        &root.display().to_string(),
                        &rm_external_pkg_name(pkg_path)?,
                    )?;
                    PkgInfo::new(
                        pkg_name.to_string(),
                        root.display().to_string(),
                        pkg_path.to_string(),
                        k_files,
                    )
                }
                None => return Ok(None),
            }));
        } else {
            return Ok(None);
        }
    }

    /// Search [`pkgpath`] among all the paths in [`pkgroots`].
    ///
    /// # Notes
    ///
    /// All paths in [`pkgpath`] must contain the kcl.mod file.
    /// It returns the parent directory of kcl.mod if present, or none if not.
    fn pkg_exists(&self, pkgroots: Vec<String>, pkgpath: &str) -> Option<String> {
        pkgroots
            .into_iter()
            .find(|root| self.pkg_exists_in_path(root.to_string(), pkgpath))
    }

    /// Search for [`pkgpath`] under [`path`].
    /// It only returns [`true`] if [`path`]/[`pkgpath`] or [`path`]/[`kcl.mod`] exists.
    fn pkg_exists_in_path(&self, path: String, pkgpath: &str) -> bool {
        let mut pathbuf = PathBuf::from(path);
        pkgpath.split('.').for_each(|s| pathbuf.push(s));
        pathbuf.exists() || pathbuf.with_extension(KCL_FILE_EXTENSION).exists()
    }
}
