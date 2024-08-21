//! Copyright The KCL Authors. All rights reserved.

use anyhow::Result;
use kclvm_utils::path::PathPrefix;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fs,
    io::Read,
    path::{Path, PathBuf},
};
use toml;

use crate::path::ModRelativePath;

pub const KCL_MOD_FILE: &str = "kcl.mod";
pub const KCL_MOD_LOCK_FILE: &str = "kcl.mod.lock";
pub const KCL_WORK_FILE: &str = "kcl.work";
pub const KCL_FILE_SUFFIX: &str = ".k";
pub const KCL_FILE_EXTENSION: &str = "k";
pub const KCL_MOD_PATH_ENV: &str = "${KCL_MOD}";
pub const KCL_PKG_PATH: &str = "KCL_PKG_PATH";
pub const DEFAULT_KCL_HOME: &str = ".kcl";
pub const DEFAULT_KPM_SUBDIR: &str = "kpm";

/// ModFile is kcl package file 'kcl.mod'.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ModFile {
    pub package: Option<Package>,
    pub profile: Option<Profile>,
    pub dependencies: Option<Dependencies>,
}

/// ModLockFile is kcl package file 'kc.mod.lock'.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ModLockFile {
    pub dependencies: Option<LockDependencies>,
}

/// Package is the kcl package section of 'kcl.mod'.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Package {
    /// The name of the package.
    pub name: Option<String>,
    /// The kcl compiler version
    pub edition: Option<String>,
    /// The version of the package.
    pub version: Option<String>,
    /// Description denotes the description of the package.
    pub description: Option<String>,
    /// Exclude denote the files to include when publishing.
    pub include: Option<Vec<String>>,
    /// Exclude denote the files to exclude when publishing.
    pub exclude: Option<Vec<String>>,
}

/// Profile is the profile section of 'kcl.mod'.
/// It is used to specify the compilation options of the current package.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Profile {
    /// A list of entry-point files.
    pub entries: Option<Vec<String>>,
    /// Flag that, when true, disables the emission of the special 'none' value in the output.
    pub disable_none: Option<bool>,
    /// Flag that, when true, ensures keys in maps are sorted.
    pub sort_keys: Option<bool>,
    /// A list of attribute selectors for conditional compilation.
    pub selectors: Option<Vec<String>>,
    /// A list of override paths.
    pub overrides: Option<Vec<String>>,
    /// A list of additional options for the KCL compiler.
    pub options: Option<Vec<String>>,
}

/// A map of package names to their respective dependency specifications.
pub type Dependencies = HashMap<String, Dependency>;
pub type LockDependencies = HashMap<String, LockDependency>;

/// Dependency represents a single dependency for a package, which may come in different forms
/// such as version, Git repository, OCI repository, or a local path.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Dependency {
    /// Specifies a version dependency, e.g., "1.0.0".
    Version(String),
    /// Specifies a Git source dependency.
    Git(GitSource),
    /// Specifies an OCI (Open Container Initiative) image source dependency.
    Oci(OciSource),
    /// Specifies a local path dependency.
    Local(LocalSource),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LockDependency {
    /* Common field */
    pub name: String,
    pub full_name: Option<String>,
    pub version: Option<String>,
    pub sum: Option<String>,

    /* OCI Source */
    pub reg: Option<String>,
    pub repo: Option<String>,
    pub oci_tag: Option<String>,

    /* Git Source */
    pub url: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub git_tag: Option<String>,

    /* Local Source */
    pub path: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GitSource {
    /// The URL of the Git repository.
    pub git: String,
    /// An optional branch name within the Git repository.
    pub branch: Option<String>,
    /// An optional commit hash to check out from the Git repository.
    pub commit: Option<String>,
    /// An optional tag name to check out from the Git repository.
    pub tag: Option<String>,
    /// An optional version specification associated with Git source.
    pub version: Option<String>,
}

/// Defines an OCI package as a source for a dependency.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct OciSource {
    // The URI of the OCI repository.
    pub oci: String,
    /// An optional tag of the OCI package in the registry.
    pub tag: Option<String>,
}

/// Defines a local filesystem path as a source for a dependency.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LocalSource {
    /// The path to the local directory or file.
    pub path: String,
}

impl ModFile {
    #[inline]
    pub fn get_entries(&self) -> Option<Vec<String>> {
        self.profile.as_ref().map(|p| p.entries.clone()).flatten()
    }
}

/// Load kcl mod file from path
pub fn load_mod_file<P: AsRef<Path>>(path: P) -> Result<ModFile> {
    let file_path = path.as_ref().join(KCL_MOD_FILE);
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer: Vec<u8> = vec![];
    file.read_to_end(&mut buffer)?;
    toml::from_slice(buffer.as_slice()).map_err(|e| anyhow::anyhow!(e))
}

/// Load kcl mod lock file from path
pub fn load_mod_lock_file<P: AsRef<Path>>(path: P) -> Result<ModLockFile> {
    let file_path = path.as_ref().join(KCL_MOD_LOCK_FILE);
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer: Vec<u8> = vec![];
    file.read_to_end(&mut buffer)?;
    toml::from_slice(buffer.as_slice()).map_err(|e| anyhow::anyhow!(e))
}

/// Get the path holding the external kcl package.
/// From the environment variable KCL_PKG_PATH.
/// If `KCL_PKG_PATH` is not present, then the user root string is returned.
/// If the user root directory cannot be found, an empty string will be returned.
pub fn get_vendor_home() -> String {
    match env::var(KCL_PKG_PATH) {
        Ok(path) => path,
        Err(_) => create_default_vendor_home().unwrap_or_default(),
    }
}

