/*

                core::Namer                          basic_resolver
                     │                                     │
                     ▼                                     ▼
          ┌─────────────────────┐               ┌─────────────────────┐
          │ core::GlobalState   │               │  ast_node_type_map  │
          └─────────────────────┘               └─────────────────────┘
                     │                                     │
                     ▼                                     ▼
  ┌──────────────────────────────────────────────────────────────────────────────┐
  │        	                    advanced_resolver                                │
  ├──────────────────────────────────────────────────────────────────────────────┤
  │        ┌─────────────────┐                                                   │
  │        │ ast::Expression │                                                   │
  │        └─────────────────┘                                                   │
  │                 │                                                            │
  │                 │ resolve_local_value                                        │
  │                 ▼                                                            │
  │        ┌─────────────────┐                                                   │
  │        │ ast::Expression │                                                   │
  │        └─────────────────┘                                                   │
  │                 │                                                            │
  │                 │ resolve_symbol_ref (map Expression to DefinitionSymbols)   |
  │                 ▼                                                            │
  │        ┌─────────────────┐                                                   │
  │        │ ast::Expression │                                                   │
  │        └─────────────────┘                                                   │
  └──────────────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼   build_sema_db (collect symbol locs and analyse scope)
                        ┌─────────────────────┐
                        │ core::GlobalState   │
                        └─────────────────────┘
*/

use indexmap::IndexSet;
use kclvm_error::Position;

use crate::{
    core::{
        global_state::GlobalState,
        package::ModuleInfo,
        scope::{LocalSymbolScope, LocalSymbolScopeKind, RootSymbolScope, ScopeKind, ScopeRef},
        symbol::SymbolRef,
    },
    resolver::scope::{NodeKey, NodeTyMap},
};

use kclvm_ast::ast::AstIndex;
use kclvm_ast::ast::Program;
use kclvm_ast::walker::MutSelfTypedResultWalker;
mod node;

///  AdvancedResolver mainly does two tasks:
///  1: Traverse AST to parse LocalSymbol and store it in GlobalState, while storing the parsed type in Symbol
///  2: Establish a mapping between expressions and SymbolRef, specifically injecting symbol information into AST
///
///  After the work of the advanced resolver is completed, the GlobalState will build the whole semantic database,
///  so that toolchain can query semantic information about the AST
pub struct AdvancedResolver<'ctx> {
    pub(crate) ctx: Context<'ctx>,
    pub(crate) gs: GlobalState,
}

pub struct Context<'ctx> {
    pub program: &'ctx Program,
    node_ty_map: NodeTyMap,
    scopes: Vec<ScopeRef>,
    current_pkgpath: Option<String>,
    current_filename: Option<String>,
    current_schema_symbol: Option<SymbolRef>,
    start_pos: Position,
    end_pos: Position,
    cur_node: AstIndex,

    // whether the identifier currently being visited may be a definition
    // it will only be true when visiting a l-value or parameter,
    // which means advanced resolver will will create the corresponding
    // ValueSymbol instead of an UnresolvedSymbol
    maybe_def: bool,
}

impl<'ctx> Context<'ctx> {
    pub fn get_node_key(&self, id: &AstIndex) -> NodeKey {
        NodeKey {
            pkgpath: self.current_pkgpath.clone().unwrap(),
            id: id.clone(),
        }
    }
}

