use std::env;
use std::ops::Index;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_error::Diagnostic as KCLDiagnostic;
use kclvm_error::Position as KCLPos;
use kclvm_sema::builtin::MATH_FUNCTION_NAMES;
use kclvm_sema::builtin::STRING_MEMBER_FUNCTIONS;
use kclvm_sema::resolver::scope::ProgramScope;
use lsp_types::request::GotoTypeDefinitionResponse;
use lsp_types::CodeAction;
use lsp_types::CodeActionKind;
use lsp_types::CodeActionOrCommand;
use lsp_types::CompletionResponse;
use lsp_types::Diagnostic;
use lsp_types::DiagnosticRelatedInformation;
use lsp_types::DiagnosticSeverity;
use lsp_types::DocumentSymbol;
use lsp_types::DocumentSymbolResponse;
use lsp_types::Location;
use lsp_types::MarkedString;
use lsp_types::NumberOrString;
use lsp_types::SymbolKind;
use lsp_types::TextEdit;
use lsp_types::Url;
use lsp_types::WorkspaceEdit;
use lsp_types::{Position, Range, TextDocumentContentChangeEvent};
use parking_lot::RwLock;
use proc_macro_crate::bench_test;

use crate::completion::KCLCompletionItem;
use crate::document_symbol::document_symbol;
use crate::formatting::format;
use crate::from_lsp::{file_path_from_url, text_range};
use crate::hover::hover;
use crate::quick_fix::quick_fix;
use crate::to_lsp::kcl_diag_to_lsp_diags;
use crate::{
    completion::{completion, into_completion_items},
    goto_def::goto_definition,
    util::{apply_document_changes, parse_param_and_compile, Param},
};

fn compile_test_file(testfile: &str) -> (String, Program, ProgramScope, IndexSet<KCLDiagnostic>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path;
    test_file.push(testfile);

    let file = test_file.to_str().unwrap().to_string();

    let (program, prog_scope, diags) = parse_param_and_compile(
        Param { file: file.clone() },
        Some(Arc::new(RwLock::new(Default::default()))),
    )
    .unwrap();
    (file, program, prog_scope, diags)
}

