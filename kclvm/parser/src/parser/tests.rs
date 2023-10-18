use crate::parse_file;
use regex::Regex;

fn check_parsing_module(filename: &str, src: &str, expect: &str) {
    let m = crate::parse_file(filename, Some(src.to_string())).expect(filename);
    let actual = format!("{}\n", serde_json::ser::to_string(&m).unwrap());
    assert_eq!(actual.trim(), expect.trim());
}

#[test]
fn test_parse_file() {
    let filenames = vec![
        "testdata/assert-01.k",
        "testdata/assert-02.k",
        "testdata/assert-03.k",
        "testdata/assert-if-0.k",
        "testdata/assert-if-1.k",
        "testdata/assert-if-2.k",
        "testdata/assign-01.k",
        "testdata/config_expr-01.k",
        "testdata/config_expr-02.k",
        "testdata/config_expr-03.k",
        "testdata/config_expr-04.k",
        "testdata/import-01.k",
        "testdata/if-01.k",
        "testdata/if-02.k",
        "testdata/if-03.k",
        "testdata/type-01.k",
        "testdata/hello_win.k",
    ];
    for filename in filenames {
        let code = std::fs::read_to_string(filename).unwrap();
        let expect = std::fs::read_to_string(filename.to_string() + ".json").unwrap();
        check_parsing_module(
            filename.trim_start_matches("testdata/"),
            code.as_str(),
            expect.as_str(),
        );
    }
}

#[test]
fn test_parse_file_not_found() {
    match parse_file("The file path is invalid", None) {
        Ok(_) => {
            panic!("unreachable")
        }
        Err(err_msg) => {
            assert!(
                Regex::new(r"^Failed to load KCL file 'The file path is invalid'. Because.*")
                    .unwrap()
                    .is_match(&err_msg)
            );
        }
    }
}
