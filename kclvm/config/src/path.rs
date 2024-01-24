//! The file provides the mod relative path type.
//!
//! The mod relative path is a path that is relative to the root package path.
//! The root package is can be specified by the prefix `${<name>:KCL_MOD}`.
//! `<name>` is the name of the root package.
//! If `<name>` is omitted, the root package is the current package.
//!
//! # Examples
//!
//! `/usr/my_pkg` is the real path of the package `my_pkg`.
//! `${my_pkg:KCL_MOD}/sub/main.k` is a mod relative path.
//! The real path of `${my_pkg:KCL_MOD}/xxx/main.k` is `/usr/my_pkg/sub/main.k`.
use anyhow::Result;
use pcre2::bytes::Regex;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
/// [`ModRelativePath`] is a path that is relative to the root package path.
/// The root package is can be specified by the prefix `${<name>:KCL_MOD}`.
/// `<name>` is the name of the root package.
/// If `<name>` is omitted, the root package is the current package.
///
/// # Examples
///
/// `/usr/my_pkg` is the real path of the package `my_pkg`.
/// `${my_pkg:KCL_MOD}/sub/main.k` is a mod relative path.
/// The real path of `${my_pkg:KCL_MOD}/xxx/main.k` is `/usr/my_pkg/sub/main.k`.
pub struct ModRelativePath {
    path: String,
}

/// The regular expression to match the mod relative path preffix.
const RELATIVE_PATH_PREFFIX: &str = r#"\$\{((?P<name>[a-zA-Z0-9_-]+):)?KCL_MOD\}/"#;

/// The name of the root package.
const ROOT_PKG_NAME_FLAG: &str = "name";

impl From<String> for ModRelativePath {
    fn from(path: String) -> Self {
        ModRelativePath::new(path)
    }
}

impl ModRelativePath {
    /// [`new`] creates a new [`ModRelativePath`] instance.
    pub fn new(path: String) -> ModRelativePath {
        ModRelativePath { path }
    }

    /// [`get_path`] returns the clone string of path of the [`ModRelativePath`].
    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    /// [`is_relative_path`] returns true if the path is a mod relative path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kclvm_config::path::ModRelativePath;
    /// let path = ModRelativePath::new("${my_pkg:KCL_MOD}/src/path.rs".to_string());
    /// assert_eq!(path.is_relative_path().unwrap(), true);
    ///
    /// let path = ModRelativePath::new("${KCL_MOD}/src/path.rs".to_string());
    /// assert_eq!(path.is_relative_path().unwrap(), true);
    ///
    /// let path = ModRelativePath::new("/usr/${my_pkg:KCL_MOD}/src/path.rs".to_string());
    /// assert_eq!(path.is_relative_path().unwrap(), false);
    ///
    /// let path = ModRelativePath::new("/src/path.rs".to_string());
    /// assert_eq!(path.is_relative_path().unwrap(), false);
    /// ```
    pub fn is_relative_path(&self) -> Result<bool> {
        Ok(Regex::new(RELATIVE_PATH_PREFFIX)?
            .find(self.path.as_bytes())?
            .map_or(false, |mat| mat.start() == 0))
    }

    /// [`get_root_pkg_name`] returns the name of the root package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kclvm_config::path::ModRelativePath;
    /// let path = ModRelativePath::new("${my_pkg:KCL_MOD}/src/path.rs".to_string());
    /// assert_eq!(path.get_root_pkg_name().unwrap(), Some("my_pkg".to_string()));
    ///
    /// let path = ModRelativePath::new("${KCL_MOD}/src/path.rs".to_string());
    /// assert_eq!(path.get_root_pkg_name().unwrap(), None);
    ///
    /// let path = ModRelativePath::new("/src/path.rs".to_string());
    /// assert_eq!(path.get_root_pkg_name().unwrap(), None);
    /// ```
    pub fn get_root_pkg_name(&self) -> Result<Option<String>> {
        if !self.is_relative_path()? {
            return Ok(None);
        }

        Ok(Regex::new(RELATIVE_PATH_PREFFIX)?
            .captures(self.path.as_bytes())?
            .and_then(|caps| caps.name(ROOT_PKG_NAME_FLAG))
            .map(|mat| std::str::from_utf8(mat.as_bytes()).map(|s| s.to_string()))
            .transpose()?)
    }

