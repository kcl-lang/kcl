//! Copyright The KCL Authors. All rights reserved.

use anyhow::Result;
use kclvm_utils::path::PathPrefix;
use serde::Deserialize;
use std::{env, fs, io::Read, path::PathBuf};
use toml;

use crate::path::ModRelativePath;

pub const KCL_MOD_FILE: &str = "kcl.mod";
pub const KCL_FILE_SUFFIX: &str = ".k";
pub const KCL_FILE_EXTENSION: &str = "k";
pub const KCL_MOD_PATH_ENV: &str = "${KCL_MOD}";
pub const KCL_PKG_PATH: &str = "KCL_PKG_PATH";
pub const DEFAULT_KCL_HOME: &str = ".kcl";
pub const DEFAULT_KPM_SUBDIR: &str = "kpm";

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
            Ok(_) => return Some(kpm_home.canonicalize().unwrap().display().to_string()),
            Err(_) => None,
        },
    }
}

#[allow(dead_code)]
#[derive(Default, Deserialize)]
pub struct KCLModFile {
    pub root: Option<String>,
    pub root_pkg: Option<String>,
    pub build: Option<KCLModFileBuildSection>,
    pub expected: Option<KCLModFileExpectedSection>,
}

#[allow(dead_code)]
#[derive(Default, Deserialize)]
pub struct KCLModFileBuildSection {
    pub enable_pkg_cache: Option<bool>,
    pub cached_pkg_prefix: Option<String>,
    pub target: Option<String>,
}

#[allow(dead_code)]
#[derive(Default, Deserialize)]
pub struct KCLModFileExpectedSection {
    pub min_build_time: Option<String>,
    pub max_build_time: Option<String>,
    pub kclvm_version: Option<String>,
    pub kcl_plugin_version: Option<String>,
    pub global_version: Option<String>,
}

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

pub fn load_mod_file(root: &str) -> KCLModFile {
    let k_mod_file_path = std::path::Path::new(root).join(KCL_MOD_FILE);
    if !k_mod_file_path.exists() {
        return KCLModFile::default();
    }
    let mut file = std::fs::File::open(k_mod_file_path.to_str().unwrap()).unwrap();
    let mut buffer: Vec<u8> = vec![];
    file.read_to_end(&mut buffer).unwrap();
    toml::from_slice(buffer.as_slice()).unwrap()
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
        let kcl_mod = load_mod_file(TEST_ROOT);
        assert!(kcl_mod.build.as_ref().unwrap().enable_pkg_cache.unwrap());
        assert_eq!(
            kcl_mod
                .build
                .as_ref()
                .unwrap()
                .cached_pkg_prefix
                .as_ref()
                .unwrap(),
            "pkg.path"
        );
        assert_eq!(
            kcl_mod
                .expected
                .as_ref()
                .unwrap()
                .kclvm_version
                .as_ref()
                .unwrap(),
            "v0.3.0"
        );
        assert_eq!(
            kcl_mod
                .expected
                .as_ref()
                .unwrap()
                .kcl_plugin_version
                .as_ref()
                .unwrap(),
            "v0.2.0"
        );
    }
}
