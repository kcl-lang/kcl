use std::path::PathBuf;

use indexmap::IndexSet;
use kclvm_error::Position as KCLPos;
use lsp_types::{Position, Url};

use crate::{
    goto_def::goto_definition,
    util::{parse_param_and_compile, Param},
};

#[test]
fn goto_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path.clone();
    test_file.push("src/test_data/goto_def_test/goto_def.k");

    let file = test_file.to_str().unwrap();

    let (pos, program, prog_scope) = parse_param_and_compile(Param {
        url: Url::from_file_path(file).unwrap(),
        pos: Position {
            line: 0,
            character: 10,
        },
    });

    let res = goto_definition(program.clone(), pos, prog_scope.clone());
    let mut expeced_files = IndexSet::new();
    let path_str = path.to_str().unwrap();
    let test_files = [
        "src/test_data/goto_def_test/pkg/schema_def1.k",
        "src/test_data/goto_def_test/pkg/schema_def.k",
    ];
    expeced_files.insert(format!("{}/{}", path_str, test_files[0]));
    expeced_files.insert(format!("{}/{}", path_str, test_files[1]));

    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Array(arr) => {
            assert_eq!(expeced_files.len(), arr.len());
            for loc in arr {
                let got_path = loc.uri.path().to_string();
                assert!(expeced_files.contains(&got_path));
            }
        }
        _ => {
            unreachable!("test error")
        }
    }

    // test goto import file: import .pkg.schema_def
    let pos = KCLPos {
        filename: file.to_string(),
        line: 2,
        column: Some(10),
    };
    let res = goto_definition(program.clone(), pos, prog_scope.clone());
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            let mut expected_path = path.clone();
            expected_path.push(test_files[1]);
            assert_eq!(got_path, expected_path.to_str().unwrap())
        }
        _ => {
            unreachable!("test error")
        }
    }

    // test goto schema definition: p = pkg.Person {
    let pos = KCLPos {
        filename: file.to_string(),
        line: 4,
        column: Some(11),
    };
    let res = goto_definition(program.clone(), pos, prog_scope.clone());

    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            let mut expected_path = path.clone();
            expected_path.push(test_files[1]);
            assert_eq!(got_path, expected_path.to_str().unwrap());

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: 0, // zero-based
                character: 0,
            };

            let expected_end = Position {
                line: 2, // zero-based
                character: 13,
            };
            assert_eq!(got_start, expected_start);
            assert_eq!(got_end, expected_end);
        }
        _ => {
            unreachable!("test error")
        }
    }

    // test goto identifier definition: p1 = p
    let pos = KCLPos {
        filename: file.to_string(),
        line: 9,
        column: Some(6),
    };
    let res = goto_definition(program.clone(), pos, prog_scope.clone());
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, file);

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: 3, // zero-based
                character: 0,
            };

            let expected_end = Position {
                line: 3, // zero-based
                character: 1,
            };

            assert_eq!(got_start, expected_start);
            assert_eq!(got_end, expected_end);
        }
        _ => {
            unreachable!("test error")
        }
    }
}
