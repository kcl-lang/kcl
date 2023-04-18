use kclvm_utils::path::PathPrefix;
use std::env::consts::DLL_SUFFIX;
use std::path::PathBuf;

const KCLVM_CLI_BIN_PATH_ENV_VAR: &str = "KCLVM_CLI_BIN_PATH";
const KCLVM_LIB_LINK_PATH_ENV_VAR: &str = "KCLVM_LIB_LINK_PATH";
const KCLVM_LIB_SHORT_NAME: &str = "kclvm_cli_cdylib";

#[derive(Debug)]
pub struct Command {
    executable_root: String,
}

impl Command {
    pub fn new() -> Self {
        let executable_root = Self::get_executable_root();

        Self { executable_root }
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
        path.adjust_canonicalization()
    }

    /// Add args for cc.
    pub(crate) fn add_args(
        &self,
        libs: &[String],
        lib_path: String,
        cmd: &mut std::process::Command,
    ) {
        #[cfg(not(target_os = "windows"))]
        self.unix_args(libs, lib_path, cmd);

        #[cfg(target_os = "windows")]
        self.msvc_win_args(libs, lib_path, cmd);
    }

    /// Add args for cc on unix os.
    pub(crate) fn unix_args(
        &self,
        libs: &[String],
        lib_path: String,
        cmd: &mut std::process::Command,
    ) {
        let path = self.runtime_lib_path(&lib_path);
        cmd.args(libs)
            .arg(&format!("-Wl,-rpath,{}", &path))
            .arg(&format!("-L{}", &path))
            .arg(&format!("-I{}/include", self.executable_root))
            .arg("-lkclvm_cli_cdylib");
    }

    /// Get the runtime library path.
    pub(crate) fn runtime_lib_path(&self, lib_path: &str) -> String {
        let path = self.get_lib_link_path();
        let lib_name = Self::get_lib_name();
        let lib_file_path = std::path::Path::new(&path).join(&lib_name);
        // Copy runtime library to target path for parallel execute.
        if let Some(target_path) = std::path::Path::new(&lib_path).parent() {
            let target_lib_file_path = std::path::Path::new(target_path).join(&lib_name);

            // Locking file for parallel file copy.
            let mut file_lock =
                fslock::LockFile::open(&format!("{}.lock", target_lib_file_path.display()))
                    .unwrap();
            file_lock.lock().unwrap();

            std::fs::copy(lib_file_path, target_lib_file_path).unwrap();
            target_path.to_string_lossy().to_string()
        } else {
            path
        }
    }

    // Add args for cc on windows os.
    #[cfg(target_os = "windows")]
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
            .arg(format!(r#"/LIBPATH:"{}""#, self.get_lib_link_path()))
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
        if let Ok(path) = std::env::var(KCLVM_CLI_BIN_PATH_ENV_VAR) {
            return path;
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
        }
        .canonicalize()
        .expect("canonicalize kclvm executable path failed");

        let p = p.parent().unwrap().parent().unwrap();
        p.to_str().unwrap().to_string()
    }

    /// Get KCLVM lib link path
    pub(crate) fn get_lib_link_path(&self) -> String {
        let mut default_path = None;
        for folder in ["lib", "bin"] {
            let path = std::path::Path::new(&self.executable_root)
                .join(folder)
                .join(&Self::get_lib_name());
            if path.exists() {
                default_path = Some(path.parent().unwrap().to_string_lossy().to_string());
                break;
            }
        }
        std::env::var(KCLVM_LIB_LINK_PATH_ENV_VAR)
            .ok()
            .or(default_path)
            .unwrap_or(self.executable_root.clone())
    }

    /// Get KCLVM lib name
    pub(crate) fn get_lib_name() -> String {
        let suffix = Self::get_lib_suffix();
        if Self::is_windows() {
            format!("{KCLVM_LIB_SHORT_NAME}{suffix}")
        } else {
            format!("lib{KCLVM_LIB_SHORT_NAME}{suffix}")
        }
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
