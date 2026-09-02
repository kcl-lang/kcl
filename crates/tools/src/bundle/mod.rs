//! Bundle a KCL program into a single self-contained source file.
//!
//! Given an entry file (or files), every package of the program that the
//! entry imports is inlined into one KCL module: import statements that
//! refer to in-tree packages are dropped, and the symbols of each package
//! are renamed with a package-derived prefix so that names from distinct
//! packages never collide. Imports of builtin/system modules (e.g. `math`)
//! are kept as-is.
//!
//! The bundled file is semantically equivalent to the original program for
//! the common case: schemas, rules, lambdas and values of imported packages
//! are all inlined with their references rewritten. Mangled names start
//! with `_` so that the values coming from the inlined packages stay out of
//! the output, exactly like when they were imports: only the variables of
//! the main package are printed.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use kcl_ast::ast::{self, Module, Stmt, Target};
use kcl_ast::walker::MutSelfMutWalker;
use kcl_ast_pretty::print_ast_module;
use kcl_config::vfs::fix_import_path;
use kcl_parser::{LoadProgramOptions, ParseSession, load_program};
use kcl_utils::pkgpath::pkgpath_to_path_buf;
#[cfg(test)]
mod tests;

/// The separator between the mangled package prefix and a symbol name.
const NAME_SEP: &str = "__";

/// Bundle the program loaded from `entries` into a single KCL source file.
///
/// # Examples
///
/// ```no_run
/// use kcl_tools::bundle::bundle;
/// let bundled = bundle(&["main.k"], None).unwrap();
/// ```
#[allow(clippy::arc_with_non_send_sync)]
pub fn bundle(entries: &[&str], opts: Option<LoadProgramOptions>) -> Result<String> {
    let sess = Arc::new(ParseSession::default());
    let mut opts = opts.unwrap_or_default();
    opts.load_plugins = true;
    let program = load_program(sess, entries, Some(opts), None)
        .map_err(|err| anyhow!(err.to_string()))?
        .program;

    // pkgpath -> modules of the package, in load order.
    let mut pkgs: BTreeMap<String, Vec<Module>> = BTreeMap::new();
    for (pkgpath, files) in &program.pkgs {
        for file in files {
            let module = program
                .modules
                .get(file)
                .and_then(|m| m.read().ok())
                .ok_or_else(|| anyhow!("module {} not found in the program", file))?;
            pkgs.entry(pkgpath.clone())
                .or_default()
                .push(module.clone());
        }
    }

    let bundler = Bundler::new(program.root, &pkgs);
    let mut body: Vec<kcl_ast::ast::NodeRef<Stmt>> = vec![];
    let mut kept_imports: HashSet<String> = HashSet::new();
    for pkgpath in bundler.package_order() {
        for module in pkgs.get_mut(&pkgpath).unwrap() {
            let mut rewriter = Rewriter {
                aliases: bundler.aliases_for(&pkgpath, module),
                locals: bundler.locals_for(&pkgpath),
                pkgpaths: &bundler.pkgpaths,
                mangled: &bundler.mangled,
            };
            rename_bindings(module, &rewriter);
            rewriter.walk_module(module);
            // Keep every statement, dropping the inlined imports and the
            // imports another inlined file already brought in.
            for stmt in &module.body {
                if let Stmt::Import(import_stmt) = &stmt.node {
                    let rawpath = &import_stmt.rawpath;
                    if !bundler.is_inlined_import(&module.filename, rawpath)
                        && kept_imports.insert(rawpath.clone())
                    {
                        body.push(stmt.clone());
                    }
                } else {
                    body.push(stmt.clone());
                }
            }
        }
    }
    let bundled = Module {
        filename: "bundled.k".to_string(),
        doc: None,
        body,
        comments: vec![],
    };
    Ok(print_ast_module(&bundled))
}

/// The mangled prefix of a package, e.g. `__main__.sub.models` -> `sub_models`.
fn mangle_prefix(pkgpath: &str) -> String {
    let local = pkgpath
        .strip_prefix(&format!("{}.", kcl_ast::MAIN_PKG))
        .unwrap_or(pkgpath);
    local.replace('.', "_")
}

