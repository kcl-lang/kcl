use std::path::PathBuf;

use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_error::Diagnostic;
use kclvm_error::ErrorKind::InvalidSyntax;
use kclvm_error::ErrorKind::TypeError;
use kclvm_error::{DiagnosticId, Position as KCLPos};
use kclvm_sema::builtin::MATH_FUNCTION_NAMES;
use kclvm_sema::builtin::STRING_MEMBER_FUNCTIONS;
use kclvm_sema::resolver::scope::ProgramScope;
use lsp_types::CompletionResponse;
use lsp_types::DocumentSymbol;
use lsp_types::DocumentSymbolResponse;
use lsp_types::MarkedString;
use lsp_types::SymbolKind;
use lsp_types::{Position, Range, TextDocumentContentChangeEvent};

use crate::document_symbol::document_symbol;
use crate::hover::hover;
use crate::{
    completion::{completion, into_completion_items},
    goto_def::goto_definition,
    util::{apply_document_changes, parse_param_and_compile, Param},
};

fn compile_test_file(testfile: &str) -> (String, Program, ProgramScope, IndexSet<Diagnostic>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path;
    test_file.push(testfile);

    let file = test_file.to_str().unwrap().to_string();

    let (program, prog_scope, diags) =
        parse_param_and_compile(Param { file: file.clone() }, None).unwrap();
    (file, program, prog_scope, diags)
}

#[test]
fn diagnostics_test() {
    let (_, _, _, diags) = compile_test_file("src/test_data/diagnostics.k");

    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].code, Some(DiagnosticId::Error(InvalidSyntax)));
    assert_eq!(diags[1].code, Some(DiagnosticId::Error(TypeError)));
}

#[test]
fn goto_import_pkg_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");
    let pos = KCLPos {
        filename: file,
        line: 1,
        column: Some(10),
    };

    let res = goto_definition(&program, &pos, &prog_scope);
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
}

#[test]
fn goto_import_file_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test goto import file: import .pkg.schema_def
    let pos = KCLPos {
        filename: file,
        line: 2,
        column: Some(10),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, expected_path.to_str().unwrap())
        }
        _ => {
            unreachable!("test error")
        }
    }
}

