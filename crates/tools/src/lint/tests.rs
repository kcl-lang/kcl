use super::lint_files;
use std::path::PathBuf;

#[test]
fn test_lint() {
    let (errors, warnings) = lint_files(&["./src/lint/test_data/lint.k"], None);
    let msgs = [
        "The import stmt should be placed at the top of the module",
        "Module 'a' is reimported multiple times",
        "Module 'a' imported but unused",
        "Module 'a' imported but unused",
        "Module 'abc' imported but unused",
    ];
    assert_eq!(warnings.len(), msgs.len());
    for (diag, m) in warnings.iter().zip(msgs.iter()) {
        assert_eq!(diag.messages[0].message, m.to_string());
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src");
    path.push("lint");
    path.push("test_data");
    path.push("abc");

    let msgs = [
        "pkgpath abc not found in the program",
        "try 'kcl mod add abc' to download the missing package",
        "browse more packages at 'https://artifacthub.io'",
        &format!("Cannot find the module abc from {}", path.to_str().unwrap()),
    ];
    assert_eq!(
        errors.len(),
        msgs.len(),
        "{:?}",
        errors
            .iter()
            .map(|e| e.messages[0].message.clone())
            .collect::<Vec<String>>()
    );
    for (diag, m) in errors.iter().zip(msgs.iter()) {
        assert_eq!(diag.messages[0].message, m.to_string());
    }
}

#[test]
fn test_unused_check_for_each_file() {
    let (errs, warnings) = lint_files(
        &[
            "./src/lint/test_data/unused_check_for_each_file/a.k",
            "./src/lint/test_data/unused_check_for_each_file/b.k",
        ],
        None,
    );
    assert_eq!(errs.len(), 0);
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0].messages[0].message,
        "Module 'math' imported but unused".to_string()
    );
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src");
    path.push("lint");
    path.push("test_data");
    path.push("unused_check_for_each_file");
    path.push("a.k");
    assert_eq!(
        warnings[0].messages[0].range.0.filename,
        path.to_str().unwrap().to_string()
    );
}

#[test]
fn test_lint_all_packages() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("src");
    root.push("lint");
    root.push("test_data");
    root.push("lint_all");
    let pattern = format!("{}/...", root.to_str().unwrap());
    let (errors, warnings) = lint_files(&[pattern.as_str()], None);
    // Every package is linted on its own, so `S` defined both in the root
    // package and in `pkg_dup` is not a redefinition, and `pkg/sub/orphan.k`,
    // which no entry file imports, is still checked.
    assert_eq!(
        errors.len(),
        0,
        "{:?}",
        errors
            .iter()
            .map(|e| e.messages[0].message.clone())
            .collect::<Vec<String>>()
    );
    let msgs = [
        ("Module 'math' imported but unused", "lint_all/main.k"),
        (
            "Module 'math' imported but unused",
            "lint_all/pkg/sub/orphan.k",
        ),
    ];
    assert_eq!(warnings.len(), msgs.len());
    for (diag, (m, file)) in warnings.iter().zip(msgs.iter()) {
        assert_eq!(diag.messages[0].message, m.to_string());
        // Compare path components, the separator differs between platforms.
        assert!(std::path::Path::new(&diag.messages[0].range.0.filename).ends_with(file));
    }
}

#[test]
fn test_lint_dir_checks_root_package_only() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("src");
    root.push("lint");
    root.push("test_data");
    root.push("lint_all");
    let (errors, warnings) = lint_files(&[root.to_str().unwrap()], None);
    // A plain directory keeps its existing meaning: only the root package
    // (the files directly in it) takes part in the lint.
    assert_eq!(errors.len(), 0);
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0].messages[0].message,
        "Module 'math' imported but unused".to_string()
    );
    assert!(
        std::path::Path::new(&warnings[0].messages[0].range.0.filename)
            .ends_with("lint_all/main.k")
    );
}
