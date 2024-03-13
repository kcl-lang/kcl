use crate::{kcl, lookup_the_nearest_file_dir};
use anyhow::{bail, Result};
use kclvm_config::modfile::KCL_MOD_FILE;
use kclvm_parser::LoadProgramOptions;
use notify::{RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::marker::Send;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::PathBuf,
    process::Command,
    sync::{mpsc::channel, Arc, Mutex},
};

/// Searches for the nearest kcl.mod directory containing the given file and fills the compilation options
/// with metadata of dependent packages.
///
/// # Arguments
///
/// * `k_file_path` - Path to the K file for which metadata is needed.
/// * `opts` - Mutable reference to the compilation options to fill.
///
/// # Returns
///
/// * `Result<()>` - Empty result if successful, error otherwise.
pub(crate) fn fill_pkg_maps_for_k_file(
    k_file_path: PathBuf,
    opts: &mut LoadProgramOptions,
) -> Result<()> {
    match lookup_the_nearest_file_dir(k_file_path, KCL_MOD_FILE) {
        Some(mod_dir) => {
            let metadata = fetch_metadata(mod_dir.canonicalize()?)?;
            let maps: HashMap<String, String> = metadata
                .packages
                .into_iter()
                .map(|(pname, pkg)| (pname, pkg.manifest_path.display().to_string()))
                .collect();
            opts.package_maps.extend(maps);
        }
        None => return Ok(()),
    };

    Ok(())
}

/// Trait for writing messages to a file.
pub trait Writer {
    fn write_message(&mut self, message: &str) -> Result<()>;
}

impl Writer for File {
    /// Writes a message to the file followed by a newline.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to write.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Empty result if successful, error otherwise.
    fn write_message(&mut self, message: &str) -> Result<()> {
        writeln!(self, "{}", message)?;
        Ok(())
    }
}

/// Watches for modifications in the kcl.mod file within the given directory and updates dependencies accordingly.
///
/// # Arguments
///
/// * `directory` - The directory containing the kcl.mod file to watch.
/// * `writer` - The writer for outputting log messages.
///
/// # Returns
///
/// * `Result<()>` - Empty result if successful, error otherwise.
pub fn watch_kcl_mod<W: Writer + Send + 'static>(directory: PathBuf, writer: W) -> Result<()> {
    let writer = Arc::new(Mutex::new(writer)); // Wrap writer in Arc<Mutex<_>> for thread safety
    let (sender, receiver) = channel();
    let writer_clone = Arc::clone(&writer); // Create a clone of writer for the closure

    let mut watcher = notify::recommended_watcher(move |res| {
        if let Err(err) = sender.send(res) {
            let mut writer = writer_clone.lock().unwrap(); // Lock the mutex before using writer
            writer
                .write_message(&format!("Error sending event to channel: {:?}", err))
                .ok();
        }
    })?;

    watcher.watch(&directory, RecursiveMode::NonRecursive)?;

    loop {
        match receiver.recv() {
            Ok(event) => {
                match event {
                    Ok(event) => match event.kind {
                        notify::event::EventKind::Modify(modify_kind) => {
                            if let notify::event::ModifyKind::Data(data_change) = modify_kind {
                                if data_change == notify::event::DataChange::Content {
                                    let mut writer = writer.lock().unwrap(); // Lock the mutex before using writer
                                    writer.write_message("kcl.mod file content modified. Updating dependencies...").ok();
                                    update_dependencies(directory.clone())?;
                                }
                            }
                        }
                        _ => {}
                    },
                    Err(err) => {
                        let mut writer = writer.lock().unwrap(); // Lock the mutex before using writer
                        writer
                            .write_message(&format!("Watcher error: {:?}", err))
                            .ok();
                    }
                }
            }
            Err(e) => {
                let mut writer = writer.lock().unwrap(); // Lock the mutex before using writer
                writer
                    .write_message(&format!("Receiver error: {:?}", e))
                    .ok();
            }
        }
    }
}

