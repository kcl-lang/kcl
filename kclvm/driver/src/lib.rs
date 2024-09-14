pub mod arguments;
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
pub mod toolchain;

#[cfg(test)]
mod tests;

use anyhow::Result;
use kclvm_config::{
    modfile::{
        get_pkg_root, load_mod_file, KCL_FILE_EXTENSION, KCL_FILE_SUFFIX, KCL_MOD_FILE,
        KCL_WORK_FILE,
    },
    settings::{build_settings_pathbuf, DEFAULT_SETTING_FILE},
    workfile::load_work_file,
};
use kclvm_parser::LoadProgramOptions;
use kclvm_utils::path::PathPrefix;
use std::iter;
use std::{collections::HashMap, env};
use std::{
    collections::HashSet,
    fs::read_dir,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};
use toolchain::{fill_pkg_maps_for_k_file, Metadata, Toolchain};
use walkdir::WalkDir;

/// Get compile workspace(files and options) from a single file input.
/// 1. Lookup entry files in kcl.yaml
/// 2. Lookup entry files in kcl.mod
/// 3. If not found, consider the path or folder where the file is
///    located as the compilation entry point
pub fn lookup_compile_workspace(
    tool: &dyn Toolchain,
    file: &str,
    load_pkg: bool,
) -> CompileUnitOptions {
    let mut default_res: CompileUnitOptions = (vec![], None, None);
    let mut load_opt = kclvm_parser::LoadProgramOptions::default();
    let metadata = fill_pkg_maps_for_k_file(tool, file.into(), &mut load_opt).unwrap_or(None);
    let path = Path::new(file);
    if let Some(ext) = path.extension() {
        if load_pkg {
            if let Some(parent) = path.parent() {
                if let Ok(files) = get_kcl_files(parent, false) {
                    default_res = (files, Some(load_opt), metadata);
                }
            }
        } else {
            if ext == KCL_FILE_EXTENSION && path.is_file() {
                default_res = (vec![file.to_string()], Some(load_opt), metadata);
            }
        }
    }
    match lookup_compile_unit_path(file) {
        Ok(CompileUnitPath::SettingFile(dir)) => {
            let settings_files = lookup_setting_files(&dir);
            let files = if settings_files.is_empty() {
                default_res.0.iter().map(|s| s.as_str()).collect()
            } else {
                vec![]
            };
            let settings_files: Vec<&str> =
                settings_files.iter().map(|f| f.to_str().unwrap()).collect();
            match build_settings_pathbuf(&files, Some(settings_files), None) {
                Ok(setting_buf) => {
                    let setting = setting_buf.settings();
                    let files = setting.input();

                    let work_dir = setting_buf
                        .path()
                        .clone()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let mut load_opt = kclvm_parser::LoadProgramOptions {
                        work_dir: work_dir.clone(),
                        ..Default::default()
                    };
                    let metadata =
                        fill_pkg_maps_for_k_file(tool, file.into(), &mut load_opt).unwrap_or(None);
                    if files.is_empty() {
                        default_res
                    } else {
                        (files, Some(load_opt), metadata)
                    }
                }
                Err(_) => default_res,
            }
        }
        Ok(CompileUnitPath::ModFile(dir)) => match load_mod_file(&dir) {
            Ok(mod_file) => {
                let mut load_opt = kclvm_parser::LoadProgramOptions::default();
                let metadata =
                    fill_pkg_maps_for_k_file(tool, file.into(), &mut load_opt).unwrap_or(None);
                if let Some(files) = mod_file.get_entries() {
                    let work_dir = dir.to_string_lossy().to_string();
                    load_opt.work_dir = work_dir.clone();
                    (files, Some(load_opt), metadata)
                } else {
                    default_res
                }
            }
            Err(_) => default_res,
        },
        Ok(CompileUnitPath::NotFound) | Err(_) => default_res,
    }
}

pub fn lookup_compile_workspaces(
    tool: &dyn Toolchain,
    path: &str,
    load_pkg: bool,
) -> (
    HashMap<WorkSpaceKind, CompileUnitOptions>,
    Option<HashMap<String, String>>,
) {
    let mut workspaces = HashMap::new();
    match lookup_workspace(path) {
        Ok(workspace) => match &workspace {
            WorkSpaceKind::WorkFile(work_file_path) => {
                if let Ok(mut workfile) = load_work_file(work_file_path) {
                    let root = work_file_path.parent().unwrap();
                    workfile.canonicalize(root.to_path_buf());
                    for work in workfile.workspaces {
                        match lookup_workspace(&work.abs_path) {
                            Ok(workspace) => {
                                workspaces.insert(
                                    workspace.clone(),
                                    lookup_compile_workspace(tool, &work.abs_path, load_pkg),
                                );
                            }
                            Err(_) => {}
                        }
                    }
                    return (workspaces, Some(workfile.failed.clone()));
                }
            }
            WorkSpaceKind::Folder(folder) => {
                let mut load_opt = kclvm_parser::LoadProgramOptions::default();
                let metadata =
                    fill_pkg_maps_for_k_file(tool, path.into(), &mut load_opt).unwrap_or(None);

                if load_pkg {
                    if folder.is_dir() {
                        if let Ok(files) = get_kcl_files(folder.clone(), false) {
                            // return (files, Some(load_opt), metadata);
                            workspaces.insert(workspace, (files, Some(load_opt), metadata));
                            return (workspaces, None);
                        }
                    }
                }
                workspaces.insert(
                    workspace,
                    (vec![path.to_string()], Some(load_opt), metadata),
                );
            }
            WorkSpaceKind::SettingFile(setting_file) => {
                workspaces.insert(
                    workspace.clone(),
                    lookup_compile_workspace(
                        tool,
                        &setting_file.as_path().adjust_canonicalization(),
                        load_pkg,
                    ),
                );
            }
            WorkSpaceKind::ModFile(mod_file) => {
                workspaces.insert(
                    workspace.clone(),
                    lookup_compile_workspace(
                        tool,
                        &mod_file.as_path().adjust_canonicalization(),
                        load_pkg,
                    ),
                );
            }
            WorkSpaceKind::File(_) | WorkSpaceKind::NotFound => {
                let pathbuf = PathBuf::from(path);
                let file_path = pathbuf.as_path();
                if file_path.is_file() {
                    workspaces.insert(workspace, lookup_compile_workspace(tool, path, load_pkg));
                }
            }
        },
        Err(_) => {}
    }

    (workspaces, None)
}

