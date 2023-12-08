use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use compiler_base_span::{span::new_byte_pos, BytePos, FilePathMapping, SourceMap};
use json_spanned_value::{self as jsv, spanned};
use kclvm_ast::ast::PosTuple;
use located_yaml::YamlLoader;

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
    // SourceMap is used to find the position of the error in the file
    sm: SourceMap,
}

impl DataLoader {
    /// If `DataLoader` is constructed using a file path, then `content` is the content of the file.
    pub(crate) fn new_with_file_path(loader_kind: LoaderKind, file_path: &str) -> Result<Self> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to Load '{}'", file_path))?;
        let sm = SourceMap::new(FilePathMapping::empty());
        sm.new_source_file(PathBuf::from(file_path).into(), content.clone());
        Ok(Self {
            kind: loader_kind,
            content,
            sm,
        })
    }

    /// If `DataLoader` is constructed using a Json/Yaml string, then `content` is the string
    #[allow(dead_code)]
    pub(crate) fn new_with_str(loader_kind: LoaderKind, content: &str) -> Result<Self> {
        let sm = SourceMap::new(FilePathMapping::empty());
        sm.new_source_file(PathBuf::from("").into(), content.to_string());
        Ok(Self {
            kind: loader_kind,
            content: content.to_string(),
            sm,
        })
    }

    pub(crate) fn get_data(&self) -> &str {
        &self.content
    }

    pub(crate) fn get_kind(&self) -> &LoaderKind {
        &self.kind
    }

    /// Convert the position in the source map to the position in the source file
    pub fn byte_pos_to_pos_in_sourcemap(&self, lo: BytePos, hi: BytePos) -> PosTuple {
        let lo = self.sm.lookup_char_pos(lo);
        let hi = self.sm.lookup_char_pos(hi);
        let filename: String = format!("{}", lo.file.name.prefer_remapped());
        (
            filename,
            lo.line as u64,
            lo.col.0 as u64,
            hi.line as u64,
            hi.col.0 as u64,
        )
    }

    pub fn file_name(&self) -> String {
        self.sm
            .lookup_char_pos(new_byte_pos(0))
            .file
            .name
            .prefer_remapped()
            .to_string()
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

/// Load data into Json value with span.
impl Loader<spanned::Value> for DataLoader {
    fn load(&self) -> Result<spanned::Value> {
        let v = match self.kind {
            LoaderKind::JSON => jsv::from_str(self.get_data())
                .with_context(|| format!("Failed to String '{}' to Json", self.get_data()))?,
            _ => {
                bail!("Failed to String to Json Value")
            }
        };

        Ok(v)
    }
}

/// Load data into Json value with span.
impl Loader<located_yaml::Yaml> for DataLoader {
    fn load(&self) -> Result<located_yaml::Yaml> {
        let v = match self.kind {
            LoaderKind::YAML => YamlLoader::load_from_str(self.get_data())
                .with_context(|| format!("Failed to String '{}' to Yaml", self.get_data()))?,
            _ => {
                bail!("Failed to String to Yaml Value")
            }
        };

        v.docs
            .get(0)
            .map_or_else(|| bail!("Failed to Load YAML"), |res| Ok(res.clone()))
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
