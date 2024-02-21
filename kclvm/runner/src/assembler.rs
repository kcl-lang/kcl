use anyhow::Result;
use compiler_base_macros::bug;
use indexmap::IndexMap;
use kclvm_ast::ast::{self, Program};
use kclvm_compiler::codegen::{
    llvm::{emit_code, OBJECT_FILE_SUFFIX},
    EmitOptions,
};
use kclvm_config::cache::{load_pkg_cache, save_pkg_cache, CacheOption, KCL_CACHE_PATH_ENV_VAR};
use kclvm_sema::resolver::scope::ProgramScope;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use crate::ExecProgramArgs;

/// IR code file suffix.
const DEFAULT_IR_FILE: &str = "_a.out";

/// LibAssembler trait is used to indicate the general interface
/// that must be implemented when different intermediate codes are assembled
/// into dynamic link libraries.
///
/// Note: LibAssembler is only for single file kcl program. For multi-file kcl programs,
/// KclvmAssembler is provided to support for multi-file parallel compilation to improve
/// the performance of the compiler.
pub(crate) trait LibAssembler {
    /// Add a suffix to the file name according to the file suffix of different intermediate code files.
    /// e.g. LLVM IR -> code_file : "/test_dir/test_code_file" -> return : "/test_dir/test_code_file.o"
    fn add_code_file_suffix(&self, code_file: &str) -> String;

    /// Return the file suffix of different intermediate code files.
    /// e.g. LLVM IR -> return : ".o"
    fn get_code_file_suffix(&self) -> String;

    /// Assemble different intermediate codes into object files for single file kcl program.
    /// Returns the path of the object file.
    ///
    /// Inputs:
    /// compile_prog: Reference of kcl program ast.
    ///
    /// "import_names" is import pkgpath and name of kcl program.
    /// Type of import_names is "IndexMap<kcl_file_name, IndexMap<import_name, import_path>>".
    ///
    /// "kcl_file_name" is the kcl file name string.
    /// "import_name" is the name string of import stmt.
    /// "import_path" is the path string of import stmt.
    ///
    /// e.g. "import test/main_pkg as main", "main" is an "import_name".
    /// e.g. "import test/main_pkg as main", "test/main_pkg" is an import_path.
    ///
    /// "code_file" is the filename of the generated intermediate code file.
    /// e.g. code_file : "/test_dir/test_code_file"
    ///
    /// "object_file_path" is the full filename of the generated intermediate code file with suffix.
    /// e.g. code_file_path : "/test_dir/test_code_file.o"
    ///
    /// "arg" is the arguments of the kclvm runtime.   
    fn assemble(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        code_file_path: &str,
        arg: &ExecProgramArgs,
    ) -> Result<String>;

    /// Clean cache lock files.
    #[inline]
    fn clean_lock_file(&self, path: &str) -> Result<()> {
        let lock_path = &format!("{}.lock", self.add_code_file_suffix(path));
        clean_path(lock_path)
    }
}

/// This enum lists all the intermediate code assemblers currently supported by kclvm.
/// Currently only supports assemble llvm intermediate code into dynamic link library.
#[derive(Clone)]
pub(crate) enum KclvmLibAssembler {
    LLVM,
}

/// KclvmLibAssembler is a dispatcher, responsible for calling corresponding methods
/// according to different types of intermediate codes.
///
/// KclvmLibAssembler implements the LibAssembler trait,
/// and calls the corresponding method according to different assembler.
impl LibAssembler for KclvmLibAssembler {
    #[inline]
    fn assemble(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        object_file_path: &str,
        args: &ExecProgramArgs,
    ) -> Result<String> {
        match &self {
            KclvmLibAssembler::LLVM => LlvmLibAssembler::default().assemble(
                compile_prog,
                import_names,
                code_file,
                object_file_path,
                args,
            ),
        }
    }

    #[inline]
    fn add_code_file_suffix(&self, code_file: &str) -> String {
        match &self {
            KclvmLibAssembler::LLVM => LlvmLibAssembler::default().add_code_file_suffix(code_file),
        }
    }

    #[inline]
    fn get_code_file_suffix(&self) -> String {
        match &self {
            KclvmLibAssembler::LLVM => LlvmLibAssembler::default().get_code_file_suffix(),
        }
    }
}

/// LlvmLibAssembler is mainly responsible for assembling the generated LLVM IR into a dynamic link library.
#[derive(Clone)]
pub(crate) struct LlvmLibAssembler;

