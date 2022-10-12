// Copyright 2021 The KCL Authors. All rights reserved.

mod lexer;
mod parser;
mod session;

#[cfg(test)]
mod tests;

extern crate kclvm_error;

use crate::session::ParseSession;
use kclvm::{ErrType, PanicInfo};
use kclvm_ast::ast;
use kclvm_span::{self, FilePathMapping, SourceMap};

use lexer::parse_token_streams;
use parser::Parser;
use rustc_span::BytePos;
use rustc_span::Pos;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use kclvm_span::create_session_globals_then;

/// parser mode
#[derive(Debug, Clone)]
pub enum ParseMode {
    Null,
    ParseComments,
}

/// Get the AST program from json file.
pub fn parse_program_from_json_file(path: &str) -> Result<ast::Program, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn parse_program(filename: &str) -> Result<ast::Program, String> {
    let abspath = std::fs::canonicalize(&std::path::PathBuf::from(filename)).unwrap();

    let mut prog = ast::Program {
        root: abspath.parent().unwrap().to_str().unwrap().to_string(),
        main: "__main__".to_string(),
        pkgs: std::collections::HashMap::new(),
        cmd_args: Vec::new(),
        cmd_overrides: Vec::new(),
    };

    let mainpkg = "__main__";

    let mut module = parse_file(abspath.to_str().unwrap(), None)?;
    module.filename = filename.to_string();
    module.pkg = mainpkg.to_string();
    module.name = mainpkg.to_string();

    prog.pkgs.insert(mainpkg.to_string(), vec![module]);

    Ok(prog)
}

pub fn parse_file(filename: &str, code: Option<String>) -> Result<ast::Module, String> {
    create_session_globals_then(move || {
        let src = if let Some(s) = code {
            s
        } else {
            match std::fs::read_to_string(filename) {
                Ok(src) => src,
                Err(_err) => {
                    let err_msg =
                        format!("Failed to load KCL file '{}'. Because '{}'", filename, _err);
                    return Err(err_msg);
                }
            }
        };

        let sm = kclvm_span::SourceMap::new(FilePathMapping::empty());
        let sf = sm.new_source_file(PathBuf::from(filename).into(), src.to_string());
        let sess = &ParseSession::with_source_map(std::sync::Arc::new(sm));

        let stream = lexer::parse_token_streams(sess, src.as_str(), sf.start_pos);
        let mut p = parser::Parser::new(sess, stream);
        let mut m = p.parse_module();

        m.filename = filename.to_string();
        m.pkg = kclvm_ast::MAIN_PKG.to_string();
        m.name = kclvm_ast::MAIN_PKG.to_string();

        Ok(m)
    })
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
        let sm = SourceMap::new(FilePathMapping::empty());
        sm.new_source_file(PathBuf::from("").into(), src.to_string());
        let sess = &ParseSession::with_source_map(Arc::new(sm));

        let expr: Option<ast::NodeRef<ast::Expr>> = Some(create_session_globals_then(|| {
            let stream = parse_token_streams(sess, src, BytePos::from_u32(0));
            let mut parser = Parser::new(sess, stream);
            parser.parse_expr()
        }));
        expr
    }
}

#[derive(Debug, Default, Clone)]
pub struct LoadProgramOptions {
    pub work_dir: String,
    pub k_code_list: Vec<String>,

    pub cmd_args: Vec<ast::CmdArgSpec>,
    pub cmd_overrides: Vec<ast::OverrideSpec>,

    pub _mode: Option<ParseMode>,
    pub _load_packages: bool,
}

pub fn load_program(
    paths: &[&str],
    opts: Option<LoadProgramOptions>,
) -> Result<ast::Program, String> {
    // todo: support cache
    if let Some(opts) = opts {
        Loader::new(paths, Some(opts)).load_main()
    } else {
        Loader::new(paths, None).load_main()
    }
}

struct Loader {
    paths: Vec<String>,
    opts: LoadProgramOptions,

    pkgroot: String,

    modfile: kclvm_config::modfile::KCLModFile,
    pkgs: std::collections::HashMap<String, Vec<ast::Module>>,
    missing_pkgs: Vec<String>,
    // todo: add shared source_map all parse_file.
}

