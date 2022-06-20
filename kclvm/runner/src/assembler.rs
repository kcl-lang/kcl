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
const IR_FILE: &str = "_a.out";
/// Default codegen timeout.
const DEFAULT_TIME_OUT: u64 = 60;

/// LibAssembler trait is used to indicate the general interface
/// that must be implemented when different intermediate codes are assembled
/// into dynamic link libraries.
///
/// Note: LibAssembler is only for single file kcl program. For multi-file kcl programs,
/// KclvmAssembler is provided to support for multi-file parallel compilation to improve
/// the performance of the compiler.
pub trait LibAssembler {
    /// Add a suffix to the file name according to the file suffix of different intermediate codes.
    /// e.g. LLVM IR
    /// code_file : "/test_dir/test_code_file"
    /// return : "/test_dir/test_code_file.ll"
    fn add_code_file_suffix(&self, code_file: &str) -> String;

    /// Return the file suffix of different intermediate codes.
    /// e.g. LLVM IR
    /// return : ".ll"
    fn get_code_file_suffix(&self) -> &str;

    /// Assemble different intermediate codes into dynamic link libraries for single file kcl program.
    ///
    /// Inputs:
    /// compile_prog: Reference of kcl program ast.
    ///
    /// "import_names" is import pkgpath and name of kcl program.
    /// Type of import_names is "IndexMap<kcl_file_name, IndexMap<import_name, import_path>>".
    /// "kcl_file_name" is the kcl file name string.
    /// "import_name" is the name string of import stmt.
    /// e.g. "import test/main_pkg as main", "main" is an import_name.
    /// "import_path" is the path string of import stmt.
    /// e.g. "import test/main_pkg as main", "test/main_pkg" is an import_path.
    /// import_names is from "ProgramScope.import_names" returned by "resolve_program" after resolving kcl ast by kclvm-sema.
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
    ///
    /// "plugin_agent" is a pointer to the plugin address.
    ///
    /// Returns the path of the dynamic link library.
    fn assemble_lib(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        code_file_path: &str,
        lib_path: &str,
        plugin_agent: &u64,
    ) -> String;

    /// This method is prepared for concurrent compilation in KclvmAssembler.
    /// It is an atomic method executed by each thread in concurrent compilation.
    ///
    /// This method will take the above method “assemble_lib” as a hook method to
    /// generate the dynamic link library, and lock the file before calling “assemble_lib”,
    /// unlocked after the call ends,
    #[inline]
    fn lock_file_and_gen_lib(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        file: &Path,
        plugin_agent: &u64,
    ) -> String {
        // e.g. LLVM IR
        // code_file: file_name
        // code_file_path: file_name.ll
        // lock_file_path: file_name.dll.lib or file_name.lib or file_name.so
        let code_file = file.to_str().unwrap();
        let code_file_path = &self.add_code_file_suffix(code_file);
        let lock_file_path = &format!("{}.lock", code_file_path);
        let lib_path = format!("{}{}", code_file, Command::get_lib_suffix());

        // Locking file for parallel code generation.
        let mut file_lock = fslock::LockFile::open(lock_file_path).unwrap();
        file_lock.lock().unwrap();

        // Calling the hook method will generate the corresponding intermediate code
        // according to the implementation of method "assemble_lib".
        let gen_lib_path = self.assemble_lib(
            compile_prog,
            import_names,
            code_file,
            code_file_path,
            &lib_path,
            plugin_agent,
        );

        // Unlock file
        file_lock.unlock().unwrap();

        gen_lib_path
    }

    // Clean file path
    // Delete the file in "path".
    #[inline]
    fn clean_path(&self, path: &str) {
        if Path::new(path).exists() {
            std::fs::remove_file(&path).unwrap();
        }
    }

    // Clean lock file
    // Clear the lock files generated during concurrent compilation.
    #[inline]
    fn clean_lock_file(&self, path: &str) {
        let lock_path = &format!("{}.lock", self.add_code_file_suffix(path));
        self.clean_path(lock_path);
    }
}

