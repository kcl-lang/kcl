//! This file primarily offers utils for working with file paths,
//! enabling them to be automatically formatted according to the OS.

use std::path::Path;

/// Util methods for file path prefixes
pub trait PathPrefix {
    /// In the Windows system, the file path returned by method [`canonicalize()`],
    /// in rust [`PathBuf`] or [`Path`], will include the '\\?\' character,
    /// which is prepared for the Windows API.
    ///
    /// Paths containing "\\?\" may sometimes result in the file being unable to be found.
    /// As such, [`adjust_canonicalization()`] is required to remove this '\\?\'.
    /// On non-Windows systems, this method does not make any modifications to the file path.
    ///
    /// For more information about "\\?\",
    /// see https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file#short-vs-long-names
    fn adjust_canonicalization(&self) -> String;
}

impl<P> PathPrefix for P
where
    P: AsRef<Path>,
{
    #[cfg(not(target_os = "windows"))]
    /// On non-Windows systems, this method does not make any modifications to the file path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use kclvm_utils::path::PathPrefix;
    ///
    /// let path = Path::new(".").canonicalize().unwrap();
    /// assert_eq!(
    ///     path.clone().adjust_canonicalization(),
    ///     path.display().to_string()
    /// );
    /// ```
    fn adjust_canonicalization(&self) -> String {
        self.as_ref().display().to_string()
    }

    #[cfg(target_os = "windows")]
    /// For kclvm on windows, the "\\?\ " will cause the obj file to not be found when linking by "cl.exe".
    ///
    /// Slicing this path directly is not a good solution,
    /// we will find a more fluent way to solve this problem in the future. @zongz
    /// Note: On windows systems, a file path that is too long may cause "cl.exe" to crash.
    /// For more information, see doc in trait [`PathPrefix`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use kclvm_utils::path::PathPrefix;
    ///
    /// let path = Path::new(".").canonicalize().unwrap();
    /// assert!(path.display().to_string().contains("\\\\?\\"));
    /// assert!(!path.adjust_canonicalization().contains("\\\\?\\"));
    /// ```
    fn adjust_canonicalization(&self) -> String {
        const VERBATIM_PREFIX: &str = r#"\\?\"#;
        let p = self.as_ref().display().to_string();
        if p.starts_with(VERBATIM_PREFIX) {
            p[VERBATIM_PREFIX.len()..].to_string()
        } else {
            p
        }
    }
}

#[test]
#[cfg(target_os = "windows")]
fn test_adjust_canonicalization() {
    let path = Path::new(".").canonicalize().unwrap();
    assert!(path.display().to_string().contains("\\\\?\\"));
    assert!(!path.adjust_canonicalization().contains("\\\\?\\"));
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_adjust_canonicalization1() {
    let path = Path::new(".").canonicalize().unwrap();
    assert_eq!(
        path.clone().adjust_canonicalization(),
        path.display().to_string()
    );
}

#[inline]
pub fn is_dir(path: &str) -> bool {
    std::path::Path::new(path).is_dir()
}

#[inline]
pub fn is_absolute(path: &str) -> bool {
    std::path::Path::new(path).is_absolute()
}

#[inline]
pub fn path_exist(path: &str) -> bool {
    std::path::Path::new(path).exists()
}
