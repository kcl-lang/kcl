// Copyright 2021 The KCL Authors. All rights reserved.

use serde::Deserialize;
use std::io::Read;
use toml;

pub const KCL_MOD_FILE: &str = "kcl.mod";
pub const KCL_FILE_SUFFIX: &str = ".k";
pub const KCL_MOD_PATH_ENV: &str = "${KCL_MOD}";

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

pub fn get_pkg_root_from_paths(file_paths: &[String]) -> Result<String, String> {
    if file_paths.is_empty() {
        return Err("No input KCL files or paths".to_string());
    }

    let mut m = std::collections::HashMap::<String, String>::new();
    let mut last_root = "".to_string();
    for s in file_paths {
        if s.contains(KCL_MOD_PATH_ENV) {
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
        return Ok(last_root);
    }

    Err(format!("conflict kcl.mod file paths: {:?}", m))
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
                return Some(module_path.to_str().unwrap().to_string());
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
                return Some(path.to_str().unwrap().to_string());
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
            get_pkg_root_from_paths(&[]),
            Err("No input KCL files or paths".to_string())
        );
        assert_eq!(
            get_pkg_root_from_paths(&["wrong_path".to_string()]),
            Ok("".to_string())
        );
        let expected_root = std::path::Path::new(TEST_ROOT).canonicalize().unwrap();
        let expected = expected_root.to_str().unwrap();
        assert_eq!(
            get_pkg_root_from_paths(&[SETTINGS_FILE.to_string()]),
            Ok(expected.to_string())
        );
    }

    #[test]
    fn test_get_pkg_root() {
        let root = get_pkg_root(SETTINGS_FILE);
        assert!(root.is_some());
        let expected_root = std::path::Path::new(TEST_ROOT).canonicalize().unwrap();
        let expected = expected_root.to_str().unwrap();
        assert_eq!(root.unwrap().as_str(), expected);
    }

    #[test]
    fn test_load_mod_file() {
        let kcl_mod = load_mod_file(TEST_ROOT);
        assert_eq!(
            kcl_mod.build.as_ref().unwrap().enable_pkg_cache.unwrap(),
            true
        );
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
