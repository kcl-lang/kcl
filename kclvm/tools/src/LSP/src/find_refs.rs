use crate::from_lsp::{file_path_from_url, kcl_pos};
use crate::goto_def::{find_def_with_gs, goto_definition_with_gs};
use crate::to_lsp::lsp_location;
use crate::util::{compile_with_params, Params};

use crate::state::{KCLVfs, KCLWordIndexMap};
use anyhow::Result;
use kclvm_ast::ast::Program;
use kclvm_error::Position as KCLPos;
use kclvm_parser::KCLModuleCache;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::resolver::scope::KCLScopeCache;
use lsp_types::Location;

const FIND_REFS_LIMIT: usize = 20;

pub(crate) fn find_refs<F: Fn(String) -> Result<(), anyhow::Error>>(
    _program: &Program,
    kcl_pos: &KCLPos,
    include_declaration: bool,
    word_index_map: KCLWordIndexMap,
    vfs: Option<KCLVfs>,
    logger: F,
    gs: &GlobalState,
    module_cache: Option<KCLModuleCache>,
    scope_cache: Option<KCLScopeCache>,
) -> Result<Vec<Location>, String> {
    let def = find_def_with_gs(kcl_pos, gs, true);
    match def {
        Some(def_ref) => match gs.get_symbols().get_symbol(def_ref) {
            Some(obj) => {
                let (start, end) = obj.get_range();
                // find all the refs of the def
                if let Some(def_loc) = lsp_location(start.filename.clone(), &start, &end) {
                    Ok(find_refs_from_def(
                        vfs,
                        word_index_map,
                        def_loc,
                        obj.get_name(),
                        include_declaration,
                        Some(FIND_REFS_LIMIT),
                        logger,
                        module_cache,
                        scope_cache,
                    ))
                } else {
                    Err(format!("Invalid file path: {0}", start.filename))
                }
            }
            None => Err(String::from(
                "Found more than one definitions, reference not supported",
            )),
        },
        None => Err(String::from(
            "Definition item not found, result in no reference",
        )),
    }
}

pub(crate) fn find_refs_from_def<F: Fn(String) -> Result<(), anyhow::Error>>(
    vfs: Option<KCLVfs>,
    word_index_map: KCLWordIndexMap,
    def_loc: Location,
    name: String,
    include_declaration: bool,
    limit: Option<usize>,
    logger: F,
    module_cache: Option<KCLModuleCache>,
    scope_cache: Option<KCLScopeCache>,
) -> Vec<Location> {
    let mut ref_locations = vec![];
    for word_index in (*word_index_map.write()).values_mut() {
        if let Some(mut locs) = word_index.get(name.as_str()).cloned() {
            if let Some(limit) = limit {
                if locs.len() >= limit {
                    let _ = logger(format!(
                        "Found more than {0} matched symbols, only the first {0} will be processed",
                        limit
                    ));
                    locs = locs[0..limit].to_vec();
                }
            }
            let matched_locs: Vec<Location> = locs
                .into_iter()
                .filter(|ref_loc| {
                    // from location to real def
                    // return if the real def location matches the def_loc
                    match file_path_from_url(&ref_loc.uri) {
                        Ok(file_path) => {
                            match compile_with_params(Params {
                                file: file_path.clone(),
                                module_cache: module_cache.clone(),
                                scope_cache: scope_cache.clone(),
                                vfs: vfs.clone(),
                            }) {
                                Ok((prog, _, gs)) => {
                                    let ref_pos = kcl_pos(&file_path, ref_loc.range.start);
                                    if *ref_loc == def_loc && !include_declaration {
                                        return false;
                                    }
                                    // find def from the ref_pos
                                    if let Some(real_def) =
                                        goto_definition_with_gs(&prog, &ref_pos, &gs)
                                    {
                                        match real_def {
                                            lsp_types::GotoDefinitionResponse::Scalar(
                                                real_def_loc,
                                            ) => real_def_loc == def_loc,
                                            _ => false,
                                        }
                                    } else {
                                        false
                                    }
                                }
                                Err(err) => {
                                    let _ =
                                        logger(format!("{file_path} compilation failed: {}", err));
                                    false
                                }
                            }
                        }
                        Err(err) => {
                            let _ = logger(format!("compilation failed: {}", err));
                            false
                        }
                    }
                })
                .collect();
            ref_locations.extend(matched_locs);
        }
    }
    ref_locations
}

