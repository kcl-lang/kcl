use crate::from_lsp::kcl_pos;
use crate::goto_def::{find_def, goto_definition};
use crate::to_lsp::lsp_location;
use crate::util::{parse_param_and_compile, Param};
use anyhow;
use kclvm_ast::ast::{Program, Stmt};
use kclvm_error::Position as KCLPos;
use kclvm_sema::resolver::scope::ProgramScope;
use lsp_types::{Location, Url};
use parking_lot::RwLock;
use ra_ap_vfs::Vfs;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) fn find_refs<F: Fn(String) -> Result<(), anyhow::Error>>(
    program: &Program,
    kcl_pos: &KCLPos,
    prog_scope: &ProgramScope,
    word_index_map: Arc<RwLock<HashMap<Url, HashMap<String, Vec<Location>>>>>,
    vfs: Option<Arc<RwLock<Vfs>>>,
    logger: F,
) -> Result<Vec<Location>, String> {
    // find the definition at the position
    let def = match program.pos_to_stmt(kcl_pos) {
        Some(node) => match node.node {
            Stmt::Import(_) => None,
            _ => find_def(node.clone(), kcl_pos, prog_scope),
        },
        None => None,
    };
    if def.is_none() {
        return Err(String::from(
            "Definition item not found, result in no reference",
        ));
    }
    let def = def.unwrap();
    if def.get_positions().len() > 1 {
        return Err(String::from(
            "Found more than one definitions, reference not supported",
        ));
    }
    let (start, end) = def.get_positions().iter().next().unwrap().clone();
    // find all the refs of the def
    if let Some(def_loc) = lsp_location(start.filename.clone(), &start, &end) {
        Ok(find_refs_from_def(
            vfs,
            word_index_map,
            def_loc,
            def.get_name(),
            logger,
        ))
    } else {
        Err(format!("Invalid file path: {0}", start.filename))
    }
}

pub(crate) fn find_refs_from_def<F: Fn(String) -> Result<(), anyhow::Error>>(
    vfs: Option<Arc<RwLock<Vfs>>>,
    word_index_map: Arc<RwLock<HashMap<Url, HashMap<String, Vec<Location>>>>>,
    def_loc: Location,
    name: String,
    logger: F,
) -> Vec<Location> {
    let mut ref_locations = vec![];
    for (_, word_index) in &mut *word_index_map.write() {
        if let Some(locs) = word_index.get(name.as_str()).cloned() {
            let matched_locs: Vec<Location> = locs
                .into_iter()
                .filter(|ref_loc| {
                    // from location to real def
                    // return if the real def location matches the def_loc
                    let file_path = ref_loc.uri.path().to_string();
                    match parse_param_and_compile(
                        Param {
                            file: file_path.clone(),
                        },
                        vfs.clone(),
                    ) {
                        Ok((prog, scope, _, _gs)) => {
                            let ref_pos = kcl_pos(&file_path, ref_loc.range.start);
                            // find def from the ref_pos
                            if let Some(real_def) = goto_definition(&prog, &ref_pos, &scope) {
                                match real_def {
                                    lsp_types::GotoDefinitionResponse::Scalar(real_def_loc) => {
                                        real_def_loc == def_loc
                                    }
                                    _ => false,
                                }
                            } else {
                                false
                            }
                        }
                        Err(_) => {
                            let file_path = def_loc.uri.path();
                            let _ = logger(format!("{file_path} compilation failed"));
                            return false;
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
    use crate::util::build_word_index;
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
            build_word_index(root.to_string(), true).unwrap(),
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
                        logger,
                    ),
                );
            }
            Err(_) => assert!(false, "file not found"),
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
                        start: Position::new(4, 0),
                        end: Position::new(7, 0),
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
                        logger,
                    ),
                );
            }
            Err(_) => assert!(false, "file not found"),
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
                        logger,
                    ),
                );
            }
            Err(_) => assert!(false, "file not found"),
        }
    }
}