impl Loader {
    fn new(paths: &[&str], opts: Option<LoadProgramOptions>) -> Self {
        Self {
            paths: paths.iter().map(|s| s.to_string()).collect(),
            opts: opts.unwrap_or_default(),

            pkgroot: "".to_string(),

            modfile: Default::default(),
            pkgs: Default::default(),
            missing_pkgs: Default::default(),
        }
    }

    fn load_main(&mut self) -> Result<ast::Program, String> {
        match self._load_main() {
            Ok(x) => Ok(x),
            Err(s) => Err(self.str_to_panic_info(&s).to_json_string()),
        }
    }

    fn _load_main(&mut self) -> Result<ast::Program, String> {
        debug_assert!(!self.paths.is_empty());

        self.pkgroot = kclvm_config::modfile::get_pkg_root_from_paths(&self.paths)?;

        if !self.pkgroot.is_empty() {
            debug_assert!(self.is_dir(self.pkgroot.as_str()));
            debug_assert!(self.path_exist(self.pkgroot.as_str()));

            self.modfile = kclvm_config::modfile::load_mod_file(self.pkgroot.as_str());
        }

        // fix path
        let mut path_list = Vec::new();
        for s in &self.paths {
            let mut s = s.clone();
            if s.contains(kclvm_config::modfile::KCL_MOD_PATH_ENV) {
                debug_assert!(!self.pkgroot.is_empty());
                s = s.replace(
                    kclvm_config::modfile::KCL_MOD_PATH_ENV,
                    self.pkgroot.as_str(),
                );
            }
            if !self.pkgroot.is_empty() && !self.is_absolute(s.as_str()) {
                let p = std::path::Path::new(s.as_str());
                if let Ok(x) = std::fs::canonicalize(p) {
                    s = x.to_str().unwrap_or(s.as_str()).to_string();
                }
            }

            path_list.push(s);
        }

        // get k files
        let mut k_files: Vec<String> = Vec::new();
        for (i, path) in path_list.iter().enumerate() {
            if path.ends_with(".k") {
                k_files.push(path.to_string());
                continue;
            }

            // read dir/*.k
            if self.is_dir(path) {
                if self.opts.k_code_list.len() > i {
                    return Err("invalid code list".to_string());
                }
                //k_code_list
                for s in self.get_dir_kfile_list(path)? {
                    k_files.push(s);
                }
                continue;
            }
        }

        if k_files.is_empty() {
            return Err("No input KCL files".to_string());
        }

        // check all file exists
        for (i, filename) in (&k_files).iter().enumerate() {
            if i < self.opts.k_code_list.len() {
                continue;
            }

            if !self.pkgroot.is_empty() {
                debug_assert!(self.is_file(filename.as_str()));
                debug_assert!(self.is_absolute(filename.as_str()), "filename={}", filename);
            }

            if !self.path_exist(filename.as_str()) {
                return Err(format!(
                    "Cannot find the kcl file, please check whether the file path {}",
                    filename.as_str(),
                ));
            }
        }

        // load module
        let mut pkg_files = Vec::new();
        for (i, filename) in (&k_files).iter().enumerate() {
            // todo: add shared source map for all files
            if i < self.opts.k_code_list.len() {
                let mut m = parse_file(filename, Some(self.opts.k_code_list[i].clone()))?;
                self.fix_rel_import_path(&mut m);
                pkg_files.push(m)
            } else {
                let mut m = parse_file(filename, None)?;
                self.fix_rel_import_path(&mut m);
                pkg_files.push(m);
            }
        }

        let __kcl_main__ = kclvm_ast::MAIN_PKG;
        let import_list = self.get_import_list(&pkg_files);

        self.pkgs.insert(__kcl_main__.to_string(), pkg_files);

        // load imported packages
        for import_spec in import_list {
            self.load_package(import_spec.path.to_string())?;
        }

        // Ok
        Ok(ast::Program {
            root: self.pkgroot.clone(),
            main: __kcl_main__.to_string(),
            pkgs: self.pkgs.clone(),
            cmd_args: Vec::new(),
            cmd_overrides: Vec::new(),
        })
    }

