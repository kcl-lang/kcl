extern crate serde;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_compiler::codegen::{llvm::emit_code, EmitOptions};
use kclvm_config::cache::*;
use kclvm_parser::load_program;
use kclvm_sema::resolver::resolve_program;

use kclvm_runner::command::Command;
use kclvm_runner::runner::*;
use kclvm_tools::query::apply_overrides;

#[no_mangle]
pub extern "C" fn kclvm_cli_run(args: *const i8, plugin_agent: *const i8) -> *const i8 {
    let prev_hook = std::panic::take_hook();

    // disable print panic info
    std::panic::set_hook(Box::new(|_info| {}));
    let kclvm_cli_run_unsafe_result =
        std::panic::catch_unwind(|| kclvm_cli_run_unsafe(args, plugin_agent));
    std::panic::set_hook(prev_hook);

    match kclvm_cli_run_unsafe_result {
        Ok(result) => match result {
            Ok(result) => {
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const i8
            }
            Err(result) => {
                let result = format!("ERROR:{}", result);
                let c_string =
                    std::ffi::CString::new(result.as_str()).expect("CString::new failed");
                let ptr = c_string.into_raw();
                ptr as *const i8
            }
        },
        Err(panic_err) => {
            let err_message = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<&String>() {
                (*s).clone()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                (*s).clone()
            } else {
                "".to_string()
            };

            let result = format!("ERROR:{:}", err_message);
            let c_string = std::ffi::CString::new(result.as_str()).expect("CString::new failed");
            let ptr = c_string.into_raw();
            ptr as *const i8
        }
    }
}

pub fn kclvm_cli_run_unsafe(args: *const i8, plugin_agent: *const i8) -> Result<String, String> {
    let args = ExecProgramArgs::from_str(kclvm::c2str(args));
    let plugin_agent = plugin_agent as u64;

    let files = args.get_files();
    let opts = args.get_load_program_options();

    // load ast
    let mut program = load_program(&files, Some(opts))?;
    apply_overrides(&mut program, &args.overrides, &[]);
    let scope = resolve_program(&mut program);
    scope.check_scope_diagnostics();
    // gen bc or ll file
    let ll_file = "_a.out";
    let path = std::path::Path::new(ll_file);
    if path.exists() {
        std::fs::remove_file(path).unwrap();
    }
    for entry in glob::glob(&format!("{}*.ll", ll_file)).unwrap() {
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
            let mut ll_path_lock = fslock::LockFile::open(&format!("{}.lock", ll_path)).unwrap();
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

                        save_pkg_cache(root, &pkgpath, dylib_relative_path, CacheOption::default());
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
    let dylib_paths = rx.iter().take(prog_count).collect::<Vec<String>>();
    let mut cmd = Command::new(plugin_agent);
    // link all dylibs
    let dylib_path = cmd.link_dylibs(&dylib_paths, "");

    // Config uild
    // run dylib
    let runner = KclvmRunner::new(
        dylib_path.as_str(),
        Some(KclvmRunnerOptions {
            plugin_agent_ptr: plugin_agent,
        }),
    );
    runner.run(&args)
}
