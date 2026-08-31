use kcl_ast::ast;
use kcl_parser::{ParseSessionRef, parse_file_with_global_session};
use kcl_tools::testing::{TEST_FILE_SUFFIX, TEST_SUITE_PREFIX};
use lsp_types::{CodeLens, Command, Position, Range};
use serde_json::json;
use std::path::Path;

/// The client-side command a test lens invokes. Clients that want the
/// "run test" action must register a handler for it, e.g. `vscode-kcl`.
pub const RUN_TEST_COMMAND: &str = "kcl.runTest";

/// Returns a `run test` lens for every test case in a KCL test suite file.
///
/// Test suites are `*_test.k` files and test cases are top level
/// `test_xxx = lambda { ... }` assignments, matching the discovery rules of
/// the `kcl test` tool in [`kcl_tools::testing`].
pub fn code_lens(file: &str, src: &str) -> Option<Vec<CodeLens>> {
    if !is_test_suite_file(file) {
        return None;
    }
    // A lenient parse: the module is still returned when the user is
    // mid-edit and the file does not parse cleanly.
    let module =
        parse_file_with_global_session(ParseSessionRef::default(), file, Some(src.to_string()))
            .ok()?;
    // `url_from_path` can fail on Windows when `file` is a Unix-style
    // absolute path (no drive letter, e.g. test fixtures). We don't
    // want that to mask the parser result — the URL only feeds the
    // command arguments, so fall back to an empty string instead of
    // dropping the whole lens list.
    let uri = crate::to_lsp::url_from_path(file)
        .map(|u| u.to_string())
        .unwrap_or_default();

    let mut lens = vec![];
    for stmt in &module.body {
        if let ast::Stmt::Assign(assign_stmt) = &stmt.node
            && let ast::Expr::Lambda(_) = &assign_stmt.value.node
        {
            for target in &assign_stmt.targets {
                let name = target.node.get_name();
                if !name.starts_with(TEST_SUITE_PREFIX) {
                    continue;
                }
                lens.push(CodeLens {
                    range: node_range(&target.node.name),
                    command: Some(Command {
                        title: "▶ run test".to_string(),
                        command: RUN_TEST_COMMAND.to_string(),
                        arguments: Some(vec![json!(uri.as_str()), json!(name)]),
                    }),
                    data: None,
                });
            }
        }
    }
    Some(lens)
}

/// Whether the file is a KCL test suite file, e.g. `path/to/pkg/func_test.k`.
#[inline]
fn is_test_suite_file(file: &str) -> bool {
    match Path::new(file).file_name().and_then(|name| name.to_str()) {
        Some(name) => !name.starts_with('_') && name.ends_with(TEST_FILE_SUFFIX),
        None => false,
    }
}

/// Convert an AST node span into an LSP range. AST lines are 1 based and
/// columns are 0 based, LSP lines and characters are both 0 based.
#[inline]
fn node_range<T>(node: &ast::Node<T>) -> Range {
    Range::new(
        Position::new(node.line.saturating_sub(1) as u32, node.column as u32),
        Position::new(
            node.end_line.saturating_sub(1) as u32,
            node.end_column as u32,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{RUN_TEST_COMMAND, code_lens};
    use std::path::PathBuf;

    fn test_file() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("test_data")
            .join("code_lens")
            .join("code_lens_test.k")
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn code_lens_test() {
        let file = test_file();
        let src = std::fs::read_to_string(&file).unwrap();
        let lens = code_lens(&file, &src).unwrap();

        assert_eq!(lens.len(), 2);
        for (lens, name) in lens.iter().zip(["test_func_0", "test_func_1"]) {
            let command = lens.command.as_ref().unwrap();
            assert_eq!(command.command, RUN_TEST_COMMAND);
            assert_eq!(command.title, "▶ run test");
            let args = command.arguments.as_ref().unwrap();
            assert_eq!(args.len(), 2);
            assert!(args[0].as_str().unwrap().starts_with("file://"));
            assert_eq!(args[1].as_str().unwrap(), name);
            // The lens range must be single line and cover the test name.
            assert_eq!(lens.range.start.line, lens.range.end.line);
            assert_eq!(lens.range.start.character, 0);
            assert_eq!(lens.range.end.character, name.len() as u32);
        }
        // Lenses are emitted in source order.
        assert!(lens[0].range.start.line < lens[1].range.start.line);
    }

    #[test]
    fn code_lens_not_test_file() {
        assert!(code_lens("/a/b/main.k", "test_func = lambda {\n    assert True\n}\n").is_none());
    }

    #[test]
    fn code_lens_underscore_prefixed_file() {
        assert!(
            code_lens(
                "/a/b/_helper_test.k",
                "test_func = lambda {\n    assert True\n}\n"
            )
            .is_none()
        );
    }

    #[test]
    fn code_lens_invalid_syntax() {
        // Requesting lenses on a half written file must not panic.
        assert!(code_lens("/a/b/invalid_test.k", "test_func = lambda {").is_some());
    }
}
