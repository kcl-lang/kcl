use std::fs;

use anyhow::{bail, Context, Result};

pub(crate) trait Loader<T> {
    fn load_ast(&self) -> Result<T>;
}

pub(crate) struct LoaderInner {
    content: String,
}

pub(crate) enum LoaderKind {
    YAML,
    JSON,
}

impl LoaderInner {
    fn new_with_file_path(file_path: &str) -> Result<Self> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to Load '{}'", file_path))?;

        Ok(Self { content })
    }

    fn new_with_str(s: &str) -> Result<Self> {
        Ok(Self {
            content: s.to_string(),
        })
    }

    fn get_content(&self) -> &str {
        &self.content
    }
}

pub(crate) struct DataLoader {
    kind: LoaderKind,
    inner: LoaderInner,
}

impl DataLoader {
    pub(crate) fn new_with_file_path(loader_kind: LoaderKind, file_path: &str) -> Result<Self> {
        Ok(Self {
            kind: loader_kind,
            inner: LoaderInner::new_with_file_path(file_path)
                .with_context(|| format!("Failed to Load '{}'", file_path))?,
        })
    }

    pub(crate) fn new_with_str(loader_kind: LoaderKind, s: &str) -> Result<Self> {
        Ok(Self {
            kind: loader_kind,
            inner: LoaderInner::new_with_str(s)
                .with_context(|| format!("Failed to Load '{}'", s))?,
        })
    }

    pub(crate) fn get_data(&self) -> &str {
        &self.inner.get_content()
    }
}

impl Loader<serde_json::Value> for DataLoader {
    fn load_ast(&self) -> Result<serde_json::Value> {
        let v = match self.kind {
            LoaderKind::JSON => serde_json::to_value(&self.get_data())
                .with_context(|| format!("Failed to String '{}' to Json", self.get_data()))?,
            _ => {
                bail!("Failed to String to Json Value")
            }
        };

        Ok(v)
    }
}

impl Loader<serde_yaml::Value> for DataLoader {
    fn load_ast(&self) -> Result<serde_yaml::Value> {
        let v = match self.kind {
            LoaderKind::YAML => serde_yaml::to_value(&self.get_data())
                .with_context(|| format!("Failed to String '{}' to Yaml", self.get_data()))?,
            _ => {
                bail!("Failed to String to Yaml Value")
            }
        };

        Ok(v)
    }
}