#[test]
fn goto_schema_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test goto schema definition: p = pkg.Person
    let pos = KCLPos {
        filename: file,
        line: 4,
        column: Some(11),
    };
    let res = goto_definition(&program, &pos, &prog_scope);

    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, expected_path.to_str().unwrap());

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: 0, // zero-based
                character: 0,
            };

            let expected_end = Position {
                line: 5, // zero-based
                character: 13,
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
fn goto_identifier_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test goto identifier definition: p1 = p
    let pos = KCLPos {
        filename: file.to_string(),
        line: 9,
        column: Some(6),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
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
fn goto_schema_attr_ty_def_test() {
    // test goto schema attr type definition: p1: pkg.Person
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    let pos = KCLPos {
        filename: file,
        line: 12,
        column: Some(15),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, expected_path.to_str().unwrap());

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: 0, // zero-based
                character: 0,
            };

            let expected_end = Position {
                line: 5, // zero-based
                character: 13,
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
fn goto_schema_attr_ty_def_test1() {
    // test goto schema attr type definition: p2: [pkg.Person]
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    let pos = KCLPos {
        filename: file,
        line: 13,
        column: Some(15),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, expected_path.to_str().unwrap());

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: 0, // zero-based
                character: 0,
            };

            let expected_end = Position {
                line: 5, // zero-based
                character: 13,
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
fn goto_schema_attr_ty_def_test3() {
    // test goto schema attr type definition: p3: {str: pkg.Person}
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    let pos = KCLPos {
        filename: file,
        line: 14,
        column: Some(22),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, expected_path.to_str().unwrap());

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: 0, // zero-based
                character: 0,
            };

            let expected_end = Position {
                line: 5, // zero-based
                character: 13,
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
fn goto_schema_attr_ty_def_test4() {
    // test goto schema attr type definition(Person): p4: pkg.Person | pkg.Person1
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    let pos = KCLPos {
        filename: file,
        line: 15,
        column: Some(17),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, expected_path.to_str().unwrap());

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: 0, // zero-based
                character: 0,
            };

            let expected_end = Position {
                line: 5, // zero-based
                character: 13,
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
fn goto_schema_attr_ty_def_test5() {
    // test goto schema attr type definition(Person1): p4: pkg.Person | pkg.Person1
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def1.k");

    let pos = KCLPos {
        filename: file,
        line: 15,
        column: Some(28),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
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
        _ => unreachable!("test error"),
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

#[test]
fn completion_test() {
    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/completion_test/dot/completion.k");

    // test completion for schema attr
    let pos = KCLPos {
        filename: file.to_owned(),
        line: 12,
        column: Some(7),
    };

    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
    let mut items = IndexSet::new();
    items.insert("name".to_string());
    items.insert("age".to_string());

    let expect: CompletionResponse = into_completion_items(&items).into();

    assert_eq!(got, expect);
    items.clear();

    let pos = KCLPos {
        filename: file.to_owned(),
        line: 14,
        column: Some(12),
    };

    // test completion for str builtin function
    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
    let binding = STRING_MEMBER_FUNCTIONS;
    for k in binding.keys() {
        items.insert(format!("{}{}", k, "()"));
    }
    let expect: CompletionResponse = into_completion_items(&items).into();

    assert_eq!(got, expect);
    items.clear();

    // test completion for import pkg path
    let pos = KCLPos {
        filename: file.to_owned(),
        line: 1,
        column: Some(12),
    };
    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
    items.insert("file1".to_string());
    items.insert("file2".to_string());
    items.insert("subpkg".to_string());

    let expect: CompletionResponse = into_completion_items(&items).into();
    assert_eq!(got, expect);
    items.clear();

    // test completion for import pkg' schema
    let pos = KCLPos {
        filename: file.to_owned(),
        line: 16,
        column: Some(12),
    };

    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
    items.insert("Person1".to_string());
    let expect: CompletionResponse = into_completion_items(&items).into();
    assert_eq!(got, expect);
    items.clear();

    let pos = KCLPos {
        filename: file.to_owned(),
        line: 19,
        column: Some(5),
    };
    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
    items.extend(MATH_FUNCTION_NAMES.iter().map(|s| s.to_string()));
    let expect: CompletionResponse = into_completion_items(&items).into();
    assert_eq!(got, expect);
    items.clear();

    let pos = KCLPos {
        filename: file,
        line: 22,
        column: Some(19),
    };
    assert!(completion(Some('.'), &program, &pos, &prog_scope).is_none());

    items.clear();
}

#[test]
fn schema_doc_hover_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test hover of schema doc: p = pkg.Person
    let pos = KCLPos {
        filename: file.clone(),
        line: 4,
        column: Some(11),
    };
    let got = hover(&program, &pos, &prog_scope).unwrap();
    match got.contents {
        lsp_types::HoverContents::Array(vec) => {
            if let MarkedString::String(s) = vec[0].clone() {
                assert_eq!(s, "Person");
            }
            if let MarkedString::String(s) = vec[1].clone() {
                assert_eq!(s, "\"\"\"\n    hover doc test \n    \"\"\"");
            }
        }
        _ => unreachable!("test error"),
    }
    let pos = KCLPos {
        filename: file,
        line: 5,
        column: Some(7),
    };
    let got = hover(&program, &pos, &prog_scope).unwrap();
    match got.contents {
        lsp_types::HoverContents::Scalar(marked_string) => {
            if let MarkedString::String(s) = marked_string {
                assert_eq!(s, "str");
            }
        }
        _ => unreachable!("test error"),
    }
}

#[allow(deprecated)]
fn build_document_symbol(
    name: &str,
    kind: SymbolKind,
    range: ((u32, u32), (u32, u32)),
    child: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
    let range: Range = Range {
        start: Position {
            line: range.0 .0,
            character: range.0 .1,
        },
        end: Position {
            line: range.1 .0,
            character: range.1 .1,
        },
    };
    DocumentSymbol {
        name: name.to_string(),
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children: child,
    }
}

#[test]
fn document_symbol_test() {
    let (file, program, prog_scope, _) = compile_test_file("src/test_data/document_symbol.k");

    let res = document_symbol(file.as_str(), &program, &prog_scope).unwrap();
    let mut expect = vec![];
    expect.push(build_document_symbol(
        "p",
        SymbolKind::VARIABLE,
        ((3, 0), (3, 1)),
        None,
    ));
    expect.push(build_document_symbol(
        "Person4",
        SymbolKind::STRUCT,
        ((0, 7), (1, 13)),
        Some(vec![build_document_symbol(
            "name",
            SymbolKind::PROPERTY,
            ((1, 4), (1, 8)),
            None,
        )]),
    ));
    let expect = DocumentSymbolResponse::Nested(expect);
    assert_eq!(res, expect)
}
