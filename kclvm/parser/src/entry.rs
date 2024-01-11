use anyhow::Result;
use kclvm_config::modfile::get_pkg_root;
use kclvm_config::modfile::KCL_FILE_SUFFIX;
use kclvm_config::path::ModRelativePath;
use kclvm_utils::path::PathPrefix;
use kclvm_utils::path::{is_absolute, is_dir, path_exist};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use crate::LoadProgramOptions;

/// [`Entries`] is a map of package name to package root path for one compilation
/// # note
///
/// The [`entries`] in [`Entries`] is ordered, and the order of Entrys may affect the result of the compilation.
/// The reason why the [`Entries`] is not an [`IndexMap`] is that the [`entries`] is duplicable and ordered.
#[derive(Default, Debug)]
pub struct Entries {
    root_path: String,
    entries: VecDeque<Entry>,
}

impl Entries {
    /// [`get_unique_normal_paths_by_name`] will return all the unique normal paths of [`Entry`] with the given name in [`Entries`].
    pub fn get_unique_normal_paths_by_name(&self, name: &str) -> Vec<String> {
        let paths = self
            .get_unique_paths_by_name(name)
            .iter()
            .filter(|path| {
                // All the paths contains the normal paths and the mod relative paths start with ${KCL_MOD}.
                // If the number of 'kcl.mod' paths is 0, except for the mod relative paths start with ${KCL_MOD},
                // then use empty path "" as the default.
                !ModRelativePath::new(path.to_string())
                    .is_relative_path()
                    .unwrap_or(false)
            })
            .map(|entry| entry.to_string())
            .collect::<Vec<String>>();
        paths
    }

    /// [`get_unique_paths_by_name`] will return all the unique paths of [`Entry`] with the given name in [`Entries`].
    pub fn get_unique_paths_by_name(&self, name: &str) -> Vec<String> {
        let mut paths = self
            .entries
            .iter()
            .filter(|entry| entry.name() == name)
            .map(|entry| entry.path().to_string())
            .collect::<Vec<String>>();
        paths.sort();
        paths.dedup();
        paths
    }

    /// [`push_entry`] will push a new [`Entry`] into [`Entries`].
    pub fn push_entry(&mut self, entry: Entry) {
        self.entries.push_back(entry);
    }

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
    #[allow(dead_code)]
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
        None
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
        Ok(())
    }

    /// [`get_root_path`] will return the root path of [`Entries`].
    pub fn get_root_path(&self) -> &String {
        &self.root_path
    }
}

/// [`Entry`] is a package name and package root path pair.
#[derive(Default, Debug)]
pub struct Entry {
    name: String,
    path: String,
    k_files: Vec<String>,
    k_code_lists: Vec<Option<String>>,
}