fn compare_goto_res(res: Option<GotoTypeDefinitionResponse>, pos: (&String, u32, u32, u32, u32)) {
    match res.unwrap() {
        lsp_types::GotoDefinitionResponse::Scalar(loc) => {
            let got_path = loc.uri.path();
            assert_eq!(got_path, pos.0);

            let (got_start, got_end) = (loc.range.start, loc.range.end);

            let expected_start = Position {
                line: pos.1, // zero-based
                character: pos.2,
            };

            let expected_end = Position {
                line: pos.3, // zero-based
                character: pos.4,
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
#[bench_test]
fn diagnostics_test() {
    fn build_lsp_diag(
        pos: (u32, u32, u32, u32),
        message: String,
        severity: Option<DiagnosticSeverity>,
        related_info: Vec<(String, (u32, u32, u32, u32), String)>,
        code: Option<NumberOrString>,
    ) -> Diagnostic {
        let related_information = if related_info.is_empty() {
            None
        } else {
            Some(
                related_info
                    .iter()
                    .map(|(file, pos, msg)| DiagnosticRelatedInformation {
                        location: Location {
                            uri: Url::from_file_path(file).unwrap(),
                            range: Range {
                                start: Position {
                                    line: pos.0,
                                    character: pos.1,
                                },
                                end: Position {
                                    line: pos.2,
                                    character: pos.3,
                                },
                            },
                        },
                        message: msg.clone(),
                    })
                    .collect(),
            )
        };
        Diagnostic {
            range: lsp_types::Range {
                start: Position {
                    line: pos.0,
                    character: pos.1,
                },
                end: Position {
                    line: pos.2,
                    character: pos.3,
                },
            },
            severity,
            code,
            code_description: None,
            source: None,
            message,
            related_information,
            tags: None,
            data: None,
        }
    }

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path.clone();
    test_file.push("src/test_data/diagnostics.k");
    let file = test_file.to_str().unwrap();

    let (_, _, diags) = parse_param_and_compile(
        Param {
            file: file.to_string(),
        },
        Some(Arc::new(RwLock::new(Default::default()))),
    )
    .unwrap();

    let diagnostics = diags
        .iter()
        .flat_map(|diag| kcl_diag_to_lsp_diags(diag, file))
        .collect::<Vec<Diagnostic>>();

    let expected_diags: Vec<Diagnostic> = vec![
        build_lsp_diag(
            (1, 4, 1, 4),
            "expected one of [\"identifier\", \"literal\", \"(\", \"[\", \"{\"] got newline"
                .to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("InvalidSyntax".to_string())),
        ),
        build_lsp_diag(
            (0, 0, 0, 10),
            "pkgpath abc not found in the program".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("CannotFindModule".to_string())),
        ),
        build_lsp_diag(
            (0, 0, 0, 10),
            format!(
                "Cannot find the module abc from {}/src/test_data/abc",
                path.to_str().unwrap()
            ),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("CannotFindModule".to_string())),
        ),
        build_lsp_diag(
            (8, 0, 8, 1),
            "Can not change the value of 'd', because it was declared immutable".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![(
                file.to_string(),
                (7, 0, 7, 1),
                "The variable 'd' is declared here".to_string(),
            )],
            Some(NumberOrString::String("ImmutableError".to_string())),
        ),
        build_lsp_diag(
            (7, 0, 7, 1),
            "The variable 'd' is declared here".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![(
                file.to_string(),
                (8, 0, 8, 1),
                "Can not change the value of 'd', because it was declared immutable".to_string(),
            )],
            Some(NumberOrString::String("ImmutableError".to_string())),
        ),
        build_lsp_diag(
            (2, 0, 2, 1),
            "expected str, got int(1)".to_string(),
            Some(DiagnosticSeverity::ERROR),
            vec![],
            Some(NumberOrString::String("TypeError".to_string())),
        ),
        build_lsp_diag(
            (0, 0, 0, 10),
            "Module 'abc' imported but unused".to_string(),
            Some(DiagnosticSeverity::WARNING),
            vec![],
            Some(NumberOrString::String("UnusedImportWarning".to_string())),
        ),
    ];
    for (get, expected) in diagnostics.iter().zip(expected_diags.iter()) {
        assert_eq!(get, expected)
    }
}

#[test]
#[bench_test]
fn quick_fix_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut test_file = path.clone();
    test_file.push("src/test_data/quick_fix.k");
    let file = test_file.to_str().unwrap();

    let (_, _, diags) = parse_param_and_compile(
        Param {
            file: file.to_string(),
        },
        Some(Arc::new(RwLock::new(Default::default()))),
    )
    .unwrap();

    let diagnostics = diags
        .iter()
        .flat_map(|diag| kcl_diag_to_lsp_diags(diag, file))
        .collect::<Vec<Diagnostic>>();

    let uri = Url::from_file_path(file).unwrap();
    let code_actions = quick_fix(&uri, &diagnostics);

    let expected = vec![
        CodeActionOrCommand::CodeAction(CodeAction {
            title: "ReimportWarning".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostics[0].clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(
                    vec![(
                        uri.clone(),
                        vec![TextEdit {
                            range: Range {
                                start: Position {
                                    line: 1,
                                    character: 0,
                                },
                                end: Position {
                                    line: 1,
                                    character: 20,
                                },
                            },
                            new_text: "".to_string(),
                        }],
                    )]
                    .into_iter()
                    .collect(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        }),
        CodeActionOrCommand::CodeAction(CodeAction {
            title: "UnusedImportWarning".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostics[1].clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(
                    vec![(
                        uri.clone(),
                        vec![TextEdit {
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: 0,
                                    character: 20,
                                },
                            },
                            new_text: "".to_string(),
                        }],
                    )]
                    .into_iter()
                    .collect(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        }),
    ];

    for (get, expected) in code_actions.iter().zip(expected.iter()) {
        assert_eq!(get, expected)
    }

    assert_eq!(expected[0], code_actions[0]);
    assert_eq!(expected[1], code_actions[1]);
}

#[test]
#[bench_test]
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
#[bench_test]
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
#[bench_test]
fn goto_pkg_prefix_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    // test goto pkg prefix def: p = pkg.Person {  <- pkg
    let pos = KCLPos {
        filename: file,
        line: 4,
        column: Some(7),
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
#[bench_test]
fn goto_schema_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test goto schema definition: p = pkg.Person <- Person
    let pos = KCLPos {
        filename: file,
        line: 4,
        column: Some(11),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
    );
}

#[test]
#[bench_test]
fn goto_var_def_in_config_and_config_if_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    let pos = KCLPos {
        filename: file.clone(),
        line: 67,
        column: Some(36),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 65, 11, 65, 14));

    let pos = KCLPos {
        filename: file.clone(),
        line: 67,
        column: Some(44),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 65, 16, 65, 21));

    let pos = KCLPos {
        filename: file.clone(),
        line: 64,
        column: Some(11),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 69, 6, 69, 10));

    let pos = KCLPos {
        filename: file.clone(),
        line: 67,
        column: Some(10),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 69, 6, 69, 10));
}

