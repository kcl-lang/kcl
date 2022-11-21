use std::fs;

use anyhow::{bail, Context, Result};

pub(crate) trait Loader<T> {
    fn load(&self) -> Result<T>;
}

/// Types of verifiable files currently supported by KCL-Vet,
/// currently only YAML files and Json files are supported.
#[derive(Clone, Copy)]
pub enum LoaderKind {
    YAML,
    JSON,
}

/// DataLoader for Json or Yaml
/// If `DataLoader` is constructed using a file path, then `content` is the content of the file.
/// If `DataLoader` is constructed using a Json/Yaml string, then `content` is the string
pub(crate) struct DataLoader {
    kind: LoaderKind,
    content: String,
}

impl DataLoader {
    /// If `DataLoader` is constructed using a file path, then `content` is the content of the file.
    pub(crate) fn new_with_file_path(loader_kind: LoaderKind, file_path: &str) -> Result<Self> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to Load '{}'", file_path))?;

        Ok(Self {
            kind: loader_kind,
            content,
        })
    }

    /// If `DataLoader` is constructed using a Json/Yaml string, then `content` is the string
    #[allow(dead_code)]
    pub(crate) fn new_with_str(loader_kind: LoaderKind, content: &str) -> Result<Self> {
        Ok(Self {
            kind: loader_kind,
            content: content.to_string(),
        })
    }

    pub(crate) fn get_data(&self) -> &str {
        &self.content
    }

    pub(crate) fn get_kind(&self) -> &LoaderKind {
        &self.kind
    }
}

impl Loader<serde_json::Value> for DataLoader {
    /// Load data into Json value.
    fn load(&self) -> Result<serde_json::Value> {
        let v = match self.kind {
            LoaderKind::JSON => serde_json::from_str(self.get_data())
                .with_context(|| format!("Failed to String '{}' to Json", self.get_data()))?,
            _ => {
                bail!("Failed to String to Json Value")
            }
        };

        Ok(v)
    }
}

impl Loader<serde_yaml::Value> for DataLoader {
    /// Load data into Yaml value.
    fn load(&self) -> Result<serde_yaml::Value> {
        let v = match self.kind {
            LoaderKind::YAML => serde_yaml::from_str(self.get_data())
                .with_context(|| format!("Failed to String '{}' to Yaml", self.get_data()))?,
            _ => {
                bail!("Failed to String to Yaml Value")
            }
        };

        Ok(v)
    }
}
