//! The `kclvm` command-line interface.

#[macro_use]
extern crate clap;

use indexmap::IndexMap;
use std::path::PathBuf;
use std::thread;
use std::{collections::HashMap, path::Path};

use clap::ArgMatches;
use kclvm_ast::ast;
use kclvm_compiler::codegen::{llvm::emit_code, EmitOptions};
use kclvm_config::cache::*;
use kclvm_config::settings::{load_file, merge_settings, SettingsFile};
use kclvm_parser::{load_program, parse_file};
use kclvm_runner::command::Command;
use kclvm_sema::resolver::resolve_program;

fn main() {
    let matches = clap_app!(kcl =>
        (@subcommand run =>
            (@arg INPUT: ... "Sets the input file to use")
            (@arg OUTPUT: -o --output +takes_value "Sets the LLVM IR/BC output file path")
            (@arg SETTING: ... -Y --setting "Sets the input file to use")
            (@arg EMIT_TYPE: --emit +takes_value "Sets the emit type, expect (ast)")
            (@arg BC_PATH: --bc +takes_value "Sets the linked LLVM bitcode file path")
            (@arg verbose: -v --verbose "Print test information verbosely")
            (@arg disable_none: -n --disable-none "Disable dumping None values")
            (@arg debug: -d --debug "Run in debug mode (for developers only)")
            (@arg sort_key: -k --sort "Sort result keys")
            (@arg ARGUMENT: ... -D --argument "Specify the top-level argument")
        )
    )
    .get_matches();
    if let Some(matches) = matches.subcommand_matches("run") {
        if let Some(files) = matches.values_of("INPUT") {
            let files: Vec<&str> = files.into_iter().collect::<Vec<&str>>();
            if let Some(emit_ty) = matches.value_of("EMIT_TYPE") {
                if emit_ty == "ast" {
                    let module = parse_file(files[0], None);
                    println!("{}", serde_json::to_string(&module).unwrap())
                }
            } else {
                // load ast
                let mut program = load_program(&files, None);
                let scope = resolve_program(&mut program);

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
                let mut theads = vec![];
                for (pkgpath, (compile_prog, import_names, cache_dir)) in compile_progs {
                    let t = thread::spawn(move || {
                        let root = &compile_prog.root;
                        let is_main_pkg = pkgpath == kclvm_ast::MAIN_PKG;
                        let file = if is_main_pkg {
                            let main_file =
                                format!("{}{}", pkgpath, chrono::Local::now().timestamp_nanos());
                            cache_dir.join(&main_file)
                        } else {
                            cache_dir.join(&pkgpath)
                        };
                        let lock_file =
                            format!("{}.lock", cache_dir.join(&pkgpath).to_str().unwrap());
                        let ll_file = file.to_str().unwrap();
                        let ll_path = format!("{}.ll", ll_file);
                        let dylib_path = format!("{}{}", ll_file, Command::get_lib_suffix());
                        let mut ll_path_lock = fslock::LockFile::open(&lock_file).unwrap();
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
                            let mut cmd = Command::new(0);
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
                                    let mut cmd = Command::new(0);
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
                        dylib_path
                    });
                    theads.push(t);
                }
                let mut dylib_paths = vec![];
                for t in theads {
                    let dylib_path = t.join().unwrap();
                    dylib_paths.push(dylib_path);
                }
                let mut cmd = Command::new(0);
                // link all dylibs
                let dylib_path = cmd.link_dylibs(&dylib_paths, "");
                // Config build
                let settings = build_settings(&matches);
                cmd.run_dylib_with_settings(&dylib_path, settings).unwrap();
                for dylib_path in dylib_paths {
                    if dylib_path.contains(kclvm_ast::MAIN_PKG) && Path::new(&dylib_path).exists() {
                        std::fs::remove_file(&dylib_path).unwrap();
                    }
                }
            }
        } else {
            println!("{}", matches.usage());
        }
    } else {
        println!("{}", matches.usage());
    }
}

/// Build settings from arg matches.
fn build_settings(matches: &ArgMatches) -> SettingsFile {
    let debug_mode = matches.occurrences_of("debug") > 0;
    let disable_none = matches.occurrences_of("disable_none") > 0;

    let mut settings = if let Some(files) = matches.values_of("SETTING") {
        let files: Vec<&str> = files.into_iter().collect::<Vec<&str>>();
        merge_settings(
            &files
                .iter()
                .map(|f| load_file(f))
                .collect::<Vec<SettingsFile>>(),
        )
    } else {
        SettingsFile::new()
    };
    if let Some(config) = &mut settings.kcl_cli_configs {
        config.debug = Some(debug_mode);
        config.disable_none = Some(disable_none);
    }
    settings
}
