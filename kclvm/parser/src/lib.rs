// Copyright 2021 The KCL Authors. All rights reserved.

mod lexer;
mod parser;
mod session;

#[cfg(test)]
mod tests;

extern crate kclvm_error;

pub use crate::session::ParseSession;
use compiler_base_macros::bug;
use compiler_base_session::Session;
use compiler_base_span::span::new_byte_pos;
use kclvm_ast::ast;
use kclvm_config::modfile::{
    get_pkg_root_from_paths, get_vendor_home, KCL_FILE_EXTENSION, KCL_FILE_SUFFIX, KCL_MOD_FILE,
    KCL_MOD_PATH_ENV,
};
use kclvm_error::ErrorKind;
use kclvm_runtime::PanicInfo;
use kclvm_sema::plugin::PLUGIN_MODULE_PREFIX;
use kclvm_utils::path::PathPrefix;

use lexer::parse_token_streams;
use parser::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use kclvm_span::create_session_globals_then;

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

    pub cmd_args: Vec<ast::CmdArgSpec>,
    pub cmd_overrides: Vec<ast::OverrideSpec>,
    /// The parser mode.
    pub mode: ParseMode,
    /// Wether to load packages.
    pub load_packages: bool,
}

impl Default for LoadProgramOptions {
    fn default() -> Self {
        Self {
            work_dir: Default::default(),
            k_code_list: Default::default(),
            vendor_dirs: vec![get_vendor_home()],
            cmd_args: Default::default(),
            cmd_overrides: Default::default(),
            mode: ParseMode::ParseComments,
            load_packages: true,
        }
    }
}

pub fn load_program(
    sess: Arc<ParseSession>,
    paths: &[&str],
    opts: Option<LoadProgramOptions>,
) -> Result<ast::Program, String> {
    // todo: support cache
    if let Some(opts) = opts {
        Loader::new(sess, paths, Some(opts)).load_main()
    } else {
        Loader::new(sess, paths, None).load_main()
    }
}

struct Loader {
    sess: Arc<ParseSession>,
    paths: Vec<String>,
    opts: LoadProgramOptions,
    missing_pkgs: Vec<String>,
}

impl Loader {
    fn new(sess: Arc<ParseSession>, paths: &[&str], opts: Option<LoadProgramOptions>) -> Self {
        Self {
            sess,
            paths: paths.iter().map(|s| s.to_string()).collect(),
            opts: opts.unwrap_or_default(),
            missing_pkgs: Default::default(),
        }
    }

    #[inline]
    fn load_main(&mut self) -> Result<ast::Program, String> {
        create_session_globals_then(move || self._load_main())
    }

    fn _load_main(&mut self) -> Result<ast::Program, String> {
        let root = get_pkg_root_from_paths(&self.paths)?;

        // Get files from options with root.
        let k_files = self.get_main_files(&root)?;

        // load module
        let mut pkgs = HashMap::new();
        let mut pkg_files = Vec::new();
        for (i, filename) in k_files.iter().enumerate() {
            if i < self.opts.k_code_list.len() {
                let mut m = parse_file_with_session(
                    self.sess.clone(),
                    filename,
                    Some(self.opts.k_code_list[i].clone()),
                )?;
                self.fix_rel_import_path(&root, &mut m);
                pkg_files.push(m)
            } else {
                let mut m = parse_file_with_session(self.sess.clone(), filename, None)?;
                self.fix_rel_import_path(&root, &mut m);
                pkg_files.push(m);
            }
        }

        let import_list = self.get_import_list(&root, &pkg_files);

        pkgs.insert(kclvm_ast::MAIN_PKG.to_string(), pkg_files);

        // load imported packages
        for import_spec in import_list {
            self.load_package(&root, import_spec.0, import_spec.1, &mut pkgs)?;
        }

        // Ok
        Ok(ast::Program {
            root,
            main: kclvm_ast::MAIN_PKG.to_string(),
            pkgs,
        })
    }