impl LlvmLibAssembler {
    #[inline]
    fn new() -> Self {
        Self {}
    }
}

impl Default for LlvmLibAssembler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// KclvmLibAssembler implements the LibAssembler trait,
impl LibAssembler for LlvmLibAssembler {
    /// "assemble_lib" will call the [kclvm_compiler::codegen::emit_code]
    /// to generate the `.o` object file.
    #[inline]
    fn assemble(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        object_file_path: &str,
        arg: &ExecProgramArgs,
    ) -> Result<String> {
        // Clean the existed "*.o" object file.
        clean_path(object_file_path)?;

        // Compile KCL code into ".o" object file.
        emit_code(
            compile_prog,
            arg.work_dir.clone().unwrap_or("".to_string()),
            import_names,
            &EmitOptions {
                from_path: None,
                emit_path: Some(code_file),
                no_link: true,
            },
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "Internal error: compile KCL to LLVM error {}",
                e.to_string()
            )
        })?;

        Ok(object_file_path.to_string())
    }

    #[inline]
    fn add_code_file_suffix(&self, code_file: &str) -> String {
        format!("{}{}", code_file, OBJECT_FILE_SUFFIX)
    }

    #[inline]
    fn get_code_file_suffix(&self) -> String {
        OBJECT_FILE_SUFFIX.to_string()
    }
}

/// KclvmAssembler is mainly responsible for assembling the generated bytecode
/// LLVM IR or other IR code into dynamic link libraries, for multi-file kcl programs,
/// and take the result of kclvm-parser, kclvm-sema and kclvm-compiler as input.
///
/// KclvmAssembler improves the performance of kclvm by concurrently compiling kcl multi-file programs.
/// The member "thread_count" of KclvmAssembler is the number of threads in multi-file compilation.
///
/// KclvmAssembler provides an atomic operation for generating a dynamic link library for a single file
/// through KclvmLibAssembler for each thread.
pub(crate) struct KclvmAssembler {
    program: ast::Program,
    scope: ProgramScope,
    entry_file: String,
    single_file_assembler: KclvmLibAssembler,
    target: String,
    external_pkgs: HashMap<String, String>,
}

impl KclvmAssembler {
    /// Constructs an KclvmAssembler instance with a default value 4
    /// for the number of threads in multi-file compilation.
    #[inline]
    pub(crate) fn new(
        program: ast::Program,
        scope: ProgramScope,
        entry_file: String,
        single_file_assembler: KclvmLibAssembler,
        external_pkgs: HashMap<String, String>,
    ) -> Self {
        Self {
            program,
            scope,
            entry_file,
            single_file_assembler,
            target: env!("KCLVM_DEFAULT_TARGET").to_string(),
            external_pkgs,
        }
    }

    /// Clean up the path of the dynamic link libraries generated.
    /// It will remove the file in "file_path" and all the files in file_path end with ir code file suffix.
    #[inline]
    pub(crate) fn clean_path_for_genlibs(&self, file_path: &str, suffix: &str) -> Result<()> {
        let path = std::path::Path::new(file_path);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        for entry in glob::glob(&format!("{}*{}", file_path, suffix))? {
            match entry {
                Ok(path) => {
                    if path.exists() {
                        std::fs::remove_file(path)?;
                    }
                }
                Err(e) => bug!("{:?}", e),
            };
        }
        Ok(())
    }