/// The mangled name of `symbol` of `pkgpath`: it starts with `_` so that the
/// values of inlined packages stay out of the output, exactly like when they
/// were imports: only the variables of the main package are printed.
fn mangled_name(pkgpath: &str, symbol: &str) -> String {
    format!("_{}{}{}", mangle_prefix(pkgpath), NAME_SEP, symbol)
}

struct Bundler {
    root: String,
    /// Every pkgpath taking part in the bundle.
    pkgpaths: HashSet<String>,
    /// pkgpath -> top level symbols defined by the package, in file order.
    symbols: HashMap<String, Vec<String>>,
    /// pkgpath -> (file -> alias -> imported pkgpath).
    imports: HashMap<String, HashMap<String, HashMap<String, String>>>,
    /// pkgpath -> symbol -> mangled name.
    mangled: HashMap<String, HashMap<String, String>>,
}

impl Bundler {
    fn new(root: String, pkgs: &BTreeMap<String, Vec<Module>>) -> Self {
        let mut bundler = Bundler {
            root: root.clone(),
            pkgpaths: pkgs.keys().cloned().collect(),
            symbols: HashMap::new(),
            imports: HashMap::new(),
            mangled: HashMap::new(),
        };
        // The main package keeps its names and cannot be mangled into.
        let mut taken = HashSet::new();
        if let Some(modules) = pkgs.get(kcl_ast::MAIN_PKG) {
            taken.extend(modules.iter().flat_map(top_level_symbols));
        }
        for (pkgpath, modules) in pkgs {
            let mut symbols: Vec<String> = vec![];
            let mut imports = HashMap::new();
            for module in modules {
                imports.insert(module.filename.clone(), module_imports(&root, module));
                for symbol in top_level_symbols(module) {
                    if !symbols.contains(&symbol) {
                        symbols.push(symbol);
                    }
                }
            }
            bundler.symbols.insert(pkgpath.clone(), symbols);
            bundler.imports.insert(pkgpath.clone(), imports);
        }
        // Assign a unique mangled name to every symbol of every non-main package.
        for pkgpath in pkgs.keys() {
            if pkgpath == kcl_ast::MAIN_PKG {
                continue;
            }
            let mut mangled = HashMap::new();
            for symbol in &bundler.symbols[pkgpath] {
                let mut name = mangled_name(pkgpath, symbol);
                while !taken.insert(name.clone()) {
                    name = format!("{}_1", name);
                }
                mangled.insert(symbol.clone(), name);
            }
            bundler.mangled.insert(pkgpath.clone(), mangled);
        }
        bundler
    }

    /// Import dependencies of a package: only imports resolved to packages
    /// that take part in the bundle, in a deterministic order.
    fn deps_of(&self, pkgpath: &str) -> Vec<String> {
        let mut deps = BTreeSet::new();
        if let Some(imports) = self.imports.get(pkgpath) {
            for file_imports in imports.values() {
                for target in file_imports.values() {
                    if self.pkgpaths.contains(target) {
                        deps.insert(target.clone());
                    }
                }
            }
        }
        deps.into_iter().collect()
    }

    /// Packages in dependency order (dependencies first, main package last).
    fn package_order(&self) -> Vec<String> {
        let mut order = vec![];
        let mut visited = HashSet::new();
        self.visit(kcl_ast::MAIN_PKG, &mut visited, &mut order);
        order
    }

    fn visit(&self, pkgpath: &str, visited: &mut HashSet<String>, order: &mut Vec<String>) {
        if !visited.insert(pkgpath.to_string()) || !self.pkgpaths.contains(pkgpath) {
            return;
        }
        for dep in self.deps_of(pkgpath) {
            self.visit(&dep, visited, order);
        }
        order.push(pkgpath.to_string());
    }

