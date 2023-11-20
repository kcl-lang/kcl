use crate::{
    from_lsp::kcl_pos,
    goto_def::find_def_with_gs,
    util::{build_word_index_for_file_paths, parse_param_and_compile, read_file, Param},
};
use anyhow::{anyhow, Result};
use kclvm_ast::ast;
use kclvm_error::diagnostic;
use kclvm_query::selector::parse_symbol_selector_spec;
use lsp_types::{Location, Position, Range, TextEdit, Url};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

pub fn rename_symbol_on_file(
    pkg_root: &str,
    symbol_path: &str,
    file_paths: &[String],
    new_name: String,
) -> Result<Vec<String>> {
    let changes = rename_symbol(pkg_root, file_paths, symbol_path, new_name)?;
    let new_codes = apply_rename_changes(&changes);
    let mut changed_paths = vec![];
    for (path, content) in new_codes {
        fs::write(path.clone(), content)?;
        changed_paths.push(path.clone());
    }
    Ok(changed_paths)
}

fn apply_rename_changes(changes: &HashMap<Url, Vec<TextEdit>>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (url, edits) in changes {
        let file_content = read_file(&url.path().to_string()).unwrap();

        let file_content_lines: Vec<&str> = file_content.lines().collect();
        let mut updated_lines: Vec<String> = file_content_lines
            .iter()
            .map(|&line| line.to_string())
            .collect();

        let mut to_removed = HashSet::new();

        for edit in edits {
            let start_line = edit.range.start.line as usize;
            let end_line = edit.range.end.line as usize;

            if start_line == end_line {
                // the text edit belongs to a single line
                let line = &file_content_lines[start_line];
                let updated_line = apply_text_edit(edit, line);
                updated_lines[start_line] = updated_line;
            } else {
                let start_line_text = &file_content_lines[start_line];
                let end_line_text = &file_content_lines[end_line];
                let start_line_edit = TextEdit {
                    range: Range {
                        start: edit.range.start,
                        end: Position {
                            line: edit.range.start.line,
                            character: start_line_text.len() as u32,
                        },
                    },
                    new_text: edit.new_text.clone(),
                };
                let end_line_edit = TextEdit {
                    range: Range {
                        start: Position {
                            line: edit.range.end.line,
                            character: 0,
                        },
                        end: edit.range.end,
                    },
                    new_text: String::new(),
                };
                let updated_start_line = apply_text_edit(&start_line_edit, start_line_text);
                let updated_end_line = apply_text_edit(&end_line_edit, end_line_text);
                updated_lines[start_line] = format!("{}{}", updated_start_line, updated_end_line);

                for line_num in (start_line + 1)..end_line + 1 {
                    // todo, record lines to be deleted, instead of update to empty string
                    // from start+1 to end
                    updated_lines[line_num] = String::new();
                    to_removed.insert(line_num);
                }
            }
        }

        let retained_lines: Vec<_> = updated_lines
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !to_removed.contains(index))
            .map(|(_, item)| item.to_string())
            .collect();

        let new_file_content = retained_lines.join("\n");
        result.insert(url.path().to_string(), new_file_content);
    }
    result
}

