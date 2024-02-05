use anyhow::{bail, Result};
use std::{collections::HashMap, env, iter, path::PathBuf, process::Command};

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

/// Get the path for the KCL executable.
fn kcl() -> PathBuf {
    get_path_for_executable("kcl")
}

/// [`get_path_for_executable`] will return the path for [`executable_name`].
pub fn get_path_for_executable(executable_name: &'static str) -> PathBuf {
    // The current implementation checks $PATH for an executable to use:
    // `<executable_name>`
    //  example: for <executable_name>, this tries just <executable_name>, which will succeed if <executable_name> is on the $PATH

    if lookup_in_path(executable_name) {
        return executable_name.into();
    }

    executable_name.into()
}

/// [`lookup_in_path`] will search for an executable file [`exec`] in the environment variable ‘PATH’.
///  If found, return true, otherwise return false.
fn lookup_in_path(exec: &str) -> bool {
    let paths = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&paths)
        .map(|path| path.join(exec))
        .find_map(probe)
        .is_some()
}

/// [`probe`] check if the given path points to a file.
/// If it does, return [`Some`] of the path.
/// If not, check if adding the current operating system's executable file extension (if any) to the path points to a file.
/// If it does, return [`Some`] of the path with the extension added.
/// If neither, return [`None`].
fn probe(path: PathBuf) -> Option<PathBuf> {
    let with_extension = match env::consts::EXE_EXTENSION {
        "" => None,
        it => Some(path.with_extension(it)),
    };
    iter::once(path)
        .chain(with_extension)
        .find(|it| it.is_file())
}

/// [`lookup_the_nearest_file_dir`] will start from [`from`] and search for file [`the_nearest_file`] in the parent directories.
/// If found, it will return the [`Some`] of [`the_nearest_file`] file path. If not found, it will return [`None`]
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
