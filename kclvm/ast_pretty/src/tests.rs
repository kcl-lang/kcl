use std::path::{Path, PathBuf};

use super::print_ast_module;
use kclvm_parser::parse_file_force_errors;
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