    /// [`canonicalize_by_root_path`] returns the canonicalized path by the root path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kclvm_config::path::ModRelativePath;
    /// let path = ModRelativePath::new("${name:KCL_MOD}/src/path".to_string());
    /// #[cfg(target_os = "windows")]
    /// assert_eq!(path.canonicalize_by_root_path("/usr/my_pkg").unwrap(), "/usr/my_pkg\\src/path");
    /// #[cfg(not(target_os = "windows"))]
    /// assert_eq!(path.canonicalize_by_root_path("/usr/my_pkg").unwrap(), "/usr/my_pkg/src/path");
    ///
    /// let path = ModRelativePath::new("/src/path".to_string());
    /// assert_eq!(path.canonicalize_by_root_path("/usr/my_pkg").unwrap(), "/src/path");
    /// ```
    pub fn canonicalize_by_root_path(&self, root_path: &str) -> Result<String> {
        if !self.is_relative_path()? {
            return Ok(self.get_path());
        }

        Ok(Regex::new(RELATIVE_PATH_PREFFIX)?
            .captures(self.path.as_bytes())?
            .map_or_else(
                || self.get_path(),
                |caps| {
                    // Due to the path format is different between windows and linux,
                    // Can not use the replace method directly
                    // by 'replace(std::str::from_utf8(caps.get(0).unwrap().as_bytes()).unwrap(), root_path)'.
                    let sub_path = self.get_path().replace(
                        std::str::from_utf8(caps.get(0).unwrap().as_bytes()).unwrap(),
                        "",
                    );
                    let res = PathBuf::from(root_path)
                        .join(sub_path)
                        .display()
                        .to_string();

                    res
                },
            ))
    }
}

#[cfg(test)]
mod test_relative_path {
    use super::*;

    #[test]
    fn test_is_relative_path() {
        let path = ModRelativePath::new("${name:KCL_MOD}/src/path.rs".to_string());
        assert!(path.is_relative_path().unwrap());
        let path = ModRelativePath::new("${KCL_MOD}/src/path.rs".to_string());
        assert!(path.is_relative_path().unwrap());
        let path = ModRelativePath::new("/usr/${name:KCL_MOD}/src/path.rs".to_string());
        assert!(!path.is_relative_path().unwrap());
        let path = ModRelativePath::new("/src/path.rs".to_string());
        assert!(!path.is_relative_path().unwrap());
        let path = ModRelativePath::new("./src/path.rs".to_string());
        assert!(!path.is_relative_path().unwrap());
        let path = ModRelativePath::new("${K_MOD}/src/path.rs".to_string());
        assert!(!path.is_relative_path().unwrap());
        let path = ModRelativePath::new("${:KCL_MOD}/src/path.rs".to_string());
        assert!(!path.is_relative_path().unwrap());
    }

    #[test]
    fn test_get_root_pkg_name() {
        let path = ModRelativePath::new("${my_pkg:KCL_MOD}/src/path.rs".to_string());
        assert_eq!(
            path.get_root_pkg_name().unwrap(),
            Some("my_pkg".to_string())
        );

        let path = ModRelativePath::new("${KCL_MOD}/src/path.rs".to_string());
        assert_eq!(path.get_root_pkg_name().unwrap(), None);

        let path = ModRelativePath::new("/src/path.rs".to_string());
        assert_eq!(path.get_root_pkg_name().unwrap(), None);
    }

    #[test]
    fn test_canonicalize_by_root_path() {
        let path = ModRelativePath::new("${name:KCL_MOD}/src/path".to_string());
        #[cfg(target_os = "windows")]
        assert_eq!(
            path.canonicalize_by_root_path("C:\\usr\\my_pkg").unwrap(),
            "C:\\usr\\my_pkg\\src/path"
        );
        #[cfg(not(target_os = "windows"))]
        assert_eq!(
            path.canonicalize_by_root_path("/usr/my_pkg").unwrap(),
            "/usr/my_pkg/src/path"
        );
        let path = ModRelativePath::new("/src/path".to_string());
        assert_eq!(
            path.canonicalize_by_root_path("/usr/my_pkg").unwrap(),
            "/src/path"
        );
    }
}
