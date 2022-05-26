use std::{env, path::PathBuf};

use super::runner::*;
use kclvm::ValueRef;
use kclvm_config::settings::SettingsFile;

#[derive(Debug)]
pub struct Command {
    clang_path: String,
    rust_libstd_dylib: String,
    executable_root: String,
    plugin_method_ptr: u64,
}

impl Command {
    pub fn new(plugin_method_ptr: u64) -> Self {
        let executable_root = Self::get_executable_root();
        let rust_libstd_dylib = Self::get_rust_libstd_dylib(executable_root.as_str());
        let clang_path = Self::get_clang_path();

        Self {
            clang_path,
            rust_libstd_dylib,
            executable_root,
            plugin_method_ptr,
        }
    }

    pub fn run_dylib(&self, dylib_path: &str) -> Result<String, String> {
        unsafe {
            let lib = libloading::Library::new(dylib_path).unwrap();

            // get kclvm_plugin_init
            let kclvm_plugin_init: libloading::Symbol<
                unsafe extern "C" fn(
                    fn_ptr: extern "C" fn(
                        method: *const i8,
                        args_json: *const i8,
                        kwargs_json: *const i8,
                    ) -> *const i8,
                ),
            > = lib.get(b"kclvm_plugin_init").unwrap();

            // get _kcl_run
            let kcl_run: libloading::Symbol<
                unsafe extern "C" fn(
                    kclvm_main_ptr: u64, // main.k => kclvm_main
                    option_len: kclvm_size_t,
                    option_keys: *const *const kclvm_char_t,
                    option_values: *const *const kclvm_char_t,
                    strict_range_check: i32,
                    disable_none: i32,
                    disable_schema_check: i32,
                    list_option_mode: i32,
                    debug_mode: i32,
                    result_buffer_len: kclvm_size_t,
                    result_buffer: *mut kclvm_char_t,
                    warn_buffer_len: kclvm_size_t,
                    warn_buffer: *mut kclvm_char_t,
                ) -> kclvm_size_t,
            > = lib.get(b"_kcl_run").unwrap();

            // get kclvm_main
            let kclvm_main: libloading::Symbol<u64> = lib.get(b"kclvm_main").unwrap();
            let kclvm_main_ptr = kclvm_main.into_raw().into_raw() as u64;

            // get plugin_method
            let plugin_method_ptr = self.plugin_method_ptr;
            let plugin_method_ptr = (plugin_method_ptr as *const u64) as *const ()
                as *const extern "C" fn(
                    method: *const i8,
                    args: *const i8,
                    kwargs: *const i8,
                ) -> *const i8;
            let plugin_method: extern "C" fn(
                method: *const i8,
                args: *const i8,
                kwargs: *const i8,
            ) -> *const i8 = std::mem::transmute(plugin_method_ptr);

            // register plugin agent
            kclvm_plugin_init(plugin_method);

            let option_len = 0;
            let option_keys = std::ptr::null();
            let option_values = std::ptr::null();
            let strict_range_check = 0;
            let disable_none = 0;
            let disable_schema_check = 0;
            let list_option_mode = 0;
            let debug_mode = 0;

            let mut result = vec![0u8; 1024 * 1024];
            let result_buffer_len = result.len() as i32 - 1;
            let result_buffer = result.as_mut_ptr() as *mut i8;

            let mut warn_buffer = vec![0u8; 1024 * 1024];
            let warn_buffer_len = warn_buffer.len() as i32 - 1;
            let warn_buffer = warn_buffer.as_mut_ptr() as *mut i8;

            let n = kcl_run(
                kclvm_main_ptr,
                option_len,
                option_keys,
                option_values,
                strict_range_check,
                disable_none,
                disable_schema_check,
                list_option_mode,
                debug_mode,
                result_buffer_len,
                result_buffer,
                warn_buffer_len,
                warn_buffer,
            );

            let s = std::str::from_utf8(&result[0..n as usize]).unwrap();
            Ok(s.to_string())
        }
    }