/// Create a '.kcl/kpm' folder in the user's root directory,
/// returning the folder path in [Option::Some] if it already exists.
///
/// If the folder does not exist, create it and return the file path
/// in [Option::Some].
///
/// If creating the folder failed, [`Option::None`] is returned.
pub fn create_default_vendor_home() -> Option<String> {
    #[cfg(target_os = "windows")]
    let root_dir = match env::var("USERPROFILE") {
        Ok(val) => val,
        Err(_) => return None,
    };
    #[cfg(not(target_os = "windows"))]
    let root_dir = match env::var("HOME") {
        Ok(val) => val,
        Err(_) => return None,
    };
    let kpm_home = PathBuf::from(root_dir)
        .join(DEFAULT_KCL_HOME)
        .join(DEFAULT_KPM_SUBDIR);
    match kpm_home.canonicalize() {
        Ok(path) => return Some(path.display().to_string()),
        Err(_) => match fs::create_dir_all(kpm_home.clone()) {
            Ok(_) => match kpm_home.canonicalize() {
                Ok(p) => Some(p.display().to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        },
    }
}

/// Get package root path from input file paths and workdir.
pub fn get_pkg_root_from_paths(file_paths: &[String], workdir: String) -> Result<String, String> {
    if file_paths.is_empty() {
        return Err("No input KCL files or paths".to_string());
    }

    let mut m = std::collections::HashMap::<String, String>::new();
    let mut last_root = "".to_string();
    for s in file_paths {
        let path = ModRelativePath::from(s.to_string());
        if path.is_relative_path().map_err(|err| err.to_string())? {
            continue;
        }

        if let Some(root) = get_pkg_root(s) {
            m.insert(root.clone(), root.clone());
            last_root = root.clone();
        }
    }
    if m.is_empty() {
        return Ok("".to_string());
    }
    if m.len() == 1 {
        Ok(last_root)
    } else if !workdir.is_empty() {
        return Ok(workdir);
    } else {
        return Ok("".to_string());
    }
}

/// Get package root path from the single input file path.
pub fn get_pkg_root(k_file_path: &str) -> Option<String> {
    if k_file_path.is_empty() {
        return None;
    }
    // # search by kcl.mod file
    if let Ok(module_path) = std::path::Path::new(k_file_path).canonicalize() {
        let mut module_path = module_path;
        while module_path.exists() {
            let kcl_mod_path = module_path.join(KCL_MOD_FILE);
            if kcl_mod_path.exists() && kcl_mod_path.is_file() {
                return Some(module_path.adjust_canonicalization());
            }
            if let Some(path) = module_path.parent() {
                module_path = path.to_path_buf();
            } else {
                break;
            }
        }
    }
    if k_file_path.ends_with(KCL_FILE_SUFFIX) {
        if let Ok(path) = std::path::Path::new(k_file_path).canonicalize() {
            if let Some(path) = path.parent() {
                return Some(path.adjust_canonicalization());
            }
        }
    }
    None
}

#[cfg(test)]
mod modfile_test {
    use crate::modfile::*;

    const TEST_ROOT: &str = "./src/testdata/";
    const SETTINGS_FILE: &str = "./src/testdata/kcl.mod";

    #[test]
    fn test_get_pkg_root_from_paths() {
        assert_eq!(
            get_pkg_root_from_paths(&[], "".to_string()),
            Err("No input KCL files or paths".to_string())
        );
        assert_eq!(
            get_pkg_root_from_paths(&["wrong_path".to_string()], "".to_string()),
            Ok("".to_string())
        );
        let expected_root = std::path::Path::new(TEST_ROOT).canonicalize().unwrap();
        let expected = expected_root.adjust_canonicalization();
        assert_eq!(
            get_pkg_root_from_paths(&[SETTINGS_FILE.to_string()], "".to_string()),
            Ok(expected.to_string())
        );
    }

    #[test]
    fn test_get_pkg_root() {
        let root = get_pkg_root(SETTINGS_FILE);
        assert!(root.is_some());
        let expected_root = std::path::Path::new(TEST_ROOT).canonicalize().unwrap();
        let expected = expected_root.adjust_canonicalization();
        assert_eq!(root.unwrap().as_str(), expected);
    }

    #[test]
    fn test_load_mod_file() {
        let kcl_mod = load_mod_file(TEST_ROOT).unwrap();
        assert_eq!(
            kcl_mod.package.as_ref().unwrap().name.as_ref().unwrap(),
            "test_add_deps"
        );
        assert_eq!(
            kcl_mod.package.as_ref().unwrap().version.as_ref().unwrap(),
            "0.0.1"
        );
        assert_eq!(
            kcl_mod.package.as_ref().unwrap().edition.as_ref().unwrap(),
            "0.0.1"
        );
        assert_eq!(
            kcl_mod.profile.as_ref().unwrap().entries.as_ref().unwrap(),
            &vec!["main.k".to_string()]
        );
        assert_eq!(
            kcl_mod.dependencies.as_ref().unwrap().get("pkg0"),
            Some(&Dependency::Git(GitSource {
                git: "test_url".to_string(),
                tag: Some("test_tag".to_string()),
                ..Default::default()
            }))
        );
        assert_eq!(
            kcl_mod.dependencies.as_ref().unwrap().get("pkg1"),
            Some(&Dependency::Version("oci_tag1".to_string()))
        );
        assert_eq!(
            kcl_mod.dependencies.as_ref().unwrap().get("pkg2"),
            Some(&Dependency::Oci(OciSource {
                oci: "oci://ghcr.io/kcl-lang/helloworld".to_string(),
                tag: Some("0.1.1".to_string()),
                ..Default::default()
            }))
        );
        assert_eq!(
            kcl_mod.dependencies.as_ref().unwrap().get("pkg3"),
            Some(&Dependency::Local(LocalSource {
                path: "../pkg".to_string(),
            }))
        );
    }
}
