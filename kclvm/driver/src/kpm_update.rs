use anyhow::{bail, Result};
use std::{path::PathBuf, process::Command};
use crate::kpm_metadata::get_path_for_executable;


const MANIFEST_FILE: &str = "kcl.mod";


pub(crate) fn update_kcl_module(manifest_path: PathBuf) -> Result<()> {match lookup_the_nearest_file_dir(manifest_path.clone(), MANIFEST_FILE) {
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
pub fn kcl() -> PathBuf {
    get_path_for_executable("kcl")
}
pub fn kpm() -> PathBuf {
    get_path_for_executable("kpm")
}

pub(crate) fn lookup_the_nearest_file_dir(
    from: PathBuf,
    the_nearest_file: &str,
) -> Option<PathBuf> {
    let mut current_dir = from;

    loop {
        let found_path = current_dir.join(the_nearest_file);
        if found_path.is_file() {
            return current_dir.canonicalize().ok();
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => return None,
        }
    }
}