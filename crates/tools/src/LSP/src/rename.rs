use crate::state::KCLVfs;
use crate::word_index::{VirtualLocation, build_virtual_word_index};
use crate::{from_lsp::kcl_pos, goto_def::find_def};
use anyhow::{Result, anyhow};
use kcl_ast::ast::{self, Program};
use kcl_error::diagnostic;
use kcl_parser::{LoadProgramOptions, ParseSessionRef, load_program};
use kcl_query::{path::parse_attribute_path, selector::parse_symbol_selector_spec};
use kcl_sema::{
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
    let f = if source_codes.keys().all(|path| path.starts_with("/")) {
        VfsPath::new_virtual_path
    } else {
        VfsPath::new_real_path
    };

    for (filepath, code) in &source_codes {
        vfs.write()
            .set_file_contents(f(filepath.clone()), Some(code.as_bytes().to_vec()));
    }
    let changes: HashMap<String, Vec<TextEdit>> =
        rename_symbol(pkg_root, vfs, symbol_path, new_name, f)?;
    apply_rename_changes(&changes, source_codes)
}

fn package_path_to_file_path(pkg_path: &str, vfs: KCLVfs) -> Vec<String> {
    let pkg = PathBuf::from(pkg_path);
    let vfs_read = vfs.read();
    let mut result: Vec<String> = vec![];

    // first search as directory(KCL package in the strict sense)
    result.extend(vfs_read.iter().filter_map(|(_, vfs_path)| {
        let path = PathBuf::from(vfs_path.to_string());
        if let Some(parent) = path.parent()
            && parent == pkg
        {
            return Some(vfs_path.to_string());
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
    ) && let Some(symbol_ref) = gs
        .get_symbols()
        .get_symbol_by_fully_qualified_name(kcl_ast::MAIN_PKG)
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
        kcl_sema::resolver::Options {
            merge_program: false,
            type_erasure: false,
            ..Default::default()
        },
        None,
    )?;

    let mut gs = GlobalState::default();
    Namer::find_symbols(&program, &mut gs);
    let node_ty_map = prog_scope.node_ty_map;
    AdvancedResolver::resolve_program(&program, &mut gs, node_ty_map)?;

    Ok((program, gs))
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

                #[allow(clippy::needless_range_loop)]
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

/// Find all the occurrences of the target symbol and return the text edit actions to rename them.
///
/// ## Parameters
/// - pkg_root: the absolute file path to the root package
/// - vfs: contains all the files and contents to be renamed
/// - symbol_path: path to the symbol. The symbol path should be in the format of: `pkg.sub_pkg:name.sub_name`
/// - new_name: the new name of the symbol
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
                            if let Some(symbol_ref) = find_def(&kcl_pos, &gs, true)
                                && let Some(symbol_def) = gs.get_symbols().get_symbol(symbol_ref)
                                && symbol_def.get_range() == range
                            {
                                refs.push(loc)
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
    use super::{
        apply_rename_changes, package_path_to_file_path, rename_symbol, rename_symbol_on_code,
        rename_symbol_on_file, select_symbol,
    };
    use crate::state::KCLVfs;
    use kcl_error::diagnostic;
    use lsp_types::{Position, Range, TextEdit};
    use ra_ap_vfs::VfsPath;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Removes `CARGO_MANIFEST_DIR` so snapshots stay stable across machines.
    fn relativize(path: &str) -> String {
        let normalized = path.replace('\\', "/");
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .display()
            .to_string()
            .replace('\\', "/");
        let root = root.trim_end_matches('/');
        if let Some(stripped) = normalized.strip_prefix(root) {
            return stripped.trim_start_matches('/').to_string();
        }
        if let Some(idx) = normalized.find("src/test_data/") {
            return normalized[idx..].to_string();
        }
        normalized
    }

    fn relativize_to(root: &Path, path: &str) -> String {
        let path_buf = PathBuf::from(path);
        if let Ok(rel) = path_buf.strip_prefix(root) {
            return rel.display().to_string().replace('\\', "/");
        }
        let root_s = root.display().to_string().replace('\\', "/");
        let root_s = root_s.trim_end_matches('/');
        let path_s = path.replace('\\', "/");
        if let Some(idx) = path_s.rfind(root_s) {
            return path_s[idx + root_s.len()..]
                .trim_start_matches('/')
                .to_string();
        }
        path_s
    }

    fn fixture_root(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
    }

    fn fixture_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                files.push(entry.into_path());
            }
        }
        files.sort();
        files
    }

    /// Loads every file under `rel` into a fresh VFS. Virtual paths back the
    /// in-memory helpers. Real paths back rename, which resolves imports from
    /// the fixture on disk.
    fn load_vfs(rel: &str, virtual_path: bool) -> (PathBuf, KCLVfs) {
        let root = fixture_root(rel);
        let vfs = KCLVfs::default();
        for path in fixture_files(&root) {
            let content =
                fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            let path_string = path.to_str().expect("utf-8 fixture path").to_string();
            let vfs_path = if virtual_path {
                VfsPath::new_virtual_path(path_string)
            } else {
                VfsPath::new_real_path(path_string)
            };
            vfs.write().set_file_contents(vfs_path, Some(content));
        }
        (root, vfs)
    }

    fn fmt_paths(files: &[String]) -> String {
        let mut files: Vec<_> = files.iter().map(|file| relativize(file)).collect();
        files.sort();
        if files.is_empty() {
            "no files".to_string()
        } else {
            files.join("\n")
        }
    }

    fn fmt_symbol(got: Option<(String, diagnostic::Range)>) -> String {
        let Some((name, (start, end))) = got else {
            return "None".to_string();
        };
        format!(
            "name: {name}\nstart: {}\nend: {}",
            fmt_diagnostic_pos(&start),
            fmt_diagnostic_pos(&end)
        )
    }

    fn fmt_diagnostic_pos(pos: &diagnostic::Position) -> String {
        match pos.column {
            Some(column) => format!("{}:{}:{column}", relativize(&pos.filename), pos.line),
            None => format!("{}:{}:-", relativize(&pos.filename), pos.line),
        }
    }

    fn fmt_text_edits(changes: &HashMap<String, Vec<TextEdit>>) -> String {
        let mut files: Vec<_> = changes.keys().cloned().collect();
        files.sort();
        if files.is_empty() {
            return "no edits".to_string();
        }
        let mut out = String::new();
        for file in files {
            out.push_str(&format!("file: {}\n", relativize(&file)));
            let mut edits = changes[&file].clone();
            edits.sort_by(|left, right| {
                (
                    left.range.start.line,
                    left.range.start.character,
                    left.range.end.line,
                    left.range.end.character,
                    &left.new_text,
                )
                    .cmp(&(
                        right.range.start.line,
                        right.range.start.character,
                        right.range.end.line,
                        right.range.end.character,
                        &right.new_text,
                    ))
            });
            for edit in edits {
                out.push_str(&format!(
                    "  {}:{}-{}:{} {:?}\n",
                    edit.range.start.line,
                    edit.range.start.character,
                    edit.range.end.line,
                    edit.range.end.character,
                    edit.new_text
                ));
            }
        }
        out
    }

    /// Renders renamed sources with `Debug` so a missing trailing newline stays visible.
    fn fmt_named_sources(rows: &[(String, String)]) -> String {
        let mut rows = rows.to_vec();
        rows.sort_by(|left, right| left.0.cmp(&right.0));
        let mut out = String::new();
        for (path, content) in rows {
            out.push_str(&path);
            out.push('\n');
            out.push_str(&format!("{content:?}"));
            out.push('\n');
        }
        out
    }

    fn fmt_renamed_sources(sources: &HashMap<String, String>) -> String {
        let rows: Vec<_> = sources
            .iter()
            .map(|(path, content)| (relativize(path), content.clone()))
            .collect();
        fmt_named_sources(&rows)
    }

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("kcl-rename-{name}-{}-{nanos}", std::process::id()));
            fs::create_dir_all(&path).unwrap_or_else(|err| panic!("create temp dir: {err}"));
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn copy_tree(src: &Path, dst: &Path) {
        for path in fixture_files(src) {
            let rel = path.strip_prefix(src).expect("file is under the fixture");
            let target = dst.join(rel);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .unwrap_or_else(|err| panic!("create {}: {err}", parent.display()));
            }
            fs::copy(&path, &target).unwrap_or_else(|err| {
                panic!("copy {} -> {}: {err}", path.display(), target.display())
            });
        }
    }

    /// Each test loads its own fixture directory into a new VFS and snapshots
    /// the helper result. Mirrors the `*_test_snapshot!` suites in this crate.
    #[macro_export]
    macro_rules! package_path_test_snapshot {
        ($name:ident, $fixture:expr, $pkg_rel:expr) => {
            #[test]
            fn $name() {
                let (root, vfs) = load_vfs($fixture, true);
                let pkg = root.join($pkg_rel);
                let files =
                    package_path_to_file_path(pkg.to_str().expect("utf-8 package path"), vfs);
                insta::assert_snapshot!(fmt_paths(&files));
            }
        };
    }

    #[macro_export]
    macro_rules! select_symbol_test_snapshot {
        ($name:ident, $fixture:expr, $pkgpath:expr, $field_path:expr) => {
            #[test]
            fn $name() {
                let (root, vfs) = load_vfs($fixture, true);
                let got = select_symbol(
                    &kcl_ast::ast::SymbolSelectorSpec {
                        pkg_root: root.to_str().expect("utf-8 fixture root").to_string(),
                        pkgpath: $pkgpath.to_string(),
                        field_path: $field_path.to_string(),
                    },
                    vfs,
                    ra_ap_vfs::VfsPath::new_virtual_path,
                );
                insta::assert_snapshot!(fmt_symbol(got));
            }
        };
    }

    #[macro_export]
    macro_rules! rename_symbol_test_snapshot {
        ($name:ident, $fixture:expr, $symbol_path:expr, $new_name:expr) => {
            #[test]
            fn $name() {
                let (root, vfs) = load_vfs($fixture, false);
                let changes = rename_symbol(
                    root.to_str().expect("utf-8 fixture root"),
                    vfs,
                    $symbol_path,
                    $new_name.to_string(),
                    ra_ap_vfs::VfsPath::new_real_path,
                )
                .unwrap_or_else(|err| panic!("rename failed: {err}"));
                insta::assert_snapshot!(fmt_text_edits(&changes));
            }
        };
    }

    #[macro_export]
    macro_rules! rename_on_code_test_snapshot {
        ($name:ident, $fixture:expr, $symbol_path:expr, $new_name:expr) => {
            #[test]
            fn $name() {
                let root = fixture_root($fixture);
                let mut source_codes = std::collections::HashMap::new();
                for path in fixture_files(&root) {
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
                    source_codes.insert(
                        path.to_str().expect("utf-8 fixture path").to_string(),
                        content,
                    );
                }
                let renamed = rename_symbol_on_code(
                    root.to_str().expect("utf-8 fixture root"),
                    $symbol_path,
                    source_codes,
                    $new_name.to_string(),
                )
                .unwrap_or_else(|err| panic!("rename failed: {err}"));
                insta::assert_snapshot!(fmt_renamed_sources(&renamed));
            }
        };
    }

    #[macro_export]
    macro_rules! rename_on_file_test_snapshot {
        ($name:ident, $fixture:expr, $symbol_path:expr, $new_name:expr) => {
            #[test]
            fn $name() {
                let src = fixture_root($fixture);
                let tmp = TempDir::new(stringify!($name));
                copy_tree(&src, &tmp.0);
                let file_paths: Vec<String> = fixture_files(&tmp.0)
                    .iter()
                    .map(|path| path.to_str().expect("utf-8 fixture path").to_string())
                    .collect();
                let changed = rename_symbol_on_file(
                    tmp.0.to_str().expect("utf-8 temp root"),
                    $symbol_path,
                    &file_paths,
                    $new_name.to_string(),
                )
                .unwrap_or_else(|err| panic!("rename failed: {err}"));
                let rows: Vec<(String, String)> = changed
                    .iter()
                    .map(|path| {
                        let content = std::fs::read_to_string(path)
                            .unwrap_or_else(|err| panic!("read {path}: {err}"));
                        (relativize_to(&tmp.0, path), content)
                    })
                    .collect();
                insta::assert_snapshot!(fmt_named_sources(&rows));
            }
        };
    }

    package_path_test_snapshot!(
        package_path_to_file_path_package,
        "src/test_data/rename/package_path_to_file_path_package",
        "base"
    );

    package_path_test_snapshot!(
        package_path_to_file_path_module,
        "src/test_data/rename/package_path_to_file_path_module",
        "base/person"
    );

    package_path_test_snapshot!(
        package_path_to_file_path_root_file,
        "src/test_data/rename/package_path_to_file_path_root_file",
        "config"
    );

    select_symbol_test_snapshot!(
        select_symbol_person_name,
        "src/test_data/rename/select_symbol_person_name",
        "base",
        "Person.name"
    );

    select_symbol_test_snapshot!(
        select_symbol_name_first,
        "src/test_data/rename/select_symbol_name_first",
        "base",
        "Name.first"
    );

    select_symbol_test_snapshot!(
        select_symbol_person,
        "src/test_data/rename/select_symbol_person",
        "base",
        "Person"
    );

    select_symbol_test_snapshot!(
        select_symbol_var_a,
        "src/test_data/rename/select_symbol_var_a",
        "base",
        "a"
    );

    select_symbol_test_snapshot!(
        select_symbol_server_name,
        "src/test_data/rename/select_symbol_server_name",
        "base",
        "Server.name"
    );

    select_symbol_test_snapshot!(
        select_symbol_missing,
        "src/test_data/rename/select_symbol_missing",
        "base",
        "name"
    );

    rename_symbol_test_snapshot!(
        rename_schema,
        "src/test_data/rename/rename_schema",
        "base:Person",
        "NewPerson"
    );

    rename_symbol_test_snapshot!(
        rename_schema_attr,
        "src/test_data/rename/rename_schema_attr",
        "base:Person.name",
        "new_name"
    );

    #[test]
    fn apply_rename_changes_single_line() {
        let path = fixture_root("src/test_data/rename/apply_rename_changes/main.k");
        let key = path.to_str().expect("utf-8 fixture path").to_string();
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let changes = HashMap::from([(
            key.clone(),
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
        )]);
        let result = apply_rename_changes(&changes, HashMap::from([(key.clone(), source)]))
            .unwrap_or_else(|err| panic!("apply rename failed: {err}"));
        insta::assert_snapshot!(format!("{:?}", result.get(&key).expect("renamed file")));
    }

    rename_on_code_test_snapshot!(
        rename_symbol_on_code_schema,
        "src/test_data/rename/rename_symbol_on_code_schema",
        "base:Person",
        "NewPerson"
    );

    rename_on_file_test_snapshot!(
        rename_symbol_on_file_schema,
        "src/test_data/rename/rename_symbol_on_file_schema",
        "base:Person",
        "NewPerson"
    );
}
