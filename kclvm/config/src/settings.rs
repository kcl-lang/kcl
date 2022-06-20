// Copyright 2021 The KCL Authors. All rights reserved.
use serde::{Deserialize, Serialize};

const INVALID_KCL_OPTIONS_MSG: &str = "invalid kcl_options";
const SETTINGS_FILE_PARA: &str = "-Y";
const ARGUMENTS_PARA: &str = "-D";
const OUTPUT_PARA: &str = "-o";
const OVERRIDES_PARA: &str = "-O";
const PATH_SELECTOR_PARA: &str = "-S";
const STRICT_RANGE_CHECK_PARA: &str = "-r";
const DISABLE_NONE_PARA: &str = "-n";
const VERBOSE_PARA: &str = "-v";
const DEBUG_PARA: &str = "-d";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsFile {
    pub kcl_cli_configs: Option<Config>,
    pub kcl_options: Option<Vec<KeyValuePair>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub files: Option<Vec<String>>,
    pub file: Option<Vec<String>>,
    pub output: Option<String>,
    pub overrides: Option<Vec<String>>,
    pub path_selector: Option<Vec<String>>,
    pub strict_range_check: Option<bool>,
    pub disable_none: Option<bool>,
    pub verbose: Option<u32>,
    pub debug: Option<bool>,
}

impl SettingsFile {
    pub fn new() -> Self {
        SettingsFile {
            kcl_cli_configs: Some(Config {
                file: Some(vec![]),
                files: Some(vec![]),
                output: None,
                overrides: Some(vec![]),
                path_selector: Some(vec![]),
                strict_range_check: Some(false),
                disable_none: Some(false),
                verbose: Some(0),
                debug: Some(false),
            }),
            kcl_options: Some(vec![]),
        }
    }
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestSettingsFile {
    kcl_options: Option<String>,
}

pub fn load_file(filename: &str) -> SettingsFile {
    let f = std::fs::File::open(filename).unwrap();
    let data: SettingsFile = serde_yaml::from_reader(f).unwrap();
    data
}

macro_rules! set_if {
    ($result: expr, $attr: ident, $setting: expr) => {
        if $setting.$attr.is_some() {
            $result.$attr = $setting.$attr.clone();
        }
    };
}

pub fn merge_settings(settings: &[SettingsFile]) -> SettingsFile {
    let mut result = SettingsFile::new();
    for setting in settings {
        if let Some(kcl_cli_configs) = &setting.kcl_cli_configs {
            let mut result_kcl_cli_configs = result.kcl_cli_configs.as_mut().unwrap();
            set_if!(result_kcl_cli_configs, files, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, file, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, output, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, overrides, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, path_selector, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, strict_range_check, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, disable_none, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, verbose, kcl_cli_configs);
            set_if!(result_kcl_cli_configs, debug, kcl_cli_configs);
            // debug: Option<bool>,
        }
        if let Some(kcl_options) = &setting.kcl_options {
            let result_kcl_options = result.kcl_options.as_mut().unwrap();
            for option in kcl_options {
                result_kcl_options.push(option.clone());
            }
        }
    }
    result
}

pub fn decode_test_format_settings_file(filename: &str, workdir: &str) -> SettingsFile {
    let f = std::fs::File::open(filename).unwrap();
    let data: TestSettingsFile = serde_yaml::from_reader(f).unwrap();
    let mut settings_file: SettingsFile = SettingsFile::new();
    match data.kcl_options {
        Some(ref arg) => {
            let args: Vec<&str> = arg.split(' ').collect();
            let mut i = 0;
            while i < args.len() {
                let arg = <&str>::clone(args.get(i).unwrap());
                match arg {
                    SETTINGS_FILE_PARA => {
                        i += 1;
                        let mut settings_vec: Vec<SettingsFile> = vec![settings_file];
                        while i < args.len() && !args.get(i).unwrap().starts_with('-') {
                            let para = args
                                .get(i)
                                .unwrap_or_else(|| panic!("{}: {}", INVALID_KCL_OPTIONS_MSG, arg));
                            let settings = load_file(
                                std::path::Path::new(workdir).join(para).to_str().unwrap(),
                            );
                            settings_vec.push(settings);
                        }
                        settings_file = merge_settings(&settings_vec);
                    }
                    ARGUMENTS_PARA => {
                        i += 1;
                        let para = args
                            .get(i)
                            .unwrap_or_else(|| panic!("{}: {}", INVALID_KCL_OPTIONS_MSG, arg));
                        let para = String::from(*para);
                        let paras: Vec<&str> = para.split('=').collect();
                        if paras.len() != 2 {
                            panic!("{}: {}", INVALID_KCL_OPTIONS_MSG, arg);
                        }
                        (*settings_file.kcl_options.as_mut().unwrap()).push(KeyValuePair {
                            key: String::from(*paras.get(0).unwrap()),
                            value: String::from(*paras.get(1).unwrap()),
                        });
                    }
                    DEBUG_PARA => {
                        (*settings_file.kcl_cli_configs.as_mut().unwrap()).debug = Some(true)
                    }
                    STRICT_RANGE_CHECK_PARA => {
                        (*settings_file.kcl_cli_configs.as_mut().unwrap()).strict_range_check =
                            Some(true)
                    }
                    DISABLE_NONE_PARA => {
                        (*settings_file.kcl_cli_configs.as_mut().unwrap()).disable_none = Some(true)
                    }
                    OVERRIDES_PARA => {
                        i += 1;
                        let para = args
                            .get(i)
                            .unwrap_or_else(|| panic!("{}: {}", INVALID_KCL_OPTIONS_MSG, arg));
                        (*settings_file.kcl_cli_configs.as_mut().unwrap())
                            .overrides
                            .as_mut()
                            .unwrap()
                            .push(para.to_string());
                    }
                    PATH_SELECTOR_PARA => {
                        i += 1;
                        let para = args
                            .get(i)
                            .unwrap_or_else(|| panic!("{}: {}", INVALID_KCL_OPTIONS_MSG, arg));
                        (*settings_file.kcl_cli_configs.as_mut().unwrap())
                            .path_selector
                            .as_mut()
                            .unwrap()
                            .push(para.to_string());
                    }
                    VERBOSE_PARA => {
                        let verbose = (*settings_file.kcl_cli_configs.as_mut().unwrap())
                            .verbose
                            .as_mut()
                            .unwrap();
                        *verbose += 1;
                    }
                    OUTPUT_PARA => {
                        i += 1;
                        let para = args
                            .get(i)
                            .unwrap_or_else(|| panic!("{}: {}", INVALID_KCL_OPTIONS_MSG, arg));
                        (*settings_file.kcl_cli_configs.as_mut().unwrap()).output =
                            Some(para.to_string());
                    }
                    _ => {
                        if arg.ends_with(".k") {
                            (*settings_file
                                .kcl_cli_configs
                                .as_mut()
                                .unwrap()
                                .files
                                .as_mut()
                                .unwrap())
                            .push(String::from(arg));
                        }
                    }
                }
                i += 1;
            }
            settings_file
        }
        None => settings_file,
    }
}

#[cfg(test)]
mod settings_test {
    use crate::settings::*;

