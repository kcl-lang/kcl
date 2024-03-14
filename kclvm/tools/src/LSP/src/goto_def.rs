//! GotoDefinition for KCL
//! Github Issue: https://github.com/kcl-lang/kcl/issues/476
//! Now supports goto definition for the following situation:
//! + variable
//! + schema definition
//! + mixin definition
//! + schema attr
//! + attr type

use crate::to_lsp::lsp_location;
use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_error::Position as KCLPos;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::core::symbol::SymbolRef;
use lsp_types::GotoDefinitionResponse;

// Navigates to the definition of an identifier.
pub(crate) fn goto_definition_with_gs(
    _program: &Program,
    kcl_pos: &KCLPos,
    gs: &GlobalState,
) -> Option<lsp_types::GotoDefinitionResponse> {
    let mut res = IndexSet::new();
    let def = find_def_with_gs(kcl_pos, gs, true);
    match def {
        Some(def_ref) => match gs.get_symbols().get_symbol(def_ref) {
            Some(def) => match def_ref.get_kind() {
                kclvm_sema::core::symbol::SymbolKind::Package => {
                    let pkg_info = gs.get_packages().get_package_info(&def.get_name()).unwrap();
                    for file in pkg_info.get_kfile_paths() {
                        let dummy_pos = KCLPos {
                            filename: file.clone(),
                            line: 1,
                            column: None,
                        };
                        res.insert((dummy_pos.clone(), dummy_pos));
                    }
                }
                _ => {
                    res.insert(def.get_range());
                }
            },
            None => {}
        },
        None => {}
    }
    positions_to_goto_def_resp(&res)
}

pub(crate) fn find_def_with_gs(
    kcl_pos: &KCLPos,
    gs: &GlobalState,
    exact: bool,
) -> Option<SymbolRef> {
    if exact {
        match gs.look_up_exact_symbol(kcl_pos) {
            Some(symbol_ref) => match gs.get_symbols().get_symbol(symbol_ref) {
                Some(symbol) => symbol.get_definition(),
                None => None,
            },
            None => None,
        }
    } else {
        match gs.look_up_closest_symbol(kcl_pos) {
            Some(symbol_ref) => match gs.get_symbols().get_symbol(symbol_ref) {
                Some(symbol) => symbol.get_definition(),
                None => None,
            },
            None => None,
        }
    }
}