    /// Get files in the main package with the package root.
    fn get_main_files(&mut self, root: &str) -> Result<Vec<String>, String> {
        // fix path
        let mut path_list = Vec::new();
        for s in &self.paths {
            let mut s = s.clone();
            if s.contains(KCL_MOD_PATH_ENV) {
                s = s.replace(KCL_MOD_PATH_ENV, root);
            }
            if !root.is_empty() && !self.is_absolute(s.as_str()) {
                let p = std::path::Path::new(s.as_str());
                if let Ok(x) = std::fs::canonicalize(p) {
                    s = x.adjust_canonicalization();
                }
            }

            path_list.push(s);
        }

        // get k files
        let mut k_files: Vec<String> = Vec::new();
        for (i, path) in path_list.iter().enumerate() {
            if path.ends_with(KCL_FILE_SUFFIX) {
                k_files.push(path.to_string());
                continue;
            }

            // read dir/*.k
            if self.is_dir(path) {
                if self.opts.k_code_list.len() > i {
                    return Err(PanicInfo::from("Invalid code list").to_json_string());
                }
                //k_code_list
                for s in self.get_dir_files(path)? {
                    k_files.push(s);
                }
                continue;
            }
        }

        if k_files.is_empty() {
            return Err(PanicInfo::from("No input KCL files").to_json_string());
        }

        // check all file exists
        for (i, filename) in k_files.iter().enumerate() {
            if i < self.opts.k_code_list.len() {
                continue;
            }

            if !self.path_exist(filename.as_str()) {
                return Err(PanicInfo::from(format!(
                    "Cannot find the kcl file, please check whether the file path {}",
                    filename.as_str(),
                ))
                .to_json_string());
            }
        }
        Ok(k_files)
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

    fn load_package(
        &mut self,
        pkgroot: &str,
        pkgpath: String,
        pos: ast::Pos,
        pkgs: &mut HashMap<String, Vec<ast::Module>>,
    ) -> Result<(), String> {
        if pkgpath.is_empty() {
            return Ok(());
        }

        if pkgs.contains_key(&pkgpath) {
            return Ok(());
        }
        if self.missing_pkgs.contains(&pkgpath) {
            return Ok(());
        }

        // plugin pkgs
        if self.is_plugin_pkg(pkgpath.as_str()) {
            return Ok(());
        }

        // builtin pkgs
        if self.is_builtin_pkg(pkgpath.as_str()) {
            return Ok(());
        }

        // Look for in the current package's directory.
        let is_internal = self.is_internal_pkg(pkgroot, &pkgpath);

        // Look for in the vendor path.
        let is_external = self.is_external_pkg(&pkgpath)?;

        if is_external.is_some() && is_internal.is_some() {
            return Err(PanicInfo::from_ast_pos(
                format!(
                    "the `{}` is found multiple times in the current package and vendor package",
                    pkgpath
                ),
                pos.into(),
            )
            .to_json_string());
        }

        let origin_pkg_path = pkgpath.to_string();

        let (pkgroot, k_files) = match is_internal {
            Some(internal_root) => (
                internal_root.to_string(),
                self.get_pkg_kfile_list(&internal_root, &pkgpath)?,
            ),
            None => match is_external {
                Some(external_root) => (
                    external_root.to_string(),
                    self.get_pkg_kfile_list(
                        &external_root,
                        &self.rm_external_pkg_name(pkgpath.as_str())?,
                    )?,
                ),
                None => {
                    return Err(PanicInfo::from_ast_pos(
                        format!("pkgpath {} not found in the program", pkgpath),
                        pos.into(),
                    )
                    .to_json_string());
                }
            },
        };

        if k_files.is_empty() {
            self.missing_pkgs.push(pkgpath);
            return Ok(());
        }

        let mut pkg_files = Vec::new();
        for filename in k_files {
            let mut m = parse_file_with_session(self.sess.clone(), filename.as_str(), None)?;

            m.pkg = origin_pkg_path.clone();
            m.name = "".to_string();
            self.fix_rel_import_path(&pkgroot, &mut m);

            pkg_files.push(m);
        }

        let import_list = self.get_import_list(&pkgroot, &pkg_files);
        pkgs.insert(origin_pkg_path, pkg_files);

        for import_spec in import_list {
            self.load_package(&pkgroot, import_spec.0, import_spec.1, pkgs)?;
        }

        Ok(())
    }

    fn get_import_list(&self, pkgroot: &str, pkg: &[ast::Module]) -> Vec<(String, ast::Pos)> {
        let mut import_list = Vec::new();
        for m in pkg {
            for stmt in &m.body {
                if let ast::Stmt::Import(import_spec) = &stmt.node {
                    let mut import_spec = import_spec.clone();
                    import_spec.path = kclvm_config::vfs::fix_import_path(
                        pkgroot,
                        &m.filename,
                        import_spec.path.as_str(),
                    );
                    import_list.push((import_spec.path, stmt.pos().into()));
                }
            }
        }
        import_list
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

        let pkgpath: String = pathbuf.as_path().to_str().unwrap().to_string();
        let abspath: String = std::path::Path::new(&pkgroot)
            .join(pkgpath)
            .to_str()
            .unwrap()
            .to_string();

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
    /// If found, return to the [`pkgroot`]， else return [`None`]
    fn is_internal_pkg(&self, pkgroot: &str, pkgpath: &str) -> Option<String> {
        self.pkg_exists(vec![pkgroot.to_string()], pkgpath)
    }

    /// Look for [`pkgpath`] in the external package's home.
    /// If found, return to the [`pkgroot`]， else return [`None`]
    fn is_external_pkg(&self, pkgpath: &str) -> Result<Option<String>, String> {
        let root_path = match self.pkg_exists(self.opts.vendor_dirs.clone(), pkgpath) {
            Some(path) => path,
            None => return Ok(None),
        };

        let pathbuf = PathBuf::from(root_path);
        let rootpkg = pathbuf
            .join(self.parse_external_pkg_name(pkgpath)?)
            .join(KCL_MOD_FILE);

        if rootpkg.exists() {
            return Ok(Some(
                match rootpkg.parent() {
                    Some(it) => it,
                    None => return Ok(None),
                }
                .display()
                .to_string(),
            ));
        } else {
            Ok(None)
        }
    }

    /// Remove the external package name prefix from the current import absolute path.
    ///
    /// # Note
    /// [`rm_external_pkg_name`] just remove the prefix of the import path,
    /// so it can't distinguish whether the current path is an internal package or an external package.
    ///
    /// # Error
    /// An error is returned if an empty string is passed in.
    fn rm_external_pkg_name(&self, pkgpath: &str) -> Result<String, String> {
        Ok(pkgpath
            .to_string()
            .trim_start_matches(self.parse_external_pkg_name(pkgpath)?.as_str())
            .to_string())
    }

    /// Remove the external package name prefix from the current import absolute path.
    ///
    /// # Note
    /// [`rm_external_pkg_name`] just remove the prefix of the import path,
    /// so it can't distinguish whether the current path is an internal package or an external package.
    ///
    /// # Error
    /// An error is returned if an empty string is passed in.
    fn parse_external_pkg_name(&self, pkgpath: &str) -> Result<String, String> {
        let mut names = pkgpath.splitn(2, '.');
        match names.next() {
            Some(it) => Ok(it.to_string()),
            None => Err(
                PanicInfo::from(format!("Invalid external package name `{}`", pkgpath))
                    .to_json_string(),
            ),
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

// utils
impl Loader {
    fn is_dir(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }

    fn is_absolute(&self, path: &str) -> bool {
        std::path::Path::new(path).is_absolute()
    }

    fn path_exist(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}