/// This enum lists all the intermediate code assemblers currently supported by kclvm.
/// Currently only supports assemble llvm intermediate code into dynamic link library.
#[derive(Clone)]
pub enum KclvmLibAssembler {
    LLVM(LlvmLibAssembler),
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
        plugin_agent: &u64,
    ) -> String {
        match &self {
            KclvmLibAssembler::LLVM(llvm_a) => llvm_a.assemble_lib(
                compile_prog,
                import_names,
                code_file,
                code_file_path,
                lib_path,
                plugin_agent,
            ),
        }
    }

    #[inline]
    fn add_code_file_suffix(&self, code_file: &str) -> String {
        match &self {
            KclvmLibAssembler::LLVM(llvm_a) => llvm_a.add_code_file_suffix(code_file),
        }
    }

    #[inline]
    fn get_code_file_suffix(&self) -> &str {
        match &self {
            KclvmLibAssembler::LLVM(llvm_a) => llvm_a.get_code_file_suffix(),
        }
    }

    #[inline]
    fn lock_file_and_gen_lib(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        file: &Path,
        plugin_agent: &u64,
    ) -> String {
        match &self {
            KclvmLibAssembler::LLVM(llvm_a) => {
                llvm_a.lock_file_and_gen_lib(compile_prog, import_names, file, plugin_agent)
            }
        }
    }
}

/// LlvmLibAssembler is mainly responsible for assembling the generated LLVM IR into a dynamic link library.
#[derive(Clone)]
pub struct LlvmLibAssembler;

/// KclvmLibAssembler implements the LibAssembler trait,
impl LibAssembler for LlvmLibAssembler {
    /// "assemble_lib" will call the [kclvm_compiler::codegen::emit_code]
    /// to generate IR file.
    ///
    /// And then assemble the dynamic link library based on the LLVM IR,
    ///
    /// At last remove the codegen temp files and return the dynamic link library path.
    /// # Examples
    ///
    /// ```no_run
    /// use kclvm_runner::runner::ExecProgramArgs;
    /// use kclvm_parser::load_program;
    /// use kclvm_sema::resolver::resolve_program;
    /// use kclvm_runner::assembler::LlvmLibAssembler;
    /// use crate::kclvm_runner::assembler::LibAssembler;
    ///
    /// // default args and configuration
    /// let mut args = ExecProgramArgs::default();
    /// let k_path = "./src/test_datas/init_check_order_0/main.k";
    /// args.k_filename_list.push(k_path.to_string());
    /// let plugin_agent = 0;
    /// let files = args.get_files();
    /// let opts = args.get_load_program_options();
    ///
    /// // parse and resolve kcl
    /// let mut program = load_program(&files, Some(opts)).unwrap();
    /// let scope = resolve_program(&mut program);
    ///
    /// // tmp file
    /// let temp_entry_file = "test_entry_file";
    /// let temp_entry_file_path = &format!("{}.ll", temp_entry_file);
    /// let temp_entry_file_lib = &format!("{}.dylib", temp_entry_file);
    ///
    /// // assemble libs
    /// let llvm_assembler = LlvmLibAssembler{};
    /// let lib_file = llvm_assembler.assemble_lib(
    ///      &program,
    ///      scope.import_names.clone(),
    ///      temp_entry_file,
    ///      temp_entry_file_path,
    ///      temp_entry_file_lib,
    ///      &plugin_agent
    /// );
    /// let lib_path = std::path::Path::new(&lib_file);
    /// assert_eq!(lib_path.exists(), true);
    /// llvm_assembler.clean_path(&lib_file);
    /// assert_eq!(lib_path.exists(), false);
    /// ```
    #[inline]
    fn assemble_lib(
        &self,
        compile_prog: &Program,
        import_names: IndexMap<String, IndexMap<String, String>>,
        code_file: &str,
        code_file_path: &str,
        lib_path: &str,
        plugin_agent: &u64,
    ) -> String {
        // clean "*.ll" file path.
        self.clean_path(&code_file_path.to_string());

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

        // assemble lib
        let mut cmd = Command::new(*plugin_agent);
        let gen_lib_path = cmd.run_clang_single(code_file_path, lib_path);

        // clean "*.ll" file path
        self.clean_path(&code_file_path.to_string());
        gen_lib_path
    }

    /// Add ".ll" suffix to a file path.
    #[inline]
    fn add_code_file_suffix(&self, code_file: &str) -> String {
        format!("{}.ll", code_file)
    }

