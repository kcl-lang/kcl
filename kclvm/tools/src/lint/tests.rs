use super::lint_files;

#[test]
fn test_lint() {
    let (_, warnings) = lint_files(&["./src/lint/test_data/lint.k"], None);
    let msgs = [
        "Importstmt should be placed at the top of the module",
        "Module 'a' is reimported multiple times",
        "Module 'import_test.a' imported but unused",
    ];
    for (diag, m) in warnings.iter().zip(msgs.iter()) {
        assert_eq!(diag.messages[0].message, m.to_string());
    }
}
