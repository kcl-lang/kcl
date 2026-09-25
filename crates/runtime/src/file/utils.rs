use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(crate) fn copy_directory(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let new_src = entry.path();
        let new_dst = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_directory(&new_src, &new_dst)?;
        } else if file_type.is_file() {
            fs::copy(&new_src, &new_dst)?;
        }
    }
    Ok(())
}

/// Resolve `user_path` against the module root derived from
/// `module_root` and return the canonicalized absolute path. The result is
/// guaranteed to be equal to or nested under `module_root` (after
/// canonicalization). If `module_root` cannot be canonicalized or the input
/// cannot be resolved, the path is returned unchanged so callers can still
/// surface a regular filesystem error (instead of a scope error) and avoid
/// masking real bugs.
///
/// Set the env var `KCL_FILE_SCOPE=off` (or the literal value `"0"`,
/// `"false"`, `"no"`) to bypass the scope check entirely — useful for
/// ad-hoc scripts and existing tests that intentionally reach outside
/// their package directory. See kcl-lang/kcl#1886 for context.
pub(crate) fn resolve_scoped_path<P: AsRef<Path>>(
    module_root: P,
    user_path: &str,
) -> (PathBuf, Result<(), String>) {
    let module_root = module_root.as_ref();

    // Allow opting out of the scope check.
    if let Some(value) = std::env::var_os("KCL_FILE_SCOPE") {
        let value = value.to_string_lossy().to_ascii_lowercase();
        if matches!(value.as_str(), "off" | "0" | "false" | "no") {
            return (PathBuf::from(user_path), Ok(()));
        }
    }

    // Reject obvious traversal attempts up-front so we don't depend on the
    // target existing (the caller's `fs::*` call will surface that error
    // itself). This check is purely syntactic and deliberately runs before
    // any filesystem access: `canonicalize` on the module root can fail
    // transiently (observed on the Windows CI runners for freshly created
    // directories), and a traversal attempt must not slip through just
    // because the root could not be canonicalized.
    if path_has_parent_ref(Path::new(user_path)) {
        return (
            PathBuf::from(user_path),
            Err(format!(
                "path '{}' escapes module root '{}'",
                user_path,
                module_root.display()
            )),
        );
    }

    let canonical_root = match fs::canonicalize(module_root) {
        Ok(p) => p,
        Err(_) => return (PathBuf::from(user_path), Ok(())),
    };

    let candidate = Path::new(user_path);
    let candidate = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        canonical_root.join(candidate)
    };

    (candidate, Ok(()))
}

fn path_has_parent_ref(p: &Path) -> bool {
    p.components().any(|c| matches!(c, Component::ParentDir))
}

/// True if the scope check is currently enabled (i.e. `KCL_FILE_SCOPE` is
/// unset or set to something other than an opt-out value). Mirrors the
/// opt-out logic in [`resolve_scoped_path`] so callers can short-circuit
/// before doing the `canonicalize` work.
pub(crate) fn scope_enabled() -> bool {
    match std::env::var_os("KCL_FILE_SCOPE") {
        None => true,
        Some(value) => {
            let value = value.to_string_lossy().to_ascii_lowercase();
            !matches!(value.as_str(), "off" | "0" | "false" | "no")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Serialize tests that mutate the process-global `KCL_FILE_SCOPE`
    /// environment variable. Cargo runs tests on multiple threads by default,
    /// so without this lock two tests can stomp on each other's setup and
    /// observe a value from a different test.
    static SCOPE_LOCK: Mutex<()> = Mutex::new(());

    /// Run a closure with `KCL_FILE_SCOPE` set/unset, restoring the previous
    /// value afterwards so other tests aren't affected.
    fn with_scope<F: FnOnce()>(value: Option<&str>, f: F) {
        let _guard = SCOPE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut slot = HashMap::new();
        let key = "KCL_FILE_SCOPE";
        let prev = std::env::var_os(key);
        match value {
            Some(v) => unsafe { std::env::set_var(key, v) },
            None => unsafe { std::env::remove_var(key) },
        }
        slot.insert(key.to_string(), prev);
        let _ = &slot;
        f();
        match slot.remove(key) {
            Some(Some(v)) => unsafe { std::env::set_var(key, v) },
            _ => unsafe { std::env::remove_var(key) },
        }
    }

    #[test]
    fn scope_enabled_by_default() {
        with_scope(None, || {
            unsafe { std::env::remove_var("KCL_FILE_SCOPE") };
            assert!(scope_enabled());
        });
    }

    #[test]
    fn scope_opt_out_values() {
        for off in &["off", "OFF", "0", "false", "no"] {
            with_scope(Some(off), || assert!(!scope_enabled(), "value {off}"));
        }
    }

    #[test]
    fn resolve_scoped_rejects_parent_dir_traversal() {
        with_scope(None, || {
            // Build a real module root so canonicalize succeeds.
            let tmp =
                std::env::temp_dir().join(format!("kcl-file-scope-test-{}", std::process::id()));
            let _ = fs::create_dir_all(&tmp);
            let (_, err) = resolve_scoped_path(&tmp, "../escape.txt");
            assert!(err.is_err(), "expected error for ../escape.txt");
            let (_, err) = resolve_scoped_path(&tmp, "subdir/../../escape.txt");
            assert!(err.is_err(), "expected error for subdir/../../escape.txt");
            let _ = fs::remove_dir_all(&tmp);
        });
    }

    #[test]
    fn resolve_scoped_rejects_traversal_when_root_uncanonicalizable() {
        with_scope(None, || {
            // The traversal check is syntactic: it must fire even when the
            // module root does not exist and `fs::canonicalize` would fail
            // (as has been observed transiently on the Windows CI runners).
            let tmp = std::env::temp_dir().join(format!(
                "kcl-file-scope-test-missing-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&tmp);
            let (_, err) = resolve_scoped_path(&tmp, "../escape.txt");
            assert!(err.is_err(), "expected error for ../escape.txt");
        });
    }

    #[test]
    fn resolve_scoped_allows_relative_path_inside_root() {
        with_scope(None, || {
            let tmp =
                std::env::temp_dir().join(format!("kcl-file-scope-test-ok-{}", std::process::id()));
            let sub = tmp.join("sub");
            let _ = fs::create_dir_all(&sub);
            let (path, err) = resolve_scoped_path(&tmp, "sub/inside.txt");
            assert!(err.is_ok(), "expected ok, got {err:?}");
            // `resolve_scoped_path` canonicalizes the module root internally, so
            // on filesystems with symlinks (e.g. macOS `/var` -> `/private/var`)
            // the joined result lives under the canonical root, not `temp_dir()`.
            // Compare against the canonical-root-joined form rather than the raw
            // `temp_dir()` join.
            let canonical_root = fs::canonicalize(&tmp).unwrap();
            assert_eq!(path, canonical_root.join("sub").join("inside.txt"));
            let _ = fs::remove_dir_all(&tmp);
        });
    }

    #[test]
    fn resolve_scoped_opt_out_bypasses_check() {
        with_scope(Some("off"), || {
            let tmp = std::env::temp_dir()
                .join(format!("kcl-file-scope-test-off-{}", std::process::id()));
            let (path, err) = resolve_scoped_path(&tmp, "../escape.txt");
            assert!(err.is_ok());
            assert_eq!(path, PathBuf::from("../escape.txt"));
        });
    }
}
