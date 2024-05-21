use kclvm_ast::ast::Program;
use kclvm_error::Position as KCLPos;
use kclvm_sema::{
    builtin::BUILTIN_DECORATORS,
    core::global_state::GlobalState,
    ty::{FunctionType, ANY_TYPE_STR},
};
use lsp_types::{Hover, HoverContents, MarkedString};

use crate::goto_def::find_def_with_gs;

/// Returns a short text describing element at position.
/// Specifically, the doc for schema and schema attr(todo)

enum MarkedStringType {
    String,
    LanguageString,
}

pub(crate) fn hover(
    _program: &Program,
    kcl_pos: &KCLPos,
    gs: &GlobalState,
) -> Option<lsp_types::Hover> {
    let mut docs: Vec<(String, MarkedStringType)> = vec![];
    let def = find_def_with_gs(kcl_pos, gs, true);
    match def {
        Some(def_ref) => match gs.get_symbols().get_symbol(def_ref) {
            Some(obj) => match def_ref.get_kind() {
                kclvm_sema::core::symbol::SymbolKind::Schema => match &obj.get_sema_info().ty {
                    Some(ty) => {
                        // Build hover content for schema definition
                        // Schema Definition hover
                        // ```
                        // pkg
                        // schema Foo(Base)[param: type]
                        // -----------------
                        // doc
                        // -----------------
                        // Attributes:
                        // attr1: type
                        // attr2? type
                        // ```
                        let schema_ty = ty.into_schema_type();

                        let schema_ty_signature_no_pkg = schema_ty.schema_ty_signature_no_pkg();

                        docs.push((schema_ty.pkgpath, MarkedStringType::String));

                        if !schema_ty.doc.is_empty() {
                            docs.push((schema_ty.doc.clone(), MarkedStringType::String));
                        }

                        // The attr of schema_ty does not contain the attrs from inherited base schema.
                        // Use the api provided by GlobalState to get all attrs
                        let module_info = gs.get_packages().get_module_info(&kcl_pos.filename);
                        let schema_attrs = obj.get_all_attributes(gs.get_symbols(), module_info);
                        let mut attrs = vec![schema_ty_signature_no_pkg];
                        for schema_attr in schema_attrs {
                            if let kclvm_sema::core::symbol::SymbolKind::Attribute =
                                schema_attr.get_kind()
                            {
                                let attr = gs.get_symbols().get_symbol(schema_attr).unwrap();
                                let name = attr.get_name();
                                let attr_symbol =
                                    gs.get_symbols().get_attr_symbol(schema_attr).unwrap();
                                let attr_ty_str = match &attr.get_sema_info().ty {
                                    Some(ty) => ty.ty_str(),
                                    None => ANY_TYPE_STR.to_string(),
                                };
                                attrs.push(format!(
                                    "\t{}{}: {}",
                                    name,
                                    if attr_symbol.is_optional() { "?" } else { "" },
                                    attr_ty_str,
                                ));
                            }
                        }
                        docs.push((attrs.join("\n\n"), MarkedStringType::LanguageString));
                    }
                    _ => {}
                },
                kclvm_sema::core::symbol::SymbolKind::Attribute => {
                    let sema_info = obj.get_sema_info();
                    match &sema_info.ty {
                        Some(ty) => {
                            docs.push((
                                format!("{}: {}", &obj.get_name(), ty.ty_str()),
                                MarkedStringType::LanguageString,
                            ));
                            if let Some(doc) = &sema_info.doc {
                                if !doc.is_empty() {
                                    docs.push((doc.clone(), MarkedStringType::String));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                kclvm_sema::core::symbol::SymbolKind::Value => match &obj.get_sema_info().ty {
                    Some(ty) => match &ty.kind {
                        kclvm_sema::ty::TypeKind::Function(func_ty) => {
                            docs.push((
                                build_func_hover_content(func_ty, obj.get_name().clone())
                                    .join("\n"),
                                MarkedStringType::LanguageString,
                            ));
                        }
                        _ => {
                            docs.push((
                                format!("{}: {}", &obj.get_name(), ty.ty_str()),
                                MarkedStringType::LanguageString,
                            ));
                        }
                    },
                    _ => {}
                },
                kclvm_sema::core::symbol::SymbolKind::Expression => return None,
                kclvm_sema::core::symbol::SymbolKind::Comment => return None,
                kclvm_sema::core::symbol::SymbolKind::Decorator => {
                    match BUILTIN_DECORATORS.get(&obj.get_name()) {
                        Some(ty) => {
                            let hover_content = build_func_hover_content(
                                &ty.into_func_type(),
                                obj.get_name().clone(),
                            )
                            .join("\n");

                            for (i, ele) in hover_content.split("fn").enumerate() {
                                if i == 0 {
                                    docs.push((ele.to_string(), MarkedStringType::String));
                                } else {
                                    docs.push((
                                        format!("fn{}", ele),
                                        MarkedStringType::LanguageString,
                                    ));
                                }
                            }

                            // docs.push((
                            //     build_func_hover_content(
                            //         &ty.into_func_type(),
                            //         obj.get_name().clone(),
                            //     )
                            //     .join("\n"),
                            //     MarkedStringType::LanguageString,
                            // ));
                        }
                        None => todo!(),
                    }
                }
                _ => {
                    let ty_str = match &obj.get_sema_info().ty {
                        Some(ty) => ty.ty_str(),
                        None => "".to_string(),
                    };
                    docs.push((
                        format!("{}: {}", &obj.get_name(), ty_str),
                        MarkedStringType::LanguageString,
                    ));
                }
            },
            None => {}
        },
        None => {}
    }
    docs_to_hover(docs)
}

fn convert_doc_to_marked_string(doc: &(String, MarkedStringType)) -> MarkedString {
    match doc.1 {
        MarkedStringType::String => MarkedString::String(doc.0.clone()),
        MarkedStringType::LanguageString => {
            MarkedString::LanguageString(lsp_types::LanguageString {
                language: "kcl".to_string(),
                value: doc.0.clone(),
            })
        }
    }
}

// Convert docs to Hover. This function will convert to
// None, Scalar or Array according to the number of positions
fn docs_to_hover(docs: Vec<(String, MarkedStringType)>) -> Option<lsp_types::Hover> {
    match docs.len() {
        0 => None,
        1 => Some(Hover {
            contents: HoverContents::Scalar(convert_doc_to_marked_string(&docs[0])),
            range: None,
        }),
        _ => Some(Hover {
            contents: HoverContents::Array(
                docs.iter()
                    .map(|doc| convert_doc_to_marked_string(doc))
                    .collect(),
            ),
            range: None,
        }),
    }
}

// Build hover content for function call
// ```
// pkg
// -----------------
// function func_name(arg1: type, arg2: type, ..) -> type
// -----------------
// doc
// ```
fn build_func_hover_content(func_ty: &FunctionType, name: String) -> Vec<String> {
    let mut docs = vec![];
    if let Some(ty) = &func_ty.self_ty {
        let self_ty = format!("{}\n\n", ty.ty_str());
        docs.push(self_ty);
    }

    let mut sig = format!("fn {}(", name);
    if func_ty.params.is_empty() {
        sig.push(')');
    } else {
        for (i, p) in func_ty.params.iter().enumerate() {
            sig.push_str(&format!("{}: {}", p.name, p.ty.ty_str()));

            if i != func_ty.params.len() - 1 {
                sig.push_str(", ");
            }
        }
        sig.push(')');
    }
    sig.push_str(&format!(" -> {}", func_ty.return_ty.ty_str()));
    docs.push(sig);

    if !func_ty.doc.is_empty() {
        docs.push(func_ty.doc.clone().replace('\n', "\n\n"));
    }
    docs
}

#[cfg(test)]
mod tests {
    use crate::hover::docs_to_hover;
    use crate::hover::MarkedStringType;
    use std::path::PathBuf;

    use kclvm_error::Position as KCLPos;
    use lsp_types::MarkedString;
    use proc_macro_crate::bench_test;

    use crate::tests::compile_test_file;

    use super::hover;

    #[test]
    #[bench_test]
    fn schema_doc_hover_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test hover of schema doc: p = pkg.Person
        let pos = KCLPos {
            filename: file.clone(),
            line: 4,
            column: Some(11),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "pkg");
                }
                if let MarkedString::LanguageString(s) = vec[1].clone() {
                    assert_eq!(s.value, "schema Person");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::String(s) = vec[2].clone() {
                    assert_eq!(s, "hover doc test");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::String(s) = vec[3].clone() {
                    assert_eq!(s, "\n\n");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::String(s) = vec[4].clone() {
                    assert_eq!(s, "Attributes:\n");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::LanguageString(s) = vec[5].clone() {
                    assert_eq!(s.value, "name: str\n\nage: int");
                } else {
                    unreachable!("Wrong type");
                }
            }
            _ => unreachable!("test error"),
        }
        let pos = KCLPos {
            filename: file,
            line: 5,
            column: Some(7),
        };
        let got = hover(&program, &pos, &gs).unwrap();
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
    fn test_docs_to_hover_multiple_docs() {
        // Given multiple documentation strings
        let docs = vec![
            (
                "Documentation string 1".to_string(),
                MarkedStringType::String,
            ),
            (
                "Documentation string 2".to_string(),
                MarkedStringType::String,
            ),
            (
                "Documentation string 3".to_string(),
                MarkedStringType::String,
            ),
        ];

        // When converting to hover content
        let hover = docs_to_hover(docs);

        // Then the result should be a Hover object with an Array of MarkedString::String
        assert!(hover.is_some());
        let hover = hover.unwrap();
        match hover.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec.len(), 3);
                assert_eq!(
                    vec[0],
                    MarkedString::String("Documentation string 1".to_string())
                );
                assert_eq!(
                    vec[1],
                    MarkedString::String("Documentation string 2".to_string())
                );
                assert_eq!(
                    vec[2],
                    MarkedString::String("Documentation string 3".to_string())
                );
            }
            _ => panic!("Unexpected hover contents"),
        }
    }

    #[test]
    #[bench_test]
    fn schema_doc_hover_test1() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/hover.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 16,
            column: Some(8),
        };
        let got = hover(&program, &pos, &gs).unwrap();

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "__main__");
                }
                if let MarkedString::LanguageString(s) = vec[1].clone() {
                    assert_eq!(s.value, "schema Person");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::String(s) = vec[2].clone() {
                    assert_eq!(s, "hover doc test");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::String(s) = vec[3].clone() {
                    assert_eq!(s, "\n\n");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::String(s) = vec[4].clone() {
                    assert_eq!(s, "Attributes:\n");
                } else {
                    unreachable!("Wrong type");
                }
                if let MarkedString::LanguageString(s) = vec[5].clone() {
                    assert_eq!(s.value, "name: str\n\nage?: int");
                } else {
                    unreachable!("Wrong type");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn schema_attr_hover_test() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/hover.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 17,
            column: Some(7),
        };
        let got = hover(&program, &pos, &gs).unwrap();

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
            line: 18,
            column: Some(7),
        };
        let got = hover(&program, &pos, &gs).unwrap();

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

    #[test]
    #[bench_test]
    fn func_def_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/hover.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 22,
            column: Some(18),
        };
        let got = hover(&program, &pos, &gs).unwrap();

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec.len(), 2);
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "fn encode(value: str, encoding: str) -> str");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(
                        s,
                        "Encode the string `value` using the codec registered for encoding."
                    );
                }
            }
            _ => unreachable!("test error"),
        }

        let pos = KCLPos {
            filename: file.clone(),
            line: 23,
            column: Some(14),
        };
        let got = hover(&program, &pos, &gs).unwrap();

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec.len(), 3);
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "str\n\n");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "fn count(sub: str, start: int, end: int) -> int");
                }
                if let MarkedString::String(s) = vec[2].clone() {
                    assert_eq!(s, "Return the number of non-overlapping occurrences of substring sub in the range [start, end]. Optional arguments start and end are interpreted as in slice notation.");
                }
            }
            _ => unreachable!("test error"),
        }

        let pos = KCLPos {
            filename: file.clone(),
            line: 25,
            column: Some(4),
        };
        let got = hover(&program, &pos, &gs).unwrap();

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec.len(), 2);
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "fn print() -> NoneType");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "Prints the values to a stream, or to the system stdout by default.\n\nOptional keyword arguments:\n\nsep:   string inserted between values, default a space.\n\nend:   string appended after the last value, default a newline.");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn complex_select_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/fib.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 14,
            column: Some(22),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Scalar(marked_string) => {
                if let MarkedString::String(s) = marked_string {
                    assert_eq!(s, "value: int");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn assignment_ty_in_lambda_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/ty_in_lambda.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 3,
            column: Some(8),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Scalar(marked_string) => {
                if let MarkedString::String(s) = marked_string {
                    assert_eq!(s, "result: {str:str}");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn str_var_func_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/hover.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 28,
            column: Some(12),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec.len(), 3);
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "str\n\n");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "fn capitalize() -> str");
                }
                if let MarkedString::String(s) = vec[2].clone() {
                    assert_eq!(s, "Return a copy of the string with its first character capitalized and the rest lowercased.");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn import_pkg_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/import_pkg.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 3,
            column: Some(7),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec.len(), 2);
                if let MarkedString::String(s) = vec[0].clone() {
                    assert_eq!(s, "fib\n\nschema Fib");
                }
                if let MarkedString::String(s) = vec[1].clone() {
                    assert_eq!(s, "Attributes:\n\nn: int\n\nvalue: int");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn expr_after_config_if_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/hover.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 41,
            column: Some(13),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Scalar(marked_string) => {
                if let MarkedString::String(s) = marked_string {
                    assert_eq!(s, "stratege: str");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn schema_scope_variable_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/fib.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 3,
            column: Some(11),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Scalar(marked_string) => {
                if let MarkedString::String(s) = marked_string {
                    assert_eq!(s, "n1: int");
                }
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn decorator_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/decorator.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 1,
            column: Some(1),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        let expect_content = vec![MarkedString::String("".to_string()), MarkedString::LanguageString(lsp_types::LanguageString {
            language: "kcl".to_string(),
            value: "fn deprecated(version: str, reason: str, strict: bool) -> any\nThis decorator is used to get the deprecation message according to the wrapped key-value pair.".to_string(),
            })];
        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec, expect_content)
            }
            _ => unreachable!("test error"),
        }

        let pos = KCLPos {
            filename: file.clone(),
            line: 3,
            column: Some(8),
        };
        let got = hover(&program, &pos, &gs).unwrap();
        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec, expect_content);
            }
            _ => unreachable!("test error"),
        }
    }

    #[test]
    #[bench_test]
    fn inherit_schema_attr_hover() {
        let (file, program, _, gs) = compile_test_file("src/test_data/hover_test/inherit.k");
        let pos = KCLPos {
            filename: file.clone(),
            line: 5,
            column: Some(9),
        };
        let got = hover(&program, &pos, &gs).unwrap();

        let expect_content = vec![
            MarkedString::String("__main__\n\nschema Data1\\[m: {str:str}](Data)".to_string()),
            MarkedString::String("Attributes:\n\nname: str\n\nage: int".to_string()),
        ];

        match got.contents {
            lsp_types::HoverContents::Array(vec) => {
                assert_eq!(vec, expect_content);
            }
            _ => unreachable!("test error"),
        }
    }
}
