use indexmap::IndexMap;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};
use threadpool::ThreadPool;

use crate::command::Command;
use kclvm_ast::ast;
use kclvm_compiler::codegen::{llvm::emit_code, EmitOptions};
use kclvm_config::cache::{load_pkg_cache, save_pkg_cache, CacheOption};
use kclvm_sema::resolver::scope::ProgramScope;

/// LLVM IR file suffix.
const LL_FILE: &str = "_a.out";

/// KclvmAssembler is mainly responsible for generating bytecode,
/// LLVM IR or dylib, and take the result of kclvm-parser and kclvm-sema
/// as input.
pub struct KclvmAssembler;
impl KclvmAssembler {
    /// Generate the dylibs and return file paths from .
    ///
    /// In the method, 4 threads will be created to concurrently
    /// generate dylibs under different package paths.
    /// This method will generate “.out” and ".ll" files,
    /// and return the file paths of the generated files in Vec<String>.
    pub fn gen_dylibs(
        program: ast::Program,
        scope: ProgramScope,
        plugin_agent: u64,
    ) -> Vec<String> {
        // gen bc or ll_file
        let path = std::path::Path::new(LL_FILE);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        for entry in glob::glob(&format!("{}*.ll", LL_FILE)).unwrap() {
            match entry {
                Ok(path) => {
                    if path.exists() {
                        std::fs::remove_file(path).unwrap();
                    }
                }
                Err(e) => println!("{:?}", e),
            };
        }

        let cache_dir = Path::new(&program.root)
            .join(".kclvm")
            .join("cache")
            .join(kclvm_version::get_full_version());
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).unwrap();
        }
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
        let pool = ThreadPool::new(4);
        let (tx, rx) = channel();
        let prog_count = compile_progs.len();
        for (pkgpath, (compile_prog, import_names, cache_dir)) in compile_progs {
            let tx = tx.clone();
            pool.execute(move || {
                let root = &compile_prog.root;
                let is_main_pkg = pkgpath == kclvm_ast::MAIN_PKG;
                let file = if is_main_pkg {
                    PathBuf::from(&pkgpath)
                } else {
                    cache_dir.join(&pkgpath)
                };
                let ll_file = file.to_str().unwrap();
                let ll_path = format!("{}.ll", ll_file);
                let dylib_path = format!("{}{}", ll_file, Command::get_lib_suffix());
                let mut ll_path_lock =
                    fslock::LockFile::open(&format!("{}.lock", ll_path)).unwrap();
                ll_path_lock.lock().unwrap();
                if Path::new(&ll_path).exists() {
                    std::fs::remove_file(&ll_path).unwrap();
                }
                let dylib_path = if is_main_pkg {
                    emit_code(
                        &compile_prog,
                        import_names,
                        &EmitOptions {
                            from_path: None,
                            emit_path: Some(&ll_file),
                            no_link: true,
                        },
                    )
                    .expect("Compile KCL to LLVM error");
                    let mut cmd = Command::new(plugin_agent);
                    cmd.run_clang_single(&ll_path, &dylib_path)
                } else {
                    // If AST module has been modified, ignore the dylib cache
                    let dylib_relative_path: Option<String> =
                        load_pkg_cache(root, &pkgpath, CacheOption::default());
                    match dylib_relative_path {
                        Some(dylib_relative_path) => {
                            if dylib_relative_path.starts_with('.') {
                                dylib_relative_path.replacen(".", root, 1)
                            } else {
                                dylib_relative_path
                            }
                        }
                        None => {
                            emit_code(
                                &compile_prog,
                                import_names,
                                &EmitOptions {
                                    from_path: None,
                                    emit_path: Some(&ll_file),
                                    no_link: true,
                                },
                            )
                            .expect("Compile KCL to LLVM error");
                            let mut cmd = Command::new(plugin_agent);
                            let dylib_path = cmd.run_clang_single(&ll_path, &dylib_path);
                            let dylib_relative_path = dylib_path.replacen(root, ".", 1);

                            save_pkg_cache(
                                root,
                                &pkgpath,
                                dylib_relative_path,
                                CacheOption::default(),
                            );
                            dylib_path
                        }
                    }
                };
                if Path::new(&ll_path).exists() {
                    std::fs::remove_file(&ll_path).unwrap();
                }
                ll_path_lock.unlock().unwrap();
                tx.send(dylib_path)
                    .expect("channel will be there waiting for the pool");
            });
        }
        rx.iter().take(prog_count).collect::<Vec<String>>()
    }
}
