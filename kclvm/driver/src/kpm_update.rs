use anyhow::{bail, Result};
use std::{path::PathBuf, process::Command};
use crate::{probe, lookup_the_nearest_file_dir, kcl};

const MANIFEST_FILE: &str = "kcl.mod";

/// Update the KCL module.
///
/// This function calls `kcl mod update` to update the KCL module.
pub(crate) fn update_kcl_module(manifest_path: PathBuf) -> Result<()> {
    match lookup_the_nearest_file_dir(manifest_path.clone(), MANIFEST_FILE) {
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
        None => bail!("Manifest file '{}' not found in directory hierarchy", MANIFEST_FILE),
    }
}