    pub fn run_dylib_with_settings(
        &self,
        dylib_path: &str,
        settings: SettingsFile,
    ) -> Result<String, String> {
        unsafe {
            let lib = libloading::Library::new(dylib_path).unwrap();

            let kcl_run: libloading::Symbol<
                unsafe extern "C" fn(
                    kclvm_main_ptr: u64, // main.k => kclvm_main
                    option_len: kclvm_size_t,
                    option_keys: *const *const kclvm_char_t,
                    option_values: *const *const kclvm_char_t,
                    strict_range_check: i32,
                    disable_none: i32,
                    disable_schema_check: i32,
                    list_option_mode: i32,
                    debug_mode: i32,
                    result_buffer_len: kclvm_size_t,
                    result_buffer: *mut kclvm_char_t,
                    warn_buffer_len: kclvm_size_t,
                    warn_buffer: *mut kclvm_char_t,
                ) -> kclvm_size_t,
            > = lib.get(b"_kcl_run").unwrap();

            let kclvm_main: libloading::Symbol<u64> = lib.get(b"kclvm_main").unwrap();
            let kclvm_main_ptr = kclvm_main.into_raw().into_raw() as u64;

            let option_len = 0;
            let option_keys = std::ptr::null();
            let option_values = std::ptr::null();
            let strict_range_check = 0;
            let disable_none = settings
                .kcl_cli_configs
                .as_ref()
                .map_or(0, |c| c.disable_none.map_or(0, |v| v as i32));
            let disable_schema_check = 0;
            let list_option_mode = 0;
            let debug_mode = settings
                .kcl_cli_configs
                .as_ref()
                .map_or(0, |c| c.debug.map_or(0, |v| v as i32));

            let mut result = vec![0u8; 1024 * 1024];
            let result_buffer_len = result.len() as i32 - 1;
            let result_buffer = result.as_mut_ptr() as *mut i8;

            let mut warn_buffer = vec![0u8; 1024 * 1024];
            let warn_buffer_len = warn_buffer.len() as i32 - 1;
            let warn_buffer = warn_buffer.as_mut_ptr() as *mut i8;

            let n = kcl_run(
                kclvm_main_ptr,
                option_len,
                option_keys,
                option_values,
                strict_range_check,
                disable_none,
                disable_schema_check,
                list_option_mode,
                debug_mode,
                result_buffer_len,
                result_buffer,
                warn_buffer_len,
                warn_buffer,
            );

            let ctx = kclvm::Context::current_context_mut();
            ctx.cfg.debug_mode = debug_mode > 0;
            ctx.cfg.disable_none = disable_none > 0;
            let s = std::str::from_utf8(&result[0..n as usize]).unwrap();
            if s.is_empty() {
                println!()
            } else {
                println!("{}", ValueRef::from_json(s).unwrap().plan_to_yaml_string());
            }
        }

        Ok("".to_string())
    }

    pub fn link_dylibs(&mut self, dylibs: &[String], dylib_path: &str) -> String {
        let mut dylib_path = dylib_path.to_string();

        if dylib_path.is_empty() {
            dylib_path = format!("{}{}", "_a.out", Self::get_lib_suffix());
        }

        let mut args: Vec<String> = vec![
            "-Wno-override-module".to_string(),
            "-Wno-error=unused-command-line-argument".to_string(),
            "-Wno-unused-command-line-argument".to_string(),
            "-shared".to_string(),
            "-undefined".to_string(),
            "dynamic_lookup".to_string(),
            format!("-Wl,-rpath,{}/lib", self.executable_root),
            format!("-L{}/lib", self.executable_root),
            "-lkclvm_native_shared".to_string(),
            format!("-I{}/include", self.executable_root),
        ];
        let mut bc_files = dylibs.to_owned();
        args.append(&mut bc_files);
        let mut more_args = vec![
            self.rust_libstd_dylib.clone(),
            "-fPIC".to_string(),
            "-o".to_string(),
            dylib_path.to_string(),
        ];
        args.append(&mut more_args);

        std::process::Command::new(self.clang_path.clone())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .args(&args)
            .output()
            .expect("clang failed");

        dylib_path
    }

    pub fn run_clang(&mut self, bc_path: &str, dylib_path: &str) -> String {
        let mut bc_path = bc_path.to_string();
        let mut dylib_path = dylib_path.to_string();

        let mut bc_files = vec![];

        for entry in glob::glob(&format!("{}*.ll", bc_path)).unwrap() {
            match entry {
                Ok(path) => {
                    if path.exists() {
                        bc_files.push(path);
                    }
                }
                Err(e) => println!("{:?}", e),
            };
        }
        let mut bc_files = bc_files
            .iter()
            .map(|f| f.to_str().unwrap().to_string())
            .collect::<Vec<String>>();

        if !Self::path_exist(bc_path.as_str()) {
            let s = format!("{}.ll", bc_path);
            if Self::path_exist(s.as_str()) {
                bc_path = s;
            } else {
                let s = format!("{}.ll", bc_path);
                if Self::path_exist(s.as_str()) {
                    bc_path = s;
                }
            }
        }

        if dylib_path.is_empty() {
            dylib_path = format!("{}{}", bc_path, Self::get_lib_suffix());
        }

        let mut args: Vec<String> = vec![
            "-Wno-override-module".to_string(),
            "-Wno-error=unused-command-line-argument".to_string(),
            "-Wno-unused-command-line-argument".to_string(),
            "-shared".to_string(),
            "-undefined".to_string(),
            "dynamic_lookup".to_string(),
            format!("-Wl,-rpath,{}/lib", self.executable_root),
            format!("-L{}/lib", self.executable_root),
            "-lkclvm_native_shared".to_string(),
            format!("-I{}/include", self.executable_root),
        ];
        args.append(&mut bc_files);
        let mut more_args = vec![
            self.rust_libstd_dylib.clone(),
            "-fPIC".to_string(),
            "-o".to_string(),
            dylib_path.to_string(),
        ];
        args.append(&mut more_args);

        std::process::Command::new(self.clang_path.clone())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .args(&args)
            .output()
            .expect("clang failed");

        dylib_path
    }