impl<'ctx> AdvancedResolver<'ctx> {
    pub fn resolve_program(
        program: &'ctx Program,
        gs: GlobalState,
        node_ty_map: NodeTyMap,
    ) -> GlobalState {
        let mut advanced_resolver = Self {
            gs,
            ctx: Context {
                program,
                node_ty_map,
                scopes: vec![],
                current_filename: None,
                current_pkgpath: None,
                current_schema_symbol: None,
                start_pos: Position::dummy_pos(),
                end_pos: Position::dummy_pos(),
                cur_node: AstIndex::default(),
                maybe_def: false,
            },
        };

        for (name, modules) in advanced_resolver.ctx.program.pkgs.iter() {
            advanced_resolver.ctx.current_pkgpath = Some(name.clone());
            if let Some(pkg_info) = advanced_resolver.gs.get_packages().get_package_info(name) {
                if modules.is_empty() {
                    continue;
                }
                if !advanced_resolver.ctx.scopes.is_empty() {
                    advanced_resolver.ctx.scopes.clear();
                }
                advanced_resolver.enter_root_scope(
                    name.clone(),
                    pkg_info.pkg_filepath.clone(),
                    pkg_info.kfile_paths.clone(),
                );
                for module in modules.iter() {
                    advanced_resolver.ctx.current_filename = Some(module.filename.clone());
                    advanced_resolver.walk_module(module);
                }
                advanced_resolver.leave_scope()
            }
        }

        advanced_resolver.gs.build_sema_db();
        advanced_resolver.gs
    }

    fn enter_root_scope(
        &mut self,
        pkgpath: String,
        filename: String,
        kfile_paths: IndexSet<String>,
    ) {
        let package_ref = self
            .gs
            .get_symbols_mut()
            .get_symbol_by_fully_qualified_name(&pkgpath)
            .unwrap();

        let root_scope = RootSymbolScope::new(pkgpath, filename, package_ref, kfile_paths);
        let scope_ref = self.gs.get_scopes_mut().alloc_root_scope(root_scope);
        self.ctx.scopes.push(scope_ref);
    }

    fn enter_local_scope(
        &mut self,
        filepath: &str,
        start: Position,
        end: Position,
        kind: LocalSymbolScopeKind,
    ) {
        let parent = *self.ctx.scopes.last().unwrap();
        let local_scope = LocalSymbolScope::new(parent, start, end, kind);
        let scope_ref = self.gs.get_scopes_mut().alloc_local_scope(local_scope);

        match parent.get_kind() {
            ScopeKind::Root => {
                self.gs
                    .get_scopes_mut()
                    .roots
                    .get_mut(parent.get_id())
                    .unwrap()
                    .add_child(filepath, scope_ref);
            }
            ScopeKind::Local => {
                self.gs
                    .get_scopes_mut()
                    .locals
                    .get_mut(parent.get_id())
                    .unwrap()
                    .add_child(scope_ref);
            }
        }
        self.ctx.scopes.push(scope_ref);
    }

    fn leave_scope(&mut self) {
        self.ctx.scopes.pop();
    }

    fn get_current_module_info(&self) -> Option<&ModuleInfo> {
        self.gs
            .get_packages()
            .get_module_info(self.ctx.current_filename.as_ref()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::advanced_resolver::AdvancedResolver;
    use crate::core::global_state::GlobalState;
    use crate::core::symbol::SymbolKind;
    use crate::namer::Namer;
    use crate::resolver;

    use kclvm_error::Position;
    use kclvm_parser::load_program;
    use kclvm_parser::ParseSession;
    use std::path::Path;
    use std::sync::Arc;

    #[cfg(not(target_os = "windows"))]
    fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
        p.as_ref().display().to_string()
    }

    #[cfg(target_os = "windows")]
    fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
        const VERBATIM_PREFIX: &str = r#"\\?\"#;
        let p = p.as_ref().display().to_string();
        if p.starts_with(VERBATIM_PREFIX) {
            p[VERBATIM_PREFIX.len()..].to_string()
        } else {
            p
        }
    }

