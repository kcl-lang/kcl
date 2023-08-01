// Copyright 2021 The KCL Authors. All rights reserved.

use anyhow::Result;
use kclvm_utils::path::PathPrefix;
use serde::Deserialize;
use std::{
    collections::{HashMap, VecDeque},
    env, fs,
    io::Read,
    path::PathBuf,
};
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
        Err(_) => create_default_vendor_home().unwrap_or(String::default()),
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

/// [`Entries`] is a map of package name to package root path for one compilation
/// # note
///
/// The [`entries`] in [`Entries`] is ordered, and the order of Entrys may affect the result of the compilation.
/// The reason why the [`Entries`] is not an [`IndexMap`] is that the [`entries`] is duplicable and ordered.
#[derive(Default, Debug)]
pub struct Entries {
    entries: VecDeque<Entry>,
}

impl Entries {
    /// [`push`] will push a new [`Entry`] into [`Entries`] with the given name and path.
    pub fn push(&mut self, name: String, path: String) {
        self.entries.push_back(Entry::new(name, path));
    }

    /// [`contains_pkg_name`] will return [`Option::Some`] if there is an [`Entry`] with the given name in [`Entries`].
    pub fn contains_pkg_name(&self, name: &String) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.name() == name)
    }

    /// [`iter`] will return an iterator of [`Entry`] in [`Entries`].
    pub fn iter(&self) -> std::collections::vec_deque::Iter<Entry> {
        self.entries.iter()
    }

    /// [`len`] will return the length of [`Entries`].
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// [`get_nth_entry`] will return the nth [`Entry`] in [`Entries`].
    pub fn get_nth_entry(&self, n: usize) -> Option<&Entry> {
        if n >= self.len() {
            return None;
        }
        let mut count = 0;
        for entry in self.entries.iter() {
            if count == n {
                return Some(entry);
            }
            count += 1;
        }
        return None;
    }

    /// [`get_nth_entry_by_name`] will return the nth [`Entry`] by name in [`Entries`].
    pub fn get_nth_entry_by_name(&self, n: usize, name: &str) -> Option<&Entry> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| name == entry.name())
            .nth(n)
            .map(|(_, entry)| entry)
    }

    /// [`apply_to_all_entries`] will apply the given function to all [`Entry`] in [`Entries`].
    pub fn apply_to_all_entries<F>(&mut self, f: F) -> Result<()>
    where
        F: FnMut(&mut Entry) -> Result<()>,
    {
        self.entries.iter_mut().try_for_each(f)?;
        return Ok(());
    }
}

/// [`Entry`] is a package name and package root path pair.
#[derive(Default, Debug)]
pub struct Entry {
    name: String,
    path: String,
}

impl Entry {
    /// [`new`] will create a new [`Entry`] with the given name and path.
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }

    /// [`name`] will return the name of [`Entry`].
    pub fn name(&self) -> &String {
        &self.name
    }

    /// [`path`] will return the path of [`Entry`].
    pub fn path(&self) -> &String {
        &self.path
    }

    /// [`set_name`] will set the name of [`Entry`] to the given name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// [`set_path`] will set the path of [`Entry`] to the given path.
    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }
}

