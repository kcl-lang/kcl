use crate::{
    from_lsp::kcl_pos,
    goto_def::find_def_with_gs,
    util::{build_word_index_for_file_paths, parse_param_and_compile, Param},
};
use anyhow::{anyhow, Result};
use kclvm_ast::ast;
use kclvm_error::diagnostic;
use kclvm_query::selector::parse_symbol_selector_spec;
use lsp_types::{Location, TextEdit, Url};
use std::collections::HashMap;
use std::path::PathBuf;

/// the rename_symbol API
/// find all the occurrences of the target symbol and return the text edit actions to rename them
/// pkg_root: the absolute file path to the root package
/// file_paths: list of files in which symbols can be renamed
/// symbol_path: path to the symbol. The symbol path should be in the format of: `pkg.sub_pkg:name.sub_name`
/// new_name: the new name of the symbol
pub fn rename_symbol(
    pkg_root: &str,
    file_paths: &[String],
    symbol_path: &str,
    new_name: String,
) -> Result<HashMap<Url, Vec<TextEdit>>> {
    // 1. from symbol path to the symbol
    let symbol_spec = parse_symbol_selector_spec(pkg_root, symbol_path)?;
    // 2. get the symbol name and definition range from symbol path

    match select_symbol(&symbol_spec) {
        Some((name, range)) => {
            // 3. build word index on file_paths, find refs within file_paths scope
            let word_index = build_word_index_for_file_paths(file_paths)?;
            if let Some(locations) = word_index.get(&name) {
                // 4. filter out the matched refs
                // 4.1 collect matched words(names) and remove Duplicates of the file paths
                let file_map =
                    locations
                        .iter()
                        .fold(HashMap::<Url, Vec<&Location>>::new(), |mut acc, loc| {
                            acc.entry(loc.uri.clone()).or_insert(Vec::new()).push(loc);
                            acc
                        });
                let refs = file_map
                    .iter()
                    .flat_map(|(_, locs)| locs.iter())
                    .filter(|&&loc| {
                        // 4.2 filter out the words and remain those whose definition is the target def
                        let p = loc.uri.path();
                        if let Ok((_, _, _, gs)) = parse_param_and_compile(
                            Param {
                                file: p.to_string(),
                            },
                            None,
                        ) {
                            let kcl_pos = kcl_pos(p, loc.range.start);
                            if let Some(symbol_ref) = find_def_with_gs(&kcl_pos, &gs, true) {
                                if let Some(real_def) = gs.get_symbols().get_symbol(symbol_ref) {
                                    return real_def.get_range() == range;
                                }
                            }
                        }
                        false
                    })
                    .cloned()
                    .collect::<Vec<&Location>>();
                // 5. refs to rename actions
                let changes = refs.into_iter().fold(HashMap::new(), |mut map, location| {
                    let uri = &location.uri;
                    map.entry(uri.clone())
                        .or_insert_with(Vec::new)
                        .push(TextEdit {
                            range: location.range,
                            new_text: new_name.clone(),
                        });
                    map
                });
                return Ok(changes);
            } else {
                return Ok(HashMap::new());
            }
        }

        None => Err(anyhow!(
            "get symbol from symbol path failed, {}",
            symbol_path
        )),
    }
}

/// Select a symbol by the symbol path
/// The symbol path should be in the format of: `pkg.sub_pkg:name.sub_name`
/// returns the symbol name and definition range
pub fn select_symbol(selector: &ast::SymbolSelectorSpec) -> Option<(String, diagnostic::Range)> {
    let mut pkg = PathBuf::from(&selector.pkg_root);
    let pkg_names = selector.pkgpath.split(".");
    for n in pkg_names {
        pkg = pkg.join(n)
    }

    let fields: Vec<&str> = selector.field_path.split(".").collect();
    match pkg.as_path().to_str() {
        Some(pkgpath) => {
            // resolve pkgpath and get the symbol data by the fully qualified name
            if let Ok((prog, _, _, gs)) = parse_param_and_compile(
                Param {
                    file: pkgpath.to_string(),
                },
                None,
            ) {
                if let Some(symbol_ref) = gs
                    .get_symbols()
                    .get_symbol_by_fully_qualified_name(&prog.main)
                {
                    let mut owner_ref = symbol_ref;
                    let mut target = None;
                    for field in fields {
                        let owner = gs.get_symbols().get_symbol(owner_ref).unwrap();
                        target = owner.get_attribute(field, gs.get_symbols(), None);
                        if let Some(target) = target {
                            owner_ref = target;
                        }
                    }

                    let target_symbol = gs.get_symbols().get_symbol(target?)?;
                    return Some((target_symbol.get_name(), target_symbol.get_range()));
                }
            }
            None
        }
        None => None,
    }
}
#[cfg(test)]
mod tests {
    use kclvm_ast::ast;
    use kclvm_error::diagnostic;
    use lsp_types::{Position, Url};
    use std::path::PathBuf;