impl Entry {
    /// [`new`] will create a new [`Entry`] with the given name and path.
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            k_files: vec![],
            k_code_lists: vec![],
        }
    }

    /// [`name`] will return the name of [`Entry`].
    pub fn name(&self) -> &String {
        &self.name
    }

    /// [`path`] will return the path of [`Entry`].
    pub fn path(&self) -> &String {
        &self.path
    }

    /// [`set_path`] will set the path of [`Entry`] to the given path.
    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    /// [`extend_k_files`] will extend the k files of [`Entry`] to the given k file.
    pub fn extend_k_files(&mut self, k_files: Vec<String>) {
        self.k_files.extend(k_files);
    }

    /// [`extend_k_files_and_codes`] will extend the k files and k codes of [`Entry`] to the given k file and k code.
    pub fn extend_k_files_and_codes(
        &mut self,
        k_files: Vec<String>,
        k_codes: &mut VecDeque<String>,
    ) {
        for k_file in k_files.iter() {
            self.k_code_lists.push(k_codes.pop_front());
            self.k_files.push(k_file.to_string());
        }
    }

    /// [`push_k_code`] will push the k code of [`Entry`] to the given k code.
    pub fn push_k_code(&mut self, k_code: Option<String>) {
        self.k_code_lists.push(k_code);
    }

    /// [`get_k_files`] will return the k files of [`Entry`].
    pub fn get_k_files(&self) -> &Vec<String> {
        &self.k_files
    }

    /// [`get_k_codes`] will return the k codes of [`Entry`].
    pub fn get_k_codes(&self) -> &Vec<Option<String>> {
        &self.k_code_lists
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
/// use kclvm_parser::entry::get_compile_entries_from_paths;
/// use kclvm_parser::LoadProgramOptions;
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
/// // It looks like `${KCL_MOD}/xxx/xxx`
/// let kcl3_path = PathBuf::from("${KCL_MOD}/main.k");
///
/// // [`package_maps`] is a map to show the real path of the mod relative path [`kcl2`].
/// let mut opts = LoadProgramOptions::default();
/// opts.package_maps.insert("kcl2".to_string(), testpath.join("kcl2").to_str().unwrap().to_string());
///
/// // [`get_compile_entries_from_paths`] will return the map of package name to package root real path.
/// let entries = get_compile_entries_from_paths(
///     &[
///         kcl1_path.to_str().unwrap().to_string(),
///         kcl2_path.display().to_string(),
///         kcl3_path.display().to_string(),
///     ],
///     &opts,
/// ).unwrap();
///
/// // [`entries`] will contain 3 entries.
/// // <__main__, "/usr/xxx/src/testdata/multimods/kcl1">
/// // <kcl2, "/usr/xxx/src/testdata/multimods/kcl2">
/// // <__main__, "/usr/xxx/src/testdata/multimods/kcl1">
/// assert_eq!(entries.len(), 3);
///
/// // <__main__, "/usr/xxx/src/testdata/multimods/kcl1">
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
/// // <kcl2, "/usr/xxx/src/testdata/multimods/kcl2">
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
/// // <__main__, "/usr/xxx/src/testdata/multimods/kcl1">
/// assert_eq!(entries.get_nth_entry(2).unwrap().name(), "__main__");
/// assert_eq!(
///     PathBuf::from(entries.get_nth_entry(2).unwrap().path())
///         .canonicalize()
///         .unwrap()
///         .to_str()
///         .unwrap(),
///     kcl1_path.canonicalize().unwrap().to_str().unwrap()
/// );
/// ```
pub fn get_compile_entries_from_paths(
    file_paths: &[String],
    opts: &LoadProgramOptions,
) -> Result<Entries> {
    if file_paths.is_empty() {
        return Err(anyhow::anyhow!("No input KCL files or paths"));
    }
    let mut result = Entries::default();
    let mut k_code_queue = VecDeque::from(opts.k_code_list.clone());
    for s in file_paths {
        let path = ModRelativePath::from(s.to_string());

        // If the path is a [`ModRelativePath`] with prefix '${<package_name>:KCL_MOD}',
        // calculate the real path and the package name.
        if let Some((pkg_name, pkg_path)) = path.get_root_pkg_name()?.and_then(|name| {
            opts.package_maps
                .get(&name)
                .map(|pkg_path: &String| (name, pkg_path))
        }) {
            // Replace the mod relative path prefix '${<pkg_name>:KCL_MOD}' with the real path.
            let s = path.canonicalize_by_root_path(pkg_path)?;
            if let Some(root) = get_pkg_root(&s) {
                let mut entry: Entry = Entry::new(pkg_name.clone(), root.clone());
                entry.extend_k_files_and_codes(
                    get_main_files_from_pkg_path(&s, &root, &pkg_name, opts)?,
                    &mut k_code_queue,
                );
                result.push_entry(entry);
                continue;
            }
            // If the [`ModRelativePath`] with prefix '${KCL_MOD}'
        } else if path.is_relative_path()? && path.get_root_pkg_name()?.is_none() {
            // Push it into `result`, and deal it later.
            let mut entry = Entry::new(kclvm_ast::MAIN_PKG.to_string(), path.get_path());
            entry.push_k_code(k_code_queue.pop_front());
            result.push_entry(entry);
            continue;
        } else if let Some(root) = get_pkg_root(s) {
            // If the path is a normal path.
            let mut entry: Entry = Entry::new(kclvm_ast::MAIN_PKG.to_string(), root.clone());
            entry.extend_k_files_and_codes(
                get_main_files_from_pkg_path(s, &root, kclvm_ast::MAIN_PKG, opts)?,
                &mut k_code_queue,
            );
            result.push_entry(entry);
        }
    }

    // The main 'kcl.mod' can not be found, the empty path "" will be took by default.
    if result
        .get_unique_normal_paths_by_name(kclvm_ast::MAIN_PKG)
        .is_empty()
    {
        let mut entry = Entry::new(kclvm_ast::MAIN_PKG.to_string(), "".to_string());
        for s in file_paths {
            entry.extend_k_files_and_codes(
                get_main_files_from_pkg_path(s, "", kclvm_ast::MAIN_PKG, opts)?,
                &mut k_code_queue,
            );
        }
        result.push_entry(entry);
    }

    let pkg_root = if result
        .get_unique_normal_paths_by_name(kclvm_ast::MAIN_PKG)
        .len()
        == 1
        && opts.work_dir.is_empty()
    {
        // If the 'kcl.mod' can be found only once, the package root path will be the path of the 'kcl.mod'.
        result
            .get_unique_normal_paths_by_name(kclvm_ast::MAIN_PKG)
            .get(0)
            .unwrap()
            .to_string()
    } else if !opts.work_dir.is_empty() {
        // If the 'kcl.mod' can be found more than once, the package root path will be the 'work_dir'.
        if let Some(root_work_dir) = get_pkg_root(&opts.work_dir) {
            root_work_dir
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    result.root_path = pkg_root.clone();
    // Replace the '${KCL_MOD}' of all the paths with package name '__main__'.
    result.apply_to_all_entries(|entry| {
        let path = ModRelativePath::from(entry.path().to_string());
        if entry.name() == kclvm_ast::MAIN_PKG && path.is_relative_path()? {
            entry.set_path(pkg_root.to_string());
            entry.extend_k_files(get_main_files_from_pkg_path(
                &path.canonicalize_by_root_path(&pkg_root)?,
                &pkg_root,
                kclvm_ast::MAIN_PKG,
                opts,
            )?);
        }
        Ok(())
    })?;

    Ok(result)
}

/// Get files in the main package with the package root.
fn get_main_files_from_pkg_path(
    pkg_path: &str,
    root: &str,
    pkg_name: &str,
    opts: &LoadProgramOptions,
) -> Result<Vec<String>> {
    // fix path
    let mut path_list = Vec::new();
    let mut s = pkg_path.to_string();

    let path = ModRelativePath::from(s.to_string());

    if path.is_relative_path()? {
        if let Some(name) = path.get_root_pkg_name()? {
            if name == pkg_name {
                s = path.canonicalize_by_root_path(root)?;
            }
        } else if path.is_relative_path()? {
            return Err(anyhow::anyhow!("Can not find {} in the path: {}", s, root));
        }
    }
    if !root.is_empty() && !is_absolute(s.as_str()) {
        let p = std::path::Path::new(s.as_str());
        if let Ok(x) = std::fs::canonicalize(p) {
            s = x.adjust_canonicalization();
        }
    }

    path_list.push(s);

    // get k files
    let mut k_files: Vec<String> = Vec::new();

    for (i, path) in path_list.iter().enumerate() {
        // read dir/*.k
        if is_dir(path) {
            if opts.k_code_list.len() > i {
                return Err(anyhow::anyhow!("Invalid code list"));
            }
            // k_code_list
            for s in get_dir_files(path, false)? {
                k_files.push(s);
            }
            continue;
        } else {
            k_files.push(path.to_string());
        }
    }

    if k_files.is_empty() {
        return Err(anyhow::anyhow!("No input KCL files"));
    }

    // check all file exists
    for (i, filename) in k_files.iter().enumerate() {
        if i < opts.k_code_list.len() {
            continue;
        }

        if !path_exist(filename.as_str()) {
            return Err(anyhow::anyhow!(
                "Cannot find the kcl file, please check the file path {}",
                filename.as_str(),
            ));
        }
    }
    Ok(k_files)
}

/// Get file list in the directory.
pub fn get_dir_files(dir: &str, is_recursive: bool) -> Result<Vec<String>> {
    if !std::path::Path::new(dir).exists() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(dir.to_string());
    // BFS all the files in the directory.
    while let Some(path) = queue.pop_front() {
        let path = Path::new(&path);
        if path.is_dir() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_dir() && is_recursive {
                                queue.push_back(path.to_string_lossy().to_string());
                            } else if !is_ignored_file(&path.display().to_string()) {
                                list.push(path.display().to_string());
                            }
                        }
                    }
                }
                Err(err) => {
                    return Err(anyhow::anyhow!(
                        "Failed to read directory: {},{}",
                        path.display(),
                        err
                    ));
                }
            }
        } else if !is_ignored_file(&path.display().to_string()) {
            list.push(path.display().to_string());
        }
    }

    list.sort();
    Ok(list)
}

/// Check if the file is ignored.
/// The file is ignored if
///     it is not a kcl file (end with '*.k')
///     or it is a test file (end with '_test.k')
///     or it is a hidden file. (start with '_')
fn is_ignored_file(filename: &str) -> bool {
    (!filename.ends_with(KCL_FILE_SUFFIX))
        || (filename.ends_with("_test.k"))
        || (filename.starts_with('_'))
}
