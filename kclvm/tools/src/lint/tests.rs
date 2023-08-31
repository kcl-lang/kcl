use super::lint_files;
use std::path::PathBuf;

#[test]
fn test_lint() {
    let (errors, warnings) = lint_files(&["./src/lint/test_data/lint.k"], None);
    let msgs = [
        "Importstmt should be placed at the top of the module",
        "Module 'a' is reimported multiple times",
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
        &format!("Cannot find the module abc from {}", path.to_str().unwrap()),
    ];
    assert_eq!(errors.len(), msgs.len());
    for (diag, m) in errors.iter().zip(msgs.iter()) {
        assert_eq!(diag.messages[0].message, m.to_string());
    }
}
