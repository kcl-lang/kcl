use super::*;
use kcl_parser::parse_file_force_errors;
use pretty_assertions::assert_eq;
use walkdir::WalkDir;

const FILE_INPUT_SUFFIX: &str = ".input";
const FILE_OUTPUT_SUFFIX: &str = ".golden";
const TEST_CASES: &[&str; 23] = &[
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
    "trailing_comment_collection",
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
    assert!(
        format(
            "./src/format/test_data/format_path_data/single_file.k",
            &FormatOptions::default()
        )
        .is_ok()
    );
}

#[test]
fn test_format_folder() {
    assert!(
        format(
            "./src/format/test_data/format_path_data/folder",
            &FormatOptions::default()
        )
        .is_ok()
    );
}

#[test]
fn test_format_with_stdout_option() {
    let opts = FormatOptions {
        is_stdout: true,
        recursively: false,
        omit_errors: false,
        ..Default::default()
    };
    let changed_files = format("./src/format/test_data/format_path_data/if.k", &opts).unwrap();
    assert_eq!(changed_files.len(), 1);
    let changed_files = format("./src/format/test_data/format_path_data/", &opts).unwrap();
    assert_eq!(changed_files.len(), 1);
    let opts = FormatOptions {
        is_stdout: true,
        recursively: true,
        omit_errors: false,
        ..Default::default()
    };
    let changed_files = format("./src/format/test_data/format_path_data/", &opts).unwrap();
    assert_eq!(changed_files.len(), 2);
}