// Convert kcl position to GotoDefinitionResponse. This function will convert to
// None, Scalar or Array according to the number of positions
fn positions_to_goto_def_resp(
    positions: &IndexSet<(KCLPos, KCLPos)>,
) -> Option<GotoDefinitionResponse> {
    match positions.len() {
        0 => None,
        1 => {
            let (start, end) = positions.iter().next().unwrap().clone();
            let loc = lsp_location(start.filename.clone(), &start, &end)?;
            Some(lsp_types::GotoDefinitionResponse::Scalar(loc))
        }
        _ => {
            let mut res = vec![];
            for (start, end) in positions {
                let loc = lsp_location(start.filename.clone(), start, end)?;
                res.push(loc)
            }
            Some(lsp_types::GotoDefinitionResponse::Array(res))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::goto_definition_with_gs;
    use crate::{
        from_lsp::file_path_from_url,
        tests::{compare_goto_res, compile_test_file},
    };
    use indexmap::IndexSet;
    use kclvm_error::Position as KCLPos;
    use proc_macro_crate::bench_test;
    use std::path::PathBuf;

    #[test]
    #[bench_test]
    fn goto_import_pkg_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");
        let pos = KCLPos {
            filename: file,
            line: 1,
            column: Some(10),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);

        let mut expected_files = IndexSet::new();
        let path_str = path.to_str().unwrap();
        let test_files = [
            "src/test_data/goto_def_test/pkg/schema_def1.k",
            "src/test_data/goto_def_test/pkg/schema_def.k",
        ];
        expected_files.insert(format!("{}/{}", path_str, test_files[0]));
        expected_files.insert(format!("{}/{}", path_str, test_files[1]));

        match res.unwrap() {
            lsp_types::GotoDefinitionResponse::Array(arr) => {
                assert_eq!(expected_files.len(), arr.len());
                for loc in arr {
                    let got_path = file_path_from_url(&loc.uri).unwrap();
                    assert!(expected_files.contains(&got_path));
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

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto import file: import .pkg.schema_def
        let pos = KCLPos {
            filename: file,
            line: 2,
            column: Some(10),
        };
        let res = goto_definition_with_gs(&program, &pos, &gs);
        match res.unwrap() {
            lsp_types::GotoDefinitionResponse::Scalar(loc) => {
                let got_path = file_path_from_url(&loc.uri).unwrap();
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

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        // test goto pkg prefix def: p = pkg.Person {  <- pkg
        let pos = KCLPos {
            filename: file,
            line: 4,
            column: Some(7),
        };
        let res = goto_definition_with_gs(&program, &pos, &gs);
        let mut expected_files = IndexSet::new();
        let path_str = path.to_str().unwrap();
        let test_files = [
            "src/test_data/goto_def_test/pkg/schema_def1.k",
            "src/test_data/goto_def_test/pkg/schema_def.k",
        ];
        expected_files.insert(format!("{}/{}", path_str, test_files[0]));
        expected_files.insert(format!("{}/{}", path_str, test_files[1]));

        match res.unwrap() {
            lsp_types::GotoDefinitionResponse::Array(arr) => {
                assert_eq!(expected_files.len(), arr.len());
                for loc in arr {
                    let got_path = file_path_from_url(&loc.uri).unwrap();
                    assert!(expected_files.contains(&got_path));
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

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto schema definition: p = pkg.Person <- Person
        let pos = KCLPos {
            filename: file,
            line: 4,
            column: Some(11),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 7, 0, 13),
        );
    }

    #[test]
    #[bench_test]
    fn goto_var_def_in_config_and_config_if_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 67,
            column: Some(36),
        };
        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 65, 11, 65, 14));

        let pos = KCLPos {
            filename: file.clone(),
            line: 67,
            column: Some(44),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 65, 16, 65, 21));
        let pos = KCLPos {
            filename: file.clone(),
            line: 64,
            column: Some(11),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 69, 6, 69, 10));
        let pos = KCLPos {
            filename: file.clone(),
            line: 67,
            column: Some(10),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 69, 6, 69, 10));
    }

    #[test]
    #[bench_test]
    fn goto_var_def_in_dict_comp_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 77,
            column: Some(68),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 76, 143, 76, 145));

        let pos = KCLPos {
            filename: file.clone(),
            line: 77,
            column: Some(61),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 76, 143, 76, 145));
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_def_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto schema attr definition: name: "alice"
        let pos = KCLPos {
            filename: file,
            line: 5,
            column: Some(7),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 4, 4, 4, 8),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_def_test1() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/goto_def.k");

        // test goto schema attr definition, goto name in: s = p2.n.name
        let pos = KCLPos {
            filename: file,
            line: 30,
            column: Some(12),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 18, 4, 18, 8),
        );
    }

    #[test]
    #[bench_test]
    fn test_goto_identifier_names() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/goto_def.k");

        // test goto p2 in: s = p2.n.name
        let pos = KCLPos {
            filename: file.clone(),
            line: 30,
            column: Some(5),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
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

        let res = goto_definition_with_gs(&program, &pos, &gs);
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

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 18, 4, 18, 8),
        );
    }

    #[test]
    #[bench_test]
    fn goto_identifier_def_test() {
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        // test goto identifier definition: p1 = p
        let pos = KCLPos {
            filename: file.to_string(),
            line: 9,
            column: Some(6),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 3, 0, 3, 1));
    }

    #[test]
    #[bench_test]
    fn goto_assign_type_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto schema attr definition: name: "alice"
        let pos = KCLPos {
            filename: file.clone(),
            line: 38,
            column: Some(17),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 33, 7, 33, 15));
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test() {
        // test goto schema attr type definition: p1: pkg.Person
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 12,
            column: Some(15),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 7, 0, 13),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test1() {
        // test goto schema attr type definition: p2: [pkg.Person]
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 13,
            column: Some(15),
        };
        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 7, 0, 13),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test3() {
        // test goto schema attr type definition: p3: {str: pkg.Person}
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 14,
            column: Some(22),
        };
        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 7, 0, 13),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test4() {
        // test goto schema attr type definition(Person): p4: pkg.Person | pkg.Person1
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        let pos = KCLPos {
            filename: file,
            line: 15,
            column: Some(17),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 7, 0, 13),
        );
    }

    #[test]
    #[bench_test]
    fn goto_schema_attr_ty_def_test5() {
        // test goto schema attr type definition(Person1): p4: pkg.Person | pkg.Person1
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def1.k");

        let pos = KCLPos {
            filename: file,
            line: 15,
            column: Some(28),
        };
        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(
            res,
            (&expected_path.to_str().unwrap().to_string(), 0, 7, 0, 14),
        );
    }

    #[test]
    #[bench_test]
    fn goto_local_var_def_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let mut expected_path = path;
        expected_path.push("src/test_data/goto_def_test/pkg/schema_def.k");

        // test goto local var def
        let pos = KCLPos {
            filename: file.clone(),
            line: 47,
            column: Some(11),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 43, 4, 43, 9));

        let pos = KCLPos {
            filename: file.clone(),
            line: 49,
            column: Some(11),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 43, 4, 43, 9));

        let pos = KCLPos {
            filename: file.clone(),
            line: 51,
            column: Some(11),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 43, 4, 43, 9));
    }

    #[test]
    #[bench_test]
    fn complex_select_goto_def() {
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 52,
            column: Some(22),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 43, 4, 43, 9));
    }

    #[test]
    #[bench_test]
    fn schema_attribute_def_goto_def() {
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 19,
            column: Some(5),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 18, 4, 18, 8));
    }

    #[test]
    #[bench_test]
    fn config_desuger_def_goto_def() {
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 82,
            column: Some(9),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 18, 4, 18, 8));
    }

    #[test]
    #[bench_test]
    fn lambda_param_goto_def() {
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 86,
            column: Some(4),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 84, 14, 84, 15));

        let pos = KCLPos {
            filename: file.clone(),
            line: 86,
            column: Some(8),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 84, 22, 84, 23));
    }

    #[test]
    #[bench_test]
    fn list_if_expr_test() {
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 91,
            column: Some(8),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 88, 0, 88, 1));
    }

    #[test]
    #[bench_test]
    fn lambda_local_var_test() {
        let (file, program, _, gs) = compile_test_file("src/test_data/goto_def_test/goto_def.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 96,
            column: Some(9),
        };

        let res = goto_definition_with_gs(&program, &pos, &gs);
        compare_goto_res(res, (&file, 94, 11, 94, 12));
    }
}
