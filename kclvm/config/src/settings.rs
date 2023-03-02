// Copyright 2021 The KCL Authors. All rights reserved.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default settings file `kcl.yaml`
pub const DEFAULT_SETTING_FILE: &str = "kcl.yaml";

/// Readonly settings with the filepath.
#[derive(Debug, Default, Clone)]
pub struct SettingsPathBuf(Option<PathBuf>, SettingsFile);

impl SettingsPathBuf {
    /// New a settings with path and settings content.
    #[inline]
    pub fn new(path: Option<PathBuf>, settings: SettingsFile) -> Self {
        Self(path, settings)
    }

    /// Get the output setting.
    #[inline]
    pub fn output(&self) -> Option<String> {
        match &self.1.kcl_cli_configs {
            Some(c) => c.output.clone(),
            None => None,
        }
    }

    /// Get the path.
    #[inline]
    pub fn path(&self) -> &Option<PathBuf> {
        &self.0
    }

    /// Get the settings.
    #[inline]
    pub fn settings(&self) -> &SettingsFile {
        &self.1
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsFile {
    pub kcl_cli_configs: Option<Config>,
    pub kcl_options: Option<Vec<KeyValuePair>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

    /// Get the output setting.
    #[inline]
    pub fn output(&self) -> Option<String> {
        match &self.kcl_cli_configs {
            Some(c) => c.output.clone(),
            None => None,
        }
    }

    /// Get the input setting.
    #[inline]
    pub fn input(&self) -> Vec<String> {
        match &self.kcl_cli_configs {
            Some(c) => match &c.file {
                Some(file) => match &c.files {
                    Some(files) if !files.is_empty() => files.clone(),
                    _ => file.clone(),
                },
                None => match &c.files {
                    Some(files) => files.clone(),
                    None => vec![],
                },
            },
            None => vec![],
        }
    }
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TestSettingsFile {
    kcl_options: Option<String>,
}

/// Load kcl settings file.
pub fn load_file(filename: &str) -> Result<SettingsFile> {
    let f = std::fs::File::open(filename)?;
    let data: SettingsFile = serde_yaml::from_reader(f)?;
    Ok(data)
}

macro_rules! set_if {
    ($result: expr, $attr: ident, $setting: expr) => {
        if $setting.$attr.is_some() {
            $result.$attr = $setting.$attr.clone();
        }
    };
}

/// Merge multiple settings into one settings.
pub fn merge_settings(settings: &[SettingsFile]) -> SettingsFile {
    let mut result = SettingsFile::new();
    for setting in settings {
        if let Some(kcl_cli_configs) = &setting.kcl_cli_configs {
            if result.kcl_cli_configs.is_none() {
                result.kcl_cli_configs = Some(Config::default());
            }
            if let Some(result_kcl_cli_configs) = result.kcl_cli_configs.as_mut() {
                set_if!(result_kcl_cli_configs, files, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, file, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, output, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, overrides, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, path_selector, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, strict_range_check, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, disable_none, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, verbose, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, debug, kcl_cli_configs);
            }
        }
        if let Some(kcl_options) = &setting.kcl_options {
            if result.kcl_options.is_none() {
                result.kcl_options = Some(vec![])
            }
            if let Some(result_kcl_options) = result.kcl_options.as_mut() {
                for option in kcl_options {
                    result_kcl_options.push(option.clone());
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod settings_test {
    use crate::settings::*;

    const SETTINGS_FILE: &str = "./src/testdata/settings.yaml";

    #[test]
    fn test_settings_load_file() {
        let settings = load_file(SETTINGS_FILE).unwrap();
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
    fn test_merge_settings() -> anyhow::Result<()> {
        let settings1 = load_file(SETTINGS_FILE)?;
        let settings2 = load_file(SETTINGS_FILE)?;
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
        Ok(())
    }
}
