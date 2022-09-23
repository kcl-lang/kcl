use crate::command::Command;
use indexmap::IndexMap;
use kclvm_ast::ast::{self, Program};
use kclvm_compiler::codegen::{llvm::emit_code, EmitOptions};
use kclvm_config::cache::{load_pkg_cache, save_pkg_cache, CacheOption};
use kclvm_error::bug;
use kclvm_sema::resolver::scope::ProgramScope;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};
use threadpool::ThreadPool;

/// IR code file suffix.
const DEFAULT_IR_FILE: &str = "_a.out";
/// Default codegen timeout.
const DEFAULT_TIME_OUT: u64 = 5;

/// LibAssembler trait is used to indicate the general interface
/// that must be implemented when different intermediate codes are assembled
/// into dynamic link libraries.
///
/// Note: LibAssembler is only for single file kcl program. For multi-file kcl programs,
/// KclvmAssembler is provided to support for multi-file parallel compilation to improve
/// the performance of the compiler.
pub(crate) trait LibAssembler {
    /// Add a suffix to the file name according to the file suffix of different intermediate code files.
    /// e.g. LLVM IR -> code_file : "/test_dir/test_code_file" -> return : "/test_dir/test_code_file.ll"
    fn add_code_file_suffix(&self, code_file: &str) -> String;

    /// Return the file suffix of different intermediate code files.
    /// e.g. LLVM IR -> return : ".ll"
    fn get_code_file_suffix(&self) -> String;

    /// Assemble different intermediate codes into dynamic link libraries for single file kcl program.
    /// Returns the path of the dynamic link library.
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
    /// "code_file_path" is the full filename of the generated intermediate code file with suffix.
    /// e.g. code_file_path : "/test_dir/test_code_file.ll"
    ///
    /// "lib_path" is the file path of the dynamic link library.
    /// e.g. lib_path : "/test_dir/test_code_file.ll.dylib" (mac)
    /// e.g. lib_path : "/test_dir/test_code_file.ll.dll.lib" (windows)
    /// e.g. lib_path : "/test_dir/test_code_file.ll.so" (ubuntu)
    fn assemble_lib(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        code_file_path: &str,
        lib_path: &str,
    ) -> String;

