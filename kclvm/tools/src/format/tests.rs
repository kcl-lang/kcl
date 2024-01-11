use super::*;
use kclvm_parser::parse_file_force_errors;
use pretty_assertions::assert_eq;
use walkdir::WalkDir;

const FILE_INPUT_SUFFIX: &str = ".input";
const FILE_OUTPUT_SUFFIX: &str = ".golden";
const TEST_CASES: &[&str; 22] = &[
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
    "import_only",
    "indent",
    "inline_comment",
    "lambda",
    "quant",
    "schema",
    "string",
    "type_alias",
    "unary",
    "union_types",
    "layout_import_stmt",
    "different_stmts_line_breaks",
    // "list_dict_schema_expr",
];

fn read_data(data_name: &str) -> (String, String) {
    let src = std::fs::read_to_string(format!(
        "./src/format/test_data/format_data/{}{}",
        data_name, FILE_INPUT_SUFFIX
    ))
    .unwrap();

    (
        format_source("", &src, &Default::default()).unwrap().0,
        std::fs::read_to_string(format!(
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
        #[cfg(target_os = "windows")]
        let data_output = data_output.replace("\r\n", "\n");
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
        omit_errors: false,
    };
    let changed_files = format("./src/format/test_data/format_path_data/if.k", &opts).unwrap();
    assert_eq!(changed_files.len(), 1);
    let changed_files = format("./src/format/test_data/format_path_data/", &opts).unwrap();
    assert_eq!(changed_files.len(), 1);
    let opts = FormatOptions {
        is_stdout: true,
        recursively: true,
        omit_errors: false,
    };
    let changed_files = format("./src/format/test_data/format_path_data/", &opts).unwrap();
    assert_eq!(changed_files.len(), 2);
}

#[test]
fn test_format_with_omit_error_option() {
    let opts = FormatOptions {
        is_stdout: false,
        recursively: false,
        omit_errors: true,
    };
    let cases = [
        (
            r#"x = {
a: {
b: 1
c: 2
}
d: 3
}       
"#,
            r#"x = {
    a: {
        b: 1
        c: 2
    }
    d: 3
}
"#,
        ),
        (
            r#"x = {
a: {
    b: 1
        c: 2
}
}
"#,
            r#"x = {
    a: {
        b: 1
        c: 2
    }
}
"#,
        ),
        (
            r#"x = {
    a: 1
   b: 2
  c: 3
}
"#,
            r#"x = {
    a: 1
    b: 2
    c: 3
}
"#,
        ),
        (
            r#"x = {
    a: 1
     b: 2
      c: 3
}
"#,
            r#"x = {
    a: 1
    b: 2
    c: 3
}
"#,
        ),
    ];
    for (code, expected_code) in cases {
        let (actual_code, _) = format_source("error_indent.k", code, &opts).unwrap();
        assert_eq!(actual_code, expected_code);
    }
}

#[test]
fn test_format_integration_konfig() -> Result<()> {
    let konfig_path = Path::new(".")
        .canonicalize()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test")
        .join("integration")
        .join("konfig");
    let files = get_files(konfig_path, true, true, ".k");
    for file in &files {
        // Skip test and hidden files.
        if file.ends_with("_test.k") || file.starts_with('_') {
            continue;
        }
        assert!(
            parse_file_force_errors(file, None).is_ok(),
            "file {} test format failed",
            file
        );
        let src = std::fs::read_to_string(file)?;
        let (formatted_src, _) = format_source("", &src, &Default::default())?;
        let parse_result = parse_file_force_errors("test.k", Some(formatted_src.clone() + "\n"));
        assert!(
            parse_result.is_ok(),
            "file {} test format failed, the formatted source is\n{}\n the parse error is\n{}",
            file,
            formatted_src,
            parse_result.err().unwrap(),
        );
    }
    Ok(())
}

/// Get kcl files from path.
fn get_files<P: AsRef<Path>>(
    path: P,
    recursively: bool,
    sorted: bool,
    suffix: &str,
) -> Vec<String> {
    let mut files = vec![];
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let file = path.to_str().unwrap();
            if file.ends_with(suffix) && (recursively || entry.depth() == 1) {
                files.push(file.to_string())
            }
        }
    }
    if sorted {
        files.sort();
    }
    files
}
