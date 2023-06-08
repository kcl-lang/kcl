use anyhow::{bail, Ok, Result};
use kclvm_parser::LoadProgramOptions;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, iter, path::PathBuf, process::Command};

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
            // 2. call `kpm metadata`.
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

    return Ok(());
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
/// [`Metadata`] is the metadata of the current KCL package,
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

/// [`fetch_metadata`] will call `kpm metadata` to obtain the metadata.
pub fn fetch_metadata(manifest_path: PathBuf) -> Result<Metadata> {
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
            return Some(current_dir.canonicalize().ok()?);
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => return None,
        }
    }
}

/// [`kpm`] will return the path for executable kpm binary.
pub fn kpm() -> PathBuf {
    get_path_for_executable("kpm")
}

/// [`get_path_for_executable`] will return the path for [`executable_name`].
pub fn get_path_for_executable(executable_name: &'static str) -> PathBuf {
    // The current implementation checks $PATH for an executable to use:
    // `<executable_name>`
    //  example: for kpm, this tries just `kpm`, which will succeed if `kpm` is on the $PATH

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
