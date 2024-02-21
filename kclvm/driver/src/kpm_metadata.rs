use crate::{get_path_for_executable, kcl, lookup_the_nearest_file_dir};
use anyhow::{bail, Ok, Result};
use kclvm_parser::LoadProgramOptions;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, process::Command};

const MANIFEST_FILE: &str = "kcl.mod";

/// [`fill_pkg_maps_for_k_file`] will call `kpm metadata` to obtain the metadata
/// of all dependent packages of the kcl package where the current file is located,
/// and fill the relevant information of the external packages into compilation option [`LoadProgramOptions`].
pub(crate) fn fill_pkg_maps_for_k_file(
    k_file_path: PathBuf,
    opts: &mut LoadProgramOptions,
) -> Result<()> {
    // 1. find the kcl.mod dir for the kcl package contains 'k_file_path'.
    match lookup_the_nearest_file_dir(k_file_path, MANIFEST_FILE) {
        Some(mod_dir) => {
            // 2. get the module metadata.
            let metadata = fetch_metadata(mod_dir.canonicalize()?)?;
            // 3. fill the external packages local paths into compilation option [`LoadProgramOptions`].
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

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
/// [`Metadata`] is the metadata of the current KCL module,
/// currently only the mapping between the name and path of the external dependent package is included.
pub struct Metadata {
    pub packages: HashMap<String, Package>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// [`Package`] is a kcl package.
pub struct Package {
    /// Name as given in the `kcl.mod`
    pub name: String,
    /// Path containing the `kcl.mod`
    pub manifest_path: PathBuf,
}

impl Metadata {
    /// [`parse`] will parse the json string into [`Metadata`].
    fn parse(data: String) -> Result<Self> {
        let meta = serde_json::from_str(data.as_ref())?;
        Ok(meta)
    }
}

/// [`fetch_metadata`] returns the KCL module metadata.
#[inline]
pub fn fetch_metadata(manifest_path: PathBuf) -> Result<Metadata> {
    use std::result::Result::Ok;
    match fetch_mod_metadata(manifest_path.clone()) {
        Ok(result) => Ok(result),
        Err(_) => fetch_kpm_metadata(manifest_path),
    }
}

/// [`fetch_kpm_metadata`] will call `kpm metadata` to obtain the metadata.
///
/// TODO: this function will be removed at kcl v0.8.0 for the command migration
/// `kpm -> kcl mod`.
pub(crate) fn fetch_kpm_metadata(manifest_path: PathBuf) -> Result<Metadata> {
    use std::result::Result::Ok;
    match Command::new(kpm())
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

/// [`fetch_mod_metadata`] will call `kcl mod metadata` to obtain the metadata.
pub(crate) fn fetch_mod_metadata(manifest_path: PathBuf) -> Result<Metadata> {
    use std::result::Result::Ok;
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

/// [`kpm`] will return the path for executable kpm binary.
pub fn kpm() -> PathBuf {
    get_path_for_executable("kpm")
}
