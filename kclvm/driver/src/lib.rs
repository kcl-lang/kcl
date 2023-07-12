use anyhow::Result;
pub mod arguments;
pub mod kpm_metadata;
pub const DEFAULT_PROJECT_FILE: &str = "project.yaml";

#[cfg(test)]
mod tests;

use kclvm_ast::ast;
use kclvm_config::{
    modfile::{KCL_FILE_EXTENSION, KCL_FILE_SUFFIX, KCL_MOD_PATH_ENV},
    settings::{build_settings_pathbuf, DEFAULT_SETTING_FILE},
};
use kclvm_parser::LoadProgramOptions;
use kclvm_utils::path::PathPrefix;
use kpm_metadata::fill_pkg_maps_for_k_file;
use std::{
    fs::{self, read_dir},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Normalize input files with the working directory and replace ${KCL_MOD} with the module root path.
pub fn canonicalize_input_files(
    k_files: &[String],
    work_dir: String,
    check_exist: bool,
) -> Result<Vec<String>, String> {
    let mut kcl_paths = Vec::<String>::new();

    // The first traversal changes the relative path to an absolute path
    for (_, file) in k_files.iter().enumerate() {
        let path = Path::new(file);
        let is_absolute = path.is_absolute();
        let is_exist_maybe_symlink = path.exists();
        // If the input file or path is a relative path and it is not a absolute path in the KCL module VFS,
        // join with the work directory path and convert it to a absolute path.

        let abs_path = if !is_absolute && !file.starts_with(KCL_MOD_PATH_ENV) {
            let filepath = Path::new(&work_dir).join(file);
            match filepath.canonicalize() {
                Ok(path) => Some(path.adjust_canonicalization()),
                Err(_) => {
                    if check_exist {
                        return Err(format!(
                            "Cannot find the kcl file, please check whether the file path {}",
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
            match PathBuf::from(file).canonicalize() {
                Ok(real_path) => Some(String::from(real_path.to_str().unwrap())),
                Err(_) => {
                    if check_exist {
                        return Err(format!(
                            "Cannot find the kcl file, please check whether the file path {}",
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
    let pkgroot = kclvm_config::modfile::get_pkg_root_from_paths(&kcl_paths)?;

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
                        cmd_args: if let Some(options) = setting.clone().kcl_options {
                            options
                                .iter()
                                .map(|o| ast::CmdArgSpec {
                                    name: o.key.to_string(),
                                    value: o.value.to_string(),
                                })
                                .collect()
                        } else {
                            vec![]
                        },
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
            return (vec![file.to_string()], Some(load_opt));
        }
    }
}

pub fn lookup_setting_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut settings = vec![];
    if let Ok(p) = lookup_kcl_yaml(dir) {
        settings.push(p);
    }
    settings
}

pub fn lookup_kcl_yaml(dir: &PathBuf) -> io::Result<PathBuf> {
    let mut path = dir.clone();
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
/// If a "project.yaml" file is found, return the path of the first directory containing a "kcl.yaml" file in that project.
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
/// If the input file is project/base/base.k, it will return Path("project/prod")
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
            } else if entry.file_name() == DEFAULT_PROJECT_FILE {
                // If find "project.yaml", the input file may be in the `base`
                // directory of a project, return the path of the first stack
                // of this project
                let project_path = PathBuf::from(p);
                for e in read_dir(project_path)? {
                    if let Ok(entry) = e {
                        let path = entry.path();
                        if path.is_dir() && lookup_kcl_yaml(&path).is_ok() {
                            return Ok(path);
                        }
                    }
                }
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
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let file = path.to_str().unwrap();
            if file.ends_with(KCL_FILE_SUFFIX) && (recursively || entry.depth() == 1) {
                files.push(file.to_string())
            }
        }
    }
    files.sort();
    Ok(files)
}