/// Lookup default setting files e.g. kcl.yaml
pub fn lookup_setting_files(dir: &Path) -> Vec<PathBuf> {
    let mut settings = vec![];
    if let Ok(p) = lookup_kcl_yaml(dir) {
        settings.push(p);
    }
    settings
}

fn lookup_kcl_yaml(dir: &Path) -> io::Result<PathBuf> {
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

pub type CompileUnitOptions = (Vec<String>, Option<LoadProgramOptions>, Option<Metadata>);

/// CompileUnitPath is the kcl program default entries that are defined
/// in the config files.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompileUnitPath {
    SettingFile(PathBuf),
    ModFile(PathBuf),
    NotFound,
}

/// LSP workspace, will replace CompileUnitPath
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum WorkSpaceKind {
    WorkFile(PathBuf),
    ModFile(PathBuf),
    SettingFile(PathBuf),
    Folder(PathBuf),
    File(PathBuf),
    NotFound,
}

/// For the KCL project, some definitions may be introduced through multi-file
/// compilation (kcl.yaml). This function is used to start from a single file and try
/// to find a `compile unit` that contains all definitions
/// Given a file path, search for the nearest "kcl.yaml" file or the nearest "kcl.mod" file.
/// If a "kcl.yaml" file is found, return the path of the directory containing the file.
/// If a "kcl.mod" file is found, return the path of the directory containing the file.
/// If none of these files are found, return an error indicating that the files were not found.
///
/// Example:
/// +-- project
/// | +-- base
/// | | +-- base.k
/// | +-- prod
/// | | +-- main.k
/// | | +-- kcl.yaml
/// | +-- test
/// | | +-- main.k
/// | | +-- kcl.yaml
/// | +-- kcl.mod
///
/// If the input file is project/prod/main.k or project/test/main.k, it will return
/// Path("project/prod") or Path("project/test")
pub fn lookup_compile_unit_path(file: &str) -> io::Result<CompileUnitPath> {
    let path = PathBuf::from(file);
    let current_dir_path = path.as_path().parent().unwrap();
    let entries = read_dir(current_dir_path)?;
    for entry in entries {
        let entry = entry?;
        // The entry priority of `kcl.yaml`` is higher than that of `kcl.mod`.
        if entry.file_name() == *DEFAULT_SETTING_FILE {
            // If find "kcl.yaml", the input file is in a compile stack, return the
            // path of this compile stack
            return Ok(CompileUnitPath::SettingFile(PathBuf::from(
                current_dir_path,
            )));
        } else if entry.file_name() == *KCL_MOD_FILE {
            return Ok(CompileUnitPath::ModFile(PathBuf::from(current_dir_path)));
        }
    }
    Ok(CompileUnitPath::NotFound)
}

/// It will replace lookup_compile_unit_path()
pub fn lookup_workspace(path: &str) -> io::Result<WorkSpaceKind> {
    let pathbuf = PathBuf::from(path);
    let path = pathbuf.as_path();
    if path.is_dir() {
        for entry in read_dir(path)? {
            let entry = entry?;
            if entry.file_name() == *KCL_WORK_FILE {
                return Ok(WorkSpaceKind::WorkFile(entry.path()));
            }
        }

        for entry in read_dir(path)? {
            let entry = entry?;
            if entry.file_name() == *KCL_MOD_FILE {
                return Ok(WorkSpaceKind::ModFile(entry.path()));
            }
        }

        for entry in read_dir(path)? {
            let entry = entry?;
            if entry.file_name() == *DEFAULT_SETTING_FILE {
                return Ok(WorkSpaceKind::SettingFile(entry.path()));
            }
        }

        return Ok(WorkSpaceKind::Folder(PathBuf::from(path)));
    }
    if path.is_file() {
        if let Some(ext) = path.extension() {
            if ext.to_str().unwrap() == KCL_FILE_EXTENSION {
                return Ok(WorkSpaceKind::File(PathBuf::from(path)));
            }
        }
    }
    Ok(WorkSpaceKind::NotFound)
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
