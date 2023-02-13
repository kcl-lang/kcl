#![allow(dead_code)]
use std::env::consts::DLL_SUFFIX;
use std::ffi::CString;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Command {
    executable_root: String,
}

impl Command {
    pub fn new() -> Self {
        let executable_root = Self::get_executable_root();

        Self { executable_root }
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
            CString::new(format!("{}/bin", self.executable_root)).unwrap(),
            CString::new(format!("-L{}/bin", self.executable_root)).unwrap(),
            // With the change from Catalina to Big Sur (11.0), Apple moved the location of
            // libraries. On Big Sur, it is required to pass the location of the System
            // library. The -lSystem option is still required for macOS 10.15.7 and
            // lower.
            // Ref: https://github.com/ponylang/ponyc/pull/3686
            CString::new("-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib").unwrap(),
            CString::new("-lSystem").unwrap(),
            // Link runtime libs.
            CString::new("-lkclvm_cli_cdylib").unwrap(),
            // Output lib path.
            CString::new("-o").unwrap(),
            CString::new(lib_path).unwrap(),
        ];

        #[cfg(target_os = "linux")]
        let args = vec![
            // clang -fPIC
            CString::new("-znotext").unwrap(),
            // Output dynamic libs `.so`.
            CString::new("--shared").unwrap(),
            // Link relative path
            CString::new("-R").unwrap(),
            CString::new(format!("{}/bin", self.executable_root)).unwrap(),
            CString::new(format!("-L{}/bin", self.executable_root)).unwrap(),
            // Link runtime libs.
            CString::new("-lkclvm_cli_cdylib").unwrap(),
            // Output lib path.
            CString::new("-o").unwrap(),
            CString::new(lib_path).unwrap(),
        ];

        #[cfg(target_os = "windows")]
        let args = vec![
            // Output dynamic libs `.dll`.
            CString::new("/dll").unwrap(),
            // Lib search path
            CString::new(format!("/libpath:{}/lib", self.executable_root)).unwrap(),
            // Output lib path.
            CString::new(format!("/out:{}", lib_path)).unwrap(),
        ];

        args
    }

    /// Link dynamic libraries into one library using cc-rs lib.
    pub(crate) fn link_libs_with_cc(&mut self, libs: &[String], lib_path: &str) -> String {
        let lib_suffix = Self::get_lib_suffix();
        let lib_path = if lib_path.is_empty() {
            format!("{}{}", "_a.out", lib_suffix)
        } else if !lib_path.ends_with(&lib_suffix) {
            format!("{}{}", lib_path, lib_suffix)
        } else {
            lib_path.to_string()
        };

        #[cfg(not(target_os = "windows"))]
        let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);

        #[cfg(target_os = "windows")]
        let target = format!("{}-{}", std::env::consts::ARCH, Self::cc_env_windows());

        let mut build = cc::Build::new();

        build
            .cargo_metadata(false)
            .no_default_flags(false)
            .pic(true)
            .shared_flag(true)
            .opt_level(0)
            .target(&target)
            .host(&target)
            .flag("-o")
            .flag(&lib_path);

        let libs: Vec<String> = libs
            .into_iter()
            .map(|lib| adjust_canonicalization(lib))
            .collect();
        let libs = libs.as_slice();
        build.files(libs);

        // Run command with cc.
        let mut cmd = build.try_get_compiler().unwrap().to_command();
        self.add_args(libs, lib_path.to_string(), &mut cmd);
        let result = cmd.output().expect("run cc command failed");
        if !result.status.success() {
            panic!(
                "run cc failed: stdout {}, stderr: {}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            )
        }
        // Use absolute path.
        let path = PathBuf::from(&lib_path)
            .canonicalize()
            .unwrap_or_else(|_| panic!("{} not found", lib_path));
        path.to_str().unwrap().to_string()
    }

    /// Add args for cc.
    pub(crate) fn add_args(
        &self,
        libs: &[String],
        _lib_path: String,
        cmd: &mut std::process::Command,
    ) {
        #[cfg(not(target_os = "windows"))]
        self.unix_args(libs, cmd);

        #[cfg(target_os = "windows")]
        self.msvc_win_args(libs, _lib_path, cmd);
    }

    // Add args for cc on unix os.
    pub(crate) fn unix_args(&self, libs: &[String], cmd: &mut std::process::Command) {
        cmd.args(libs)
            .arg(&format!("-Wl,-rpath,{}/bin", self.executable_root))
            .arg(&format!("-L{}/bin", self.executable_root))
            .arg(&format!("-I{}/include", self.executable_root))
            .arg("-lkclvm_cli_cdylib");
    }

    // Add args for cc on windows os.
    pub(crate) fn msvc_win_args(
        &self,
        libs: &[String],
        lib_path: String,
        cmd: &mut std::process::Command,
    ) {
        cmd.args(libs)
            .arg("kclvm_cli_cdylib.lib")
            .arg("/link")
            .arg("/NOENTRY")
            .arg("/NOLOGO")
            .arg(format!(r#"/LIBPATH:"{}\bin""#, self.executable_root))
            .arg("/DEFAULTLIB:msvcrt.lib")
            .arg("/DEFAULTLIB:libcmt.lib")
            .arg("/DLL")
            .arg(format!("/OUT:{}", lib_path))
            .arg("/EXPORT:_kcl_run")
            .arg("/EXPORT:kclvm_main")
            .arg("/EXPORT:kclvm_plugin_init");
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

    #[cfg(target_os = "windows")]
    fn cc_env_windows() -> String {
        "msvc".to_string()
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

#[cfg(not(target_os = "windows"))]
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(target_os = "windows")]
/// On windows, the "\\? \ " will cause the obj file to not be found when linking by "cl.exe".
/// Slicing this path directly is not a good solution,
/// we will find a more fluent way to solve this problem in the future. @zongz
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}
