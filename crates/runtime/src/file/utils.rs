use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
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
/// `module_root` and return the canonicalized absolute path. Relative paths
/// are joined onto the canonical root; absolute paths are only accepted when
/// they lie inside the root (compared after canonicalization, so symlinks
/// pointing outside the root are rejected as well). Paths that would escape
/// — via `..` or by targeting a location outside the root — come back with
/// an error. If `module_root` itself cannot be canonicalized, the path is
/// returned unchanged with `Ok(())` so callers can still surface a regular
/// filesystem error (instead of a scope error) and avoid masking real bugs.
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

    // Reject obvious traversal attempts up-front so we don't depend on
    // the target existing (the caller's `fs::*` call will surface that error
    // itself). This check is purely syntactic and deliberately runs before
    // any filesystem access: `canonicalize` on the module root can fail
    // transiently (observed on the Windows CI runners for freshly created
    // directories), and a traversal attempt must not slip through just
    // because the root could not be canonicalized. It is also load-bearing
    // on Windows, where a `..` inside an otherwise verbatim path is treated
    // literally by the filesystem.
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

    // Absolute paths (and relative paths routed through a symlink that
    // points outside the root) must not escape either. Compare
    // canonicalized forms so symlink components are resolved before the
    // check; paths that don't exist yet are canonicalized through their
    // nearest existing ancestor so writes to new files still get checked.
    if !path_within_root(&candidate, &canonical_root) {
        return (
            candidate,
            Err(format!(
                "path '{}' escapes module root '{}'",
                user_path,
                canonical_root.display()
            )),
        );
    }

    (candidate, Ok(()))
}

/// Canonicalize `p`, falling back to the nearest existing ancestor for paths
/// that do not exist yet (writes to new files), re-appending the missing
/// trailing components. Returns `None` only when no ancestor can be
/// canonicalized at all (e.g. a path on a drive that doesn't exist).
fn canonicalize_existing(p: &Path) -> Option<PathBuf> {
    let mut missing = Vec::new();
    let mut cur = p;
    loop {
        if let Ok(c) = fs::canonicalize(cur) {
            let mut out = c;
            for comp in missing.iter().rev() {
                out.push(comp);
            }
            return Some(out);
        }
        missing.push(cur.file_name()?);
        cur = cur.parent()?;
    }
}

fn path_within_root(candidate: &Path, canonical_root: &Path) -> bool {
    match canonicalize_existing(candidate) {
        Some(cc) => cc.starts_with(canonical_root),
        // Nothing exists up to the filesystem root (e.g. a nonexistent
        // drive): the path cannot be inside the module root.
        None => false,
    }
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

/// Split a `filepath[:ref]` argument into its path and (optional) git
/// ref parts.
///
/// The supported grammar is `path:ref` where:
/// * `path` is any string accepted by `fs::read_to_string`, including
///   absolute paths and POSIX/Windows paths,
/// * `ref` is a git ref (branch, tag, commit SHA, or `HEAD`) — i.e. it
///   only contains `[A-Za-z0-9_./-]` characters and never begins with
///   `.`, `/`, or contains a `..` segment.
///
/// Windows drive letters (`C:foo`) are skipped when searching for the
/// separator so `C:\path\to\file.txt` is left untouched.
pub(crate) fn split_path_ref(input: &str) -> (&str, Option<&str>) {
    let bytes = input.as_bytes();

    // Skip a leading Windows drive letter ("C:" followed by a path
    // separator or end of string) so we don't split on it. Real drive
    // letters are always followed by `\` or `/`, never by an
    // alphanumeric — which keeps `x:HEAD` from being misread.
    let scan_start = if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        2
    } else {
        0
    };

    let Some(rel_idx) = input[scan_start..].rfind(':') else {
        return (input, None);
    };
    let sep_idx = scan_start + rel_idx;
    let rest = &input[sep_idx + 1..];

    // Empty ref ("path:") is treated as no ref — fall back to fs::read.
    if rest.is_empty() {
        return (input, None);
    }

    let first = rest.as_bytes()[0];
    if first == b'.' || first == b'/' || first == b'\\' {
        return (input, None);
    }

    // A ref is made of safe, ref-name characters only.
    if !rest
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-' | '/'))
    {
        return (input, None);
    }

    if rest.contains("..") {
        return (input, None);
    }

    (&input[..sep_idx], Some(rest))
}

/// Walk up from `start` looking for a directory that contains `.git`.
/// Returns the absolute path to the git working tree root, or `None`
/// if no git repository is found.
pub(crate) fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start.to_path_buf());
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Some(dir);
        }
        current = dir.parent().map(Path::to_path_buf);
    }
    None
}