/// [`get_compile_entries_from_paths`] returns all the [`Entries`] for compilation from the given [`file_paths`].
///
/// # Note
/// If the path in [`file_paths`] is a normal path or a [`ModRelativePath`] with prefix `${KCL_MOD}`, the package will be named as `__main__`.
/// If the path in [`file_paths`] is a [`ModRelativePath`], the package will be named by the suffix of [`ModRelativePath`].
///
/// # Error
/// The package root path for package name `__main__` is only used once. If there are multiple
/// package root paths for `__main__`, an error `conflict kcl.mod file` is returned.
///
/// # Example
///
/// ```rust
/// use std::path::PathBuf;
/// use kclvm_config::modfile::get_compile_entries_from_paths;
/// let testpath = PathBuf::from("./src/testdata/multimods").canonicalize().unwrap();
///
/// // [`kcl1_path`] is a normal path of the package [`kcl1`] root directory.
/// // It looks like `/xxx/xxx/xxx`.
/// let kcl1_path = testpath.join("kcl1");
///
/// // [`kcl2_path`] is a mod relative path of the packege [`kcl2`] root directory.
/// // It looks like `${kcl2:KCL_MOD}/xxx/xxx`
/// let kcl2_path = PathBuf::from("${kcl2:KCL_MOD}/main.k");
///
/// // [`kcl3_path`] is a mod relative path of the [`__main__`] packege.
/// let kcl3_path = PathBuf::from("${KCL_MOD}/main.k");
///
/// // [`external_pkgs`] is a map to show the real path of the mod relative path [`kcl2`].
/// let mut external_pkgs = std::collections::HashMap::<String, String>::new();
/// external_pkgs.insert("kcl2".to_string(), testpath.join("kcl2").to_str().unwrap().to_string());
///
/// // [`get_compile_entries_from_paths`] will return the map of package name to package root real path.
/// let entries = get_compile_entries_from_paths(
///     &[
///         kcl1_path.to_str().unwrap().to_string(),
///         kcl2_path.display().to_string(),
///         kcl3_path.display().to_string(),
///     ],
/// external_pkgs).unwrap();
///
/// assert_eq!(entries.len(), 3);
///
/// assert_eq!(entries.get_nth_entry(0).unwrap().name(), "__main__");
/// assert_eq!(
///     PathBuf::from(entries.get_nth_entry(0).unwrap().path())
///         .canonicalize()
///         .unwrap()
///         .display()
///         .to_string(),
///     kcl1_path.canonicalize().unwrap().to_str().unwrap()
/// );
///
/// assert_eq!(entries.get_nth_entry(1).unwrap().name(), "kcl2");
/// assert_eq!(
///     PathBuf::from(entries.get_nth_entry(1).unwrap().path())
///         .canonicalize()
///         .unwrap()
///         .display()
///         .to_string(),
///     testpath
///         .join("kcl2")
///         .canonicalize()
///         .unwrap()
///         .to_str()
///         .unwrap()
/// );
///
/// assert_eq!(entries.get_nth_entry(2).unwrap().name(), "__main__");
/// assert_eq!(
///     PathBuf::from(entries.get_nth_entry(2).unwrap().path())
///         .display()
///         .to_string(),
///     kcl1_path.join("main.k").canonicalize().unwrap().to_str().unwrap()
/// );
///
/// ```
pub fn get_compile_entries_from_paths(
    file_paths: &[String],
    external_pkgs: HashMap<String, String>,
) -> Result<Entries, String> {
    if file_paths.is_empty() {
        return Err("No input KCL files or paths".to_string());
    }
    let mut result = Entries::default();
    let mut m = std::collections::HashMap::<String, String>::new();
    for s in file_paths {
        let path = ModRelativePath::from(s.to_string());
        // If the path is a [`ModRelativePath`],
        // calculate the real path and the package name.
        if let Some((pkg_name, pkg_path)) = path
            .get_root_pkg_name()
            .map_err(|err| err.to_string())?
            .and_then(|name| {
                external_pkgs
                    .get(&name)
                    .map(|pkg_path: &String| (name, pkg_path))
            })
        {
            let s = path
                .canonicalize_by_root_path(pkg_path)
                .map_err(|err| err.to_string())?;
            if let Some(root) = get_pkg_root(&s) {
                result.push(pkg_name, root);
                continue;
            } else {
                return Err(format!(
                    "can not find the package name of path {} in {:?}",
                    s, external_pkgs
                ));
            }
        } else if path.is_relative_path().map_err(|err| err.to_string())?
            && path
                .get_root_pkg_name()
                .map_err(|err| err.to_string())?
                .is_none()
        {
            result.push(kclvm_ast::MAIN_PKG.to_string(), path.get_path());
            continue;
        } else if let Some(root) = get_pkg_root(s) {
            // If the path is a normal path.
            m.insert(root.clone(), root.clone());
            result.push(kclvm_ast::MAIN_PKG.to_string(), root);
        } else {
            return Err(format!(
                "can not find the package name of path {} in {:?}",
                s, external_pkgs
            ));
        }
    }

    if m.is_empty() {
        result.push(kclvm_ast::MAIN_PKG.to_string(), "".to_string());
    }
    if m.len() == 1 {
        let pkg_root;
        let main_entry = result
            .get_nth_entry_by_name(0, kclvm_ast::MAIN_PKG)
            .ok_or_else(|| format!("program entry not found in {:?}", file_paths))?;
        pkg_root = main_entry.path().to_string();

        result
            .apply_to_all_entries(|entry| {
                if entry.name() == kclvm_ast::MAIN_PKG {
                    entry.set_path(
                        ModRelativePath::from(entry.path().to_string())
                            .canonicalize_by_root_path(&pkg_root)?,
                    );
                }
                return Ok(());
            })
            .map_err(|err| err.to_string())?;
        return Ok(result);
    }

    Err(format!("conflict kcl.mod file paths: {:?}", m))
}

pub fn get_pkg_root_from_paths(file_paths: &[String]) -> Result<String, String> {
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
            get_pkg_root_from_paths(&[]),
            Err("No input KCL files or paths".to_string())
        );
        assert_eq!(
            get_pkg_root_from_paths(&["wrong_path".to_string()]),
            Ok("".to_string())
        );
        let expected_root = std::path::Path::new(TEST_ROOT).canonicalize().unwrap();
        let expected = expected_root.adjust_canonicalization();
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