    /// Alias table of one file of a package: alias -> imported pkgpath, for
    /// the imports inlined into the bundle.
    fn aliases_for(&self, pkgpath: &str, module: &Module) -> HashMap<String, String> {
        let mut aliases = HashMap::new();
        if let Some(imports) = self.imports.get(pkgpath)
            && let Some(file_imports) = imports.get(&module.filename)
        {
            for (alias, target) in file_imports {
                // Only imports inlined into the bundle rewrite references.
                if self.pkgpaths.contains(target) {
                    aliases.insert(alias.clone(), target.clone());
                }
            }
        }
        aliases
    }

    /// Local-symbol rename table of a package.
    fn locals_for(&self, pkgpath: &str) -> HashMap<String, String> {
        self.mangled.get(pkgpath).cloned().unwrap_or_default()
    }

    /// Whether the import statement of `file` with `rawpath` has been inlined.
    fn is_inlined_import(&self, file: &str, rawpath: &str) -> bool {
        match import_target(&self.root, file, rawpath) {
            Some(target) => self.pkgpaths.contains(&target),
            None => false,
        }
    }
}

/// Resolve the pkgpath an import statement refers to, `None` if it cannot be
/// resolved (e.g. builtin modules such as `math`).
///
/// The last segment of an import path denotes a package only when it is a
/// directory: `import .types.nested` imports the package `types.nested`,
/// while `import .sub.orphan` imports the module `orphan.k` of the package
/// `sub`.
fn import_target(root: &str, file: &str, rawpath: &str) -> Option<String> {
    if !rawpath.starts_with('.') {
        return Some(rawpath.to_string());
    }
    let target = fix_import_path(root, file, rawpath);
    if target.is_empty() {
        return None;
    }
    let dir = pkgpath_to_path_buf(Path::new(root), &target);
    if dir.is_dir() {
        return Some(target);
    }
    if let Some((parent_pkgpath, last_segment)) = target.rsplit_once('.') {
        let mut module = pkgpath_to_path_buf(Path::new(root), parent_pkgpath);
        module.push(format!("{last_segment}.k"));
        if module.is_file() {
            return Some(parent_pkgpath.to_string());
        }
    }
    Some(target)
}

/// Local alias -> imported pkgpath for all the import statements of a module.
fn module_imports(root: &str, module: &Module) -> HashMap<String, String> {
    let mut imports = HashMap::new();
    for stmt in &module.body {
        if let Stmt::Import(import_stmt) = &stmt.node {
            let rawpath = &import_stmt.rawpath;
            if let Some(target) = import_target(root, &module.filename, rawpath) {
                let alias = match &import_stmt.asname {
                    Some(asname) => asname.node.clone(),
                    None => rawpath
                        .trim_start_matches('.')
                        .split('.')
                        .next_back()
                        .unwrap_or_default()
                        .to_string(),
                };
                imports.insert(alias, target);
            }
        }
    }
    imports
}

/// Names bound at the top level of a module, in statement order.
fn top_level_symbols(module: &Module) -> Vec<String> {
    let mut symbols = vec![];
    let mut push = |name: String| {
        if !name.is_empty() && !symbols.contains(&name) {
            symbols.push(name);
        }
    };
    for stmt in &module.body {
        match &stmt.node {
            Stmt::Assign(assign_stmt) => {
                for target in &assign_stmt.targets {
                    if target.node.paths.is_empty() {
                        push(target.node.name.node.clone());
                    }
                }
            }
            Stmt::AugAssign(aug_assign_stmt) => {
                if aug_assign_stmt.target.node.paths.is_empty() {
                    push(aug_assign_stmt.target.node.name.node.clone());
                }
            }
            Stmt::Schema(schema_stmt) => push(schema_stmt.name.node.clone()),
            Stmt::Rule(rule_stmt) => push(rule_stmt.name.node.clone()),
            Stmt::TypeAlias(type_alias_stmt) => push(type_alias_stmt.type_name.node.get_name()),
            Stmt::Unification(unification_stmt) => push(unification_stmt.target.node.get_name()),
            _ => {}
        }
    }
    symbols
}

