use super::*;
use pretty_assertions::assert_eq;

const FILE_INPUT_SUFFIX: &str = ".input";
const FILE_OUTPUT_SUFFIX: &str = ".golden";
const TEST_CASES: &[&'static str; 18] = &[
    "assert",
    "check",
    "blankline",
    "breakline",
    "codelayout",
    "collection_if",
    "comment",
    "comp_for",
    "empty",
    "import",
    "indent",
    "inline_comment",
    "lambda",
    "quant",
    "schema",
    "string",
    "type_alias",
    "unary",
];

fn read_data(data_name: &str) -> (String, String) {
    let src = std::fs::read_to_string(&format!(
        "./src/format/test_data/format_data/{}{}",
        data_name, FILE_INPUT_SUFFIX
    ))
    .unwrap();

    (
        format_source(&src).unwrap().0,
        std::fs::read_to_string(&format!(
            "./src/format/test_data/format_data/{}{}",
            data_name, FILE_OUTPUT_SUFFIX
        ))
        .unwrap(),
    )
}

#[test]
fn test_format_source() {
    for case in TEST_CASES {
        let (data_input, data_output) = read_data(case);
        assert_eq!(data_input, data_output, "Test failed on {}", case);
    }
}

#[test]
fn test_format_single_file() {
    assert!(format(
        "./src/format/test_data/format_path_data/single_file.k",
        &FormatOptions::default()
    )
    .is_ok());
}

#[test]
fn test_format_folder() {
    assert!(format(
        "./src/format/test_data/format_path_data/folder",
        &FormatOptions::default()
    )
    .is_ok());
}

#[test]
fn test_format_with_stdout_option() {
    let opts = FormatOptions {
        is_stdout: true,
        recursively: false,
    };
    let changed_files = format("./src/format/test_data/format_path_data/if.k", &opts).unwrap();
    assert_eq!(changed_files.len(), 1);
    let changed_files = format("./src/format/test_data/format_path_data/", &opts).unwrap();
    assert_eq!(changed_files.len(), 1);
    let opts = FormatOptions {
        is_stdout: true,
        recursively: true,
    };
    let changed_files = format("./src/format/test_data/format_path_data/", &opts).unwrap();
    assert_eq!(changed_files.len(), 2);
}