    #[allow(unused)]
    fn print_symbols_info(gs: &GlobalState) {
        let base_path = Path::new(".").canonicalize().unwrap();
        let symbols = gs.get_symbols();
        println!("vec![");
        for (key, val) in gs.sema_db.file_sema_map.iter() {
            let key_path = Path::new(key)
                .strip_prefix(base_path.clone())
                .unwrap_or_else(|_| Path::new(key))
                .to_str()
                .unwrap()
                .to_string();
            println!("    (\n        \"{}\".to_string().replace(\"/\", &std::path::MAIN_SEPARATOR.to_string()),", key_path);
            println!("        vec![");
            for symbol_ref in val.symbols.iter() {
                let symbol = symbols.get_symbol(*symbol_ref).unwrap();
                let (start, end) = symbol.get_range();
                println!(
                    "            ({},{},{},{},\"{}\".to_string(),SymbolKind::{:?}),",
                    start.line,
                    start.column.unwrap_or(0),
                    end.line,
                    end.column.unwrap_or(0),
                    symbol.get_name(),
                    symbol_ref.get_kind(),
                );
                if let SymbolKind::Unresolved = symbol_ref.get_kind() {
                    let def_symbol_ref = symbol.get_definition().unwrap();
                    let def_symbol = symbols.get_symbol(def_symbol_ref).unwrap();
                    let (def_start, def_end) = def_symbol.get_range();
                    let def_path = Path::new(&def_start.filename)
                        .strip_prefix(base_path.clone())
                        .unwrap_or_else(|_| Path::new(&def_start.filename))
                        .to_str()
                        .unwrap()
                        .to_string();
                    println!(
                        "            ({},{},{},{},\"{}\".to_string().replace(\"/\", &std::path::MAIN_SEPARATOR.to_string()),SymbolKind::{:?}),",
                        def_start.line,
                        def_start.column.unwrap_or(0),
                        def_end.line,
                        def_end.column.unwrap_or(0),
                        def_path,
                        def_symbol_ref.get_kind(),
                    );
                }
            }
            println!("        ],\n    ),")
        }
        println!("]");
    }