    pub fn run_clang_single(&mut self, bc_path: &str, dylib_path: &str) -> String {
        let mut bc_path = bc_path.to_string();
        let mut dylib_path = dylib_path.to_string();

        if !Self::path_exist(bc_path.as_str()) {
            let s = format!("{}.ll", bc_path);
            if Self::path_exist(s.as_str()) {
                bc_path = s;
            } else {
                let s = format!("{}.ll", bc_path);
                if Self::path_exist(s.as_str()) {
                    bc_path = s;
                }
            }
        }

        if dylib_path.is_empty() {
            dylib_path = format!("{}{}", bc_path, Self::get_lib_suffix());
        }

        let mut args: Vec<String> = vec![
            "-Wno-override-module".to_string(),
            "-Wno-error=unused-command-line-argument".to_string(),
            "-Wno-unused-command-line-argument".to_string(),
            "-shared".to_string(),
            "-undefined".to_string(),
            "dynamic_lookup".to_string(),
            format!("-Wl,-rpath,{}/lib", self.executable_root),
            format!("-L{}/lib", self.executable_root),
            "-lkclvm_native_shared".to_string(),
            format!("-I{}/include", self.executable_root),
        ];
        let mut bc_files = vec![bc_path];
        args.append(&mut bc_files);
        let mut more_args = vec![
            self.rust_libstd_dylib.clone(),
            "-fPIC".to_string(),
            "-o".to_string(),
            dylib_path.to_string(),
        ];
        args.append(&mut more_args);

        std::process::Command::new(self.clang_path.clone())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .args(&args)
            .output()
            .expect("clang failed");
        // Use absolute path.
        let path = PathBuf::from(&dylib_path).canonicalize().unwrap();
        path.to_str().unwrap().to_string()
    }

    fn get_executable_root() -> String {
        if Self::is_windows() {
            todo!();
        }

        let kclvm_cli_exe = if Self::is_windows() {
            "kclvm_cli.exe"
        } else {
            "kclvm_cli"
        };
        let p = if let Some(x) = Self::find_it(kclvm_cli_exe) {
            x
        } else {
            std::env::current_exe().unwrap()
        };

        let p = p.parent().unwrap().parent().unwrap();
        p.to_str().unwrap().to_string()
    }

    fn get_rust_libstd_dylib(executable_root: &str) -> String {
        let txt_path = std::path::Path::new(&executable_root)
            .join(if Self::is_windows() { "libs" } else { "lib" })
            .join("rust-libstd-name.txt");
        let rust_libstd_name = std::fs::read_to_string(txt_path).unwrap();
        let rust_libstd_name = rust_libstd_name.trim();
        format!("{}/lib/{}", executable_root, rust_libstd_name)
    }

    fn get_clang_path() -> String {
        // ${KCLVM_CLANG}
        let env_kclvm_clang = env::var("KCLVM_CLANG");
        if let Ok(clang_path) = env_kclvm_clang {
            if !clang_path.is_empty() {
                if Self::is_windows() {
                    return format!("{}.exe", clang_path);
                } else {
                    return clang_path;
                }
            }
        }

        // {root}/tools/clang/bin/clang
        let executable_root = Self::get_executable_root();
        let clang_path = std::path::Path::new(&executable_root)
            .join("tools")
            .join("clang")
            .join("bin")
            .join(if Self::is_windows() {
                "clang.exe"
            } else {
                "clang"
            });
        if clang_path.exists() {
            return clang_path.to_str().unwrap().to_string();
        }

        let clang_exe = if Self::is_windows() {
            "clang.exe"
        } else {
            "clang"
        };

        if let Some(s) = Self::find_it(clang_exe) {
            return s.to_str().unwrap().to_string();
        }

        panic!("get_clang_path failed")
    }

    pub fn get_lib_suffix() -> String {
        if Self::is_windows() {
            return ".dll.lib".to_string();
        }
        if Self::is_macos() {
            return ".dylib".to_string();
        }
        if Self::is_linux() {
            return ".so".to_string();
        }
        panic!("unsupport os")
    }

    fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }
    fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }
    fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    fn path_exist(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    fn find_it<P>(exe_name: P) -> Option<std::path::PathBuf>
    where
        P: AsRef<std::path::Path>,
    {
        std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths)
                .filter_map(|dir| {
                    let full_path = dir.join(&exe_name);
                    if full_path.is_file() {
                        Some(full_path)
                    } else {
                        None
                    }
                })
                .next()
        })
    }
}
