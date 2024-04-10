use crate::config_manager::Config;
use std::collections::HashMap;
use std::fs::Metadata;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use walkdir::DirEntry;
use walkdir::WalkDir;

/// Define a trait for file handlers
pub trait FileHandler: Send + Sync {
    fn handle(&self, file: &File);
}

/// File structure to hold file metadata and data
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    name: String,
    path: PathBuf,
    data: FileData,
}

/// Data structure for file metadata
#[derive(Debug, Clone, PartialEq)]
struct FileData {
    last_accesed: SystemTime,
    last_modified: SystemTime,
}

impl File {
    /// Create a new File instance from a DirEntry
    pub fn new(file: &DirEntry) -> Self {
        let metadata = file.metadata().unwrap();
        File {
            name: file.file_name().to_str().unwrap().to_string(),
            path: file.path().to_path_buf(),
            data: FileData::new(metadata),
        }
    }

    /// Get the file name
    pub fn name(&self) -> String {
        Arc::new(&self.name).to_string()
    }

    /// Get the file extension
    pub fn extension(&self) -> Option<String> {
        self.path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
    }

    /// Get the display path of the file
    pub fn ds_path(&self) -> String {
        Arc::new(&self.path)
            .to_path_buf()
            .to_str()
            .unwrap()
            .to_string()
    }

    /// Check if the file was deleted
    pub fn was_deleted(&self) -> bool {
        !self.path.exists()
    }

    /// Get the last modification time of the file
    pub fn last_modification(&self) -> SystemTime {
        self.data.last_modified
    }

    /// Set the last modification time of the file
    pub fn set_modification(&mut self, time: SystemTime) {
        self.data.last_modified = time;
    }

    /// Detect file type based on extension
    pub fn detect_file_type(&self) -> Option<String> {
        self.extension().map(|ext| {
            match ext.as_str() {
                "k" => "K File",
                "mod" => "Mod File",
                "JSON" | "json" => "JSON File",
                "YAML" | "yaml" => "YAML File",
                _ => "Unknown File Type",
            }
            .to_string()
        })
    }
}

impl FileData {
    /// Create a new FileData instance from Metadata
    pub fn new(metadata: Metadata) -> Self {
        FileData {
            last_accesed: metadata.accessed().unwrap(),
            last_modified: metadata.modified().unwrap(),
        }
    }
}

/// Define file events
#[derive(Debug)]
pub enum FileEvent {
    Modified(File),
}

/// Observer structure to watch files
#[derive(Debug)]
pub struct Observer {
    config: Config,
    files: HashMap<String, File>,
}

impl Observer {
    /// Initialize a new Observer instance with a configuration
    pub fn new(config: Config) -> Self {
        Observer {
            files: get_files(&config),
            config,
        }
    }

    /// Iterator for file events
    pub fn iter_events(&mut self) -> impl Iterator<Item = FileEvent> + '_ {
        let interval = Duration::from_millis(500);
        let last_files = self.files.clone();
        std::iter::from_fn(move || {
            let current_files = get_files(&self.config);

            let mut events = Vec::new();
            for (name, file) in current_files.iter() {
                if let Some(last_file) = last_files.get(name) {
                    if file.last_modification() > last_file.last_modification() {
                        events.push(FileEvent::Modified(file.clone()));
                    }
                }
            }
            std::thread::sleep(interval);
            if !events.is_empty() {
                self.files = current_files;
                Some(events.remove(0))
            } else {
                None
            }
        })
    }
}

/// Get files based on configuration
fn get_files(config: &Config) -> HashMap<String, File> {
    let files = match config.is_recursive() {
        true => WalkDir::new(config.path()).min_depth(1),
        false => WalkDir::new(config.path()).min_depth(1).max_depth(1),
    }
    .into_iter()
    .filter(|x| x.as_ref().unwrap().metadata().unwrap().is_file())
    .map(|x| File::new(&x.unwrap()))
    .map(|f| (f.name(), f))
    .collect::<HashMap<_, _>>();

    if config.patterns().is_empty() {
        return files;
    }
    let mut filtered_files = HashMap::new();
    for (name, file) in files {
        let ext = file.extension().unwrap_or_default();
        if config.patterns().contains(&ext) {
            filtered_files.insert(name, file);
        }
    }
    filtered_files
}
