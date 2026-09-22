use crate::{kcl, lookup_the_nearest_file_dir};
use anyhow::{Result, anyhow, bail};
use kcl_config::modfile::{KCL_MOD_FILE, KCL_MOD_LOCK_FILE, load_mod_file, load_mod_lock_file};
use kcl_parser::LoadProgramOptions;
use kcl_utils::pkgpath::external_pkgpath_to_rel_path_buf;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf, process::Command};
#[cfg(not(target_arch = "wasm32"))]
use {crate::client::ModClient, parking_lot::Mutex, std::sync::Arc};

/// `Toolchain` is a trait that outlines a standard set of operations that must be
/// implemented for a KCL module (mod), typically involving fetching metadata from,
/// and updating dependencies within, a specified path.
pub trait Toolchain: Send + Sync {
    /// Fetches the metadata from the given manifest file path.
    ///
    /// The `manifest_path` parameter is generic over P, meaning it can be any type that
    /// implements the `AsRef<Path>` trait. It is commonly a reference to a file path or a type
    /// that can be converted into a file path reference, such as `String` or `PathBuf`.
    ///
    /// The return type `Result<Metadata>` indicates that this method will either return an
    /// instance of `Metadata` or an error.
    ///
    /// # Parameters
    ///
    /// * `manifest_path` - A reference to the path of the manifest file, expected to be a type
    ///   that can be converted into a reference to a filesystem path.
    fn fetch_metadata(&self, manifest_path: PathBuf) -> Result<Metadata>;

    /// Updates the dependencies as defined within the given manifest file path.
    ///
    /// The `manifest_path` parameter is generic over P, just like in the `fetch_metadata` method,
    /// and is used to specify the location of the manifest file.
    ///
    /// The return type `Result<()>` indicates that this method will execute without returning a
    /// value upon success but may return an error.
    ///
    /// # Parameters
    ///
    /// * `manifest_path` - A reference to the path of the manifest file, expected to be a type
    ///   that can be converted into a reference to a filesystem path.
    fn update_dependencies(&self, manifest_path: PathBuf) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct CommandToolchain<S: AsRef<OsStr>> {
    path: S,
}

impl Default for CommandToolchain<PathBuf> {
    fn default() -> Self {
        Self { path: kcl() }
    }
}

/// Builds a helpful error when the `kcl` executable cannot be spawned,
/// e.g. when the CLI is not installed or not on `PATH`.
fn command_spawn_error(operation: &str, err: std::io::Error) -> anyhow::Error {
    if err.kind() == std::io::ErrorKind::NotFound {
        anyhow!(
            "failed to {operation}: the `kcl` CLI executable was not found in PATH. \
             Install it from https://kcl-lang.io/docs/user_docs/getting-started/install"
        )
    } else {
        anyhow!("failed to {operation} with error: {err}")
    }
}

impl<S: AsRef<OsStr> + Send + Sync> Toolchain for CommandToolchain<S> {
    fn fetch_metadata(&self, manifest_path: PathBuf) -> Result<Metadata> {
        match Command::new(&self.path)
            .arg("mod")
            .arg("metadata")
            .arg("--update")
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
            Err(err) => Err(command_spawn_error("fetch metadata", err)),
        }
    }