#[test]
#[bench_test]
fn goto_var_def_in_dict_comp_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    let pos = KCLPos {
        filename: file.clone(),
        line: 77,
        column: Some(68),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 76, 143, 76, 145));

    let pos = KCLPos {
        filename: file.clone(),
        line: 77,
        column: Some(61),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 76, 143, 76, 145));
}

#[test]
fn goto_dict_key_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    let pos = KCLPos {
        filename: file.clone(),
        line: 26,
        column: Some(24),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 8, 4, 8, 8),
    );

    let pos = KCLPos {
        filename: file.clone(),
        line: 59,
        column: Some(28),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 18, 4, 18, 8));
}

#[test]
#[bench_test]
fn goto_schema_attr_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test goto schema attr definition: name: "alice"
    let pos = KCLPos {
        filename: file,
        line: 5,
        column: Some(7),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 4, 4, 4, 8),
    );
}

#[test]
#[bench_test]
fn goto_schema_attr_def_test1() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/goto_def.k");

    // test goto schema attr definition, goto name in: s = p2.n.name
    let pos = KCLPos {
        filename: file,
        line: 30,
        column: Some(12),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 18, 4, 18, 8),
    );
}

#[test]
#[bench_test]
fn test_goto_identifier_names() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/goto_def.k");

    // test goto p2 in: s = p2.n.name
    let pos = KCLPos {
        filename: file.clone(),
        line: 30,
        column: Some(5),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 23, 0, 23, 2),
    );

    // test goto n in: s = p2.n.name
    let pos = KCLPos {
        filename: file.clone(),
        line: 30,
        column: Some(8),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 21, 1, 21, 2),
    );

    // test goto name in: s = p2.n.name
    let pos = KCLPos {
        filename: file,
        line: 30,
        column: Some(12),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 18, 4, 18, 8),
    );
}

#[test]
#[bench_test]
fn goto_identifier_def_test() {
    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    // test goto identifier definition: p1 = p
    let pos = KCLPos {
        filename: file.to_string(),
        line: 9,
        column: Some(6),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 3, 0, 3, 1));
}

#[test]
#[bench_test]
fn goto_assign_type_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test goto schema attr definition: name: "alice"
    let pos = KCLPos {
        filename: file.clone(),
        line: 38,
        column: Some(17),
    };

    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 33, 0, 37, 0));
}

#[test]
#[bench_test]
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
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
    );
}

#[test]
#[bench_test]
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
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
    );
}

