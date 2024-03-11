use anyhow::Result;
pub mod arguments;
pub mod kpm;
pub const DEFAULT_PROJECT_FILE: &str = "project.yaml";

#[cfg(test)]
mod tests;

use glob::glob;
use kclvm_config::{
    modfile::{get_pkg_root, KCL_FILE_EXTENSION, KCL_FILE_SUFFIX, KCL_MOD_PATH_ENV},
    path::ModRelativePath,
    settings::{build_settings_pathbuf, DEFAULT_SETTING_FILE},
};
use kclvm_parser::LoadProgramOptions;
use kclvm_utils::{path::PathPrefix, pkgpath::rm_external_pkg_name};
use kpm::{fetch_metadata, fill_pkg_maps_for_k_file};
use std::env;
use std::iter;
use std::{
    collections::HashSet,
    fs::read_dir,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Expand the file pattern to a list of files.
pub fn expand_if_file_pattern(file_pattern: String) -> Result<Vec<String>, String> {
    let paths = glob(&file_pattern).map_err(|_| format!("invalid file pattern {file_pattern}"))?;
    let mut matched_files = vec![];

    for path in paths.flatten() {
        matched_files.push(path.to_string_lossy().to_string());
    }

    Ok(matched_files)
}

pub fn expand_input_files(k_files: &[String]) -> Vec<String> {
    let mut res = vec![];
    for file in k_files {
        if let Ok(files) = expand_if_file_pattern(file.to_string()) {
            if !files.is_empty() {
                res.extend(files);
            } else {
                res.push(file.to_string())
            }
        } else {
            res.push(file.to_string())
        }
    }
    res
}

/// Normalize input files with the working directory and replace ${KCL_MOD} with the module root path.
pub fn canonicalize_input_files(
    k_files: &[String],
    work_dir: String,
    check_exist: bool,
) -> Result<Vec<String>, String> {
    let mut kcl_paths = Vec::<String>::new();
    // The first traversal changes the relative path to an absolute path
    for file in k_files.iter() {
        let path = Path::new(file);

        let is_absolute = path.is_absolute();
        let is_exist_maybe_symlink = path.exists();
        // If the input file or path is a relative path and it is not a absolute path in the KCL module VFS,
        // join with the work directory path and convert it to a absolute path.
        let path = ModRelativePath::from(file.to_string());
        let abs_path = if !is_absolute && !path.is_relative_path().map_err(|err| err.to_string())? {
            let filepath = Path::new(&work_dir).join(file);
            match filepath.canonicalize() {
                Ok(path) => Some(path.adjust_canonicalization()),
                Err(_) => {
                    if check_exist {
                        return Err(format!(
                            "Cannot find the kcl file, please check the file path {}",
                            file
                        ));
                    }
                    Some(filepath.to_string_lossy().to_string())
                }
            }
        } else {
            None
        };
        // If the input file or path is a symlink, convert it to a real path.
        let real_path = if is_exist_maybe_symlink {
            match PathBuf::from(file.to_string()).canonicalize() {
                Ok(real_path) => Some(String::from(real_path.to_str().unwrap())),
                Err(_) => {
                    if check_exist {
                        return Err(format!(
                            "Cannot find the kcl file, please check the file path {}",
                            file
                        ));
                    }
                    Some(file.to_string())
                }
            }
        } else {
            None
        };

        kcl_paths.push(abs_path.unwrap_or(real_path.unwrap_or(file.to_string())));
    }

    // Get the root path of the project
    let pkgroot = kclvm_config::modfile::get_pkg_root_from_paths(&kcl_paths, work_dir)?;

    // The second traversal replaces ${KCL_MOD} with the project root path
    kcl_paths = kcl_paths
        .iter()
        .map(|file| {
            if file.contains(KCL_MOD_PATH_ENV) {
                file.replace(KCL_MOD_PATH_ENV, pkgroot.as_str())
            } else {
                file.clone()
            }
        })
        .collect();
    Ok(kcl_paths)
}

/// Get compile uint(files and options) from a single file
pub fn lookup_compile_unit(
    file: &str,
    load_pkg: bool,
) -> (Vec<String>, Option<LoadProgramOptions>) {
    let compiled_file: String = file.to_string();
    match lookup_compile_unit_path(file) {
        Ok(dir) => {
            let settings_files = lookup_setting_files(&dir);
            let files = if settings_files.is_empty() {
                vec![file]
            } else {
                vec![]
            };

            let settings_files = settings_files.iter().map(|f| f.to_str().unwrap()).collect();
            match build_settings_pathbuf(&files, Some(settings_files), None) {
                Ok(setting_buf) => {
                    let setting = setting_buf.settings();
                    let files = if let Some(cli_configs) = setting.clone().kcl_cli_configs {
                        let mut k_filename_list = cli_configs.files.unwrap_or_default();
                        if k_filename_list.is_empty() {
                            k_filename_list = cli_configs.file.unwrap_or_default();
                        }
                        k_filename_list
                    } else {
                        vec![]
                    };

                    let work_dir = setting_buf
                        .path()
                        .clone()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let mut load_opt = kclvm_parser::LoadProgramOptions {
                        work_dir: work_dir.clone(),
                        ..Default::default()
                    };
                    match canonicalize_input_files(&files, work_dir, true) {
                        Ok(kcl_paths) => {
                            // 1. find the kcl.mod path
                            let _ = fill_pkg_maps_for_k_file(compiled_file.into(), &mut load_opt);
                            (kcl_paths, Some(load_opt))
                        }
                        Err(_) => (vec![file.to_string()], None),
                    }
                }
                Err(_) => (vec![file.to_string()], None),
            }
        }
        Err(_) => {
            let mut load_opt = kclvm_parser::LoadProgramOptions::default();
            let _ = fill_pkg_maps_for_k_file(compiled_file.into(), &mut load_opt);

            if load_pkg {
                let path = Path::new(file);
                if let Some(ext) = path.extension() {
                    if ext == KCL_FILE_EXTENSION && path.is_file() {
                        if let Some(parent) = path.parent() {
                            if let Ok(files) = get_kcl_files(parent, false) {
                                return (files, Some(load_opt));
                            }
                        }
                    }
                }
            }
            (vec![file.to_string()], Some(load_opt))
        }
    }
}

pub fn lookup_setting_files(dir: &Path) -> Vec<PathBuf> {
    let mut settings = vec![];
    if let Ok(p) = lookup_kcl_yaml(dir) {
        settings.push(p);
    }
    settings
}

pub fn lookup_kcl_yaml(dir: &Path) -> io::Result<PathBuf> {
    let mut path = dir.to_path_buf();
    path.push(DEFAULT_SETTING_FILE);
    if path.is_file() {
        Ok(path)
    } else {
        Err(io::Error::new(
            ErrorKind::NotFound,
            "Ran out of places to find kcl.yaml",
        ))
    }
}

/// For the KCL project, some definitions may be introduced through multi-file
/// compilation (kcl.yaml). This function is used to start from a single file and try
/// to find a `compile unit` that contains all definitions
/// Given a file path, search for the nearest "kcl.yaml" file or the nearest "project.yaml" file.
/// If a "kcl.yaml" file is found, return the path of the directory containing the file.
/// If none of these files are found, return an error indicating that the files were not found.
///
/// Example:
/// +-- project
/// | +-- base
/// | | +-- base.k
/// | +-- prod
/// | | +-- main.k
/// | | +-- kcl.yaml
/// | | +-- stack.yaml
/// | +-- test
/// | | +-- main.k
/// | | +-- kcl.yaml
/// | | +-- stack.yaml
/// | +-- project.yaml
///
/// If the input file is project/prod/main.k or project/test/main.k, it will return
/// Path("project/prod") or Path("project/test")
pub fn lookup_compile_unit_path(file: &str) -> io::Result<PathBuf> {
    let path = PathBuf::from(file);
    let path_ancestors = path.as_path().parent().unwrap().ancestors();
    for p in path_ancestors {
        let entrys = read_dir(p)?;
        for entry in entrys {
            let entry = entry?;
            if entry.file_name() == *DEFAULT_SETTING_FILE {
                // If find "kcl.yaml", the input file is in a stack, return the
                // path of this stack
                return Ok(PathBuf::from(p));
            }
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find kcl.yaml",
    ))
}

/// Get kcl files from path.
pub fn get_kcl_files<P: AsRef<Path>>(path: P, recursively: bool) -> Result<Vec<String>> {
    let mut files = vec![];
    let walkdir = if recursively {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let file = path.to_str().unwrap();
            if file.ends_with(KCL_FILE_SUFFIX) {
                files.push(file.to_string())
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Get the package string list form the package path.
pub fn get_pkg_list(pkgpath: &str) -> Result<Vec<String>> {
    let mut dir_list: Vec<String> = Vec::new();
    let mut dir_map: HashSet<String> = HashSet::new();
    let cwd = std::env::current_dir()?;

    let pkgpath = if pkgpath.is_empty() {
        cwd.to_string_lossy().to_string()
    } else {
        pkgpath.to_string()
    };

    let include_sub_pkg = pkgpath.ends_with("/...");
    let pkgpath = if include_sub_pkg {
        pkgpath.trim_end_matches("/...").to_string()
    } else {
        pkgpath
    };

    if pkgpath != "." && pkgpath.ends_with('.') {
        return Ok(Vec::new());
    }

    if pkgpath.is_empty() {
        return Ok(Vec::new());
    }

    match pkgpath.chars().next() {
        Some('.') => {
            let pkgpath = Path::new(&cwd).join(&pkgpath);
            pkgpath.to_string_lossy().to_string()
        }
        _ => {
            if Path::new(&pkgpath).is_absolute() {
                pkgpath.clone()
            } else if !pkgpath.contains('/') && !pkgpath.contains('\\') {
                pkgpath.replace('.', "/")
            } else {
                let pkgroot =
                    get_pkg_root(cwd.to_str().ok_or(anyhow::anyhow!("cwd path not found"))?)
                        .unwrap_or_default();
                if !pkgroot.is_empty() {
                    PathBuf::from(pkgroot)
                        .join(&pkgpath)
                        .to_string_lossy()
                        .to_string()
                } else {
                    Path::new(&cwd).join(&pkgpath).to_string_lossy().to_string()
                }
            }
        }
    };

    if !include_sub_pkg {
        return Ok(vec![pkgpath]);
    }

    for entry in WalkDir::new(&pkgpath).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir()
            && path.extension().and_then(|ext| ext.to_str()) == Some(KCL_FILE_EXTENSION)
            && !path
                .file_name()
                .map(|name| name.to_string_lossy().starts_with('_'))
                .unwrap_or(false)
        {
            if let Some(dir) = path.parent().map(|p| p.to_string_lossy().to_string()) {
                if !dir_map.contains(&dir) {
                    dir_list.push(dir.clone());
                    dir_map.insert(dir);
                }
            }
        }
    }

    Ok(dir_list)
}

/// [`lookup_the_nearest_file_dir`] will start from [`from`] and search for file [`the_nearest_file`] in the parent directories.
/// If found, it will return the [`Some`] of [`the_nearest_file`] file path. If not found, it will return [`None`]
pub(crate) fn lookup_the_nearest_file_dir(
    from: PathBuf,
    the_nearest_file: &str,
) -> Option<PathBuf> {
    let mut current_dir = from;

    loop {
        let found_path = current_dir.join(the_nearest_file);
        if found_path.is_file() {
            return current_dir.canonicalize().ok();
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => return None,
        }
    }
}

/// [`kcl`] will return the path for executable kcl binary.
pub fn kcl() -> PathBuf {
    get_path_for_executable("kcl")
}

/// [`get_path_for_executable`] will return the path for [`executable_name`].
pub fn get_path_for_executable(executable_name: &'static str) -> PathBuf {
    // The current implementation checks $PATH for an executable to use:
    // `<executable_name>`
    //  example: for <executable_name>, this tries just <executable_name>, which will succeed if <executable_name> is on the $PATH

    if lookup_in_path(executable_name) {
        return executable_name.into();
    }

    executable_name.into()
}

/// [`lookup_in_path`] will search for an executable file [`exec`] in the environment variable ‘PATH’.
///  If found, return true, otherwise return false.
fn lookup_in_path(exec: &str) -> bool {
    let paths = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&paths)
        .map(|path| path.join(exec))
        .find_map(probe)
        .is_some()
}

/// [`probe`] check if the given path points to a file.
/// If it does, return [`Some`] of the path.
/// If not, check if adding the current operating system's executable file extension (if any) to the path points to a file.
/// If it does, return [`Some`] of the path with the extension added.
/// If neither, return [`None`].
fn probe(path: PathBuf) -> Option<PathBuf> {
    let with_extension = match env::consts::EXE_EXTENSION {
        "" => None,
        it => Some(path.with_extension(it)),
    };
    iter::once(path)
        .chain(with_extension)
        .find(|it| it.is_file())
}

/// [`get_real_path_from_external`] will ask for the local path for [`pkg_name`] with subdir [`pkgpath`].
/// If the external package, whose [`pkg_name`] is 'my_package', is stored in '\user\my_package_v0.0.1'.
/// The [`pkgpath`] is 'my_package.examples.apps'.
///
/// [`get_real_path_from_external`] will return '\user\my_package_v0.0.1\examples\apps'
///
/// # Note
/// [`get_real_path_from_external`] is just a method for calculating a path, it doesn't check whether a path exists.
pub fn get_real_path_from_external(
    pkg_name: &str,
    pkgpath: &str,
    current_pkg_path: PathBuf,
) -> PathBuf {
    let mut real_path = PathBuf::new();
    let pkg_root = fetch_metadata(current_pkg_path)
        .map(|metadata| {
            metadata
                .packages
                .get(pkg_name)
                .map_or(PathBuf::new(), |pkg| pkg.manifest_path.clone())
        })
        .unwrap_or_else(|_| PathBuf::new());
    real_path = real_path.join(pkg_root);

    let pkgpath = match rm_external_pkg_name(pkgpath) {
        Ok(path) => path,
        Err(_) => String::new(),
    };
    pkgpath.split('.').for_each(|s| real_path.push(s));
    real_path
}