/// Express the absolute `abs` path relative to `git_root`. Errors out if
/// the path escapes the repo. `original` is the user-supplied path, used
/// for error messages.
pub(crate) fn path_relative_to_git_root(
    abs: &Path,
    git_root: &Path,
    original: &str,
) -> Result<String, String> {
    let rel = abs
        .strip_prefix(git_root)
        .map_err(|_| format!("file '{}' is not inside the git repository", original))?;
    // `git show <ref>:<path>` wants forward slashes even on Windows.
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

/// Run `git show <ref>:<repo_rel_path>` inside the git working tree
/// `repo` and return its stdout as a `String`. Running with the repo as
/// the child cwd keeps the lookup anchored to the repository discovered
/// by `find_git_root` instead of the process-wide current working
/// directory (which tests and embedders may relocate). Any failure (git
/// missing, bad ref, bad path, non-zero exit) is reported as an error
/// string.
pub(crate) fn git_show(repo: &Path, repo_rel_path: &str, ref_name: &str) -> Result<String, String> {
    let spec = format!("{}:{}", ref_name, repo_rel_path);
    let output = Command::new("git")
        .args(["show", &spec])
        .current_dir(repo)
        .output()
        .map_err(|e| format!("failed to invoke git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let trimmed = stderr.trim();
        return Err(if trimmed.is_empty() {
            format!("git show '{}' exited with status {}", spec, output.status)
        } else {
            format!("git show '{}' failed: {}", spec, trimmed)
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Read the contents of the already scope-resolved `filepath` pinned to
/// the git revision `ref_name`, by walking up from the resolved path to
/// the enclosing git repository and running `git show <ref>:<path>`.
/// `filepath` is the original user-supplied string, used for error
/// messages.
pub(crate) fn read_git_ref(
    filepath: &str,
    resolved: &Path,
    ref_name: &str,
) -> Result<String, String> {
    let abs = absolutize(resolved)?;
    let git_root = find_git_root(abs.parent().unwrap_or(&abs)).ok_or_else(|| {
        format!(
            "file '{}' uses ':ref' but no git repository was found",
            filepath
        )
    })?;
    let rel = path_relative_to_git_root(&abs, &git_root, filepath)?;
    git_show(&git_root, &rel, ref_name)
}

/// Byte-oriented counterpart of [`read_git_ref`] for `readbase64`.
pub(crate) fn read_git_ref_bytes(
    filepath: &str,
    resolved: &Path,
    ref_name: &str,
) -> Result<Vec<u8>, String> {
    Ok(read_git_ref(filepath, resolved, ref_name)?.into_bytes())
}

/// Join a still-relative `resolved` path (scope check skipped or no
/// module root was found) onto the process working directory.
fn absolutize(resolved: &Path) -> Result<PathBuf, String> {
    if resolved.is_absolute() {
        Ok(resolved.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(resolved))
            .map_err(|e| format!("failed to get cwd: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

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
    fn resolve_scoped_rejects_absolute_path_outside_root() {
        with_scope(None, || {
            let tmp = std::env::temp_dir()
                .join(format!("kcl-file-scope-test-abs-{}", std::process::id()));
            let _ = fs::create_dir_all(&tmp);
            // An absolute path without any `..` still escapes the root.
            let outside = tmp.parent().unwrap().join(format!(
                "kcl-file-scope-abs-outside-{}.txt",
                std::process::id()
            ));
            let (_, err) = resolve_scoped_path(&tmp, outside.to_str().unwrap());
            assert!(err.is_err(), "expected error for {outside:?}");
            let _ = fs::remove_dir_all(&tmp);
        });
    }

    #[test]
    fn resolve_scoped_allows_absolute_path_inside_root() {
        with_scope(None, || {
            let tmp = std::env::temp_dir()
                .join(format!("kcl-file-scope-test-absok-{}", std::process::id()));
            let _ = fs::create_dir_all(&tmp);
            // Absolute paths that stay inside the root remain allowed.
            let inside = tmp.join("inside.txt");
            let (path, err) = resolve_scoped_path(&tmp, inside.to_str().unwrap());
            assert!(err.is_ok(), "expected ok, got {err:?}");
            assert_eq!(path, inside);
            let _ = fs::remove_dir_all(&tmp);
        });
    }

    #[cfg(unix)]
    #[test]
    fn resolve_scoped_rejects_symlink_escape() {
        with_scope(None, || {
            let tmp = std::env::temp_dir()
                .join(format!("kcl-file-scope-test-sym-{}", std::process::id()));
            let _ = fs::create_dir_all(&tmp);
            // A relative path that stays lexically inside the root but reaches
            // outside through a symlink must be rejected.
            let outside = tmp
                .parent()
                .unwrap()
                .join(format!("kcl-file-scope-sym-outside-{}", std::process::id()));
            let _ = fs::create_dir_all(&outside);
            std::os::unix::fs::symlink(&outside, tmp.join("link")).unwrap();
            let (_, err) = resolve_scoped_path(&tmp, "link/escape.txt");
            assert!(err.is_err(), "expected error for symlink escape");
            let _ = fs::remove_dir_all(&tmp);
            let _ = fs::remove_dir_all(&outside);
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

    #[test]
    fn split_path_ref_no_colon_is_just_path() {
        assert_eq!(split_path_ref("foo.txt"), ("foo.txt", None));
        assert_eq!(split_path_ref("/abs/path.txt"), ("/abs/path.txt", None));
        assert_eq!(split_path_ref(""), ("", None));
    }

    #[test]
    fn split_path_ref_with_ref() {
        assert_eq!(
            split_path_ref("assets/x.lua:07bf1e2ee"),
            ("assets/x.lua", Some("07bf1e2ee"))
        );
        assert_eq!(split_path_ref("x:HEAD"), ("x", Some("HEAD")));
        assert_eq!(split_path_ref("x:1.2"), ("x", Some("1.2")));
        assert_eq!(split_path_ref("x:my/branch"), ("x", Some("my/branch")));
    }

    #[test]
    fn split_path_ref_skips_windows_drive_letter() {
        assert_eq!(
            split_path_ref(r"C:\path\to\file.txt"),
            (r"C:\path\to\file.txt", None)
        );
        assert_eq!(
            split_path_ref(r"C:\path\file.txt:HEAD"),
            (r"C:\path\file.txt", Some("HEAD"))
        );
    }

    #[test]
    fn split_path_ref_rejects_unsafe_suffixes() {
        // Trailing colon with empty ref.
        assert_eq!(split_path_ref("foo:"), ("foo:", None));
        // Ref starting with a dot / slash / backslash.
        assert_eq!(split_path_ref("foo:.hidden"), ("foo:.hidden", None));
        assert_eq!(split_path_ref("foo:/abs"), ("foo:/abs", None));
        // Path traversal markers.
        assert_eq!(split_path_ref("foo:../etc"), ("foo:../etc", None));
        // Disallowed characters.
        assert_eq!(split_path_ref("foo:bad ref"), ("foo:bad ref", None));
        assert_eq!(split_path_ref("foo:bad$ref"), ("foo:bad$ref", None));
    }

    /// Serialises tests that mutate the process-wide current working
    /// directory. Cargo runs tests in parallel by default, so without
    /// this lock the `set_current_dir` calls here would race and
    /// produce flaky failures.
    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Build a two-commit git repo with `hello.txt` in a temp dir; the
    /// first commit is tagged `v1.0` and HEAD has the second revision.
    /// `tag` makes the directory name unique per caller.
    fn build_two_commit_repo(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("kcl_file_git_test_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("hello.txt");
        fs::write(&file, "v1\n").unwrap();

        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_AUTHOR_NAME", "test")
                .env("GIT_AUTHOR_EMAIL", "test@example.com")
                .env("GIT_COMMITTER_NAME", "test")
                .env("GIT_COMMITTER_EMAIL", "test@example.com")
                .output()
                .unwrap()
        };
        assert!(run(&["init", "-q"]).status.success());
        assert!(run(&["add", "hello.txt"]).status.success());
        assert!(run(&["commit", "-q", "-m", "v1"]).status.success());
        assert!(run(&["tag", "v1.0"]).status.success());

        fs::write(&file, "v2\n").unwrap();
        assert!(run(&["add", "hello.txt"]).status.success());
        assert!(run(&["commit", "-q", "-m", "v2"]).status.success());
        dir
    }

    #[test]
    fn read_git_ref_resolves_relative_to_cwd() {
        let _guard = cwd_lock().lock().unwrap();
        let dir = build_two_commit_repo("rel");
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let head_result = read_git_ref("hello.txt:HEAD", Path::new("hello.txt"), "HEAD");
        let pinned_result = read_git_ref("hello.txt:v1.0", Path::new("hello.txt"), "v1.0");
        std::env::set_current_dir(&cwd).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(head_result.unwrap(), "v2\n");
        assert_eq!(pinned_result.unwrap(), "v1\n");
    }

    #[test]
    fn read_git_ref_uses_resolved_path_regardless_of_cwd() {
        // With an absolute resolved path the enclosing repository is
        // found next to the file, not next to the process cwd.
        let dir = build_two_commit_repo("abs");
        let resolved = dir.join("hello.txt");
        let pinned = read_git_ref("hello.txt:v1.0", &resolved, "v1.0");
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(pinned.unwrap(), "v1\n");
    }

    #[test]
    fn read_git_ref_errors_outside_repo() {
        let _guard = cwd_lock().lock().unwrap();
        // A temp directory that is *not* a git repo should make the
        // ref-style call fail with a clear message.
        let dir = std::env::temp_dir().join(format!("kcl_file_nogit_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("hello.txt"), "v1").unwrap();

        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let result = read_git_ref("hello.txt:HEAD", Path::new("hello.txt"), "HEAD");
        std::env::set_current_dir(&cwd).unwrap();
        let _ = fs::remove_dir_all(&dir);

        let err = result.unwrap_err();
        assert!(
            err.contains("no git repository"),
            "expected 'no git repository' in error, got: {}",
            err
        );
    }

    #[test]
    fn path_relative_to_git_root_rejects_escape() {
        // A resolved path that does not live under the discovered git
        // root cannot be expressed repo-relatively. This is a pure
        // path computation, exercised directly for determinism.
        let err = path_relative_to_git_root(
            Path::new("/other/dir/file.txt"),
            Path::new("/repo"),
            "../dir/file.txt",
        )
        .unwrap_err();
        assert!(
            err.contains("is not inside the git repository"),
            "expected 'not inside the git repository' in error, got: {}",
            err
        );
    }
}