    fn fix_rel_import_path(&mut self, m: &mut ast::Module) {
        for stmt in &mut m.body {
            if let ast::Stmt::Import(ref mut import_spec) = &mut stmt.node {
                import_spec.path = kclvm_config::vfs::fix_import_path(
                    &self.pkgroot,
                    &m.filename,
                    import_spec.path.as_str(),
                );
            }
        }
    }

    fn load_package(&mut self, pkgpath: String) -> Result<(), String> {
        if pkgpath.is_empty() {
            return Ok(());
        }

        if self.pkgs.contains_key(&pkgpath) {
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

        let k_files = self.get_pkg_kfile_list(pkgpath.as_str())?;

        if k_files.is_empty() {
            self.missing_pkgs.push(pkgpath);
            return Ok(());
        }

        let mut pkg_files = Vec::new();
        for filename in k_files {
            debug_assert!(self.is_file(filename.as_str()));
            debug_assert!(self.path_exist(filename.as_str()));

            let mut m = parse_file(filename.as_str(), None)?;

            m.pkg = pkgpath.clone();
            m.name = "".to_string();
            self.fix_rel_import_path(&mut m);

            pkg_files.push(m);
        }

        let import_list = self.get_import_list(&pkg_files);
        self.pkgs.insert(pkgpath, pkg_files);

        for import_spec in import_list {
            self.load_package(import_spec.path.to_string())?;
        }

        return Ok(());
    }

    fn get_import_list(&self, pkg: &[ast::Module]) -> Vec<ast::ImportStmt> {
        let mut import_list = Vec::new();
        for m in pkg {
            for stmt in &m.body {
                if let ast::Stmt::Import(import_spec) = &stmt.node {
                    let mut import_spec = import_spec.clone();
                    import_spec.path = kclvm_config::vfs::fix_import_path(
                        &self.pkgroot,
                        &m.filename,
                        import_spec.path.as_str(),
                    );
                    import_list.push(import_spec);
                }
            }
        }
        import_list
    }

    fn get_pkg_kfile_list(&self, pkgpath: &str) -> Result<Vec<String>, String> {
        debug_assert!(!pkgpath.is_empty());

        // plugin pkgs
        if self.is_plugin_pkg(pkgpath) {
            return Ok(Vec::new());
        }

        // builtin pkgs
        if self.is_builtin_pkg(pkgpath) {
            return Ok(Vec::new());
        }

        if self.pkgroot.is_empty() {
            return Err("pkgroot not found".to_string());
        }

        let mut pathbuf = std::path::PathBuf::new();
        pathbuf.push(&self.pkgroot);
        for s in pkgpath.split('.') {
            pathbuf.push(s);
        }

        let pkgpath: String = pathbuf.as_path().to_str().unwrap().to_string();
        let abspath: String = std::path::Path::new(&self.pkgroot)
            .join(pkgpath)
            .to_str()
            .unwrap()
            .to_string();

        if std::path::Path::new(abspath.as_str()).exists() {
            return self.get_dir_kfile_list(abspath.as_str());
        }

        let as_k_path = abspath + ".k";
        if std::path::Path::new((&as_k_path).as_str()).exists() {
            return Ok(vec![as_k_path]);
        }

        Ok(Vec::new())
    }
    fn get_dir_kfile_list(&self, dir: &str) -> Result<Vec<String>, String> {
        if !std::path::Path::new(dir).exists() {
            return Ok(Vec::new());
        }

        let mut list = Vec::new();

        for path in std::fs::read_dir(dir).unwrap() {
            let path = path.unwrap();
            if !path.file_name().to_str().unwrap().ends_with(".k") {
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
        pkgpath.starts_with("kcl_plugin.")
    }
}

// utils
impl Loader {
    fn is_file(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }
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

impl Loader {
    fn str_to_panic_info(&self, s: &str) -> PanicInfo {
        let mut panic_info = PanicInfo::default();

        panic_info.__kcl_PanicInfo__ = true;
        panic_info.message = format!("{}", s);
        panic_info.err_type_code = ErrType::CompileError_TYPE as i32;

        panic_info
    }
}