    /// Get String ".ll"
    #[inline]
    fn get_code_file_suffix(&self) -> &str {
        ".ll"
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
pub struct KclvmAssembler {
    thread_count: usize,
}

impl KclvmAssembler {
    /// Constructs an KclvmAssembler instance with a default value 4
    /// for the number of threads in multi-file compilation.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_runner::assembler::KclvmAssembler;
    ///
    /// let assembler = KclvmAssembler::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self { thread_count: 4 }
    }

    /// Constructs an KclvmAssembler instance with a value
    /// for the number of threads in multi-file compilation.
    ///
    /// # Examples
    ///
    /// ```
    /// use kclvm_runner::assembler::KclvmAssembler;
    ///
    /// let assembler = KclvmAssembler::new_with_thread_count(5);
    /// ```
    #[inline]
    pub fn new_with_thread_count(thread_count: usize) -> Self {
        if thread_count == 0 {
            bug!("Illegal thread count in multi-file compilation");
        }
        Self { thread_count }
    }

    /// Clean up the path of the dynamic link libraries generated.
    /// It will remove the file in "file_path" and all the files in file_path end with ir code file suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use std::fs::File;
    /// use kclvm_runner::assembler::KclvmAssembler;
    ///
    /// // create test dir
    /// std::fs::create_dir_all("./src/test_datas/test_clean").unwrap();
    ///
    /// // File name and suffix for test
    /// let test_path = "./src/test_datas/test_clean/test.out";
    /// let file_suffix = ".ll";
    ///
    /// // Create file "./src/test_datas/test_clean/test.out"
    /// File::create(test_path);
    /// let path = std::path::Path::new(test_path);
    /// assert_eq!(path.exists(), true);
    ///
    /// // Delete file "./src/test_datas/test_clean/test.out" and "./src/test_datas/test_clean/test.out*.ll"
    /// KclvmAssembler::new().clean_path_for_genlibs(test_path, file_suffix);
    /// assert_eq!(path.exists(), false);
    ///
    /// // Delete files whose filename end with "*.ll"
    /// let test1 = &format!("{}{}", test_path, ".test1.ll");
    /// let test2 = &format!("{}{}", test_path, ".test2.ll");
    /// File::create(test1);
    /// File::create(test2);
    /// let path1 = std::path::Path::new(test1);
    /// let path2 = std::path::Path::new(test2);
    /// assert_eq!(path1.exists(), true);
    /// assert_eq!(path2.exists(), true);
    ///
    /// // Delete file "./src/test_datas/test_clean/test.out" and "./src/test_datas/test_clean/test.out*.ll"
    /// KclvmAssembler::new().clean_path_for_genlibs(test_path, file_suffix);
    /// assert_eq!(path1.exists(), false);
    /// assert_eq!(path2.exists(), false);
    /// ```
    #[inline]
    pub fn clean_path_for_genlibs(&self, file_path: &str, suffix: &str) {
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

    /// Generate cache dir from the program root path. Create cache dir if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use kclvm_runner::assembler::KclvmAssembler;
    ///
    /// let expected_dir = "test_prog_name/.kclvm/cache/0.4.2-e07ed7af0d9bd1e86a3131714e4bd20c";
    /// let path = std::path::Path::new(expected_dir);
    /// assert_eq!(path.exists(), false);
    ///
    /// let cache_dir = KclvmAssembler::new().load_cache_dir("test_prog_name");
    /// assert_eq!(cache_dir.display().to_string(), expected_dir);
    ///
    /// let path = std::path::Path::new(expected_dir);
    /// assert_eq!(path.exists(), true);
    ///
    /// fs::remove_dir(expected_dir);
    /// assert_eq!(path.exists(), false);
    /// ```
    #[inline]
    pub fn load_cache_dir(&self, prog_root_name: &str) -> PathBuf {
        let cache_dir = Path::new(prog_root_name)
            .join(".kclvm")
            .join("cache")
            .join(kclvm_version::get_full_version());
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).unwrap();
        }
        cache_dir
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs;
    /// use kclvm_parser::load_program;
    /// use kclvm_runner::runner::ExecProgramArgs;
    /// use kclvm_runner::assembler::KclvmAssembler;
    /// use kclvm_runner::assembler::KclvmLibAssembler;
    /// use kclvm_runner::assembler::LlvmLibAssembler;
    /// use kclvm_sema::resolver::resolve_program;
    ///
    /// let plugin_agent = 0;
    ///
    /// let args = ExecProgramArgs::default();
    /// let opts = args.get_load_program_options();
    ///
    /// let kcl_path = "./src/test_datas/init_check_order_0/main.k";
    /// let mut prog = load_program(&[kcl_path], Some(opts)).unwrap();
    /// let scope = resolve_program(&mut prog);
    ///        
    /// let lib_paths = KclvmAssembler::new().gen_libs(
    ///                     prog,
    ///                     scope,
    ///                     plugin_agent,
    ///                     &("test_entry_file_name".to_string()),
    ///                     KclvmLibAssembler::LLVM(LlvmLibAssembler {}));
    /// assert_eq!(lib_paths.len(), 1);
    ///
    /// let expected_lib_path = fs::canonicalize("./test_entry_file_name.dylib").unwrap().display().to_string();
    /// assert_eq!(*lib_paths.get(0).unwrap(), expected_lib_path);
    ///
    /// let path = std::path::Path::new(&expected_lib_path);
    /// assert_eq!(path.exists(), true);
    ///
    /// KclvmAssembler::new().clean_path_for_genlibs(&expected_lib_path, ".dylib");
    /// assert_eq!(path.exists(), false);
    ///```
    pub fn gen_libs(
        &self,
        program: ast::Program,
        scope: ProgramScope,
        plugin_agent: u64,
        entry_file: &String,
        single_file_assembler: KclvmLibAssembler,
    ) -> Vec<String> {
        // Clean the code generated path.
        self.clean_path_for_genlibs(IR_FILE, single_file_assembler.get_code_file_suffix());
        // Load cache
        let cache_dir = self.load_cache_dir(&program.root);

        let mut compile_progs: IndexMap<
            String,
            (
                ast::Program,
                IndexMap<String, IndexMap<String, String>>,
                PathBuf,
            ),
        > = IndexMap::default();
        for (pkgpath, modules) in program.pkgs {
            let mut pkgs = HashMap::new();
            pkgs.insert(pkgpath.clone(), modules);
            let compile_prog = ast::Program {
                root: program.root.clone(),
                main: program.main.clone(),
                pkgs,
                cmd_args: vec![],
                cmd_overrides: vec![],
            };
            compile_progs.insert(
                pkgpath,
                (compile_prog, scope.import_names.clone(), cache_dir.clone()),
            );
        }
        let pool = ThreadPool::new(self.thread_count);
        let (tx, rx) = channel();
        let prog_count = compile_progs.len();
        for (pkgpath, (compile_prog, import_names, cache_dir)) in compile_progs {
            let tx = tx.clone();
            let temp_entry_file = entry_file.clone();
            // clone a single file assembler for one thread.
            let assembler = single_file_assembler.clone();
            pool.execute(move || {
                let root = &compile_prog.root;
                let is_main_pkg = pkgpath == kclvm_ast::MAIN_PKG;
                // The main package does not perform cache reading and writing,
                // and other packages perform read and write caching. Because
                // KCL supports multi-file compilation, it is impossible to
                // specify a standard entry for these multi-files and cannot
                // be shared, so the cache of the main package is not read and
                // written.
                let lib_path = if is_main_pkg {
                    let file = PathBuf::from(&temp_entry_file);
                    // generate dynamic link library for single file kcl program
                    assembler.lock_file_and_gen_lib(
                        &compile_prog,
                        import_names,
                        &file,
                        &plugin_agent,
                    )
                } else {
                    let file = cache_dir.join(&pkgpath);
                    // Read the lib cache
                    let lib_relative_path: Option<String> =
                        load_pkg_cache(root, &pkgpath, CacheOption::default());
                    match lib_relative_path {
                        Some(lib_relative_path) => {
                            if lib_relative_path.starts_with('.') {
                                lib_relative_path.replacen('.', root, 1)
                            } else {
                                lib_relative_path
                            }
                        }
                        None => {
                            // generate dynamic link library for single file kcl program
                            let lib_path = assembler.lock_file_and_gen_lib(
                                &compile_prog,
                                import_names,
                                &file,
                                &plugin_agent,
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
        // Clean the lock file.
        single_file_assembler.clean_lock_file(entry_file);
        lib_paths
    }
}

impl Default for KclvmAssembler {
    fn default() -> Self {
        Self::new()
    }
}
