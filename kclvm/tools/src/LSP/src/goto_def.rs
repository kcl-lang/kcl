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
use kclvm_error::Position as KCLPos;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::core::symbol::SymbolRef;
use lsp_types::GotoDefinitionResponse;

/// Navigates to the definition of an identifier.
pub fn goto_def(kcl_pos: &KCLPos, gs: &GlobalState) -> Option<lsp_types::GotoDefinitionResponse> {
    let mut res = IndexSet::new();
    let def = find_def(kcl_pos, gs, true);
    match def {
        Some(def_ref) => match gs.get_symbols().get_symbol(def_ref) {
            Some(def) => match def_ref.get_kind() {
                kclvm_sema::core::symbol::SymbolKind::Package => {
                    let pkg_info = match gs.get_packages().get_package_info(&def.get_name()) {
                        Some(pkg_info) => pkg_info,
                        None => return None,
                    };
                    if pkg_info.is_system() {
                        return None;
                    }
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

pub(crate) fn find_def(kcl_pos: &KCLPos, gs: &GlobalState, exact: bool) -> Option<SymbolRef> {
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

pub(crate) fn find_symbol(kcl_pos: &KCLPos, gs: &GlobalState, exact: bool) -> Option<SymbolRef> {
    if exact {
        gs.look_up_exact_symbol(kcl_pos)
    } else {
        gs.look_up_closest_symbol(kcl_pos)
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
    use super::goto_def;
    use crate::{from_lsp::file_path_from_url, tests::compile_test_file};
    use kclvm_error::Position as KCLPos;
    use std::path::{Path, PathBuf};

    #[macro_export]
    macro_rules! goto_def_test_snapshot {
        ($name:ident, $file:expr, $line:expr, $column: expr) => {
            #[test]
            fn $name() {
                let (file, _program, _, gs) = compile_test_file($file);

                let pos = KCLPos {
                    filename: file.clone(),
                    line: $line,
                    column: Some($column),
                };
                let res = goto_def(&pos, &gs);
                insta::assert_snapshot!(format!("{:?}", { fmt_resp(&res) }));
            }
        };
    }

    fn fmt_resp(resp: &Option<lsp_types::GotoDefinitionResponse>) -> String {
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        match resp {
            Some(resp) => match resp {
                lsp_types::GotoDefinitionResponse::Scalar(loc) => {
                    let url = file_path_from_url(&loc.uri).unwrap();
                    let got_path = Path::new(&url);
                    let relative_path = got_path.strip_prefix(root_path).unwrap();
                    format!("path: {:?}, range: {:?}", relative_path, loc.range)
                }
                lsp_types::GotoDefinitionResponse::Array(vec_location) => {
                    let mut res = String::new();
                    for loc in vec_location {
                        let url = file_path_from_url(&loc.uri).unwrap();
                        let got_path = Path::new(&url);
                        let relative_path = got_path.strip_prefix(root_path.clone()).unwrap();
                        res.push_str(&format!(
                            "path: {:?}, range: {:?}\n",
                            relative_path, loc.range
                        ));
                    }
                    res
                }
                lsp_types::GotoDefinitionResponse::Link(vec_location_link) => {
                    let mut res = String::new();
                    for loc in vec_location_link {
                        let url = file_path_from_url(&loc.target_uri).unwrap();
                        let got_path = Path::new(&url);
                        let relative_path = got_path.strip_prefix(root_path.clone()).unwrap();
                        res.push_str(&format!(
                            "path: {:?}, range: {:?}\n",
                            relative_path, loc.target_selection_range
                        ));
                    }
                    res
                }
            },
            None => "None".to_string(),
        }
    }

    goto_def_test_snapshot!(
        goto_import_pkg_test,
        "src/test_data/goto_def_test/goto_import_pkg_test/goto_import_pkg_test.k",
        1,
        11
    );

    goto_def_test_snapshot!(
        goto_pkg_prefix_def_test,
        "src/test_data/goto_def_test/goto_pkg_prefix_def_test/goto_pkg_prefix_def_test.k",
        3,
        7
    );

    goto_def_test_snapshot!(
        goto_var_def_in_config_and_config_if_test1,
        "src/test_data/goto_def_test/goto_var_def_in_config_and_config_if_test/goto_var_def_in_config_and_config_if_test.k",
        7,
        36
    );

    goto_def_test_snapshot!(
        goto_var_def_in_config_and_config_if_test2,
        "src/test_data/goto_def_test/goto_var_def_in_config_and_config_if_test/goto_var_def_in_config_and_config_if_test.k",
        7,
        44
    );

    goto_def_test_snapshot!(
        goto_var_def_in_config_and_config_if_test3,
        "src/test_data/goto_def_test/goto_var_def_in_config_and_config_if_test/goto_var_def_in_config_and_config_if_test.k",
        4,
        11
    );

    goto_def_test_snapshot!(
        goto_var_def_in_config_and_config_if_test4,
        "src/test_data/goto_def_test/goto_var_def_in_config_and_config_if_test/goto_var_def_in_config_and_config_if_test.k",
        7,
        10
    );

    goto_def_test_snapshot!(
        goto_var_def_in_dict_comp_test1,
        "src/test_data/goto_def_test/goto_var_def_in_dict_comp_test/goto_var_def_in_dict_comp_test.k",
        5,
        68
    );

    goto_def_test_snapshot!(
        goto_var_def_in_dict_comp_test2,
        "src/test_data/goto_def_test/goto_var_def_in_dict_comp_test/goto_var_def_in_dict_comp_test.k",
        5,
        61
    );

    goto_def_test_snapshot!(
        test_goto_identifier_names1,
        "src/test_data/goto_def_test/test_goto_identifier_names/test_goto_identifier_names.k",
        13,
        5
    );

    goto_def_test_snapshot!(
        test_goto_identifier_names2,
        "src/test_data/goto_def_test/test_goto_identifier_names/test_goto_identifier_names.k",
        13,
        8
    );

    goto_def_test_snapshot!(
        test_goto_identifier_names3,
        "src/test_data/goto_def_test/test_goto_identifier_names/test_goto_identifier_names.k",
        13,
        12
    );

    goto_def_test_snapshot!(
        goto_local_var_def_test1,
        "src/test_data/goto_def_test/goto_local_var_def_test/goto_local_var_def_test.k",
        7,
        11
    );

    goto_def_test_snapshot!(
        goto_local_var_def_test2,
        "src/test_data/goto_def_test/goto_local_var_def_test/goto_local_var_def_test.k",
        9,
        11
    );

    goto_def_test_snapshot!(
        goto_local_var_def_test3,
        "src/test_data/goto_def_test/goto_local_var_def_test/goto_local_var_def_test.k",
        11,
        11
    );

    goto_def_test_snapshot!(
        goto_lambda_param_goto_def1,
        "src/test_data/goto_def_test/goto_lambda_param_goto_def/goto_lambda_param_goto_def.k",
        3,
        5
    );

    goto_def_test_snapshot!(
        goto_lambda_param_goto_def2,
        "src/test_data/goto_def_test/goto_lambda_param_goto_def/goto_lambda_param_goto_def.k",
        3,
        9
    );

    // To implement
    goto_def_test_snapshot!(
        goto_system_pkg_test,
        "src/test_data/goto_def_test/goto_system_pkg_test/goto_system_pkg_test.k",
        1,
        1
    );

    goto_def_test_snapshot!(
        lambda_local_var_test,
        "src/test_data/goto_def_test/lambda_local_var_test/lambda_local_var_test.k",
        2,
        9
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test1,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        13,
        15
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test2,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        15,
        7
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test3,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        19,
        7
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test4,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        26,
        11
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test5,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        33,
        11
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test6,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        52,
        7
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test7,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        55,
        7
    );

    goto_def_test_snapshot!(
        goto_dict_to_schema_attr_test8,
        "src/test_data/goto_def_test/dict_to_schema/dict_to_schema.k",
        58,
        7
    );

    goto_def_test_snapshot!(
        list_if_expr_test,
        "src/test_data/goto_def_test/list_if_expr_test/list_if_expr_test.k",
        3,
        8
    );

    goto_def_test_snapshot!(
        goto_identifier_def_test,
        "src/test_data/goto_def_test/goto_identifier_def_test/goto_identifier_def_test.k",
        8,
        6
    );

    goto_def_test_snapshot!(
        complex_select_goto_def,
        "src/test_data/goto_def_test/complex_select_goto_def/complex_select_goto_def.k",
        13,
        22
    );

    goto_def_test_snapshot!(
        schema_attribute_def_goto_def,
        "src/test_data/goto_def_test/schema_attribute_def_goto_def/schema_attribute_def_goto_def.k",
        2,
        5
    );

    goto_def_test_snapshot!(
        config_desuger_def_goto_def,
        "src/test_data/goto_def_test/config_desuger_def_goto_def/config_desuger_def_goto_def.k",
        7,
        9
    );

    goto_def_test_snapshot!(
        goto_schema_attr_ty_def_test5,
        "src/test_data/goto_def_test/goto_schema_attr_ty_def_test/goto_schema_attr_ty_def_test.k",
        7,
        28
    );

    goto_def_test_snapshot!(
        goto_schema_attr_ty_def_test4,
        "src/test_data/goto_def_test/goto_schema_attr_ty_def_test/goto_schema_attr_ty_def_test.k",
        7,
        17
    );

    goto_def_test_snapshot!(
        goto_schema_attr_ty_def_test3,
        "src/test_data/goto_def_test/goto_schema_attr_ty_def_test/goto_schema_attr_ty_def_test.k",
        6,
        22
    );

    goto_def_test_snapshot!(
        goto_schema_attr_ty_def_test2,
        "src/test_data/goto_def_test/goto_schema_attr_ty_def_test/goto_schema_attr_ty_def_test.k",
        5,
        15
    );

    goto_def_test_snapshot!(
        goto_schema_attr_ty_def_test1,
        "src/test_data/goto_def_test/goto_schema_attr_ty_def_test/goto_schema_attr_ty_def_test.k",
        4,
        15
    );

    goto_def_test_snapshot!(
        goto_assign_type_test,
        "src/test_data/goto_def_test/goto_assign_type_test/goto_assign_type_test.k",
        5,
        17
    );

    goto_def_test_snapshot!(
        goto_schema_def_test,
        "src/test_data/goto_def_test/goto_schema_def_test/goto_schema_def_test.k",
        3,
        11
    );

    goto_def_test_snapshot!(
        goto_schema_attr_def_test1,
        "src/test_data/goto_def_test/goto_schema_attr_def_test/goto_schema_attr_def_test.k",
        4,
        7
    );

    goto_def_test_snapshot!(
        goto_schema_attr_def_test2,
        "src/test_data/goto_def_test/goto_schema_attr_def_test/goto_schema_attr_def_test.k",
        18,
        12
    );

    goto_def_test_snapshot!(
        goto_lambda_param_schema_test,
        "src/test_data/goto_def_test/goto_lambda_param_schema_test/goto_lambda_param_schema_test.k",
        8,
        10
    );

    goto_def_test_snapshot!(
        goto_lambda_return_schema_test,
        "src/test_data/goto_def_test/goto_lambda_return_schema_test/goto_lambda_return_schema_test.k",
        6,
        10
    );

    goto_def_test_snapshot!(
        goto_nested_schema_attr_test,
        "src/test_data/goto_def_test/goto_nested_schema_attr_test/goto_nested_schema_attr_test.k",
        22,
        22
    );

    goto_def_test_snapshot!(
        goto_base_schema_attr_test,
        "src/test_data/goto_def_test/goto_base_schema_attr_test/goto_base_schema_attr_test.k",
        8,
        12
    );

    goto_def_test_snapshot!(
        goto_base_schema_attr_1_test,
        "src/test_data/goto_def_test/goto_base_schema_attr_1_test/goto_base_schema_attr_1_test.k",
        4,
        12
    );

    goto_def_test_snapshot!(
        goto_unification_schema_attr_test,
        "src/test_data/goto_def_test/goto_unification_schema_attr_test/goto_unification_schema_attr_test.k",
        7,
        7
    );

    goto_def_test_snapshot!(
        goto_duplicate_var_name_in_schema,
        "src/test_data/goto_def_test/duplicate_var_name_test/duplicate_var_name.k",
        8,
        11
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_1,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        9,
        14
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_2,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        10,
        14
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_3,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        11,
        14
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_4,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        17,
        12
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_5,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        32,
        15
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_6,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        33,
        15
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_7,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        32,
        10
    );

    goto_def_test_snapshot!(
        goto_attr_in_schema_def_8,
        "src/test_data/goto_def_test/goto_attr_in_schema_def/goto_attr_in_schema_def.k",
        33,
        10
    );

    goto_def_test_snapshot!(
        goto_protocol_attr,
        "src/test_data/goto_def_test/goto_protocol/goto_protocol.k",
        6,
        17
    );

    goto_def_test_snapshot!(
        goto_protocol_attr_1,
        "src/test_data/goto_def_test/goto_protocol/goto_protocol.k",
        8,
        13
    );
}