    const SETTINGS_FILE: &str = "./src/testdata/settings.yaml";

    #[test]
    fn test_settings_load_file() {
        let settings = load_file(SETTINGS_FILE);
        assert!(settings.kcl_cli_configs.is_some());
        assert!(settings.kcl_options.is_some());
        if let Some(kcl_cli_configs) = settings.kcl_cli_configs {
            let files = vec![
                String::from("../main.k"),
                String::from("./before/base.k"),
                String::from("./main.k"),
                String::from("./sub/sub.k"),
            ];
            assert!(kcl_cli_configs.files.is_some());
            assert!(kcl_cli_configs.disable_none.is_some());
            assert!(kcl_cli_configs.strict_range_check.is_some());
            assert!(kcl_cli_configs.debug.is_some());
            assert!(kcl_cli_configs.path_selector.is_none());
            assert!(kcl_cli_configs.overrides.is_none());
            if let Some(config_files) = kcl_cli_configs.files {
                assert!(config_files == files);
            }
        }
        if let Some(kcl_options) = settings.kcl_options {
            assert!(kcl_options.len() == 2);
        }
    }

    #[test]
    fn test_merge_settings() {
        let settings1 = load_file(SETTINGS_FILE);
        let settings2 = load_file(SETTINGS_FILE);
        let settings = merge_settings(&vec![settings1, settings2]);
        if let Some(kcl_cli_configs) = settings.kcl_cli_configs {
            let files = vec![
                String::from("../main.k"),
                String::from("./before/base.k"),
                String::from("./main.k"),
                String::from("./sub/sub.k"),
            ];
            assert!(kcl_cli_configs.files.is_some());
            assert!(kcl_cli_configs.disable_none.is_some());
            assert!(kcl_cli_configs.strict_range_check.is_some());
            assert!(kcl_cli_configs.debug.is_some());
            assert!(kcl_cli_configs.path_selector.is_some());
            assert!(kcl_cli_configs.overrides.is_some());
            if let Some(config_files) = kcl_cli_configs.files {
                assert!(config_files == files);
            }
        }
        if let Some(kcl_options) = settings.kcl_options {
            assert!(kcl_options.len() == 4);
        }
    }

    #[test]
    fn test_decode_test_format_settings_file() {
        let settings = decode_test_format_settings_file("./src/testdata/test_settings.yaml", "");
        assert!(settings.kcl_cli_configs.as_ref().unwrap().debug.unwrap() == true);
        assert!(
            settings
                .kcl_cli_configs
                .as_ref()
                .unwrap()
                .strict_range_check
                .unwrap()
                == true
        );
    }
}