    /// Generate cache dir from the program root path.
    /// Create cache dir if it doesn't exist.
    #[inline]
    pub(crate) fn load_cache_dir(&self, root: &str) -> Result<PathBuf> {
        let cache_dir = self.construct_cache_dir(root);
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }
        Ok(cache_dir)
    }

    #[inline]
    pub(crate) fn construct_cache_dir(&self, root: &str) -> PathBuf {
        let root = std::env::var(KCL_CACHE_PATH_ENV_VAR).unwrap_or(root.to_string());
        Path::new(&root)
            .join(".kclvm")
            .join("cache")
            .join(kclvm_version::get_version_string())
            .join(&self.target)
    }

    /// Generate the dynamic link libraries and return file paths.
    ///
    /// In the method, multiple threads will be created to concurrently generate dynamic link libraries
    /// under different package paths.
    ///
    /// This method will generate dynamic link library files (such as "*.dylib", "*.dll.lib", "*.so")
    /// and ir code files, and return the file paths of the dynamic link library files in [Vec<String>].
    ///
    /// `gen_libs` will create multiple threads and call the method provided by [KclvmLibAssembler] in each thread
    /// to generate the dynamic link library in parallel.
    pub(crate) fn gen_libs(self, args: &ExecProgramArgs) -> Result<Vec<String>> {
        self.clean_path_for_genlibs(
            DEFAULT_IR_FILE,
            &self.single_file_assembler.get_code_file_suffix(),
        )?;
        let cache_dir = self.load_cache_dir(&self.program.root)?;
        let mut compile_progs: IndexMap<
            String,
            (
                ast::Program,
                IndexMap<String, IndexMap<String, String>>,
                PathBuf,
            ),
        > = IndexMap::default();
        for (pkgpath, modules) in self.program.pkgs {
            let mut pkgs = HashMap::new();
            pkgs.insert(pkgpath.clone(), modules);
            let compile_prog = ast::Program {
                root: self.program.root.clone(),
                pkgs,
            };
            compile_progs.insert(
                pkgpath,
                (
                    compile_prog,
                    self.scope.import_names.clone(),
                    cache_dir.clone(),
                ),
            );
        }
        let mut lib_paths = vec![];
        for (pkgpath, (compile_prog, import_names, cache_dir)) in compile_progs {
            // Clone a single file assembler for one thread.
            let assembler = self.single_file_assembler.clone();
            // Generate paths for some intermediate files (*.o, *.lock).
            let entry_file = self.entry_file.clone();
            let is_main_pkg = pkgpath == kclvm_ast::MAIN_PKG;
            let file = if is_main_pkg {
                // The path to the generated files(*.o or *.lock) when the main package is compiled.
                PathBuf::from(entry_file)
            } else {
                // The path to the generated files(*.o or *.lock) when the non-main package is compiled.
                cache_dir.join(&pkgpath)
            };
            let code_file = file
                .to_str()
                .ok_or(anyhow::anyhow!("Internal error: get cache file failed"))?
                .to_string();
            let code_file_path = assembler.add_code_file_suffix(&code_file);
            let lock_file_path = format!("{}.lock", code_file_path);
            let target = self.target.clone();
            {
                // Locking file for parallel code generation.
                let mut file_lock = fslock::LockFile::open(&lock_file_path)?;
                file_lock.lock()?;

                let root = &compile_prog.root;
                // The main package does not perform cache reading and writing,
                // and other packages perform read and write caching. Because
                // KCL supports multi-file compilation, it is impossible to
                // specify a standard entry for these multi-files and cannot
                // be shared, so the cache of the main package is not read and
                // written.
                let file_path = if is_main_pkg {
                    // generate dynamic link library for single file kcl program
                    assembler.assemble(
                        &compile_prog,
                        import_names,
                        &code_file,
                        &code_file_path,
                        args,
                    )?
                } else {
                    // Read the lib path cache
                    let file_relative_path: Option<String> = load_pkg_cache(
                        root,
                        &target,
                        &pkgpath,
                        CacheOption::default(),
                        &self.external_pkgs,
                    );
                    let file_abs_path = match file_relative_path {
                        Some(file_relative_path) => {
                            let path = if file_relative_path.starts_with('.') {
                                file_relative_path.replacen('.', root, 1)
                            } else {
                                file_relative_path
                            };
                            if Path::new(&path).exists() {
                                Some(path)
                            } else {
                                None
                            }
                        }
                        None => None,
                    };
                    match file_abs_path {
                        Some(path) => path,
                        None => {
                            // Generate the object file for single file kcl program.
                            let file_path = assembler.assemble(
                                &compile_prog,
                                import_names,
                                &code_file,
                                &code_file_path,
                                args,
                            )?;
                            let lib_relative_path = file_path.replacen(root, ".", 1);
                            let _ = save_pkg_cache(
                                root,
                                &target,
                                &pkgpath,
                                lib_relative_path,
                                CacheOption::default(),
                                &self.external_pkgs,
                            );
                            file_path
                        }
                    }
                };
                file_lock.unlock()?;
                lib_paths.push(file_path);
            };
        }
        self.single_file_assembler
            .clean_lock_file(&self.entry_file)?;
        Ok(lib_paths)
    }
}

#[inline]
pub(crate) fn clean_path(path: &str) -> Result<()> {
    if Path::new(path).exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
