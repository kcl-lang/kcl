use std::path::{Path, PathBuf};

use super::{Config, print_ast_module, print_ast_module_with_config};
use kcl_parser::parse_file_force_errors;
use pretty_assertions::assert_eq;

const FILE_INPUT_SUFFIX: &str = ".input";
const FILE_OUTPUT_SUFFIX: &str = ".output";
const TEST_CASES: &[&str] = &[
    "arguments",
    "empty",
    "if_stmt",
    "import",
    "unary",
    "codelayout",
    "collection_if",
    "comment",
    "index_sign",
    "inline_trailing_comment",
    "joined_str",
    "lambda",
    "orelse",
    "quant",
    "rule",
    "str",
    "type_alias",
    "unification",
];

fn read_data(data_name: &str) -> (String, String) {
    let mut filename = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filename.push(
        Path::new("src")
            .join("test_data")
            .join(format!("{}{}", data_name, FILE_INPUT_SUFFIX))
            .display()
            .to_string(),
    );

    let module = parse_file_force_errors(filename.to_str().unwrap(), None);

    let mut filename_expect = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filename_expect.push(
        Path::new("src")
            .join("test_data")
            .join(format!("{}{}", data_name, FILE_OUTPUT_SUFFIX))
            .display()
            .to_string(),
    );
    (
        print_ast_module(&module.unwrap()),
        std::fs::read_to_string(filename_expect.to_str().unwrap()).unwrap(),
    )
}

#[test]
fn test_ast_printer() {
    for case in TEST_CASES {
        let (data_input, data_output) = read_data(case);

        #[cfg(target_os = "windows")]
        let data_output = data_output.replace("\r\n", "\n");

        assert_eq!(data_input, data_output, "Test failed on {}", case);
    }
}

/// Helper: a small fixture containing a nested schema that exercises indentation.
const NESTED_SRC: &str = "x = {\n    a: {\n        b: 1\n    }\n}\n";

#[test]
fn test_print_ast_module_default_equivalent() {
    // Calling print_ast_module must match calling print_ast_module_with_config
    // with Config::default() byte-for-byte.
    let module = parse_file_force_errors("nested.k", Some(NESTED_SRC.to_string())).unwrap();
    let a = print_ast_module(&module);
    let b = print_ast_module_with_config(&module, Config::default());
    assert_eq!(a, b);
}

#[test]
fn test_print_ast_module_two_space_indent() {
    let module = parse_file_force_errors("nested.k", Some(NESTED_SRC.to_string())).unwrap();
    let cfg = Config {
        indent_len: 2,
        ..Config::default()
    };
    let out = print_ast_module_with_config(&module, cfg);
    // Every nested key gets exactly 2 spaces per level.
    assert!(
        out.contains("  a: {\n    b: 1\n  }\n}"),
        "expected 2-space indent, got:\n{out}"
    );
}

#[test]
fn test_print_ast_module_tab_indent() {
    let module = parse_file_force_errors("nested.k", Some(NESTED_SRC.to_string())).unwrap();
    let cfg = Config {
        use_spaces: false,
        indent_len: 4,
        tab_len: 4,
        ..Config::default()
    };
    let out = print_ast_module_with_config(&module, cfg);
    // In tab mode the printer emits a single TAB per indent level, regardless
    // of `tab_len` or `indent_len`. The first nested level is one TAB and the
    // deeper nested level is two TABs.
    assert!(
        out.contains("\n\ta: {\n\t\tb: 1\n\t}\n}\n"),
        "expected one TAB per level, got:\n{out}"
    );
}

#[test]
fn test_print_ast_module_tab_len_does_not_affect_tab_mode_bytes() {
    // `tab_len` is a display hint per the EditorConfig spec. Even setting it
    // to 8 (the classic default) must not change the bytes emitted in tab mode.
    let module = parse_file_force_errors("nested.k", Some(NESTED_SRC.to_string())).unwrap();
    let cfg_small = Config {
        use_spaces: false,
        indent_len: 4,
        tab_len: 2,
        ..Config::default()
    };
    let cfg_large = Config {
        use_spaces: false,
        indent_len: 4,
        tab_len: 8,
        ..Config::default()
    };
    let small = print_ast_module_with_config(&module, cfg_small);
    let large = print_ast_module_with_config(&module, cfg_large);
    assert_eq!(small, large);
}
