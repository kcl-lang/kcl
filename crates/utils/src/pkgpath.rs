//! This file primarily offers utils for working with kcl package paths.

use anyhow::{anyhow, Result};

/// Remove the external package name prefix from the current import absolute path.
///
/// # Note
/// [`rm_external_pkg_name`] just remove the prefix of the import path,
/// so it can't distinguish whether the current path is an internal package or an external package.
///
/// # Error
/// An error is returned if an empty string is passed in.
pub fn rm_external_pkg_name(pkgpath: &str) -> Result<String> {
    Ok(pkgpath
        .to_string()
        .trim_start_matches(parse_external_pkg_name(pkgpath)?.as_str())
        .to_string())
}

/// Remove the external package name prefix from the current import absolute path.
///
/// # Note
/// [`rm_external_pkg_name`] just remove the prefix of the import path,
/// so it can't distinguish whether the current path is an internal package or an external package.
///
/// # Error
/// An error is returned if an empty string is passed in.
pub fn parse_external_pkg_name(pkgpath: &str) -> Result<String> {
    let mut names = pkgpath.splitn(2, '.');
    match names.next() {
        Some(it) => Ok(it.to_string()),
        None => Err(anyhow!("Invalid external package name `{}`", pkgpath)),
    }
}
