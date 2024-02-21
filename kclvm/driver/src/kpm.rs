use crate::kcl;
use crate::lookup_the_nearest_file_dir;
use anyhow::{bail, Result};
use kclvm_config::modfile::KCL_MOD_FILE;
use kclvm_parser::LoadProgramOptions;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, process::Command};

/// [`fill_pkg_maps_for_k_file`] will call `kpm metadata` to obtain the metadata
/// of all dependent packages of the kcl package where the current file is located,
/// and fill the relevant information of the external packages into compilation option [`LoadProgramOptions`].
pub(crate) fn fill_pkg_maps_for_k_file(
    k_file_path: PathBuf,
    opts: &mut LoadProgramOptions,
) -> Result<()> {
    // 1. find the kcl.mod dir for the kcl package contains 'k_file_path'.
    match lookup_the_nearest_file_dir(k_file_path, KCL_MOD_FILE) {
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

/// [`fetch_metadata`] will call `kcl mod metadata` to obtain the metadata.
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

/// [`update_dependencies`] will call `kcl mod update` to update the dependencies.
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