#[test]
fn test_format_with_dry_run_option() {
    let opts = FormatOptions {
        dry_run: true,
        ..Default::default()
    };
    let before = std::fs::read_to_string("./src/format/test_data/format_path_data/if.k").unwrap();
    let changed_files = format("./src/format/test_data/format_path_data/if.k", &opts).unwrap();
    let after = std::fs::read_to_string("./src/format/test_data/format_path_data/if.k").unwrap();
    assert_eq!(changed_files.len(), 1);
    assert_eq!(
        before, after,
        "dry_run option should not modify input files"
    );
    let opts = FormatOptions {
        recursively: true,
        dry_run: true,
        ..Default::default()
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
        ..Default::default()
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

/// When the file contains a syntax error (here: `-` is not a valid character
/// in an import path), the parser's error recovery still produces a partial
/// AST, but the pretty-printer re-renders it into well-formed but wrong code.
/// The formatter must leave the source unchanged in that case so we don't
/// silently corrupt the user's file. See issue #1882.
#[test]
fn test_format_leaves_source_unchanged_on_parse_error() {
    let opts = FormatOptions {
        is_stdout: false,
        recursively: false,
        omit_errors: true,
        ..Default::default()
    };
    let src = "import gateway-api.v1 as gatewayApi_v1\n\n\nmyGateway = gatewayApi.Gateway {\n    spec = {}\n}\n";
    let (formatted, changed) = format_source("issue1882.k", src, &opts).unwrap();
    assert!(
        !changed,
        "format_source must report unchanged on syntax error"
    );
    assert_eq!(
        formatted, src,
        "format_source must leave the source unchanged on syntax error"
    );
}

/// Regression test for #2140: with `omit_errors: true`, the formatter must
/// only bail out on actual syntax errors. Semantic errors such as
/// `CannotFindModule` (e.g. an import that can't be resolved on disk)
/// don't corrupt the AST, and the pretty-printer can still re-render it.
#[test]
fn test_format_proceeds_with_semantic_errors() {
    let opts = FormatOptions {
        is_stdout: false,
        recursively: false,
        omit_errors: true,
        ..Default::default()
    };
    // `a` does not exist on disk, so the parser reports `CannotFindModule`,
    // but the source itself is syntactically valid KCL. The input is
    // intentionally mis-formatted (extra blank lines, no blank line before
    // `a=1`) so the formatter has something to do.
    let src = "import a\n\n\n\na=1\n";
    let (formatted, changed) = format_source("semantic_only.k", src, &opts).unwrap();
    assert!(
        changed,
        "format_source should still reformat when only semantic errors are present; got {formatted:?}"
    );
    assert_ne!(
        formatted, src,
        "format_source should still reformat when only semantic errors are present"
    );
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

#[test]
fn test_format_trailing_newlines() {
    let schema_end = "foo = \"bar\"\n\nschema Name:\n    first: str\n";
    let var_end = "schema Name:\n    first: str\n\nfoo = \"bar\"\n";

    let (schema_formatted, _) = format_source("test.k", schema_end, &Default::default()).unwrap();
    let (var_formatted, _) = format_source("test.k", var_end, &Default::default()).unwrap();

    // Count trailing newlines
    let schema_trailing = schema_formatted
        .chars()
        .rev()
        .take_while(|c| *c == '\n')
        .count();
    let var_trailing = var_formatted
        .chars()
        .rev()
        .take_while(|c| *c == '\n')
        .count();

    println!("Schema ending: {} trailing newlines", schema_trailing);
    println!("Schema output:\n{}", schema_formatted);
    println!("Var ending: {} trailing newlines", var_trailing);
    println!("Var output:\n{}", var_formatted);

    // Both should have exactly 1 trailing newline
    assert_eq!(
        schema_trailing, 1,
        "Schema ending should have 1 trailing newline, got {}",
        schema_trailing
    );
    assert_eq!(
        var_trailing, 1,
        "Var ending should have 1 trailing newline, got {}",
        var_trailing
    );
}

// ---------------------------------------------------------------------------
// .editorconfig end-to-end tests
// ---------------------------------------------------------------------------

mod editorconfig_e2e {
    use std::fs;
    use tempfile::TempDir;

    use super::*;

    /// Helper: write an `.editorconfig` with `root = true` so the walk
    /// cannot escape the temp directory.
    fn write_ec(dir: &std::path::Path, body: &str) {
        let mut full = String::from("root = true\n\n");
        full.push_str(body);
        fs::write(dir.join(".editorconfig"), full).unwrap();
    }

    #[test]
    fn editorconfig_two_space_indent() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_style = space\nindent_size = 2\n");
        let k = dir.path().join("sample.k");
        fs::write(&k, "x = {\n    a: {\n        b: 1\n    }\n}\n").unwrap();

        let src = std::fs::read_to_string(&k).unwrap();
        let (formatted, _) =
            format_source(k.to_str().unwrap(), &src, &FormatOptions::default()).unwrap();

        // Two-space indent from .editorconfig — every nested key gets 2 spaces.
        assert!(
            formatted.contains("  a: {\n    b: 1\n  }\n}"),
            "expected 2-space indent from .editorconfig, got:\n{formatted}"
        );
    }

    #[test]
    fn editorconfig_tab_indent_style() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_style = tab\n");
        let k = dir.path().join("sample.k");
        fs::write(&k, "x = {\n    a: {\n        b: 1\n    }\n}\n").unwrap();

        let src = std::fs::read_to_string(&k).unwrap();
        let (formatted, _) =
            format_source(k.to_str().unwrap(), &src, &FormatOptions::default()).unwrap();

        // Tab mode emits one TAB per indent level regardless of indent_len.
        assert!(
            formatted.contains("\n\ta: {\n\t\tb: 1\n\t}\n}\n"),
            "expected TAB indent from .editorconfig, got:\n{formatted}"
        );
    }

    #[test]
    fn format_options_override_editorconfig() {
        let dir = TempDir::new().unwrap();
        write_ec(dir.path(), "[*.k]\nindent_style = space\nindent_size = 8\n");
        let k = dir.path().join("sample.k");
        fs::write(&k, "x = {\n    a: 1\n}\n").unwrap();

        let src = std::fs::read_to_string(&k).unwrap();
        // Override: ask for 2-space indent regardless of what the file's
        // .editorconfig says.
        let opts = FormatOptions {
            indent_width: Some(2),
            ..FormatOptions::default()
        };
        let (formatted, _) = format_source(k.to_str().unwrap(), &src, &opts).unwrap();

        assert!(
            formatted.contains("  a: 1\n}\n"),
            "expected explicit 2-space override, got:\n{formatted}"
        );
    }
}