    fn update_dependencies(&self, manifest_path: PathBuf) -> Result<()> {
        match Command::new(&self.path)
            .arg("mod")
            .arg("update")
            .current_dir(manifest_path)
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
            Err(err) => Err(command_spawn_error("update dependencies", err)),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct NativeToolchain {
    client: Arc<Mutex<ModClient>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Toolchain for NativeToolchain {
    fn fetch_metadata(&self, manifest_path: PathBuf) -> Result<Metadata> {
        let mut client = self.client.lock();
        client.change_work_dir(manifest_path)?;
        match client.get_metadata_from_mod_lock_file() {
            Some(metadata) => Ok(metadata),
            None => client.resolve_all_deps(false),
        }
    }

    fn update_dependencies(&self, manifest_path: PathBuf) -> Result<()> {
        let mut client = self.client.lock();
        client.change_work_dir(manifest_path)?;
        let _ = client.resolve_all_deps(true)?;
        Ok(())
    }
}

/// [`Metadata`] is the metadata of the current KCL module,
/// currently only the mapping between the name and path of the external dependent package is included.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Metadata {
    pub packages: HashMap<String, Package>,
}

/// [`Package`] is a structure representing a package.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// [`default`] returns the default toolchain.
#[inline]
pub fn default() -> impl Toolchain {
    CommandToolchain::default()
}

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
    tool: &dyn Toolchain,
    k_file_path: PathBuf,
    opts: &mut LoadProgramOptions,
) -> Result<Option<Metadata>> {
    match lookup_the_nearest_file_dir(k_file_path, KCL_MOD_FILE) {
        Some(mod_dir) => {
            let mod_dir = mod_dir.canonicalize()?;
            // Prefer the toolchain-provided metadata (which may pull in OCI/Git
            // packages) and fall back to reading `kcl.mod` + `kcl.mod.lock`
            // directly when the toolchain is unavailable (e.g. the `kcl` CLI is
            // not installed in a test/CI environment).
            match tool.fetch_metadata(mod_dir.clone()) {
                Ok(metadata) => {
                    extend_pkg_maps_from_metadata(&metadata, opts);
                    Ok(Some(metadata))
                }
                Err(_) => match fill_pkg_maps_from_lock_file(&mod_dir, opts) {
                    Ok(Some(metadata)) => Ok(Some(metadata)),
                    Ok(None) => Ok(None),
                    Err(_) => Ok(None),
                },
            }
        }
        None => Ok(None),
    }
}

fn extend_pkg_maps_from_metadata(metadata: &Metadata, opts: &mut LoadProgramOptions) {
    let maps: HashMap<String, String> = metadata
        .packages
        .iter()
        .map(|(name, pkg)| (name.clone(), pkg.manifest_path.display().to_string()))
        .collect();
    opts.package_maps.extend(maps);
}

/// Fallback implementation that builds the package map directly from
/// `kcl.mod` and `kcl.mod.lock` when the `kcl` CLI toolchain is unavailable.
///
/// Only local-path dependencies can be resolved without the CLI (we have
/// neither the OCI registry nor the Git cache); any non-local dependency is
/// silently skipped because the compile step has no way to find it.
fn fill_pkg_maps_from_lock_file(
    mod_dir: &Path,
    opts: &mut LoadProgramOptions,
) -> Result<Option<Metadata>> {
    if !mod_dir.join(KCL_MOD_LOCK_FILE).is_file() {
        return Ok(None);
    }
    let mod_lock = match load_mod_lock_file(mod_dir) {
        Ok(m) => m,
        Err(_) => return Ok(None),
    };
    let mod_file = match load_mod_file(mod_dir) {
        Ok(m) => m,
        Err(_) => return Ok(None),
    };
    let deps = match (&mod_lock.dependencies, &mod_file.dependencies) {
        (Some(lock), Some(mod_deps)) => (lock.clone(), mod_deps.clone()),
        _ => return Ok(None),
    };

    let mut metadata = Metadata::default();
    for (name, lock_dep) in deps.0.iter() {
        let Some(mod_dep) = deps.1.get(name) else {
            continue;
        };
        let kcl_config::modfile::Dependency::Local(local) = mod_dep else {
            continue;
        };
        let local_path = PathBuf::from(&local.path);
        let manifest_path = if local_path.is_absolute() {
            local.path.clone()
        } else {
            mod_dir
                .join(&local.path)
                .canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| mod_dir.join(&local.path).to_string_lossy().to_string())
        };
        // The map key is the on-disk package name (`name` with hyphens
        // replaced by underscores), which is what the parser uses to look
        // up packages. `full_name` is only relevant for vendored copies.
        let map_key = lock_dep.name.replace('-', "_");
        metadata.packages.insert(
            map_key.clone(),
            Package {
                name: lock_dep.name.clone(),
                manifest_path: PathBuf::from(&manifest_path),
            },
        );
        opts.package_maps.insert(map_key, manifest_path);
    }
    Ok(Some(metadata))
}

/// [`get_real_path_from_external`] will ask for the local path for [`pkg_name`] with subdir [`pkgpath`].
/// If the external package, whose [`pkg_name`] is 'my_package', is stored in '\user\my_package_v0.0.1'.
/// The [`pkgpath`] is 'my_package.examples.apps'.
///
/// [`get_real_path_from_external`] will return '\user\my_package_v0.0.1\examples\apps'
///
/// # Note
/// [`get_real_path_from_external`] is just a method for calculating a path, it doesn't check whether a path exists.
pub fn get_real_path_from_external(
    tool: &dyn Toolchain,
    pkg_name: &str,
    pkgpath: &str,
    current_pkg_path: PathBuf,
) -> PathBuf {
    let mut real_path = PathBuf::new();
    let pkg_root = tool
        .fetch_metadata(current_pkg_path)
        .map(|metadata| {
            metadata
                .packages
                .get(pkg_name)
                .map_or(PathBuf::new(), |pkg| pkg.manifest_path.clone())
        })
        .unwrap_or_else(|_| PathBuf::new());
    real_path = real_path.join(pkg_root);

    real_path.push(external_pkgpath_to_rel_path_buf(pkgpath));
    real_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_cli_reports_install_hint() {
        let tool = CommandToolchain {
            path: PathBuf::from("kcl-binary-that-does-not-exist"),
        };
        let err = tool
            .update_dependencies(PathBuf::from("."))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not found in PATH"),
            "unexpected error message: {err}"
        );

        let err = tool
            .fetch_metadata(PathBuf::from("."))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not found in PATH"),
            "unexpected error message: {err}"
        );
    }
}