/// Rewrite the names bound by the top level statements of a module: schema
/// and rule names, and the head of assignment targets.
fn rename_bindings(module: &mut Module, ctx: &Rewriter) {
    for stmt in &mut module.body {
        match &mut stmt.node {
            Stmt::Assign(assign_stmt) => {
                for target in assign_stmt.targets.iter_mut() {
                    ctx.rename_target(&mut target.node);
                }
            }
            Stmt::AugAssign(aug_assign_stmt) => ctx.rename_target(&mut aug_assign_stmt.target.node),
            Stmt::Schema(schema_stmt) => {
                if let Some(new) = ctx.locals.get(&schema_stmt.name.node) {
                    schema_stmt.name.node = new.clone();
                }
            }
            Stmt::Rule(rule_stmt) => {
                if let Some(new) = ctx.locals.get(&rule_stmt.name.node) {
                    rule_stmt.name.node = new.clone();
                }
            }
            _ => {}
        }
    }
}

/// Rewrite the identifiers of one module: references through inlined import
/// aliases and local top level symbols are renamed to their mangled names.
struct Rewriter<'a> {
    /// alias -> pkgpath, for the imports of the module being rewritten.
    aliases: HashMap<String, String>,
    /// local top level symbol -> mangled name of the package owning it.
    locals: HashMap<String, String>,
    /// every pkgpath taking part in the bundle.
    pkgpaths: &'a HashSet<String>,
    /// pkgpath -> symbol -> mangled name, shared by every package.
    mangled: &'a HashMap<String, HashMap<String, String>>,
}

impl Rewriter<'_> {
    /// The mangled name of `symbol` of `pkgpath`.
    fn name_of(&self, pkgpath: &str, symbol: &str) -> String {
        self.mangled
            .get(pkgpath)
            .and_then(|symbols| symbols.get(symbol))
            .cloned()
            .unwrap_or_else(|| mangled_name(pkgpath, symbol))
    }

    /// Rewrite the head of an assignment target:
    /// - `foo.x = 1` where `foo` is an inlined import assigns to the symbol
    ///   `x` of the imported package, i.e. its mangled name after inlining;
    /// - a local symbol head is simply renamed.
    fn rename_target(&self, target: &mut Target) {
        if let Some(pkgpath) = self.aliases.get(&target.name.node) {
            match target.paths.first() {
                Some(ast::MemberOrIndex::Member(member)) => {
                    target.name.node = self.name_of(pkgpath, &member.node);
                    target.paths.remove(0);
                }
                _ => target.name.node = mangle_prefix(pkgpath),
            }
        } else if let Some(new) = self.locals.get(&target.name.node) {
            target.name.node = new.clone();
        }
    }

    /// Rewrite the names of an identifier in place:
    /// - `foo.bar.attr` where `foo` is an inlined import refers to the symbol
    ///   `bar` (or a nested package `foo.bar`) of the imported package;
    /// - a single name that is a local top level symbol is renamed.
    fn rewrite_names(&self, names: &mut Vec<kcl_ast::ast::Node<String>>) {
        if names.is_empty() {
            return;
        }
        let head = names[0].node.clone();
        if let Some(target) = self.aliases.get(&head) {
            if names.len() > 1 {
                // Fold the leading names that denote nested packages.
                let mut folded = target.clone();
                let mut end = 1;
                while end + 1 < names.len() {
                    let next = format!("{}.{}", folded, names[end].node);
                    if self.pkgpaths.contains(&next) {
                        folded = next;
                        end += 1;
                    } else {
                        break;
                    }
                }
                names[0].node = self.name_of(&folded, &names[end].node);
                names.drain(1..=end);
            }
        } else if names.len() == 1
            && let Some(new) = self.locals.get(&head)
        {
            names[0].node = new.clone();
        }
    }
}

impl<'ctx> MutSelfMutWalker<'ctx> for Rewriter<'_> {
    fn walk_identifier(&mut self, identifier: &'ctx mut ast::Identifier) {
        self.rewrite_names(&mut identifier.names);
    }
}