/// apply_text_edit applys the text edit to a single line
fn apply_text_edit(edit: &TextEdit, line: &str) -> String {
    let range = edit.range;
    let start = range.start.character as usize;
    let end = range.end.character as usize;

    let mut updated_line = line.to_owned();
    updated_line.replace_range(start..end, &edit.new_text);
    updated_line
}

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
            let word_index = build_word_index_for_file_paths(file_paths, true)?;
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
                                module_cache: None,
                            },
                            None,
                        ) {
                            let kcl_pos = kcl_pos(p, loc.range.start);
                            if let Some(symbol_ref) = find_def_with_gs(&kcl_pos, &gs, true) {
                                if let Some(symbol_def) = gs.get_symbols().get_symbol(symbol_ref) {
                                    return symbol_def.get_range() == range;
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
                    module_cache: None,
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
                    return Some((target_symbol.get_name(), target_symbol.get_range().clone()));
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
    use lsp_types::ChangeAnnotation;
    use lsp_types::{Position, Range, TextEdit, Url};
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::PathBuf;

    use super::{apply_rename_changes, rename_symbol, rename_symbol_on_file, select_symbol};

    #[test]
    fn test_select_symbol() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("src/test_data/rename_test/");
        let pkg_root = root.to_str().unwrap().to_string();

        let mut main_path = root.clone();
        main_path = main_path.join("base").join("person.k");
        let mut server_path = root.clone();
        server_path = server_path.join("server.k");

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

        if let Some((name, range)) = select_symbol(&ast::SymbolSelectorSpec {
            pkg_root: pkg_root.clone(),
            pkgpath: "".to_string(),
            field_path: "Server.name".to_string(),
        }) {
            assert_eq!(name, "name");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: server_path.as_path().to_str().unwrap().to_string(),
                        line: 2,
                        column: Some(4),
                    },
                    diagnostic::Position {
                        filename: server_path.as_path().to_str().unwrap().to_string(),
                        line: 2,
                        column: Some(8),
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

    #[test]
    fn test_apply_rename_changes() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("src/test_data/rename_test/main.k");
        let path = root.to_str().unwrap().to_string();

        struct TestCase {
            changes: HashMap<Url, Vec<TextEdit>>,
            expected: String,
        }

        let test_cases = vec![
            TestCase {
                changes: HashMap::from([(
                    Url::from_file_path(path.clone()).unwrap(),
                    vec![TextEdit {
                        range: Range {
                            start: Position {
                                line: 2,
                                character: 11,
                            },
                            end: Position {
                                line: 2,
                                character: 17,
                            },
                        },
                        new_text: "Person2".to_string(),
                    }],
                )]),
                expected:
                    "import .pkg.vars\n\nBob = vars.Person2 {\n    name: \"Bob\"\n    age: 30\n}"
                        .to_string(),
            },
            TestCase {
                changes: HashMap::from([(
                    Url::from_file_path(path.clone()).unwrap(),
                    vec![TextEdit {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 2,
                                character: 6,
                            },
                        },
                        new_text: "".to_string(),
                    }],
                )]),
                expected: "vars.Person {\n    name: \"Bob\"\n    age: 30\n}".to_string(),
            },
            TestCase {
                changes: HashMap::from([(
                    Url::from_file_path(path.clone()).unwrap(),
                    vec![TextEdit {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 2,
                                character: 6,
                            },
                        },
                        new_text: "person = ".to_string(),
                    }],
                )]),
                expected: "person = vars.Person {\n    name: \"Bob\"\n    age: 30\n}".to_string(),
            },
        ];

        for test_case in test_cases {
            let result = apply_rename_changes(&test_case.changes);
            assert_eq!(result.get(&path).unwrap(), &test_case.expected);
        }
    }

    #[test]
    fn test_rename_symbol_on_file() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("src/test_data/rename_test/");

        let mut main_path = root.clone();
        let mut base_path = root.clone();
        base_path.push("base/person.k");
        main_path.push("config.k");
        let base_path_string = base_path.to_str().unwrap().to_string();
        let main_path_string = main_path.to_str().unwrap().to_string();

        // before test, back up the old file content
        for path in vec![base_path.clone(), main_path.clone()] {
            let content = fs::read_to_string(path.clone()).unwrap();
            let backup_path = path.with_extension("bak");
            fs::write(backup_path.clone(), content).unwrap();
        }

        let result = rename_symbol_on_file(
            root.to_str().unwrap(),
            "base:Person",
            &vec![base_path_string.clone(), main_path_string.clone()],
            "NewPerson".to_string(),
        );
        let expect_changed_paths: HashSet<_> = [base_path_string.clone(), main_path_string.clone()]
            .iter()
            .cloned()
            .collect();
        let got_changed_paths: HashSet<_> = result.unwrap().iter().cloned().collect();
        assert_eq!(expect_changed_paths, got_changed_paths);
        let base_new_content = fs::read_to_string(base_path.clone()).unwrap();
        let main_new_content = fs::read_to_string(main_path.clone()).unwrap();
        assert_eq!(
            base_new_content,
            r#"schema NewPerson:
    name: Name
    age: int

schema Name:
    first: str

a = {
    abc: "d"
}

d = a.abc
e = a["abc"]"#
        );
        assert_eq!(main_new_content, "import .base\n\na = base.NewPerson {\n    age: 1,\n    name: {\n        first: \"aa\"\n    }\n}");

        // after test, restore the old file content
        for path in vec![base_path.clone(), main_path.clone()] {
            let backup_path = path.with_extension("bak");
            let content = fs::read_to_string(backup_path.clone()).unwrap();
            fs::write(path.clone(), content).unwrap();
            fs::remove_file(backup_path.clone()).unwrap();
        }
    }
}
