//! Resolve a `kcl_ast_pretty::Config` for a file path by consulting
//! `.editorconfig` (EditorConfig, https://editorconfig.org/) when present.
//!
//! Precedence (highest first):
//!   1. Explicit `FormatOptions` fields (`use_spaces`, `indent_width`).
//!   2. Matching section in the closest `.editorconfig` walking up from the file.
//!   3. `kcl_ast_pretty::Config::default()` (4 spaces).
//!
//! This module does NOT modify `Config`'s `tab_len` semantics: when
//! `indent_style = tab`, the printer emits one TAB per indent level regardless
//! of `tab_len`. `tab_len` is only updated to reflect the EditorConfig
//! `tab_width` for callers that want it as a display hint.

use std::path::Path;

use ec4rs::property::{IndentSize, IndentStyle, TabWidth};
use kcl_ast_pretty::Config;

use super::FormatOptions;

/// Build the printer `Config` to use for `file_path`, honoring the explicit
/// `FormatOptions` overrides plus any matching `.editorconfig`.
pub(crate) fn resolve_config(file_path: &str, opts: &FormatOptions) -> Config {
    let mut cfg = Config::default();
    if should_discover(file_path) {
        apply_editorconfig(&mut cfg, Path::new(file_path));
    }
    if let Some(use_spaces) = opts.use_spaces {
        cfg.use_spaces = use_spaces;
    }
    if let Some(w) = opts.indent_width {
        if w > 0 {
            cfg.indent_len = w;
        }
    }
    cfg
}

/// Guard against `ec4rs::properties_of("")` walking from CWD: only consult
/// `.editorconfig` when the path actually points to a real file on disk.
fn should_discover(file_path: &str) -> bool {
    !file_path.is_empty() && Path::new(file_path).is_file()
}

