use crate::to_lsp::lsp_location;
use kclvm_error::Position as KCLPos;
use kclvm_sema::core::global_state::GlobalState;
use lsp_types::Location;
use std::collections::HashSet;

pub fn find_refs(kcl_pos: &KCLPos, gs: &GlobalState) -> Option<Vec<Location>> {
    match gs.look_up_exact_symbol(kcl_pos) {
        Some(symbol_ref) => match gs.get_symbols().get_symbol(symbol_ref) {
            Some(symbol) => match symbol.get_definition() {
                Some(def_ref) => {
                    if let Some(def) = gs.get_symbols().get_symbol(def_ref) {
                        let refs = def.get_references();
                        let mut refs_locs: HashSet<(KCLPos, KCLPos)> = refs
                            .iter()
                            .filter_map(|symbol| {
                                gs.get_symbols()
                                    .get_symbol(*symbol)
                                    .map(|sym| sym.get_range())
                            })
                            .collect();
                        refs_locs.insert(symbol.get_range());
                        refs_locs.insert(def.get_range());
                        let mut res: Vec<Location> = refs_locs
                            .iter()
                            .filter_map(|(start, end)| {
                                lsp_location(start.filename.clone(), &start, &end).map(|loc| loc)
                            })
                            .collect();
                        res.sort_by_key(|e| e.range.start.line);
                        return Some(res);
                    }
                }
                None => {}
            },
            None => {}
        },
        None => {}
    };
    None
}

#[cfg(test)]
mod tests {
    use crate::find_refs::find_refs;
    use crate::from_lsp::file_path_from_url;
    use lsp_types::Location;
    use std::path::{Path, PathBuf};

    use crate::tests::compile_test_file;
    use kclvm_error::Position as KCLPos;

    #[macro_export]
    macro_rules! find_ref_test_snapshot {
        ($name:ident, $file:expr, $line:expr, $column: expr) => {
            #[test]
            fn $name() {
                let (file, _program, _, gs) = compile_test_file($file);

                let pos = KCLPos {
                    filename: file.clone(),
                    line: $line,
                    column: Some($column),
                };
                let res = find_refs(&pos, &gs);
                insta::assert_snapshot!(format!("{}", { fmt_resp(&res) }));
            }
        };
    }

    fn fmt_resp(resp: &Option<Vec<Location>>) -> String {
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        match resp {
            Some(resp) => {
                let mut res = String::new();
                for loc in resp {
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
            None => "None".to_string(),
        }
    }

    find_ref_test_snapshot!(
        find_refs_variable_def_test,
        "src/test_data/find_refs_test/main.k",
        1,
        1
    );

    find_ref_test_snapshot!(
        find_refs_variable_ref_test,
        "src/test_data/find_refs_test/main.k",
        2,
        5
    );

    find_ref_test_snapshot!(
        find_refs_schema_name_test,
        "src/test_data/find_refs_test/main.k",
        5,
        8
    );

    find_ref_test_snapshot!(
        find_refs_schema_name_ref_test,
        "src/test_data/find_refs_test/main.k",
        9,
        8
    );

    find_ref_test_snapshot!(
        find_refs_schema_attr_test,
        "src/test_data/find_refs_test/main.k",
        6,
        7
    );

    find_ref_test_snapshot!(
        find_refs_schema_attr_ref_test,
        "src/test_data/find_refs_test/main.k",
        13,
        11
    );

    find_ref_test_snapshot!(
        find_refs_schema_arg_test,
        "src/test_data/find_refs_test/main.k",
        17,
        17
    );

    find_ref_test_snapshot!(
        find_refs_schema_arg_1_test,
        "src/test_data/find_refs_test/main.k",
        18,
        17
    );
}