#[test]
#[bench_test]
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
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
    );
}

#[test]
#[bench_test]
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
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 7, 0),
    );
}

#[test]
#[bench_test]
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
    compare_goto_res(
        res,
        (&expected_path.to_str().unwrap().to_string(), 0, 0, 2, 13),
    );
}

#[test]
#[bench_test]
fn goto_local_var_def_test() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/goto_def_test/goto_def.k");

    let mut expected_path = path;
    expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

    // test goto local var def
    let pos = KCLPos {
        filename: file.clone(),
        line: 47,
        column: Some(11),
    };

    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 43, 4, 43, 9));

    let pos = KCLPos {
        filename: file.clone(),
        line: 49,
        column: Some(11),
    };

    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 43, 4, 43, 9));

    let pos = KCLPos {
        filename: file.clone(),
        line: 51,
        column: Some(11),
    };

    let res = goto_definition(&program, &pos, &prog_scope);
    compare_goto_res(res, (&file, 43, 4, 43, 9));
}

#[test]
#[bench_test]
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
#[bench_test]
fn var_completion_test() {
    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/completion_test/dot/completion.k");

    let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();

    // test completion for var
    let pos = KCLPos {
        filename: file.to_owned(),
        line: 26,
        column: Some(5),
    };

    let got = completion(None, &program, &pos, &prog_scope).unwrap();

    items.extend(
        vec![
            "", // generate from error recovery of "pkg."
            "subpkg", "math", "Person", "p", "p1", "p2", "p3", "p4",
        ]
        .iter()
        .map(|name| KCLCompletionItem {
            label: name.to_string(),
        })
        .collect::<IndexSet<KCLCompletionItem>>(),
    );

    let expect: CompletionResponse = into_completion_items(&items).into();

    assert_eq!(expect, got);

    // test completion for schema attr
    let pos = KCLPos {
        filename: file.to_owned(),
        line: 24,
        column: Some(4),
    };

    let got = completion(None, &program, &pos, &prog_scope).unwrap();

    items.extend(
        vec![
            "__settings__",
            "name",
            "age", // attr of schema `Person`
        ]
        .iter()
        .map(|name| KCLCompletionItem {
            label: name.to_string(),
        })
        .collect::<IndexSet<KCLCompletionItem>>(),
    );
    let expect: CompletionResponse = into_completion_items(&items).into();

    assert_eq!(expect, got);
}

#[test]
#[bench_test]
fn dot_completion_test() {
    let (file, program, prog_scope, _) =
        compile_test_file("src/test_data/completion_test/dot/completion.k");
    let mut items: IndexSet<KCLCompletionItem> = IndexSet::new();

    // test completion for schema attr
    let pos = KCLPos {
        filename: file.to_owned(),
        line: 12,
        column: Some(7),
    };

    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();

    items.insert(KCLCompletionItem {
        label: "name".to_string(),
    });
    items.insert(KCLCompletionItem {
        label: "age".to_string(),
    });

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
        items.insert(KCLCompletionItem {
            label: format!("{}{}", k, "()"),
        });
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
    items.insert(KCLCompletionItem {
        label: "file1".to_string(),
    });
    items.insert(KCLCompletionItem {
        label: "file2".to_string(),
    });
    items.insert(KCLCompletionItem {
        label: "subpkg".to_string(),
    });

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
    items.insert(KCLCompletionItem {
        label: "Person1".to_string(),
    });

    let expect: CompletionResponse = into_completion_items(&items).into();
    assert_eq!(got, expect);
    items.clear();

    let pos = KCLPos {
        filename: file.to_owned(),
        line: 19,
        column: Some(5),
    };
    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();

    items.extend(MATH_FUNCTION_NAMES.iter().map(|s| KCLCompletionItem {
        label: s.to_string(),
    }));
    let expect: CompletionResponse = into_completion_items(&items).into();
    assert_eq!(got, expect);
    items.clear();

    // test completion for literal str builtin function
    let pos = KCLPos {
        filename: file,
        line: 21,
        column: Some(4),
    };

    let got = completion(Some('.'), &program, &pos, &prog_scope).unwrap();
    let binding = STRING_MEMBER_FUNCTIONS;
    for k in binding.keys() {
        items.insert(KCLCompletionItem {
            label: format!("{}{}", k, "()"),
        });
    }
    let expect: CompletionResponse = into_completion_items(&items).into();

    assert_eq!(got, expect);
}

