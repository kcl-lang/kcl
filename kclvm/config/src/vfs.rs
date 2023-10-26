extern crate pathdiff;

use std::path::Path;

pub fn is_abs_pkgpath(pkgpath: &str) -> bool {
    if pkgpath.is_empty() {
        return false;
    }
    if pkgpath.starts_with('.') {
        return false;
    }
    if std::path::Path::new(pkgpath).is_absolute() {
        return false;
    }
    if pkgpath.contains("..") {
        return false;
    }
    if pkgpath.contains(char::is_whitespace) {
        return false;
    }

    true
}

pub fn is_rel_pkgpath(pkgpath: &str) -> bool {
    let pkgpath = pkgpath.trim();
    pkgpath.starts_with('.')
}

pub fn fix_import_path(root: &str, filepath: &str, import_path: &str) -> String {
    // relpath: import .sub
    // FixImportPath(root, "path/to/app/file.k", ".sub")        => path.to.app.sub
    // FixImportPath(root, "path/to/app/file.k", "..sub")       => path.to.sub
    // FixImportPath(root, "path/to/app/file.k", "...sub")      => path.sub
    // FixImportPath(root, "path/to/app/file.k", "....sub")     => sub
    // FixImportPath(root, "path/to/app/file.k", ".....sub")    => ""
    //
    // abspath: import path.to.sub
    // FixImportPath(root, "path/to/app/file.k", "path.to.sub") => path.to.sub

    if !import_path.starts_with('.') {
        return import_path.to_string();
    }

    // Filepath to pkgpath
    let pkgpath = {
        let base = Path::new(&root);
        let dirpath = std::path::Path::new(&filepath).parent().unwrap();

        let pkgpath = if let Some(x) = pathdiff::diff_paths(dirpath, base) {
            x.to_str().unwrap().to_string()
        } else {
            dirpath.to_str().unwrap().to_string()
        };

        let pkgpath = pkgpath.replace(['/', '\\'], ".");
        pkgpath.trim_end_matches('.').to_string()
    };

    let mut leading_dot_count = import_path.len();
    for (i, c) in import_path.chars().enumerate() {
        if c != '.' {
            leading_dot_count = i;
            break;
        }
    }

    // The pkgpath is the current root path
    if pkgpath.is_empty() {
        if leading_dot_count <= 1 {
            return import_path.trim_matches('.').to_string();
        } else {
            return "".to_string();
        }
    }

    if leading_dot_count == 1 {
        return pkgpath + import_path;
    }

    let ss = pkgpath.split('.').collect::<Vec<&str>>();

    if (leading_dot_count - 1) < ss.len() {
        let prefix = ss[..(ss.len() - leading_dot_count + 1)].join(".");
        let suffix = import_path[leading_dot_count..].to_string();

        return format!("{}.{}", prefix, suffix);
    }

    if leading_dot_count - 1 == ss.len() {
        return import_path[leading_dot_count..].to_string();
    }

    "".to_string()
}

#[test]
fn test_fix_import_path() {
    #[cfg(not(target_os = "windows"))]
    let root = "/home/konfig";
    #[cfg(target_os = "windows")]
    let root = r#"c:\home\konfig"#;

    let s = fix_import_path(root, "path/to/app/file.k", ".sub");
    assert_eq!(s, "path.to.app.sub");

    let s = fix_import_path(root, "path/to/app/file.k", "..sub");
    assert_eq!(s, "path.to.sub");

    let s = fix_import_path(root, "path/to/app/file.k", "...sub");
    assert_eq!(s, "path.sub");

    let s = fix_import_path(root, "path/to/app/file.k", "....sub");
    assert_eq!(s, "sub");

    let s = fix_import_path(root, "path/to/app/file.k", ".....sub");
    assert_eq!(s, "");
}