impl<W> Writer for Arc<Mutex<W>>
where
    W: Writer,
{
    /// Writes a message using the wrapped writer.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to write.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Empty result if successful, error otherwise.
    fn write_message(&mut self, message: &str) -> Result<()> {
        self.lock().unwrap().write_message(message)
    }
}

/// Tracks changes in the kcl.mod file within the given working directory and watches for updates.
///
/// # Arguments
///
/// * `work_dir` - The working directory where the kcl.mod file is located.
///
/// # Returns
///
/// * `Result<()>` - Empty result if successful, error otherwise.
pub fn kcl_mod_file_track<W>(work_dir: PathBuf, writer: W) -> Result<()>
where
    W: Writer + Send + 'static,
{
    let writer = Arc::new(Mutex::new(writer)); // Wrap writer in Arc<Mutex<_>> for thread safety

    let directory = match lookup_the_nearest_file_dir(work_dir.clone(), KCL_MOD_FILE) {
        Some(mod_dir) => mod_dir,
        None => {
            let mut writer = writer.lock().unwrap(); // Lock the writer
            writer.write_message(&format!(
                "Manifest file '{}' not found in directory hierarchy",
                KCL_MOD_FILE
            ))?;
            return Ok(());
        }
    };

    if let Err(err) = watch_kcl_mod(directory, Arc::clone(&writer)) {
        let mut writer = writer.lock().unwrap(); // Lock the writer
        writer.write_message(&format!("Error watching kcl.mod file: {:?}", err))?;
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]

/// [`Metadata`] is the metadata of the current KCL module,
/// currently only the mapping between the name and path of the external dependent package is included.
pub struct Metadata {
    pub packages: HashMap<String, Package>,
}

/// Structure representing a package.
#[derive(Clone, Debug, Serialize, Deserialize)]
/// [`Package`] is a kcl package.
pub struct Package {
    /// Name as given in the `kcl.mod`
    pub name: String,
    /// Path containing the `kcl.mod`
    pub manifest_path: PathBuf,
}

impl Metadata {
    /// Parses metadata from a string.
    ///
    /// # Arguments
    ///
    /// * `data` - The string containing metadata.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Metadata if successful, error otherwise.
    fn parse(data: String) -> Result<Self> {
        let meta = serde_json::from_str(data.as_ref())?;
        Ok(meta)
    }
}

/// Fetches metadata of packages from the kcl.mod file within the given directory.
///
/// # Arguments
///
/// * `manifest_path` - The path to the directory containing the kcl.mod file.
///
/// # Returns
///
/// * `Result<Metadata>` - Metadata if successful, error otherwise.
pub fn fetch_metadata(manifest_path: PathBuf) -> Result<Metadata> {
    match Command::new(kcl())
        .arg("mod")
        .arg("metadata")
        .current_dir(manifest_path)
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                bail!(
                    "fetch metadata failed with error: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok(Metadata::parse(
                String::from_utf8_lossy(&output.stdout).to_string(),
            )?)
        }
        Err(err) => bail!("fetch metadata failed with error: {}", err),
    }
}

/// Updates dependencies for the kcl.mod file within the given directory.
///
/// # Arguments
///
/// * `work_dir` - The working directory containing the kcl.mod file.
///
/// # Returns
///
/// * `Result<()>` - Empty result if successful, error otherwise.
pub fn update_dependencies(work_dir: PathBuf) -> Result<()> {
    match lookup_the_nearest_file_dir(work_dir.clone(), KCL_MOD_FILE) {
        Some(mod_dir) => {
            match Command::new(kcl())
                .arg("mod")
                .arg("update")
                .current_dir(mod_dir)
                .output()
            {
                Ok(output) => {
                    if !output.status.success() {
                        bail!(
                            "update failed with error: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                    Ok(())
                }
                Err(err) => bail!("update failed with error: {}", err),
            }
        }
        None => bail!(
            "Manifest file '{}' not found in directory hierarchy",
            KCL_MOD_FILE
        ),
    }
}