fn apply_editorconfig(cfg: &mut Config, path: &Path) {
    // properties_of returns Err on malformed/IO issues — silently fall back.
    let Ok(mut props) = ec4rs::properties_of(path) else {
        return;
    };
    // Per the EditorConfig spec, this fills in tab_width / indent_size
    // defaults from each other when one is unset.
    props.use_fallbacks();

    if let Ok(style) = props.get::<IndentStyle>() {
        cfg.use_spaces = matches!(style, IndentStyle::Spaces);
    }

    // indent_size takes priority over tab_width.
    match props.get::<IndentSize>() {
        Ok(IndentSize::Value(n)) if n > 0 => {
            cfg.indent_len = n;
        }
        Ok(IndentSize::UseTabWidth) => {
            if let Ok(TabWidth::Value(n)) = props.get::<TabWidth>() {
                if n > 0 {
                    cfg.indent_len = n;
                    cfg.tab_len = n;
                }
            }
        }
        _ => {}
    }

    // tab_width is a *display* hint. Update only when the printer is in
    // tab mode — in space mode `tab_len` is unused (see `fill`).
    if !cfg.use_spaces {
        if let Ok(TabWidth::Value(n)) = props.get::<TabWidth>() {
            if n > 0 {
                cfg.tab_len = n;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn opts() -> FormatOptions {
        FormatOptions::default()
    }

    /// Write an .editorconfig to `dir/.editorconfig` with `root = true` so the
    /// walk cannot escape the temp directory and contaminate other tests.
    fn write_ec(dir: &Path, body: &str) {
        let mut full = String::from("root = true\n\n");
        full.push_str(body);
        fs::write(dir.join(".editorconfig"), full).unwrap();
    }

    /// Create a fresh `.k` file at `dir/path` with `contents`.
    fn write_k(dir: &Path, path: &str, contents: &str) -> std::path::PathBuf {
        let p = dir.join(path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, contents).unwrap();
        p
    }

    #[test]
    fn no_editorconfig_uses_defaults() {
        let dir = TempDir::new().unwrap();
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert_eq!(
            cfg,
            Config::default(),
            "without .editorconfig we must use defaults"
        );
    }

    #[test]
    fn space_indent_size_two() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_style = space\nindent_size = 2\n");
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert!(cfg.use_spaces);
        assert_eq!(cfg.indent_len, 2);
    }

    #[test]
    fn tab_indent_style() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_style = tab\n");
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert!(!cfg.use_spaces, "tab indent_style must flip use_spaces");
    }

    #[test]
    fn indent_size_overrides_tab_width() {
        let dir = TempDir::new().unwrap();
        write_ec(
            dir.path(),
            "[*.k]\nindent_style = space\nindent_size = 4\ntab_width = 8\n",
        );
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert_eq!(cfg.indent_len, 4, "indent_size wins over tab_width");
    }

    #[test]
    fn indent_size_tab_falls_back_to_tab_width() {
        let dir = TempDir::new().unwrap();
        write_ec(
            dir.path(),
            "[*.k]\nindent_style = space\nindent_size = tab\ntab_width = 6\n",
        );
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert_eq!(
            cfg.indent_len, 6,
            "indent_size = tab must fall back to tab_width"
        );
    }

    #[test]
    fn parent_editorconfig_applies_to_nested_file() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_size = 2\n");
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let k = write_k(&nested, "deep.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert_eq!(cfg.indent_len, 2);
    }

    #[test]
    fn nearest_editorconfig_wins() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_size = 2\n");
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        write_ec(&sub, "[*.k]\nindent_size = 8\n");
        let k = write_k(&sub, "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert_eq!(cfg.indent_len, 8, "the closer .editorconfig must win");
    }

    #[test]
    fn root_true_stops_parent_lookup() {
        // Parent dir has indent_size = 2, child dir declares root = true and
        // indent_size = 6. The walk must stop at the child.
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_size = 2\n");
        let child = dir.path().join("child");
        fs::create_dir(&child).unwrap();
        write_ec(&child, "[*.k]\nindent_size = 6\n");
        let k = write_k(&child, "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert_eq!(cfg.indent_len, 6);
    }

    #[test]
    fn format_options_overrides_editorconfig() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_size = 2\n");
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let mut o = FormatOptions::default();
        o.indent_width = Some(8);
        let cfg = resolve_config(k.to_str().unwrap(), &o);
        assert_eq!(cfg.indent_len, 8, "explicit FormatOptions must win");
    }

    #[test]
    fn use_spaces_override_flips_style() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_style = tab\nindent_size = 4\n");
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let mut o = FormatOptions::default();
        o.use_spaces = Some(true);
        let cfg = resolve_config(k.to_str().unwrap(), &o);
        assert!(
            cfg.use_spaces,
            "explicit use_spaces must override .editorconfig"
        );
    }

    #[test]
    fn non_matching_section_is_ignored() {
        let dir = TempDir::new().unwrap();
        // .editorconfig exists, but only has a [*.py] section. The KCL file
        // must still get defaults.
        write_ec(dir.path(), "[*.py]\nindent_size = 2\n");
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn empty_path_skips_discovery() {
        // Guard: ec4rs::properties_of("") would walk from CWD and silently
        // inherit any .editorconfig above the server's working directory.
        let cfg = resolve_config("", &opts());
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn nonexistent_path_skips_discovery() {
        let dir = TempDir::new().unwrap();
        // No file is created — the path must not cause a walk from CWD.
        let bogus = dir.path().join("does_not_exist.k");
        let cfg = resolve_config(bogus.to_str().unwrap(), &opts());
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn malformed_editorconfig_is_ignored_without_panic() {
        let dir = TempDir::new().unwrap();
        // Not a valid EditorConfig file at all.
        fs::write(
            dir.path().join(".editorconfig"),
            "this is not valid !!! editorconfig content [[[",
        )
        .unwrap();
        let k = write_k(dir.path(), "a.k", "x = 1\n");
        let cfg = resolve_config(k.to_str().unwrap(), &opts());
        // We don't care exactly which defaults survive — only that we don't
        // panic and don't crash the formatter.
        assert_eq!(cfg.indent_len, Config::default().indent_len);
    }
}
