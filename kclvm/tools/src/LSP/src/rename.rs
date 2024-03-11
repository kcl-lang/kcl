use crate::state::KCLVfs;
use crate::word_index::{build_virtual_word_index, VirtualLocation};
use crate::{from_lsp::kcl_pos, goto_def::find_def_with_gs};
use anyhow::{anyhow, Result};
use chumsky::chain::Chain;
use kclvm_ast::ast::{self, Program};
use kclvm_error::diagnostic;
use kclvm_parser::{load_program, LoadProgramOptions, ParseSessionRef};
use kclvm_query::{path::parse_attribute_path, selector::parse_symbol_selector_spec};
use kclvm_sema::{
    advanced_resolver::AdvancedResolver, core::global_state::GlobalState, namer::Namer,
    resolver::resolve_program_with_opts,
};
use lsp_types::{Position, Range, TextEdit};
use ra_ap_vfs::VfsPath;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// [`rename_symbol_on_file`] will rename the symbol in the given files
/// It will load the file content from file system and save to vfs, and then call [`rename_symbol`] to rename the symbol
pub fn rename_symbol_on_file(
    pkg_root: &str,
    symbol_path: &str,
    file_paths: &[String],
    new_name: String,
) -> Result<Vec<String>> {
    // load file content from file system and save to vfs
    let vfs = KCLVfs::default();
    let mut source_codes = HashMap::<String, String>::new();
    for path in file_paths {
        let content = fs::read_to_string(path.clone())?;
        vfs.write().set_file_contents(
            VfsPath::new_real_path(path.to_string()),
            Some(content.clone().into_bytes()),
        );
        source_codes.insert(path.to_string(), content.clone());
    }
    let changes = rename_symbol(pkg_root, vfs, symbol_path, new_name, VfsPath::new_real_path)?;
    let new_codes = apply_rename_changes(&changes, source_codes)?;
    let mut changed_paths = vec![];
    for (path, content) in new_codes.iter() {
        fs::write(path.clone(), content)?;
        changed_paths.push(path.clone());
    }
    Ok(changed_paths)
}

/// [`rename_symbol_on_code`] will rename the symbol in the given code
/// It will create a vfs from the file paths and codes, and then call [`rename_symbol`] to rename the symbol
pub fn rename_symbol_on_code(
    pkg_root: &str,
    symbol_path: &str,
    source_codes: HashMap<String, String>,
    new_name: String,
) -> Result<HashMap<String, String>> {
    // prepare a vfs from given file_paths
    let vfs = KCLVfs::default();
    for (filepath, code) in &source_codes {
        vfs.write().set_file_contents(
            VfsPath::new_virtual_path(filepath.clone()),
            Some(code.as_bytes().to_vec()),
        );
    }
    let changes: HashMap<String, Vec<TextEdit>> = rename_symbol(
        pkg_root,
        vfs,
        symbol_path,
        new_name,
        VfsPath::new_virtual_path,
    )?;
    apply_rename_changes(&changes, source_codes)
}

fn package_path_to_file_path(pkg_path: &str, vfs: KCLVfs) -> Vec<String> {
    let pkg = PathBuf::from(pkg_path);
    let vfs_read = vfs.read();
    let mut result: Vec<String> = vec![];

    // first search as directory(KCL package in the strict sense)
    result.extend(vfs_read.iter().filter_map(|(_, vfs_path)| {
        let path = PathBuf::from(vfs_path.to_string());
        if let Some(parent) = path.parent() {
            if parent == pkg {
                return Some(vfs_path.to_string());
            }
        }
        None
    }));

    if result.is_empty() {
        // then search as file(KCL module)
        result.extend(vfs_read.iter().filter_map(|(_, vfs_path)| {
            let path = PathBuf::from(vfs_path.to_string());
            if pkg.with_extension("k") == path {
                return Some(vfs_path.to_string());
            }
            None
        }));
    }

    result
}

