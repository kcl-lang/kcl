use anyhow::Result;
use kclvm_config::modfile::KCL_FILE_SUFFIX;
use std::path::Path;
use walkdir::WalkDir;

pub mod loader;
#[cfg(test)]
mod tests;

/// Get kcl files from path.
pub(crate) fn get_kcl_files<P: AsRef<Path>>(path: P, recursively: bool) -> Result<Vec<String>> {
    let mut files = vec![];
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let file = path.to_str().unwrap();
            if file.ends_with(KCL_FILE_SUFFIX) && (recursively || entry.depth() == 1) {
                files.push(file.to_string())
            }
        }
    }
    Ok(files)
}
