//! Copyright The KCL Authors. All rights reserved.
use anyhow::{Context, Result};
use serde::{
    de::{DeserializeSeed, Error, MapAccess, SeqAccess, Unexpected, Visitor},
    Deserialize, Serialize,
};
use std::{collections::HashMap, ops::Deref, path::PathBuf};

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
    pub sort_keys: Option<bool>,
    pub show_hidden: Option<bool>,
    /// Whether including schema type in JSON/YAML result.
    pub include_schema_type_path: Option<bool>,
    /// kcl needs a mapping between the package name and the package path
    /// to determine the source code path corresponding to different version package.
    pub package_maps: Option<HashMap<String, String>>,
    /// Use the evaluator to execute the AST program instead of AOT.
    pub fast_eval: Option<bool>,
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
                sort_keys: Some(false),
                show_hidden: Some(false),
                fast_eval: Some(false),
                include_schema_type_path: Some(false),
                package_maps: Some(HashMap::default()),
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

/// Top level argument key value pair.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KeyValuePair {
    /// key is the top level argument key.
    pub key: String,
    // Note: here is a normal json string including int, float, string, bool list and dict.
    pub value: ValueString,
}

#[macro_export]
macro_rules! tri {
    ($e:expr $(,)?) => {
        match $e {
            core::result::Result::Ok(val) => val,
            core::result::Result::Err(err) => return core::result::Result::Err(err),
        }
    };
}

/// MapStringKey denotes the map deserialize key.
struct MapStringKey;
impl<'de> DeserializeSeed<'de> for MapStringKey {
    type Value = String;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for MapStringKey {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s.to_owned())
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(s)
    }
}

/// Top level argument value string.
/// Note: here is a normal json string including int, float, string, bool list and dict.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct ValueString(pub String);

impl Deref for ValueString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ValueString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ValueString {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl<'de> Deserialize<'de> for ValueString {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<ValueString, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ValueString;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid JSON value or KCL value expression")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ValueString(serde_json::to_string(&value).map_err(
                    |_| Error::invalid_type(Unexpected::Bool(value), &self),
                )?))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ValueString(serde_json::to_string(&value).map_err(
                    |_| Error::invalid_type(Unexpected::Signed(value), &self),
                )?))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ValueString(serde_json::to_string(&value).map_err(
                    |_| Error::invalid_type(Unexpected::Unsigned(value), &self),
                )?))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ValueString(serde_json::to_string(&value).map_err(
                    |_| Error::invalid_type(Unexpected::Float(value), &self),
                )?))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ValueString(serde_json::to_string(&value).map_err(
                    |_| Error::invalid_type(Unexpected::Str(&value), &self),
                )?))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ValueString("null".into()))
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
                D::Error: Error,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(ValueString("null".into()))
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
                V::Error: Error,
            {
                let mut vec: Vec<serde_json::Value> = Vec::new();

                while let Some(elem) = tri!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(ValueString(serde_json::to_string(&vec).map_err(|_| {
                    Error::invalid_type(Unexpected::Seq, &self)
                })?))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
                V::Error: Error,
            {
                match visitor.next_key_seed(MapStringKey)? {
                    Some(first_key) => {
                        let mut values: HashMap<String, serde_json::Value> = HashMap::new();

                        values.insert(first_key, tri!(visitor.next_value()));
                        while let Some((key, value)) = tri!(visitor.next_entry()) {
                            values.insert(key, value);
                        }

                        Ok(ValueString(serde_json::to_string(&values).map_err(
                            |_| Error::invalid_type(Unexpected::Map, &self),
                        )?))
                    }
                    None => Ok(ValueString("{}".into())),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TestSettingsFile {
    kcl_options: Option<String>,
}

/// Load kcl settings file.
pub fn load_file(filename: &str) -> Result<SettingsFile> {
    let f = std::fs::File::open(filename)
        .with_context(|| format!("Failed to load '{}', no such file or directory", filename))?;
    let data: SettingsFile = serde_yaml::from_reader(f)
        .with_context(|| format!("Failed to load '{}', invalid setting file format", filename))?;
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
                set_if!(result_kcl_cli_configs, sort_keys, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, show_hidden, kcl_cli_configs);
                set_if!(result_kcl_cli_configs, fast_eval, kcl_cli_configs);
                set_if!(
                    result_kcl_cli_configs,
                    include_schema_type_path,
                    kcl_cli_configs
                );
                set_if!(result_kcl_cli_configs, package_maps, kcl_cli_configs);
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

/// Build SettingsPathBuf from args.
pub fn build_settings_pathbuf(
    files: &[&str],
    setting_files: Option<Vec<&str>>,
    setting_config: Option<SettingsFile>,
) -> Result<SettingsPathBuf> {
    let mut path = None;
    let settings = if let Some(files) = setting_files {
        let mut settings = vec![];
        for file in &files {
            let s = load_file(file)?;
            if !s.input().is_empty() {
                path = Some(
                    PathBuf::from(file)
                        .parent()
                        .map(|p| p.to_path_buf())
                        .ok_or(anyhow::anyhow!("The parent path of {file} is not found"))?,
                )
            }
            settings.push(s);
        }
        merge_settings(&settings)
    // If exists default kcl.yaml, load it.
    } else if std::fs::metadata(DEFAULT_SETTING_FILE).is_ok() {
        path = Some(
            PathBuf::from(DEFAULT_SETTING_FILE)
                .parent()
                .map(|p| p.to_path_buf())
                .ok_or(anyhow::anyhow!(
                    "The parent path of {DEFAULT_SETTING_FILE} is not found"
                ))?,
        );
        load_file(DEFAULT_SETTING_FILE)?
    } else {
        SettingsFile::default()
    };
    let mut settings = if let Some(setting_config) = setting_config {
        merge_settings(&[settings, setting_config])
    } else {
        settings
    };
    if let Some(config) = &mut settings.kcl_cli_configs {
        if !files.is_empty() {
            config.files = Some(files.iter().map(|f| f.to_string()).collect());
        }
    }
    Ok(SettingsPathBuf::new(path, settings))
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
            assert!(kcl_cli_configs.include_schema_type_path.is_none());
            assert!(kcl_cli_configs.show_hidden.is_none());
            assert!(kcl_cli_configs.fast_eval.is_none());
            assert_eq!(kcl_cli_configs.sort_keys, Some(true));
            if let Some(config_files) = kcl_cli_configs.files {
                assert!(config_files == files);
            }
        }
        if let Some(kcl_options) = settings.kcl_options {
            assert!(kcl_options.len() == 6);
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
            assert!(kcl_options.len() == 12);
        }
        Ok(())
    }
}