    #[test]
    fn test_look_up_exact_symbol() {
        let sess = Arc::new(ParseSession::default());

        let path = "src/advanced_resolver/test_data/schema_symbols.k"
            .to_string()
            .replace("/", &std::path::MAIN_SEPARATOR.to_string());
        let mut program = load_program(sess.clone(), &[&path], None, None)
            .unwrap()
            .program;
        let gs = GlobalState::default();
        let gs = Namer::find_symbols(&program, gs);

        let node_ty_map = resolver::resolve_program_with_opts(
            &mut program,
            resolver::Options {
                merge_program: false,
                type_erasure: false,
                ..Default::default()
            },
            None,
        )
        .node_ty_map;
        let gs = AdvancedResolver::resolve_program(&program, gs, node_ty_map);
        let base_path = Path::new(".").canonicalize().unwrap();
        // print_symbols_info(&gs);
        let except_symbols = vec![
            (
                "src/advanced_resolver/test_data/import_test/e.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![
                    (1, 7, 1, 16, "UnionType".to_string(), SymbolKind::Schema),
                    (2, 4, 2, 5, "a".to_string(), SymbolKind::Attribute),
                ],
            ),
            (
                "src/advanced_resolver/test_data/import_test/a.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![
                    (1, 0, 1, 2, "_a".to_string(), SymbolKind::Value),
                    (2, 7, 2, 11, "Name".to_string(), SymbolKind::Schema),
                    (3, 4, 3, 13, "firstName".to_string(), SymbolKind::Attribute),
                    (4, 4, 4, 12, "lastName".to_string(), SymbolKind::Attribute),
                    (6, 7, 6, 13, "Person".to_string(), SymbolKind::Schema),
                    (7, 4, 7, 8, "name".to_string(), SymbolKind::Attribute),
                    (7, 10, 7, 14, "Name".to_string(), SymbolKind::Unresolved),
                    (
                        2,
                        7,
                        2,
                        11,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (8, 4, 8, 7, "age".to_string(), SymbolKind::Attribute),
                    (10, 0, 10, 7, "_person".to_string(), SymbolKind::Value),
                    (10, 10, 10, 16, "Person".to_string(), SymbolKind::Unresolved),
                    (
                        6,
                        7,
                        6,
                        13,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (11, 4, 11, 8, "name".to_string(), SymbolKind::Unresolved),
                    (
                        7,
                        4,
                        7,
                        8,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (11, 11, 11, 15, "Name".to_string(), SymbolKind::Unresolved),
                    (
                        2,
                        7,
                        2,
                        11,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (
                        12,
                        8,
                        12,
                        17,
                        "firstName".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        3,
                        4,
                        3,
                        13,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (
                        13,
                        8,
                        13,
                        16,
                        "lastName".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        4,
                        4,
                        4,
                        12,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (15, 4, 15, 7, "age".to_string(), SymbolKind::Unresolved),
                    (
                        8,
                        4,
                        8,
                        7,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                ],
            ),
            (
                "src/advanced_resolver/test_data/import_test/d.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![
                    (1, 7, 1, 13, "Parent".to_string(), SymbolKind::Schema),
                    (2, 4, 2, 8, "age1".to_string(), SymbolKind::Attribute),
                ],
            ),
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![
                    (
                        1,
                        7,
                        1,
                        20,
                        "import_test.a".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/a"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        2,
                        7,
                        2,
                        20,
                        "import_test.b".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/b"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        3,
                        7,
                        3,
                        20,
                        "import_test.c".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/c"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        4,
                        7,
                        4,
                        20,
                        "import_test.d".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/d"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        5,
                        7,
                        5,
                        20,
                        "import_test.e".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/e"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        6,
                        24,
                        6,
                        25,
                        "import_test.f".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/f"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (7, 7, 7, 10, "pkg".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/pkg"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (8, 7, 8, 12, "regex".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        0,
                        1,
                        0,
                        "".to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (10, 7, 10, 11, "Main".to_string(), SymbolKind::Schema),
                    (10, 12, 10, 13, "d".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/d"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (10, 14, 10, 20, "Parent".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        7,
                        1,
                        13,
                        "src/advanced_resolver/test_data/import_test/d.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (11, 11, 11, 12, "c".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/c"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        11,
                        13,
                        11,
                        24,
                        "TestOfMixin".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        1,
                        7,
                        1,
                        18,
                        "src/advanced_resolver/test_data/import_test/c.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (12, 4, 12, 8, "name".to_string(), SymbolKind::Attribute),
                    (13, 4, 13, 7, "age".to_string(), SymbolKind::Attribute),
                    (14, 4, 14, 10, "person".to_string(), SymbolKind::Attribute),
                    (14, 13, 14, 14, "a".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/a"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (14, 15, 14, 21, "Person".to_string(), SymbolKind::Unresolved),
                    (
                        6,
                        7,
                        6,
                        13,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (
                        15,
                        4,
                        15,
                        19,
                        "list_union_type".to_string(),
                        SymbolKind::Attribute,
                    ),
                    (15, 23, 15, 24, "e".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/e"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        15,
                        25,
                        15,
                        34,
                        "UnionType".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        1,
                        7,
                        1,
                        16,
                        "src/advanced_resolver/test_data/import_test/e.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (
                        16,
                        4,
                        16,
                        19,
                        "dict_union_type".to_string(),
                        SymbolKind::Attribute,
                    ),
                    (16, 23, 16, 24, "g".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/f"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        16,
                        25,
                        16,
                        34,
                        "UnionType".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        1,
                        7,
                        1,
                        16,
                        "src/advanced_resolver/test_data/import_test/f.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (19, 8, 19, 13, "regex".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        0,
                        1,
                        0,
                        "".to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (19, 14, 19, 19, "match".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        0,
                        1,
                        0,
                        "".to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (19, 20, 19, 24, "name".to_string(), SymbolKind::Unresolved),
                    (
                        12,
                        4,
                        12,
                        8,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (19, 97, 19, 101, "name".to_string(), SymbolKind::Unresolved),
                    (
                        12,
                        4,
                        12,
                        8,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (21, 3, 21, 4, "a".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/a"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (21, 5, 21, 7, "_a".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        0,
                        1,
                        2,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (22, 4, 22, 6, "_c".to_string(), SymbolKind::Value),
                    (23, 5, 23, 6, "a".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/a"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (23, 7, 23, 9, "_a".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        0,
                        1,
                        2,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (24, 4, 24, 6, "_c".to_string(), SymbolKind::Unresolved),
                    (
                        22,
                        4,
                        22,
                        6,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (26, 4, 26, 6, "_c".to_string(), SymbolKind::Unresolved),
                    (
                        22,
                        4,
                        22,
                        6,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (28, 0, 28, 1, "p".to_string(), SymbolKind::Value),
                    (28, 4, 28, 8, "Main".to_string(), SymbolKind::Unresolved),
                    (
                        10,
                        7,
                        10,
                        11,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (29, 4, 29, 8, "name".to_string(), SymbolKind::Unresolved),
                    (
                        12,
                        4,
                        12,
                        8,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (29, 11, 29, 12, "a".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/a"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        29,
                        13,
                        29,
                        20,
                        "_person".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        10,
                        0,
                        10,
                        7,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (29, 21, 29, 25, "name".to_string(), SymbolKind::Unresolved),
                    (
                        7,
                        4,
                        7,
                        8,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (
                        29,
                        26,
                        29,
                        35,
                        "firstName".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        3,
                        4,
                        3,
                        13,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (29, 45, 29, 46, "a".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/a"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        29,
                        47,
                        29,
                        54,
                        "_person".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        10,
                        0,
                        10,
                        7,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (29, 56, 29, 60, "name".to_string(), SymbolKind::Unresolved),
                    (
                        7,
                        4,
                        7,
                        8,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (
                        29,
                        61,
                        29,
                        69,
                        "lastName".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        4,
                        4,
                        4,
                        12,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (30, 4, 30, 7, "age".to_string(), SymbolKind::Unresolved),
                    (
                        13,
                        4,
                        13,
                        7,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (30, 10, 30, 11, "b".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/b"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (30, 12, 30, 14, "_b".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        0,
                        1,
                        2,
                        "src/advanced_resolver/test_data/import_test/b.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (30, 17, 30, 18, "a".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/import_test/a"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (
                        30,
                        19,
                        30,
                        26,
                        "_person".to_string(),
                        SymbolKind::Unresolved,
                    ),
                    (
                        10,
                        0,
                        10,
                        7,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                    (30, 28, 30, 31, "age".to_string(), SymbolKind::Unresolved),
                    (
                        8,
                        4,
                        8,
                        7,
                        "src/advanced_resolver/test_data/import_test/a.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (33, 0, 33, 6, "person".to_string(), SymbolKind::Value),
                    (33, 9, 33, 12, "pkg".to_string(), SymbolKind::Unresolved),
                    (
                        0,
                        0,
                        0,
                        0,
                        "src/advanced_resolver/test_data/pkg"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Package,
                    ),
                    (33, 13, 33, 19, "Person".to_string(), SymbolKind::Unresolved),
                    (
                        4,
                        7,
                        4,
                        13,
                        "src/advanced_resolver/test_data/pkg/pkg.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (34, 4, 34, 8, "name".to_string(), SymbolKind::Unresolved),
                    (
                        5,
                        4,
                        5,
                        8,
                        "src/advanced_resolver/test_data/pkg/pkg.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (34, 9, 34, 13, "name".to_string(), SymbolKind::Unresolved),
                    (
                        2,
                        4,
                        2,
                        8,
                        "src/advanced_resolver/test_data/pkg/pkg.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                    (37, 0, 37, 1, "x".to_string(), SymbolKind::Value),
                    (38, 16, 38, 17, "x".to_string(), SymbolKind::Unresolved),
                    (
                        37,
                        0,
                        37,
                        1,
                        "src/advanced_resolver/test_data/schema_symbols.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Value,
                    ),
                ],
            ),
            (
                "src/advanced_resolver/test_data/import_test/f.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![
                    (1, 7, 1, 16, "UnionType".to_string(), SymbolKind::Schema),
                    (2, 4, 2, 5, "b".to_string(), SymbolKind::Attribute),
                ],
            ),
            (
                "src/advanced_resolver/test_data/import_test/c.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![
                    (1, 7, 1, 18, "TestOfMixin".to_string(), SymbolKind::Schema),
                    (2, 4, 2, 7, "age".to_string(), SymbolKind::Attribute),
                ],
            ),
            (
                "src/advanced_resolver/test_data/pkg/pkg.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![
                    (1, 7, 1, 11, "Name".to_string(), SymbolKind::Schema),
                    (2, 4, 2, 8, "name".to_string(), SymbolKind::Attribute),
                    (4, 7, 4, 13, "Person".to_string(), SymbolKind::Schema),
                    (5, 4, 5, 8, "name".to_string(), SymbolKind::Attribute),
                    (5, 10, 5, 14, "Name".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        7,
                        1,
                        11,
                        "src/advanced_resolver/test_data/pkg/pkg.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (5, 17, 5, 21, "Name".to_string(), SymbolKind::Unresolved),
                    (
                        1,
                        7,
                        1,
                        11,
                        "src/advanced_resolver/test_data/pkg/pkg.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Schema,
                    ),
                    (5, 23, 5, 27, "name".to_string(), SymbolKind::Unresolved),
                    (
                        2,
                        4,
                        2,
                        8,
                        "src/advanced_resolver/test_data/pkg/pkg.k"
                            .to_string()
                            .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                        SymbolKind::Attribute,
                    ),
                ],
            ),
            (
                "src/advanced_resolver/test_data/import_test/b.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                vec![(1, 0, 1, 2, "_b".to_string(), SymbolKind::Value)],
            ),
        ];
        let mut skip_def_info = false;
        for (filepath, symbols) in except_symbols.iter() {
            let abs_filepath = adjust_canonicalization(base_path.join(filepath));
            // symbols will be sorted according to their position in the file
            // now we check all symbols
            for (index, symbol_info) in symbols.iter().enumerate() {
                if skip_def_info {
                    skip_def_info = false;
                    continue;
                }
                let (start_line, start_col, end_line, end_col, name, kind) = symbol_info;
                if abs_filepath.is_empty() {
                    continue;
                }
                // test look up symbols
                let inner_pos = Position {
                    filename: abs_filepath.clone(),
                    line: (start_line + end_line) / 2,
                    column: Some((start_col + end_col) / 2),
                };
                let looked_symbol_ref = gs.look_up_exact_symbol(&inner_pos).unwrap();
                let looked_symbol = gs.get_symbols().get_symbol(looked_symbol_ref).unwrap();
                let (start, end) = looked_symbol.get_range();
                // test symbol basic infomation
                assert_eq!(start.filename, abs_filepath);
                assert_eq!(start.line, *start_line);
                assert_eq!(start.column.unwrap_or(0), *start_col);
                assert_eq!(end.line, *end_line);
                assert_eq!(end.column.unwrap_or(0), *end_col);
                assert_eq!(*name, looked_symbol.get_name());
                assert_eq!(looked_symbol_ref.get_kind(), *kind);

                // test find def
                if SymbolKind::Unresolved == looked_symbol_ref.get_kind() {
                    let (start_line, start_col, end_line, end_col, path, kind) =
                        symbols.get(index + 1).unwrap();
                    let def_ref = looked_symbol.get_definition().unwrap();
                    let def = gs.get_symbols().get_symbol(def_ref).unwrap();
                    let (start, end) = def.get_range();
                    let def_filepath = adjust_canonicalization(base_path.join(path));
                    assert_eq!(start.line, *start_line);
                    assert_eq!(start.column.unwrap_or(0), *start_col);
                    assert_eq!(end.line, *end_line);
                    assert_eq!(end.column.unwrap_or(0), *end_col);
                    if !path.is_empty() {
                        assert_eq!(start.filename, def_filepath);
                    }
                    assert_eq!(def_ref.get_kind(), *kind);
                    skip_def_info = true;
                }
            }
        }
    }

    #[test]
    fn test_look_up_cloest_symbol() {
        let sess = Arc::new(ParseSession::default());

        let path = "src/advanced_resolver/test_data/schema_symbols.k"
            .to_string()
            .replace("/", &std::path::MAIN_SEPARATOR.to_string());
        let mut program = load_program(sess.clone(), &[&path], None, None)
            .unwrap()
            .program;
        let gs = GlobalState::default();
        let gs = Namer::find_symbols(&program, gs);
        let node_ty_map = resolver::resolve_program(&mut program).node_ty_map;
        let gs = AdvancedResolver::resolve_program(&program, gs, node_ty_map);
        let base_path = Path::new(".").canonicalize().unwrap();

        let test_cases = vec![
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                19_u64,
                25_u64,
                Some((19, 20, 19, 24, "name".to_string(), SymbolKind::Unresolved)),
            ),
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                32_u64,
                7_u64,
                Some((28, 4, 28, 8, "Main".to_string(), SymbolKind::Unresolved)),
            ),
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                35_u64,
                5_u64,
                Some((33, 13, 33, 19, "Person".to_string(), SymbolKind::Unresolved)),
            ),
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                28_u64,
                30_u64,
                None,
            ),
        ];

        for (filepath, line, col, symbol_info) in test_cases.iter() {
            let abs_scope_file_path = adjust_canonicalization(base_path.join(filepath));
            let symbol_ref = gs.look_up_closest_symbol(&Position {
                filename: abs_scope_file_path.clone(),
                line: *line,
                column: Some(*col),
            });

            match symbol_info {
                Some((start_line, start_col, end_line, end_col, name, kind)) => {
                    let symbol_ref = symbol_ref.unwrap();
                    let symbol = gs.get_symbols().get_symbol(symbol_ref).unwrap();

                    let (start, end) = symbol.get_range();
                    assert_eq!(start.line, *start_line);
                    assert_eq!(start.column.unwrap_or(0), *start_col);
                    assert_eq!(end.line, *end_line);
                    assert_eq!(end.column.unwrap_or(0), *end_col);
                    assert_eq!(*name, symbol.get_name());
                    assert_eq!(symbol_ref.get_kind(), *kind);
                }
                None => assert!(symbol_ref.is_none()),
            }
        }
    }

    #[test]
    fn test_look_up_scope() {
        let sess = Arc::new(ParseSession::default());

        let path = "src/advanced_resolver/test_data/schema_symbols.k"
            .to_string()
            .replace("/", &std::path::MAIN_SEPARATOR.to_string());
        let mut program = load_program(sess.clone(), &[&path], None, None)
            .unwrap()
            .program;
        let gs = GlobalState::default();
        let gs = Namer::find_symbols(&program, gs);
        let node_ty_map = resolver::resolve_program(&mut program).node_ty_map;
        let gs = AdvancedResolver::resolve_program(&program, gs, node_ty_map);
        let base_path = Path::new(".").canonicalize().unwrap();

        let scope_test_cases = vec![
            // __main__.Main schema stmt scope
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                17_u64,
                26_u64,
                10_usize,
            ),
            // __main__.Main schema expr scope
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                30,
                6,
                6,
            ),
            // __main__.Main schema config entry value scope
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                30,
                20,
                10,
            ),
            // pkg.Person schema expr scope
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                33,
                21,
                1,
            ),
            // pkg.Person schema config entry value scope
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                34,
                17,
                6,
            ),
            // __main__ package scope
            (
                "src/advanced_resolver/test_data/schema_symbols.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                36,
                31,
                5,
            ),
            // import_test.a.Person expr scope
            (
                "src/advanced_resolver/test_data/import_test/a.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                15,
                11,
                6,
            ),
            // import_test.a.Name expr scope
            (
                "src/advanced_resolver/test_data/import_test/a.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                12,
                5,
                2,
            ),
            // import_test.a.Name config entry value scope
            (
                "src/advanced_resolver/test_data/import_test/a.k"
                    .to_string()
                    .replace("/", &std::path::MAIN_SEPARATOR.to_string()),
                12,
                21,
                8,
            ),
        ];

        for (filepath, line, col, def_num) in scope_test_cases.iter() {
            let abs_scope_file_path = adjust_canonicalization(base_path.join(filepath));
            let scope_ref = gs
                .look_up_scope(&Position {
                    filename: abs_scope_file_path.clone(),
                    line: *line,
                    column: Some(*col),
                })
                .unwrap();

            let all_defs = gs.get_all_defs_in_scope(scope_ref).unwrap();
            assert_eq!(all_defs.len(), *def_num)
        }
    }
}
