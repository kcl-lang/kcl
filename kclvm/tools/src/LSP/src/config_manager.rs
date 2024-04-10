use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct JsonFile {
    watch: PathBuf,
    recursive: Option<bool>,
    patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Config {
    path: PathBuf,
    recursive: bool,
    patterns: Vec<String>,
}

impl Config {
    /// Load configuration from file
    pub fn load_from_file(file_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(file_path)?;
        let config: JsonFile = serde_json::from_str(&file_content)?;
        Ok(Config {
            path: config.watch,
            recursive: config.recursive.unwrap_or(false),
            patterns: config.patterns.unwrap_or_default(),
        })
    }

    /// Get the path from configuration
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Check if the configuration is recursive
    pub fn is_recursive(&self) -> bool {
        self.recursive
    }

    /// Get the file patterns from configuration
    pub fn patterns(&self) -> &Vec<String> {
        &self.patterns
    }
}

/// Get the configuration file path
pub fn get_config_file() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let config_path = current_dir.join("observer.json");

    if config_path.exists() {
        Some(config_path)
    } else {
        None
    }
}
