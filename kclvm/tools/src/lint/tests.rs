use super::lint_files;
use std::path::PathBuf;

#[test]
fn test_lint() {
    let (errors, warnings) = lint_files(&["./src/lint/test_data/lint.k"], None);
    let msgs = [
        "Importstmt should be placed at the top of the module",
        "Module 'a' is reimported multiple times",
        "Module 'import_test.a' imported but unused",
        "Module 'import_test.a' imported but unused",
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
        "try 'kcl mod add abc' to download the package not found",
        "find more package on 'https://artifacthub.io'",
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
