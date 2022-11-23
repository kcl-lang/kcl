use crate::linker::lld_main;
use std::env::consts::DLL_SUFFIX;
use std::ffi::CString;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Command {
    rust_stdlib: String,
    executable_root: String,
}

impl Command {
    pub fn new() -> Self {
        let executable_root = Self::get_executable_root();
        let rust_stdlib = Self::get_rust_stdlib(executable_root.as_str());

        Self {
            rust_stdlib,
            executable_root,
        }
    }

    /// Get lld linker args
    fn lld_args(&self, lib_path: &str) -> Vec<CString> {
        #[cfg(target_os = "macos")]
        let args = vec![
            // Arch
            CString::new("-arch").unwrap(),
            CString::new(std::env::consts::ARCH).unwrap(),
            CString::new("-sdk_version").unwrap(),
            CString::new("10.5.0").unwrap(),
            // Output dynamic libs `.dylib`.
            CString::new("-dylib").unwrap(),
            // Link relative path
            CString::new("-rpath").unwrap(),
            CString::new(format!("{}/lib", self.executable_root)).unwrap(),
            CString::new(format!("-L{}/lib", self.executable_root)).unwrap(),
            // With the change from Catalina to Big Sur (11.0), Apple moved the location of
            // libraries. On Big Sur, it is required to pass the location of the System
            // library. The -lSystem option is still required for macOS 10.15.7 and
            // lower.
            // Ref: https://github.com/ponylang/ponyc/pull/3686
            CString::new("-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib").unwrap(),
            CString::new("-lSystem").unwrap(),
            // Link runtime libs.
            CString::new("-lkclvm").unwrap(),
            // Output lib path.
            CString::new("-o").unwrap(),
            CString::new(lib_path).unwrap(),
            // Link rust std
            CString::new(self.rust_stdlib.as_str()).unwrap(),
        ];

        #[cfg(target_os = "linux")]
        let args = vec![
            // clang -fPIC
            CString::new("-znotext").unwrap(),
            // Output dynamic libs `.so`.
            CString::new("--shared").unwrap(),
            // Link relative path
            CString::new("-R").unwrap(),
            CString::new(format!("{}/lib", self.executable_root)).unwrap(),
            CString::new(format!("-L{}/lib", self.executable_root)).unwrap(),
            // Link runtime libs.
            CString::new("-lkclvm").unwrap(),
            // Output lib path.
            CString::new("-o").unwrap(),
            CString::new(lib_path).unwrap(),
            // Link rust std
            CString::new(self.rust_stdlib.as_str()).unwrap(),
        ];

        #[cfg(target_os = "windows")]
        let args = vec![
            // Output dynamic libs `.dll`.
            CString::new("/dll").unwrap(),
            // Lib search path
            CString::new(format!("/libpath:{}/lib", self.executable_root)).unwrap(),
            // Output lib path.
            CString::new(format!("/out:{}", lib_path)).unwrap(),
            // Link rust std
            CString::new(self.rust_stdlib.as_str()).unwrap(),
        ];

        args
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

        let mut args = self.lld_args(&lib_path);

        for lib in libs {
            args.push(CString::new(lib.as_str()).unwrap())
        }

        // Call lld main function with args.
        assert!(!lld_main(&args), "Run LLD linker failed");

        // Use absolute path.
        let path = PathBuf::from(&lib_path)
            .canonicalize()
            .unwrap_or_else(|_| panic!("{} not found", lib_path));
        path.to_str().unwrap().to_string()
    }

    /// Get the kclvm executable root.
    fn get_executable_root() -> String {
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