#[test]
#[bench_test]
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
                assert_eq!(s, "pkg\n\nschema Person");
            }
            if let MarkedString::String(s) = vec[1].clone() {
                assert_eq!(s, "hover doc test");
            }
            if let MarkedString::String(s) = vec[2].clone() {
                assert_eq!(
                    s,
                    "Attributes:\n\n__settings__?: {str:any}\n\nname: str\n\nage: int"
                );
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
                assert_eq!(s, "name: str");
            }
        }
        _ => unreachable!("test error"),
    }
}

#[test]
#[bench_test]
fn schema_doc_hover_test1() {
    let (file, program, prog_scope, _) = compile_test_file("src/test_data/hover_test/hover.k");

    let pos = KCLPos {
        filename: file.clone(),
        line: 15,
        column: Some(11),
    };
    let got = hover(&program, &pos, &prog_scope).unwrap();

    match got.contents {
        lsp_types::HoverContents::Array(vec) => {
            if let MarkedString::String(s) = vec[0].clone() {
                assert_eq!(s, "__main__\n\nschema Person");
            }
            if let MarkedString::String(s) = vec[1].clone() {
                assert_eq!(s, "hover doc test");
            }
            if let MarkedString::String(s) = vec[2].clone() {
                assert_eq!(
                    s,
                    "Attributes:\n\n__settings__?: {str:any}\n\nname: str\n\nage?: int"
                );
            }
        }
        _ => unreachable!("test error"),
    }
}

#[test]
#[bench_test]
fn schema_attr_hover_test() {
    let (file, program, prog_scope, _) = compile_test_file("src/test_data/hover_test/hover.k");

    let pos = KCLPos {
        filename: file.clone(),
        line: 16,
        column: Some(11),
    };
    let got = hover(&program, &pos, &prog_scope).unwrap();

    match got.contents {
        lsp_types::HoverContents::Array(vec) => {
            if let MarkedString::String(s) = vec[0].clone() {
                assert_eq!(s, "name: str");
            }
            if let MarkedString::String(s) = vec[1].clone() {
                assert_eq!(s, "name doc test");
            }
        }
        _ => unreachable!("test error"),
    }

    let pos = KCLPos {
        filename: file.clone(),
        line: 17,
        column: Some(11),
    };
    let got = hover(&program, &pos, &prog_scope).unwrap();

    match got.contents {
        lsp_types::HoverContents::Array(vec) => {
            if let MarkedString::String(s) = vec[0].clone() {
                assert_eq!(s, "age: int");
            }
            if let MarkedString::String(s) = vec[1].clone() {
                assert_eq!(s, "age doc test");
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
    detail: Option<String>,
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
        detail,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children: child,
    }
}

#[test]
#[bench_test]
fn document_symbol_test() {
    let (file, program, prog_scope, _) = compile_test_file("src/test_data/document_symbol.k");

    let res = document_symbol(file.as_str(), &program, &prog_scope).unwrap();
    let mut expect = vec![];
    expect.push(build_document_symbol(
        "p",
        SymbolKind::VARIABLE,
        ((3, 0), (3, 1)),
        None,
        Some("Person4".to_string()),
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
            Some("str".to_string()),
        )]),
        Some("schema".to_string()),
    ));
    let expect = DocumentSymbolResponse::Nested(expect);
    assert_eq!(res, expect)
}