/// Select a symbol by the symbol path
/// The symbol path should be in the format of: `pkg.sub_pkg:name.sub_name`
/// returns the symbol name and definition range
fn select_symbol<F>(
    symbol_spec: &ast::SymbolSelectorSpec,
    vfs: KCLVfs,
    trans_vfs_path: F,
) -> Option<(String, diagnostic::Range)>
where
    F: Fn(String) -> VfsPath,
{
    let mut pkg = PathBuf::from(&symbol_spec.pkg_root);
    let fields = parse_attribute_path(&symbol_spec.field_path).unwrap_or_default();
    if !symbol_spec.pkgpath.is_empty() {
        let pkg_names = symbol_spec.pkgpath.split('.');
        for n in pkg_names {
            pkg = pkg.join(n)
        }
    }
    let pkg_path = pkg.as_path().to_str().unwrap();

    let file_paths = package_path_to_file_path(pkg_path, vfs.clone());

    if let Ok((_, gs)) = parse_files_with_vfs(
        pkg_path.to_string(),
        file_paths,
        vfs.clone(),
        trans_vfs_path,
    ) {
        if let Some(symbol_ref) = gs
            .get_symbols()
            .get_symbol_by_fully_qualified_name(kclvm_ast::MAIN_PKG)
        {
            let mut owner_ref = symbol_ref;
            let mut target = None;
            for field in &fields {
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

fn parse_files_with_vfs<F>(
    work_dir: String,
    file_paths: Vec<String>,
    vfs: KCLVfs,
    trans_vfs_path: F,
) -> anyhow::Result<(Program, GlobalState)>
where
    F: Fn(String) -> VfsPath,
{
    let opts = LoadProgramOptions {
        work_dir,
        load_plugins: true,
        k_code_list: {
            let mut list = vec![];
            let vfs = &vfs.read();
            for file in &file_paths {
                if let Some(id) = vfs.file_id(&trans_vfs_path(file.clone())) {
                    // Load code from vfs
                    list.push(String::from_utf8(vfs.file_contents(id).to_vec()).unwrap());
                }
            }
            list
        },
        ..Default::default()
    };
    let files: Vec<&str> = file_paths.iter().map(|s| s.as_str()).collect();
    let sess: ParseSessionRef = ParseSessionRef::default();
    let mut program = load_program(sess.clone(), &files, Some(opts), None)?.program;

    let prog_scope = resolve_program_with_opts(
        &mut program,
        kclvm_sema::resolver::Options {
            merge_program: false,
            type_erasure: false,
            ..Default::default()
        },
        None,
    );

    let gs = GlobalState::default();
    let gs = Namer::find_symbols(&program, gs);
    let node_ty_map = prog_scope.node_ty_map.clone();
    let global_state = AdvancedResolver::resolve_program(&program, gs, node_ty_map);

    Ok((program, global_state))
}

fn apply_rename_changes(
    changes: &HashMap<String, Vec<TextEdit>>,
    source_codes: HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    for (file_path, edits) in changes {
        let file_content = source_codes
            .get(file_path)
            .ok_or(anyhow!("File content is None"))?
            .to_string();
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
        result.insert(file_path.to_string(), new_file_content);
    }
    Ok(result)
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

/// match_pkgpath_and_code matches the pkgpath and code from the symbol selector spec
pub fn match_pkgpath_and_code(
    selector: &ast::SymbolSelectorSpec,
) -> (Option<String>, Option<String>) {
    let mut pkg = PathBuf::from(&selector.pkg_root);
    let pkg_names = selector.pkgpath.split('.');
    if !selector.pkgpath.is_empty() {
        for n in pkg_names {
            pkg = pkg.join(n)
        }
    }

    match pkg.as_path().to_str() {
        Some(pkgpath) => (Some(pkgpath.to_string()), None),
        None => (None, None),
    }
}

/// the rename_symbol API
/// find all the occurrences of the target symbol and return the text edit actions to rename them
/// pkg_root: the absolute file path to the root package
/// vfs: contains all the files and contents to be renamed
/// symbol_path: path to the symbol. The symbol path should be in the format of: `pkg.sub_pkg:name.sub_name`
/// new_name: the new name of the symbol
pub fn rename_symbol<F>(
    pkg_root: &str,
    vfs: KCLVfs,
    symbol_path: &str,
    new_name: String,
    trans_vfs_path: F,
) -> Result<HashMap<String, Vec<TextEdit>>>
where
    F: Fn(String) -> VfsPath,
{
    // 1. from symbol path to the symbol
    let symbol_spec = parse_symbol_selector_spec(pkg_root, symbol_path)?;
    // 2. get the symbol name and definition range from symbol path
    match select_symbol(&symbol_spec, vfs.clone(), &trans_vfs_path) {
        Some((name, range)) => {
            // 3. build word index, find refs within given scope
            // vfs to source code contents
            let mut source_codes = HashMap::<String, String>::new();
            let vfs_content = vfs.read();
            for (file_id, vfspath) in vfs_content.iter() {
                let content = std::str::from_utf8(vfs_content.file_contents(file_id)).unwrap();
                source_codes.insert(vfspath.to_string(), content.to_string());
            }
            let word_index = build_virtual_word_index(source_codes, true)?;
            if let Some(locations) = word_index.get(&name) {
                // 4. filter out the matched refs
                // 4.1 collect matched words(names) and remove Duplicates of the file paths
                let file_map = locations.iter().fold(
                    HashMap::<String, Vec<&VirtualLocation>>::new(),
                    |mut acc, loc| {
                        acc.entry(loc.filepath.clone()).or_default().push(loc);
                        acc
                    },
                );
                let mut refs = vec![];
                for (fp, locs) in file_map.iter() {
                    if let Ok((_, gs)) = parse_files_with_vfs(
                        pkg_root.to_string(),
                        vec![fp.to_string()],
                        vfs.clone(),
                        &trans_vfs_path,
                    ) {
                        for loc in locs {
                            let kcl_pos = kcl_pos(fp, loc.range.start);
                            if let Some(symbol_ref) = find_def_with_gs(&kcl_pos, &gs, true) {
                                if let Some(symbol_def) = gs.get_symbols().get_symbol(symbol_ref) {
                                    if symbol_def.get_range() == range {
                                        refs.push(loc)
                                    }
                                }
                            }
                        }
                    };
                }
                // 5. refs to rename actions
                let changes = refs.into_iter().fold(HashMap::new(), |mut map, location| {
                    map.entry(location.filepath.clone())
                        .or_insert_with(Vec::new)
                        .push(TextEdit {
                            range: location.range,
                            new_text: new_name.clone(),
                        });
                    map
                });
                Ok(changes)
            } else {
                Ok(HashMap::new())
            }
        }
        None => Err(anyhow!(
            "get symbol from symbol path failed, {}",
            symbol_path
        )),
    }
}

#[cfg(test)]
mod tests {
    use kclvm_ast::ast;
    use kclvm_error::diagnostic;
    use lsp_types::{Position, Range, TextEdit};
    use maplit::hashmap;
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::PathBuf;

    use crate::rename::rename_symbol_on_code;

    use super::{
        apply_rename_changes, package_path_to_file_path, rename_symbol, rename_symbol_on_file,
        select_symbol,
    };

    use crate::state::KCLVfs;
    use ra_ap_vfs::VfsPath;

    /// prepare_vfs constructs a vfs for test:
    /// /mock_root
    /// ├── config.k
    /// └── base
    ///     ├── server.k
    ///     └── person.k
    fn prepare_vfs() -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf, KCLVfs) {
        // mock paths
        let root = PathBuf::from("/mock_root");
        let base_path = root.join("base");
        let server_path = root.join("base").join("server.k");
        let person_path = root.join("base").join("person.k");
        let config_path = root.join("config.k");
        // mock file contents
        let person_content = r#"schema Person:
    name: Name
    age: int

schema Name:
    first: str

a = {
    abc: "d"
}

d = a.abc
e = a["abc"]
"#;
        let server_content = r#"schema Server:
    name: str
"#;
        let config_content = r#""#;

        // set vfs
        let vfs = KCLVfs::default();
        vfs.write().set_file_contents(
            VfsPath::new_virtual_path(person_path.as_path().to_str().unwrap().to_string()),
            Some(person_content.as_bytes().to_owned()),
        );
        vfs.write().set_file_contents(
            VfsPath::new_virtual_path(server_path.as_path().to_str().unwrap().to_string()),
            Some(server_content.as_bytes().to_owned()),
        );
        vfs.write().set_file_contents(
            VfsPath::new_virtual_path(config_path.as_path().to_str().unwrap().to_string()),
            Some(config_content.as_bytes().to_owned()),
        );

        (root, base_path, person_path, server_path, config_path, vfs)
    }

    #[test]
    fn test_package_path_to_file_path() {
        let (root, base_path, person_path, server_path, config_path, vfs) = prepare_vfs();

        let files = package_path_to_file_path(base_path.as_path().to_str().unwrap(), vfs.clone());
        assert_eq!(
            files,
            vec![
                person_path.as_path().to_str().unwrap().to_string(),
                server_path.as_path().to_str().unwrap().to_string()
            ]
        );

        let files = package_path_to_file_path(
            root.join("base").join("person").as_path().to_str().unwrap(),
            vfs.clone(),
        );
        assert_eq!(
            files,
            vec![person_path.as_path().to_str().unwrap().to_string()]
        );

        let files =
            package_path_to_file_path(root.join("config").as_path().to_str().unwrap(), vfs.clone());
        assert_eq!(
            files,
            vec![config_path.as_path().to_str().unwrap().to_string()]
        );
    }

    #[test]
    fn test_select_symbol() {
        let (root, _, person_path, server_path, _config_path, vfs) = prepare_vfs();
        let pkg_root = root.as_path().to_str().unwrap().to_string();

        if let Some((name, range)) = select_symbol(
            &ast::SymbolSelectorSpec {
                pkg_root: pkg_root.clone(),
                pkgpath: "base".to_string(),
                field_path: "Person.name".to_string(),
            },
            vfs.clone(),
            VfsPath::new_virtual_path,
        ) {
            assert_eq!(name, "name");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 2,
                        column: Some(4),
                    },
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 2,
                        column: Some(8),
                    },
                )
            );
        } else {
            unreachable!("select symbol failed")
        }

        if let Some((name, range)) = select_symbol(
            &ast::SymbolSelectorSpec {
                pkg_root: pkg_root.clone(),
                pkgpath: "base".to_string(),
                field_path: "Name.first".to_string(),
            },
            vfs.clone(),
            VfsPath::new_virtual_path,
        ) {
            assert_eq!(name, "first");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 6,
                        column: Some(4),
                    },
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 6,
                        column: Some(9),
                    },
                )
            );
        } else {
            unreachable!("select symbol failed")
        }

        if let Some((name, range)) = select_symbol(
            &ast::SymbolSelectorSpec {
                pkg_root: pkg_root.clone(),
                pkgpath: "base".to_string(),
                field_path: "Person".to_string(),
            },
            vfs.clone(),
            VfsPath::new_virtual_path,
        ) {
            assert_eq!(name, "Person");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 1,
                        column: Some(7),
                    },
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 1,
                        column: Some(13),
                    },
                )
            );
        } else {
            unreachable!("select symbol failed")
        }

        if let Some((name, range)) = select_symbol(
            &ast::SymbolSelectorSpec {
                pkg_root: pkg_root.clone(),
                pkgpath: "base".to_string(),
                field_path: "a".to_string(),
            },
            vfs.clone(),
            VfsPath::new_virtual_path,
        ) {
            assert_eq!(name, "a");
            assert_eq!(
                range,
                (
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 8,
                        column: Some(0),
                    },
                    diagnostic::Position {
                        filename: person_path.as_path().to_str().unwrap().to_string(),
                        line: 8,
                        column: Some(1),
                    },
                )
            );
        } else {
            unreachable!("select symbol failed")
        }

        if let Some((name, range)) = select_symbol(
            &ast::SymbolSelectorSpec {
                pkg_root: pkg_root.clone(),
                pkgpath: "base".to_string(),
                field_path: "Server.name".to_string(),
            },
            vfs.clone(),
            VfsPath::new_virtual_path,
        ) {
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
            unreachable!("select symbol failed")
        }
    }

    #[test]
    fn test_select_symbol_failed() {
        let (root, _, _, _, _, vfs) = prepare_vfs();
        let pkg_root = root.as_path().to_str().unwrap().to_string();
        let result = select_symbol(
            &ast::SymbolSelectorSpec {
                pkg_root: pkg_root.clone(),
                pkgpath: "base".to_string(),
                field_path: "name".to_string(),
            },
            vfs.clone(),
            VfsPath::new_virtual_path,
        );
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

        let base_path = base_path.to_str().unwrap();
        let main_path = main_path.to_str().unwrap();

        let vfs = KCLVfs::default();
        for path in [base_path, main_path] {
            let content = fs::read_to_string(path).unwrap();
            vfs.write().set_file_contents(
                VfsPath::new_virtual_path(path.to_string()),
                Some(content.into_bytes()),
            );
        }

        if let Ok(changes) = rename_symbol(
            root.to_str().unwrap(),
            vfs.clone(),
            "base:Person",
            "NewPerson".to_string(),
            VfsPath::new_virtual_path,
        ) {
            assert_eq!(changes.len(), 2);
            assert!(changes.contains_key(base_path));
            assert!(changes.contains_key(main_path));
            assert!(changes.get(base_path).unwrap().len() == 1);
            assert!(changes.get(base_path).unwrap()[0].range.start == Position::new(0, 7));
            assert!(changes.get(main_path).unwrap().len() == 1);
            assert!(changes.get(main_path).unwrap()[0].range.start == Position::new(2, 9));
            assert!(changes.get(main_path).unwrap()[0].new_text == "NewPerson");
        } else {
            unreachable!("rename failed")
        }

        if let Ok(changes) = rename_symbol(
            root.to_str().unwrap(),
            vfs.clone(),
            "base:Person.name",
            "new_name".to_string(),
            VfsPath::new_virtual_path,
        ) {
            assert_eq!(changes.len(), 2);
            assert!(changes.contains_key(base_path));
            assert!(changes.contains_key(main_path));
            assert!(changes.get(base_path).unwrap().len() == 1);
            assert!(changes.get(base_path).unwrap()[0].range.start == Position::new(1, 4));
            assert!(changes.get(main_path).unwrap().len() == 1);
            assert!(changes.get(main_path).unwrap()[0].range.start == Position::new(4, 4));
            assert!(changes.get(main_path).unwrap()[0].new_text == "new_name");
        } else {
            unreachable!("rename failed")
        }
    }

    #[test]
    fn test_apply_rename_changes() {
        let path = "/mock_root/main.k".to_string();
        let mut source_codes = HashMap::new();
        source_codes.insert(
            path.clone(),
            r#"import .pkg.vars

Bob = vars.Person {
    name: "Bob"
    age: 30
}"#
            .to_string(),
        );

        struct TestCase {
            changes: HashMap<String, Vec<TextEdit>>,
            expected: String,
        }

        let test_cases = vec![TestCase {
            changes: HashMap::from([(
                path.clone(),
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
            expected: "import .pkg.vars\n\nBob = vars.Person2 {\n    name: \"Bob\"\n    age: 30\n}"
                .to_string(),
        }];

        for test_case in test_cases {
            let result = apply_rename_changes(&test_case.changes, source_codes.clone());
            assert_eq!(result.unwrap().get(&path).unwrap(), &test_case.expected);
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
        for path in [base_path.clone(), main_path.clone()] {
            let content = fs::read_to_string(path.clone()).unwrap();
            let backup_path = path.with_extension("bak");
            fs::write(backup_path.clone(), content).unwrap();
        }

        let result = rename_symbol_on_file(
            root.to_str().unwrap(),
            "base:Person",
            &[base_path_string.clone(), main_path_string.clone()],
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
        for path in [base_path.clone(), main_path.clone()] {
            let backup_path = path.with_extension("bak");
            let content = fs::read_to_string(backup_path.clone()).unwrap();
            fs::write(path.clone(), content).unwrap();
            fs::remove_file(backup_path.clone()).unwrap();
        }
    }

    #[test]
    fn test_rename_symbol_on_code() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        root.push("src/test_data/rename_test/");

        let mut base_path = root.clone();
        let mut main_path = root.clone();

        base_path.push("base/person.k");
        main_path.push("config.k");

        let base_path_string = base_path.to_str().unwrap().to_string();
        let main_path_string = main_path.to_str().unwrap().to_string();

        let base_source_code = r#"schema Person:
    name: Name
    age: int

schema Name:
    first: str

a = {
    abc: "d"
}

d = a.abc
e = a["abc"]"#;

        let main_source_code = r#"import .base

a = base.Person {
    age: 1,
    name: {
        first: "aa"
    }
}"#;

        let result: HashMap<String, String> = rename_symbol_on_code(
            root.to_str().unwrap(),
            "base:Person",
            hashmap! {
                base_path_string.clone() => base_source_code.to_string(),
                main_path_string.clone() => main_source_code.to_string(),
            },
            "NewPerson".to_string(),
        )
        .unwrap();

        let base_new_content = result.get(base_path_string.clone().as_str()).unwrap();
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

        let main_new_content = result.get(main_path_string.clone().as_str()).unwrap();
        assert_eq!(
            main_new_content,
            r#"import .base

a = base.NewPerson {
    age: 1,
    name: {
        first: "aa"
    }
}"#
        );
    }
}
