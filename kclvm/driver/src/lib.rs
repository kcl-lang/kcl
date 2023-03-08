use kclvm_ast::ast;
use kclvm_config::{modfile::KCL_MOD_PATH_ENV, settings::{build_settings_pathbuf, DEFAULT_SETTING_FILE}};
use kclvm_parser::LoadProgramOptions;
use kclvm_runtime::PanicInfo;
use kclvm_utils::path::PathPrefix;
use std::{path::{Path, PathBuf}, fs::read_dir, io::{ErrorKind, self}, ffi::OsString};

/// Normalize input files with the working directory and replace ${KCL_MOD} with the module root path.
pub fn canonicalize_input_files(
    k_files: &Vec<String>,
    work_dir: String,
) -> Result<Vec<String>, String> {
    let mut kcl_paths = Vec::<String>::new();
    for (_, file) in k_files.iter().enumerate() {
        // If the input file or path is a relative path and it is not a absolute path in the KCL module VFS,
        // join with the work directory path and convert it to a absolute path.
        if !file.starts_with(KCL_MOD_PATH_ENV) && !Path::new(file).is_absolute() {
            match Path::new(&work_dir).join(file).canonicalize() {
                Ok(path) => kcl_paths.push(String::from(path.adjust_canonicalization())),
                Err(_) => {
                    return Err(PanicInfo::from_string(&format!(
                        "Cannot find the kcl file, please check whether the file path {}",
                        file
                    ))
                    .to_json_string())
                }
            }
        } else {
            kcl_paths.push(String::from(file))
        }
    }
    return Ok(kcl_paths);
}


/// Get compile uint(files and options) from a single file
pub fn lookup_compile_unit(file: &str) -> (Vec<String>, Option<LoadProgramOptions>) {
    match lookup_compile_unit_path(file) {
        Ok(dir) => {
            let settings_files = lookup_setting_files(&dir);
            let files = if settings_files.is_empty() {
                vec![file]
            } else {
                vec![]
            };

            let settings_files = settings_files.iter().map(|f| f.to_str().unwrap()).collect();
            match build_settings_pathbuf(&files, None, Some(settings_files), false, false) {
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

                    let load_opt = kclvm_parser::LoadProgramOptions {
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
                    match canonicalize_input_files(&files, work_dir) {
                        Ok(kcl_paths) => return (kcl_paths, Some(load_opt)),
                        Err(_) => return (vec![file.to_string()], None),
                    }
                }
                Err(_) => return (vec![file.to_string()], None),
            }
        }
        Err(_) => return (vec![file.to_string()], None),
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
        return Ok(path);
    } else {
        Err(io::Error::new(
            ErrorKind::NotFound,
            "Ran out of places to find kcl.yaml",
        ))
    }
}

pub fn lookup_compile_unit_path(file: &str) -> io::Result<PathBuf> {
    let path = PathBuf::from(file);
    let mut path_ancestors = path.as_path().parent().unwrap().ancestors();
    while let Some(p) = path_ancestors.next() {
        let has_kcl_yaml = read_dir(p)?
            .into_iter()
            .any(|p| p.unwrap().file_name() == OsString::from(DEFAULT_SETTING_FILE));
        if has_kcl_yaml {
            return Ok(PathBuf::from(p));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find kcl.yaml",
    ))
}