#[test]
#[bench_test]
fn file_path_from_url_test() {
    if cfg!(windows) {
        let url =
            Url::parse("file:///c%3A/Users/abc/Desktop/%E4%B8%AD%E6%96%87/ab%20c/abc.k").unwrap();
        let path = file_path_from_url(&url).unwrap();
        assert_eq!(path, "c:\\Users\\abc\\Desktop\\中文\\ab c\\abc.k");
    } else {
        let url = Url::parse("file:///Users/abc/Desktop/%E4%B8%AD%E6%96%87/ab%20c/abc.k").unwrap();
        let path = file_path_from_url(&url).unwrap();
        assert_eq!(path, "/Users/abc/Desktop/中文/ab c/abc.k");
    }
}

#[test]
fn goto_import_external_file_test() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("goto_import_def_test")
        .join("main.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();

    let _ = Command::new("kpm")
        .arg("metadata")
        .arg("--update")
        .current_dir(
            PathBuf::from(".")
                .join("src")
                .join("test_data")
                .join("goto_import_def_test")
                .canonicalize()
                .unwrap()
                .display()
                .to_string(),
        )
        .output()
        .unwrap();

    let (program, prog_scope, diags) = parse_param_and_compile(
        Param {
            file: path.to_string(),
        },
        Some(Arc::new(RwLock::new(Default::default()))),
    )
    .unwrap();

    assert_eq!(diags.len(), 0);

    // test goto import file: import .pkg.schema_def
    let pos = KCLPos {
        filename: path.to_string(),
        line: 1,
        column: Some(15),
    };
    let res = goto_definition(&program, &pos, &prog_scope);
    assert!(res.is_some());
}

#[test]
fn format_signle_file_test() {
    const FILE_INPUT_SUFFIX: &str = ".input";
    const FILE_OUTPUT_SUFFIX: &str = ".golden";
    const TEST_CASES: &[&str; 17] = &[
        "assert",
        "check",
        "blankline",
        "breakline",
        "codelayout",
        "collection_if",
        "comment",
        "comp_for",
        // "empty",
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

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = path;
    let test_dir = test_file
        .parent()
        .unwrap()
        .join("format")
        .join("test_data")
        .join("format_data");
    for case in TEST_CASES {
        let test_file = test_dir
            .join(format!("{}{}", case, FILE_INPUT_SUFFIX))
            .to_str()
            .unwrap()
            .to_string();
        let test_src = std::fs::read_to_string(&test_file).unwrap();
        let got = format(test_file, test_src, None).unwrap().unwrap();
        let data_output = std::fs::read_to_string(
            &test_dir
                .join(format!("{}{}", case, FILE_OUTPUT_SUFFIX))
                .to_str()
                .unwrap()
                .to_string(),
        )
        .unwrap();

        #[cfg(target_os = "windows")]
        let data_output = data_output.replace("\r\n", "\n");

        let expect = vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)),
            new_text: data_output,
        }];

        assert_eq!(expect, got);
    }

    // empty test case, without change after fmt
    let test_file = test_dir
        .join(format!("{}{}", "empty", FILE_INPUT_SUFFIX))
        .to_str()
        .unwrap()
        .to_string();
    let test_src = std::fs::read_to_string(&test_file).unwrap();
    let got = format(test_file, test_src, None).unwrap();
    assert_eq!(got, None)
}

#[test]
#[bench_test]
fn format_range_test() {
    let (file, program, prog_scope, _) = compile_test_file("src/test_data/format/format_range.k");
    let lsp_range = Range::new(Position::new(0, 0), Position::new(11, 0));
    let text = std::fs::read_to_string(file.clone()).unwrap();

    let range = text_range(&text, lsp_range);
    let src = text.index(range);

    let got = format(file, src.to_owned(), Some(lsp_range))
        .unwrap()
        .unwrap();

    let expected = vec![TextEdit {
        range: lsp_range,
        new_text: "a = 1\nb = 2\nc = 3\n".to_string(),
    }];
    assert_eq!(got, expected)
}
