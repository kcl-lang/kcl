//! This file primarily offers utils for working with kcl package paths.

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

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

/// Convert a dotted pkgpath (e.g. `a.b.c`) into a relative path (e.g.
/// `a/b/c` on Unix, `a\b\c` on Windows) by appending each segment to an
/// empty path buffer with the platform-correct separator.
pub fn pkgpath_to_rel_path_buf(pkgpath: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for s in pkgpath.split('.') {
        path.push(s);
    }
    path
}

/// Append each segment of a dotted pkgpath (e.g. `a.b.c`) to `root` with
/// the platform-correct separator (e.g. `root/a/b/c` on Unix,
/// `root\a\b\c` on Windows).
pub fn pkgpath_to_path_buf(root: &Path, pkgpath: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for s in pkgpath.split('.') {
        path.push(s);
    }
    path
}

/// Strip the external package name prefix from a dotted pkgpath (e.g.
/// `my_pkg.sub.dir` → `sub.dir`) and convert the remainder to a relative
/// filesystem path with the platform-correct separator (e.g. `sub/dir`).
///
/// Empty input yields an empty path. Callers should pass valid external
/// pkgpaths (no leading dot) and can push the package's manifest root on
/// top to get a full filesystem path.
pub fn external_pkgpath_to_rel_path_buf(pkgpath: &str) -> PathBuf {
    let sub_path = rm_external_pkg_name(pkgpath).unwrap_or_default();
    pkgpath_to_rel_path_buf(&sub_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::MAIN_SEPARATOR_STR;

    #[test]
    fn test_pkgpath_to_rel_path_buf() {
        let sep = MAIN_SEPARATOR_STR;
        assert_eq!(
            pkgpath_to_rel_path_buf("a.b.c"),
            PathBuf::from(["a", "b", "c"].join(sep))
        );
        assert_eq!(pkgpath_to_rel_path_buf("a"), PathBuf::from("a"));
        // An empty pkgpath yields an empty relative path.
        assert_eq!(pkgpath_to_rel_path_buf(""), PathBuf::new());
        // Leading/trailing dots produce empty segments, which `PathBuf::push`
        // skips without adding separators.
        assert_eq!(
            pkgpath_to_rel_path_buf(".a.b."),
            PathBuf::from(["a", "b"].join(sep))
        );
    }

    #[test]
    fn test_pkgpath_to_path_buf() {
        let sep = MAIN_SEPARATOR_STR;
        assert_eq!(
            pkgpath_to_path_buf(Path::new("root"), "a.b.c"),
            PathBuf::from(["root", "a", "b", "c"].join(sep))
        );
        assert_eq!(
            pkgpath_to_path_buf(Path::new("root"), "a"),
            PathBuf::from("root").join("a")
        );
        // An empty pkgpath leaves the root untouched.
        assert_eq!(
            pkgpath_to_path_buf(Path::new("root"), ""),
            PathBuf::from("root")
        );
        // An absolute root keeps its prefix.
        let abs = if cfg!(windows) { r"C:\root" } else { "/root" };
        assert_eq!(
            pkgpath_to_path_buf(Path::new(abs), "a.b"),
            Path::new(abs).join(["a", "b"].join(sep))
        );
    }

    #[test]
    fn test_external_pkgpath_to_rel_path_buf() {
        let sep = MAIN_SEPARATOR_STR;
        // External prefix is stripped; remaining dotted segments become
        // a relative path using the platform separator.
        assert_eq!(
            external_pkgpath_to_rel_path_buf("my_pkg.sub.dir"),
            PathBuf::from(["sub", "dir"].join(sep))
        );
        // Bare external name (no dotted suffix) yields an empty path.
        assert_eq!(external_pkgpath_to_rel_path_buf("my_pkg"), PathBuf::new());
        // Empty input also yields an empty path.
        assert_eq!(external_pkgpath_to_rel_path_buf(""), PathBuf::new());
        // A leading dot yields an empty first segment that
        // `parse_external_pkg_name` returns as Ok(""), so the strip is a
        // no-op and the rest of the dotted path is joined normally.
        // Callers should not pass non-external inputs.
        assert_eq!(
            external_pkgpath_to_rel_path_buf(".local.path"),
            PathBuf::from(["local", "path"].join(sep))
        );
    }
}