#[cfg(test)]
mod tests {
    use super::find_refs_from_def;
    use crate::word_index::build_word_index;
    use lsp_types::{Location, Position, Range, Url};
    use parking_lot::RwLock;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn logger(msg: String) -> Result<(), anyhow::Error> {
        println!("{}", msg);
        anyhow::Ok(())
    }

    fn check_locations_match(expect: Vec<Location>, actual: Vec<Location>) {
        assert_eq!(expect, actual)
    }

    fn setup_word_index_map(root: &str) -> HashMap<Url, HashMap<String, Vec<Location>>> {
        HashMap::from([(
            Url::from_file_path(root).unwrap(),
            build_word_index(root, true).unwrap(),
        )])
    }

    #[test]
    fn find_refs_from_variable_test() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut path = root.clone();
        path.push("src/test_data/find_refs_test/main.k");
        let path = path.to_str().unwrap();

        match lsp_types::Url::from_file_path(path) {
            Ok(url) => {
                let def_loc = Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                };
                let expect = vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(1, 4),
                            end: Position::new(1, 5),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(2, 4),
                            end: Position::new(2, 5),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(12, 14),
                            end: Position::new(12, 15),
                        },
                    },
                ];
                check_locations_match(
                    expect,
                    find_refs_from_def(
                        None,
                        Arc::new(RwLock::new(setup_word_index_map(path))),
                        def_loc,
                        "a".to_string(),
                        true,
                        Some(20),
                        logger,
                        None,
                        None,
                    ),
                );
            }
            Err(_) => unreachable!("file not found"),
        }
    }

    #[test]
    fn find_refs_include_declaration_test() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut path = root.clone();
        path.push("src/test_data/find_refs_test/main.k");
        let path = path.to_str().unwrap();
        match lsp_types::Url::from_file_path(path) {
            Ok(url) => {
                let def_loc = Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                };
                let expect = vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(1, 4),
                            end: Position::new(1, 5),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(2, 4),
                            end: Position::new(2, 5),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(12, 14),
                            end: Position::new(12, 15),
                        },
                    },
                ];
                check_locations_match(
                    expect,
                    find_refs_from_def(
                        None,
                        Arc::new(RwLock::new(setup_word_index_map(path))),
                        def_loc,
                        "a".to_string(),
                        false,
                        Some(20),
                        logger,
                        None,
                        None,
                    ),
                );
            }
            Err(_) => unreachable!("file not found"),
        }
    }

    #[test]
    fn find_refs_from_schema_name_test() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut path = root.clone();
        path.push("src/test_data/find_refs_test/main.k");
        let path = path.to_str().unwrap();
        match lsp_types::Url::from_file_path(path) {
            Ok(url) => {
                let def_loc = Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(4, 7),
                        end: Position::new(4, 11),
                    },
                };
                let expect = vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(4, 7),
                            end: Position::new(4, 11),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(8, 7),
                            end: Position::new(8, 11),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(11, 7),
                            end: Position::new(11, 11),
                        },
                    },
                ];
                check_locations_match(
                    expect,
                    find_refs_from_def(
                        None,
                        Arc::new(RwLock::new(setup_word_index_map(path))),
                        def_loc,
                        "Name".to_string(),
                        true,
                        Some(20),
                        logger,
                        None,
                        None,
                    ),
                );
            }
            Err(_) => unreachable!("file not found"),
        }
    }

    #[test]
    fn find_refs_from_schema_attr_test() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut path = root.clone();
        path.push("src/test_data/find_refs_test/main.k");
        let path = path.to_str().unwrap();
        match lsp_types::Url::from_file_path(path) {
            Ok(url) => {
                let def_loc = Location {
                    uri: url.clone(),
                    range: Range {
                        start: Position::new(5, 4),
                        end: Position::new(5, 8),
                    },
                };
                let expect = vec![
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(5, 4),
                            end: Position::new(5, 8),
                        },
                    },
                    Location {
                        uri: url.clone(),
                        range: Range {
                            start: Position::new(12, 8),
                            end: Position::new(12, 12),
                        },
                    },
                ];
                check_locations_match(
                    expect,
                    find_refs_from_def(
                        None,
                        Arc::new(RwLock::new(setup_word_index_map(path))),
                        def_loc,
                        "name".to_string(),
                        true,
                        Some(20),
                        logger,
                        None,
                        None,
                    ),
                );
            }
            Err(_) => unreachable!("file not found"),
        }
    }
}