    use super::{rename_symbol, select_symbol};

    #[test]
    fn test_select_symbol() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("src/test_data/rename_test/");
        let pkg_root = root.to_str().unwrap().to_string();

        let mut main_path = root.clone();
        main_path = main_path.join("base").join("person.k");

        if let Some((name, range)) = select_symbol(&ast::SymbolSelectorSpec {
            pkg_root: pkg_root.clone(),
            pkgpath: "base".to_string(),
            field_path: "Person.name".to_string(),
        }) {
            assert_eq!(name, "name");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 2,
                        column: Some(4),
                    },
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 2,
                        column: Some(8),
                    },
                )
            );
        } else {
            assert!(false, "select symbol failed")
        }

        if let Some((name, range)) = select_symbol(&ast::SymbolSelectorSpec {
            pkg_root: pkg_root.clone(),
            pkgpath: "base".to_string(),
            field_path: "Name.first".to_string(),
        }) {
            assert_eq!(name, "first");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 6,
                        column: Some(4),
                    },
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 6,
                        column: Some(9),
                    },
                )
            );
        } else {
            assert!(false, "select symbol failed")
        }

        if let Some((name, range)) = select_symbol(&ast::SymbolSelectorSpec {
            pkg_root: pkg_root.clone(),
            pkgpath: "base".to_string(),
            field_path: "Person".to_string(),
        }) {
            assert_eq!(name, "Person");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 1,
                        column: Some(7),
                    },
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 1,
                        column: Some(13),
                    },
                )
            );
        } else {
            assert!(false, "select symbol failed")
        }

        if let Some((name, range)) = select_symbol(&ast::SymbolSelectorSpec {
            pkg_root: pkg_root.clone(),
            pkgpath: "base".to_string(),
            field_path: "a".to_string(),
        }) {
            assert_eq!(name, "a");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 8,
                        column: Some(0),
                    },
                    diagnostic::Position {
                        filename: main_path.as_path().to_str().unwrap().to_string(),
                        line: 8,
                        column: Some(1),
                    },
                )
            );
        } else {
            assert!(false, "select symbol failed")
        }
    }

    #[test]
    fn test_select_symbol_failed() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("src/test_data/rename_test/");

        let result = select_symbol(&ast::SymbolSelectorSpec {
            pkg_root: root.to_str().unwrap().to_string(),
            pkgpath: "base".to_string(),
            field_path: "name".to_string(),
        });
        assert!(result.is_none(), "should not find the target symbol")
    }

    #[test]
    fn test_rename() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("src/test_data/rename_test/");

        let mut main_path = root.clone();
        let mut base_path = root.clone();
        base_path.push("base/person.k");
        main_path.push("config.k");

        let base_url = Url::from_file_path(base_path.clone()).unwrap();
        let main_url = Url::from_file_path(main_path.clone()).unwrap();

        if let Ok(changes) = rename_symbol(
            root.to_str().unwrap(),
            &vec![
                base_path.to_str().unwrap().to_string(),
                main_path.to_str().unwrap().to_string(),
            ],
            "base:Person",
            "NewPerson".to_string(),
        ) {
            assert_eq!(changes.len(), 2);
            assert!(changes.contains_key(&base_url));
            assert!(changes.contains_key(&main_url));
            assert!(changes.get(&base_url).unwrap().len() == 1);
            assert!(changes.get(&base_url).unwrap()[0].range.start == Position::new(0, 7));
            assert!(changes.get(&main_url).unwrap().len() == 1);
            assert!(changes.get(&main_url).unwrap()[0].range.start == Position::new(2, 9));
            assert!(changes.get(&main_url).unwrap()[0].new_text == "NewPerson".to_string());
        } else {
            assert!(false, "rename failed")
        }

        if let Ok(changes) = rename_symbol(
            root.to_str().unwrap(),
            &vec![
                base_path.to_str().unwrap().to_string(),
                main_path.to_str().unwrap().to_string(),
            ],
            "base:Person.name",
            "new_name".to_string(),
        ) {
            assert_eq!(changes.len(), 2);
            assert!(changes.contains_key(&base_url));
            assert!(changes.contains_key(&main_url));
            assert!(changes.get(&base_url).unwrap().len() == 1);
            assert!(changes.get(&base_url).unwrap()[0].range.start == Position::new(1, 4));
            assert!(changes.get(&main_url).unwrap().len() == 1);
            assert!(changes.get(&main_url).unwrap()[0].range.start == Position::new(4, 4));
            assert!(changes.get(&main_url).unwrap()[0].new_text == "new_name".to_string());
        } else {
            assert!(false, "rename failed")
        }
    }
}
