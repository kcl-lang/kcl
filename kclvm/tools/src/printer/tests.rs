use super::*;
use kclvm_parser::parse_file;
use pretty_assertions::assert_eq;

const FILE_INPUT_SUFFIX: &str = ".input";
const FILE_OUTPUT_SUFFIX: &str = ".output";
const TEST_CASES: &[&'static str; 15] = &[
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
    "quant",
    "rule",
    "type_alias",
    "unification",
];

fn read_data(data_name: &str) -> (String, String) {
    let module = parse_file(
        &format!("./src/printer/test_data/{}{}", data_name, FILE_INPUT_SUFFIX),
        None,
    );

    (
        print_ast_module(&module.unwrap()),
        std::fs::read_to_string(&format!(
            "./src/printer/test_data/{}{}",
            data_name, FILE_OUTPUT_SUFFIX
        ))
        .unwrap(),
    )
}

#[test]
fn test_ast_printer() {
    for case in TEST_CASES {
        let (data_input, data_output) = read_data(case);
        assert_eq!(data_input, data_output, "Test failed on {}", case);
    }
}
