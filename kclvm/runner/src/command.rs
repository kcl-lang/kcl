use std::env::consts::DLL_SUFFIX;
use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Command {
    clang_path: String,
    rust_stdlib: String,
    executable_root: String,
}

impl Command {
    pub fn new() -> Self {
        let executable_root = Self::get_executable_root();
        let rust_stdlib = Self::get_rust_stdlib(executable_root.as_str());
        let clang_path = Self::get_clang_path();

        Self {
            clang_path,
            rust_stdlib,
            executable_root,
        }
    }

    /// Link dynamic libraries into one library.
    pub(crate) fn link_libs(&mut self, libs: &[String], lib_path: &str) -> String {
        let lib_suffix = Self::get_lib_suffix();
        let lib_path = if lib_path.is_empty() {
            format!("{}{}", "_a.out", lib_suffix)
        } else if !lib_path.ends_with(&lib_suffix) {
            format!("{}{}", lib_path, lib_suffix)
        } else {
            lib_path.to_string()
        };

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
        let mut bc_files = libs.to_owned();
        args.append(&mut bc_files);
        let mut more_args = vec![
            self.rust_stdlib.clone(),
            "-fPIC".to_string(),
            "-o".to_string(),
            lib_path.to_string(),
        ];
        args.append(&mut more_args);

        let result = std::process::Command::new(self.clang_path.clone())
            .args(&args)
            .output()
            .expect("run clang failed");

        if !result.status.success() {
            panic!(
                "run clang failed: stdout {}, stderr: {}",
                String::from_utf8(result.stdout).unwrap(),
                String::from_utf8(result.stderr).unwrap()
            )
        }

        // Use absolute path.
        let path = PathBuf::from(&lib_path)
            .canonicalize()
            .unwrap_or_else(|_| panic!("{} not found", lib_path));
        path.to_str().unwrap().to_string()
    }

    /// Get the kclvm executable root.
    fn get_executable_root() -> String {
        if Self::is_windows() {
            todo!();
        }

        let kclvm_exe = if Self::is_windows() {
            "kclvm.exe"
        } else {
            "kclvm"
        };
        let p = if let Some(x) = Self::find_it(kclvm_exe) {
            x
        } else {
            std::env::current_exe().unwrap()
        };

        let p = p.parent().unwrap().parent().unwrap();
        p.to_str().unwrap().to_string()
    }

    fn get_rust_stdlib(executable_root: &str) -> String {
        let txt_path = std::path::Path::new(&executable_root)
            .join(if Self::is_windows() { "libs" } else { "lib" })
            .join("rust-libstd-name.txt");
        let rust_libstd_name = std::fs::read_to_string(txt_path).expect("rust libstd not found");
        let rust_libstd_name = rust_libstd_name.trim();
        format!("{}/lib/{}", executable_root, rust_libstd_name)
    }

    fn get_clang_path() -> String {
        // ${KCLVM_CLANG}
        let env_kclvm_clang = env::var("KCLVM_CLANG");
        if let Ok(clang_path) = env_kclvm_clang {
            if !clang_path.is_empty() {
                let clang_path = if Self::is_windows() {
                    format!("{}.exe", clang_path)
                } else {
                    clang_path
                };
                if std::path::Path::new(&clang_path).exists() {
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

    /// Specifies the filename suffix used for shared libraries on this
    /// platform. Example value is `.so`.
    ///
    /// Some possible values:
    ///
    /// - .so
    /// - .dylib
    /// - .dll
    pub(crate) fn get_lib_suffix() -> String {
        DLL_SUFFIX.to_string()
    }

    fn is_windows() -> bool {
        cfg!(target_os = "windows")
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
