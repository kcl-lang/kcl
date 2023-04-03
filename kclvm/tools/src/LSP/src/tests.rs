use std::path::PathBuf;

use indexmap::IndexSet;
use kclvm_error::ErrorKind::InvalidSyntax;
use kclvm_error::ErrorKind::TypeError;
use kclvm_error::{DiagnosticId, Position as KCLPos};
use lsp_types::{Position, Range, TextDocumentContentChangeEvent};

use crate::{
    goto_def::goto_definition,
    util::{apply_document_changes, parse_param_and_compile, Param},
};

#[test]
fn diagnostics_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path.clone();
    test_file.push("src/test_data/diagnostics.k");
    let file = test_file.to_str().unwrap();

    let (_, _, diags) = parse_param_and_compile(
        Param {
            file: file.to_string(),
        },
        None,
    )
    .unwrap();
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].code, Some(DiagnosticId::Error(InvalidSyntax)));
    assert_eq!(diags[1].code, Some(DiagnosticId::Error(TypeError)));
}

#[test]
fn goto_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path.clone();
    test_file.push("src/test_data/goto_def_test/goto_def.k");

    let file = test_file.to_str().unwrap();

    let (program, prog_scope, _) = parse_param_and_compile(
        Param {
            file: file.to_string(),
        },
        None,
    )
    .unwrap();

    let pos = KCLPos {
        filename: file.to_owned(),
        line: 1,
        column: Some(10),
    };

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

#[test]
fn test_apply_document_changes() {
    macro_rules! change {
        [$($sl:expr, $sc:expr; $el:expr, $ec:expr => $text:expr),+] => {
            vec![$(TextDocumentContentChangeEvent {
                range: Some(Range {
                    start: Position { line: $sl, character: $sc },
                    end: Position { line: $el, character: $ec },
                }),
                range_length: None,
                text: String::from($text),
            }),+]
        };
    }

    let mut text = String::new();
    apply_document_changes(&mut text, vec![]);
    assert_eq!(text, "");

    // Test if full updates work (without a range)
    apply_document_changes(
        &mut text,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: String::from("the"),
        }],
    );

    assert_eq!(text, "the");
    apply_document_changes(&mut text, change![0, 3; 0, 3 => " quick"]);
    assert_eq!(text, "the quick");

    apply_document_changes(&mut text, change![0, 0; 0, 4 => "", 0, 5; 0, 5 => " foxes"]);
    assert_eq!(text, "quick foxes");

    apply_document_changes(&mut text, change![0, 11; 0, 11 => "\ndream"]);
    assert_eq!(text, "quick foxes\ndream");

    apply_document_changes(&mut text, change![1, 0; 1, 0 => "have "]);
    assert_eq!(text, "quick foxes\nhave dream");

    apply_document_changes(
        &mut text,
        change![0, 0; 0, 0 => "the ", 1, 4; 1, 4 => " quiet", 1, 16; 1, 16 => "s\n"],
    );
    assert_eq!(text, "the quick foxes\nhave quiet dreams\n");

    apply_document_changes(
        &mut text,
        change![0, 15; 0, 15 => "\n", 2, 17; 2, 17 => "\n"],
    );
    assert_eq!(text, "the quick foxes\n\nhave quiet dreams\n\n");

    apply_document_changes(
        &mut text,
        change![1, 0; 1, 0 => "DREAM", 2, 0; 2, 0 => "they ", 3, 0; 3, 0 => "DON'T THEY?"],
    );
    assert_eq!(
        text,
        "the quick foxes\nDREAM\nthey have quiet dreams\nDON'T THEY?\n"
    );

    apply_document_changes(&mut text, change![0, 10; 1, 5 => "", 2, 0; 2, 12 => ""]);
    assert_eq!(text, "the quick \nthey have quiet dreams\n");

    text = String::from("❤️");
    apply_document_changes(&mut text, change![0, 0; 0, 0 => "a"]);
    assert_eq!(text, "a❤️");

    // todo: Non-ASCII char
    // text = String::from("a\nb");
    // apply_document_changes(&mut text, change![0, 1; 1, 0 => "\nțc", 0, 1; 1, 1 => "d"]);
    // assert_eq!(text, "adcb");

    // text = String::from("a\nb");
    // apply_document_changes(&mut text, change![0, 1; 1, 0 => "ț\nc", 0, 2; 0, 2 => "c"]);
    // assert_eq!(text, "ațc\ncb");
}