    #[inline]
    fn clean_lock_file(&self, path: &str) {
        let lock_path = &format!("{}.lock", self.add_code_file_suffix(path));
        clean_path(lock_path);
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
    fn assemble_lib(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        code_file_path: &str,
        lib_path: &str,
    ) -> String {
        match &self {
            KclvmLibAssembler::LLVM => LlvmLibAssembler::default().assemble_lib(
                compile_prog,
                import_names,
                code_file,
                code_file_path,
                lib_path,
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
    /// to generate IR file.
    ///
    /// And then assemble the dynamic link library based on the LLVM IR,
    ///
    /// At last remove the codegen temp files and return the dynamic link library path.
    #[inline]
    fn assemble_lib(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        code_file_path: &str,
        lib_path: &str,
    ) -> String {
        // clean "*.ll" file path.
        clean_path(&code_file_path.to_string());

        // gen LLVM IR code into ".ll" file.
        emit_code(
            compile_prog,
            import_names,
            &EmitOptions {
                from_path: None,
                emit_path: Some(code_file),
                no_link: true,
            },
        )
        .expect("Compile KCL to LLVM error");

        let mut cmd = Command::new();
        let gen_lib_path = cmd.run_clang_single(code_file_path, lib_path);

        clean_path(&code_file_path.to_string());
        gen_lib_path
    }

    #[inline]
    fn add_code_file_suffix(&self, code_file: &str) -> String {
        format!("{}.ll", code_file)
    }

    #[inline]
    fn get_code_file_suffix(&self) -> String {
        ".ll".to_string()
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
    thread_count: usize,
    program: ast::Program,
    scope: ProgramScope,
    entry_file: String,
    single_file_assembler: KclvmLibAssembler,
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
    ) -> Self {
        Self {
            thread_count: 4,
            program,
            scope,
            entry_file,
            single_file_assembler,
        }
    }

    /// Clean up the path of the dynamic link libraries generated.
    /// It will remove the file in "file_path" and all the files in file_path end with ir code file suffix.
    #[inline]
    pub(crate) fn clean_path_for_genlibs(&self, file_path: &str, suffix: &str) {
        let path = std::path::Path::new(file_path);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        for entry in glob::glob(&format!("{}*{}", file_path, suffix)).unwrap() {
            match entry {
                Ok(path) => {
                    if path.exists() {
                        std::fs::remove_file(path).unwrap();
                    }
                }
                Err(e) => bug!("{:?}", e),
            };
        }
    }

    /// Generate cache dir from the program root path.
    /// Create cache dir if it doesn't exist.
    #[inline]
    pub(crate) fn load_cache_dir(&self, prog_root_name: &str) -> PathBuf {
        let cache_dir = self.construct_cache_dir(prog_root_name);
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).unwrap();
        }
        cache_dir
    }

    #[inline]
    pub(crate) fn construct_cache_dir(&self, prog_root_name: &str) -> PathBuf {
        Path::new(prog_root_name)
            .join(".kclvm")
            .join("cache")
            .join(kclvm_version::get_full_version())
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
    pub(crate) fn gen_libs(self) -> Vec<String> {
        self.clean_path_for_genlibs(
            DEFAULT_IR_FILE,
            &self.single_file_assembler.get_code_file_suffix(),
        );
        let cache_dir = self.load_cache_dir(&self.program.root);
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
                main: self.program.main.clone(),
                pkgs,
                cmd_args: vec![],
                cmd_overrides: vec![],
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
        let pool = ThreadPool::new(self.thread_count);
        let (tx, rx) = channel();
        let prog_count = compile_progs.len();
        for (pkgpath, (compile_prog, import_names, cache_dir)) in compile_progs {
            let tx = tx.clone();
            // clone a single file assembler for one thread.
            let assembler = self.single_file_assembler.clone();

            let code_file = self.entry_file.clone();
            let code_file_path = assembler.add_code_file_suffix(&code_file);
            let lock_file_path = format!("{}.lock", code_file_path);
            let lib_path = format!("{}{}", code_file, Command::get_lib_suffix());

            pool.execute(move || {
                // Locking file for parallel code generation.
                let mut file_lock = fslock::LockFile::open(&lock_file_path)
                    .expect(&format!("{} not found", lock_file_path));
                file_lock.lock().unwrap();

                let root = &compile_prog.root;
                let is_main_pkg = pkgpath == kclvm_ast::MAIN_PKG;
                // The main package does not perform cache reading and writing,
                // and other packages perform read and write caching. Because
                // KCL supports multi-file compilation, it is impossible to
                // specify a standard entry for these multi-files and cannot
                // be shared, so the cache of the main package is not read and
                // written.
                let lib_path = if is_main_pkg {
                    // generate dynamic link library for single file kcl program
                    assembler.assemble_lib(
                        &compile_prog,
                        import_names,
                        &code_file,
                        &code_file_path,
                        &lib_path,
                    )
                } else {
                    let file = cache_dir.join(&pkgpath);
                    // Read the lib path cache
                    let lib_relative_path: Option<String> =
                        load_pkg_cache(root, &pkgpath, CacheOption::default());
                    let lib_abs_path = match lib_relative_path {
                        Some(lib_relative_path) => {
                            let path = if lib_relative_path.starts_with('.') {
                                lib_relative_path.replacen('.', root, 1)
                            } else {
                                lib_relative_path
                            };
                            if Path::new(&path).exists() {
                                Some(path)
                            } else {
                                None
                            }
                        }
                        None => None,
                    };
                    match lib_abs_path {
                        Some(path) => path,
                        None => {
                            let code_file = file.to_str().unwrap();
                            let code_file_path = assembler.add_code_file_suffix(&code_file);
                            let lib_path = format!("{}{}", code_file, Command::get_lib_suffix());
                            // generate dynamic link library for single file kcl program
                            let lib_path = assembler.assemble_lib(
                                &compile_prog,
                                import_names,
                                &code_file,
                                &code_file_path,
                                &lib_path,
                            );
                            let lib_relative_path = lib_path.replacen(root, ".", 1);
                            save_pkg_cache(
                                root,
                                &pkgpath,
                                lib_relative_path,
                                CacheOption::default(),
                            );
                            lib_path
                        }
                    }
                };
                file_lock.unlock().unwrap();
                tx.send(lib_path)
                    .expect("channel will be there waiting for the pool");
            });
        }
        // Get all codegen results from the channel with timeout
        let timeout: u64 = match env::var("KCLVM_CODE_GEN_TIMEOUT") {
            Ok(timeout_str) => timeout_str.parse().unwrap_or(DEFAULT_TIME_OUT),
            Err(_) => DEFAULT_TIME_OUT,
        };
        let mut lib_paths = vec![];
        for _ in 0..prog_count {
            let lib_path = rx
                .recv_timeout(std::time::Duration::from_secs(timeout))
                .unwrap();
            lib_paths.push(lib_path);
        }
        self.single_file_assembler.clean_lock_file(&self.entry_file);
        lib_paths
    }
}

#[inline]
pub(crate) fn clean_path(path: &str) {
    if Path::new(path).exists() {
        std::fs::remove_file(&path).unwrap();
    }
}
