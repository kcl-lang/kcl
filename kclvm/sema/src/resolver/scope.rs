use anyhow::bail;
use compiler_base_session::Session;
use indexmap::{IndexMap, IndexSet};
use kclvm_ast::ast::NodeRef;
use kclvm_ast::ast::Stmt;
use kclvm_ast::ast::Stmt::Import;
use kclvm_ast::{ast, MAIN_PKG};
use kclvm_error::diagnostic::Range;
use kclvm_error::{Handler, Level};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::resolver::Resolver;
use crate::ty::TypeRef;
use crate::{builtin::BUILTIN_FUNCTIONS, ty::TypeInferMethods};
use kclvm_ast::ast::AstIndex;
use kclvm_ast::pos::ContainsPos;
use kclvm_ast::pos::GetPos;
use kclvm_error::Position;
use serde::Serialize;

/// The object stored in the scope.
#[derive(PartialEq, Clone, Debug)]
pub struct ScopeObject {
    /// The scope object name.
    pub name: String,
    /// The scope object start position.
    pub start: Position,
    /// The scope object end position.
    pub end: Position,
    /// The type of the scope object.
    pub ty: TypeRef,
    /// The scope object kind.
    pub kind: ScopeObjectKind,
    /// The doc of the scope object, will be None unless the scope object represents a schema or schema attribute.
    pub doc: Option<String>,
}

impl ScopeObject {
    /// Positions of the scope object are valid.
    #[inline]
    pub fn pos_is_valid(&self) -> bool {
        self.start.is_valid() && self.end.is_valid()
    }
}

impl ContainsPos for ScopeObject {
    fn contains_pos(&self, pos: &Position) -> bool {
        self.start.less_equal(pos) && pos.less_equal(&self.end)
    }
}

impl GetPos for ScopeObject {
    fn get_span_pos(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }
    fn get_pos(&self) -> Position {
        self.start.clone()
    }

    fn get_end_pos(&self) -> Position {
        self.end.clone()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum ScopeObjectKind {
    Variable,
    Attribute,
    Definition,
    Parameter,
    TypeAlias,
    FunctionCall,
    Module(Module),
}

/// A scope object of module type represents an import stmt on an AST and
/// is used to record information on the AST
#[derive(PartialEq, Clone, Debug)]
pub struct Module {
    /// Record stmts which import this module and whether has been used, for check unused imported module and var definition
    pub import_stmts: Vec<(NodeRef<Stmt>, bool)>,
}

/// A Scope maintains a set of objects and links to its containing
/// (parent) and contained (children) scopes. Objects may be inserted
/// and looked up by name. The zero value for Scope is a ready-to-use
/// empty scope.
#[derive(Clone, Debug)]
pub struct Scope {
    /// The parent scope.
    pub parent: Option<Weak<RefCell<Scope>>>,
    /// The child scope list.
    pub children: Vec<Rc<RefCell<Scope>>>,
    /// The scope object mapping with its name.
    pub elems: IndexMap<String, Rc<RefCell<ScopeObject>>>,
    /// The scope start position.
    pub start: Position,
    /// The scope end position.
    pub end: Position,
    /// The scope kind.
    pub kind: ScopeKind,
}

impl Scope {
    /// Lookup the scope object recursively with the name.
    pub fn lookup(&self, name: &str) -> Option<Rc<RefCell<ScopeObject>>> {
        match self.elems.get(name) {
            Some(obj) => Some(obj.clone()),
            None => match &self.parent {
                Some(parent) => match parent.upgrade() {
                    Some(parent) => {
                        let parent = parent.borrow();
                        parent.lookup(name)
                    }
                    None => None,
                },
                None => None,
            },
        }
    }

    /// Get all usable scope objects in current and parent scope.
    pub fn all_usable_objects(&self) -> IndexMap<String, Rc<RefCell<ScopeObject>>> {
        let mut res = match &self.parent {
            Some(parent) => match parent.upgrade() {
                Some(parent) => parent.borrow().all_usable_objects(),
                None => IndexMap::new(),
            },
            None => IndexMap::new(),
        };

        for (name, obj) in &self.elems {
            match &obj.borrow().kind {
                ScopeObjectKind::Module(module) => {
                    for stmt in &module.import_stmts {
                        if let Import(import_stmt) = &stmt.0.node {
                            res.insert(import_stmt.name.clone(), obj.clone());
                        }
                    }
                }
                _ => {
                    res.insert(name.clone(), obj.clone());
                }
            }
        }
        res
    }

    /// Set a type by name to existed object, return true if found.
    pub fn set_ty(&mut self, name: &str, ty: TypeRef) -> bool {
        match self.elems.get_mut(name) {
            Some(obj) => {
                let mut obj = obj.borrow_mut();
                obj.ty = ty;
                true
            }
            None => false,
        }
    }
}

impl ContainsPos for Scope {
    /// Check if current scope contains a position
    fn contains_pos(&self, pos: &Position) -> bool {
        match &self.kind {
            ScopeKind::Package(files) => files.contains(&pos.filename),
            _ => self.start.less_equal(pos) && pos.less_equal(&self.end),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ScopeKind {
    /// Package scope.
    Package(IndexSet<String>),
    /// Builtin scope.
    Builtin,
    /// Schema name string.
    Schema(String),
    /// Loop scope.
    Loop,
    /// Condition statement.
    CondStmt,
    /// Lambda expression.
    Lambda,
    /// Config expression
    Config,
}

impl Scope {
    /// Check if current scope contains a position on the AST statement.
    pub fn contains_pos_on_stmt<T>(&self, pos: &Position, stmt: &ast::Node<ast::Stmt>) -> bool {
        match &stmt.node {
            ast::Stmt::Schema(schema) => {
                schema.body.iter().any(|n| n.contains_pos(pos))
                    || schema.checks.iter().any(|n| n.contains_pos(pos))
                    || schema
                        .index_signature
                        .as_ref()
                        .map(|n| n.contains_pos(pos))
                        .is_some()
            }
            ast::Stmt::Rule(rule) => rule.checks.iter().any(|n| n.contains_pos(pos)),
            _ => self.contains_pos(pos),
        }
    }
    /// Returns the inner most scope on the position.
    pub fn inner_most(&self, pos: &Position) -> Option<Scope> {
        // Builtin scope
        if self.parent.is_none() {
            for child in self.children.iter() {
                let child = child.borrow();
                if child.contains_pos(pos) {
                    return child.inner_most(pos);
                }
            }
            return None;
        }
        // self is not BUILTIN_SCOPE
        if self.contains_pos(pos) {
            for child in self.children.iter() {
                let child = child.borrow();
                if child.contains_pos(pos) {
                    return child.inner_most(pos);
                }
            }
            return Some(self.clone());
        }
        None
    }

    /// Get the enclosing scope
    #[inline]
    pub fn get_enclosing_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent.as_ref().map(|scope| scope.upgrade().unwrap())
    }

    /// Search child scope by the scope name.
    pub fn search_child_scope_by_name(&self, name: &str) -> Option<Rc<RefCell<Scope>>> {
        match self.elems.get(name) {
            Some(_) => {
                for child in self.children.iter() {
                    let child_ref = child.borrow();
                    if let ScopeKind::Schema(schema_name) = &child_ref.kind {
                        if name == schema_name {
                            return Some(Rc::clone(child));
                        }
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Search scope obj by the object name.
    pub fn search_obj_by_name(&self, name: &str) -> Vec<ScopeObject> {
        let mut res = vec![];
        for (obj_name, obj) in &self.elems {
            if obj_name == name {
                res.push(obj.borrow().clone())
            }
        }
        for c in &self.children {
            let c = c.borrow();
            let mut objs = c.search_obj_by_name(name);
            res.append(&mut objs);
        }
        res
    }
}

/// Program scope is scope contains a multiple scopes related to the
/// package path.
#[derive(Clone, Debug, Default)]
pub struct ProgramScope {
    pub scope_map: IndexMap<String, Rc<RefCell<Scope>>>,
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    pub node_ty_map: NodeTyMap,
    pub handler: Handler,
}

unsafe impl Send for ProgramScope {}

unsafe impl Send for Scope {}
unsafe impl Sync for Scope {}

impl ProgramScope {
    /// Get all package paths.
    #[inline]
    pub fn pkgpaths(&self) -> Vec<String> {
        self.scope_map.keys().cloned().collect::<Vec<String>>()
    }

    /// Get the scope in the main package.
    #[inline]
    pub fn main_scope(&self) -> Option<&Rc<RefCell<Scope>>> {
        self.scope_map.get(MAIN_PKG)
    }

    /// Return diagnostic pretty string but do not abort if the session exists any diagnostic.
    pub fn emit_diagnostics_to_string(
        &self,
        sess: Arc<Session>,
        include_warning: bool,
    ) -> Result<(), String> {
        let emit_error = || -> anyhow::Result<()> {
            // Add resolve errors into the session
            for diag in &self.handler.diagnostics {
                if matches!(diag.level, Level::Error) || matches!(diag.level, Level::Suggestions) {
                    sess.add_err(diag.clone())?;
                }
                if include_warning && matches!(diag.level, Level::Warning) {
                    sess.add_err(diag.clone())?;
                }
            }
            // If has syntax and resolve errors, return its string format.
            if sess.diag_handler.has_errors()? {
                let errors = sess.emit_all_diags_into_string()?;
                let mut error_strings = vec![];
                for error in errors {
                    error_strings.push(error?);
                }
                bail!(error_strings.join("\n"))
            } else {
                Ok(())
            }
        };
        emit_error().map_err(|e| e.to_string())
    }

    /// Returns the inner most scope on the position.
    pub fn inner_most_scope(&self, pos: &Position) -> Option<Scope> {
        for (_, scope) in &self.scope_map {
            match scope.borrow().inner_most(&pos) {
                Some(scope) => return Some(scope),
                None => continue,
            }
        }
        None
    }
}

/// Construct a builtin scope
pub fn builtin_scope() -> Scope {
    let mut elems = IndexMap::default();
    for (name, builtin_func) in BUILTIN_FUNCTIONS.iter() {
        elems.insert(
            name.to_string(),
            Rc::new(RefCell::new(ScopeObject {
                name: name.to_string(),
                start: Position::dummy_pos(),
                end: Position::dummy_pos(),
                ty: Arc::new(builtin_func.clone()),
                kind: ScopeObjectKind::Definition,
                doc: None,
            })),
        );
    }
    Scope {
        parent: None,
        children: vec![],
        elems,
        start: Position::dummy_pos(),
        end: Position::dummy_pos(),
        kind: ScopeKind::Builtin,
    }
}

impl<'ctx> Resolver<'ctx> {
    /// Enter scope such as schema statement, for loop expressions.
    pub fn enter_scope(&mut self, start: Position, end: Position, kind: ScopeKind) {
        let scope = Scope {
            parent: Some(Rc::downgrade(&self.scope)),
            children: vec![],
            elems: IndexMap::default(),
            start,
            end,
            kind,
        };
        let scope = Rc::new(RefCell::new(scope));
        {
            // Borrow self.scope
            let mut scope_ref = self.scope.borrow_mut();
            let children = &mut scope_ref.children;
            children.push(Rc::clone(&scope));
            // Deref self.scope
        }
        self.scope_level += 1;
        self.scope = Rc::clone(&scope);
    }

    /// Leave scope.
    pub fn leave_scope(&mut self) {
        self.ctx.local_vars.clear();
        let parent = match &self.scope.borrow().parent {
            Some(parent) => parent.upgrade().unwrap(),
            None => bug!("the scope parent is empty, can't leave the scope"),
        };
        self.scope_level -= 1;
        self.scope = Rc::clone(&parent);
    }

    /// Find scope object type by name.
    #[inline]
    pub fn find_type_in_scope(&mut self, name: &str) -> Option<TypeRef> {
        self.scope
            .borrow()
            .lookup(name)
            .map(|obj| obj.borrow().ty.clone())
    }

    /// Lookup type from the scope by name, if not found, emit a compile error and
    /// return the any type.
    pub fn lookup_type_from_scope(&mut self, name: &str, range: Range) -> TypeRef {
        match self.find_type_in_scope(name) {
            Some(ty) => ty,
            None => {
                let mut suggestion = String::new();
                let names = self
                    .scope
                    .borrow()
                    .all_usable_objects()
                    .keys()
                    .cloned()
                    .collect::<Vec<String>>();
                let suggs = suggestions::provide_suggestions(name, &names);
                if suggs.len() > 0 {
                    suggestion = format!(", did you mean '{:?}'?", suggs);
                }
                self.handler.add_compile_error_with_suggestions(
                    &format!(
                        "name '{}' is not defined{}",
                        name.replace('@', ""),
                        suggestion
                    ),
                    range,
                    Some(suggs.clone()),
                );
                self.any_ty()
            }
        }
    }

    /// Set type to the scope exited object, if not found, emit a compile error.
    pub fn set_type_to_scope<T>(&mut self, name: &str, ty: TypeRef, node: &ast::Node<T>) {
        let mut scope = self.scope.borrow_mut();
        match scope.elems.get_mut(name) {
            Some(obj) => {
                let mut obj = obj.borrow_mut();
                let infer_ty = self.ctx.ty_ctx.infer_to_variable_type(ty);
                self.node_ty_map
                    .insert(self.get_node_key(node.id.clone()), infer_ty.clone());
                obj.ty = infer_ty;
            }
            None => {
                self.handler.add_compile_error(
                    &format!("name '{}' is not defined", name.replace('@', "")),
                    node.get_span_pos(),
                );
            }
        }
    }

    /// Insert object into the current scope.
    #[inline]
    pub fn insert_object(&mut self, name: &str, obj: ScopeObject) {
        let mut scope = self.scope.borrow_mut();
        scope
            .elems
            .insert(name.to_string(), Rc::new(RefCell::new(obj)));
    }

    /// Contains object into the current scope.
    #[inline]
    pub fn contains_object(&mut self, name: &str) -> bool {
        self.scope.borrow().elems.contains_key(name)
    }

    pub fn get_node_key(&self, id: AstIndex) -> NodeKey {
        NodeKey {
            pkgpath: self.ctx.pkgpath.clone(),
            id,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub struct NodeKey {
    pub pkgpath: String,
    pub id: AstIndex,
}

pub type NodeTyMap = IndexMap<NodeKey, TypeRef>;
pub type KCLScopeCache = Arc<Mutex<CachedScope>>;

/// For CachedScope, we assume that all changed files must be located in kclvm_ast::MAIN_PKG ,
/// if this is not the case, please clear the cache directly
#[derive(Debug, Clone, Default)]
pub struct CachedScope {
    pub program_root: String,
    pub scope_map: IndexMap<String, Rc<RefCell<Scope>>>,
    pub node_ty_map: NodeTyMap,
    dependency_graph: DependencyGraph,
}

unsafe impl Send for CachedScope {}
unsafe impl Sync for CachedScope {}

#[derive(Debug, Clone, Default)]
struct DependencyGraph {
    /// map filename to pkgpath
    pub module_map: HashMap<String, HashSet<String>>,
    /// map pkgpath to node
    pub node_map: HashMap<String, DependencyNode>,
}

impl DependencyGraph {
    pub fn clear(&mut self) {
        self.module_map.clear();
        self.node_map.clear();
    }

    pub fn update(&mut self, program: &ast::Program) -> Result<HashSet<String>, String> {
        let mut new_modules = HashMap::new();
        for (pkgpath, modules) in program.pkgs.iter() {
            if pkgpath == kclvm_ast::MAIN_PKG {
                continue;
            }
            if !self.node_map.contains_key(pkgpath) {
                self.node_map.insert(
                    pkgpath.to_string(),
                    DependencyNode {
                        pkgpath: pkgpath.to_string(),
                        parents: HashSet::new(),
                        children: HashSet::new(),
                    },
                );
            }
            for module in modules {
                if !self.module_map.contains_key(&module.filename) {
                    new_modules.insert(module.filename.to_string(), module);
                    self.module_map
                        .insert(module.filename.to_string(), HashSet::new());
                }
                self.module_map
                    .get_mut(&module.filename)
                    .unwrap()
                    .insert(pkgpath.to_string());
            }
        }

        for new_module in new_modules.values() {
            self.add_new_module(new_module);
        }
        let mut invalidated_set = HashSet::new();
        if let Some(main_modules) = program.pkgs.get(kclvm_ast::MAIN_PKG) {
            for module in main_modules {
                let result = self.invalidate_module(module)?;
                for pkg in result {
                    invalidated_set.insert(pkg);
                }
                self.remove_dependency_from_pkg(&module.filename);
                self.add_new_module(module);
            }
        }
        Ok(invalidated_set)
    }

    fn add_new_module(&mut self, new_module: &ast::Module) {
        let module_file = new_module.filename.clone();
        if let Some(pkgpaths) = self.module_map.get(&module_file) {
            for stmt in &new_module.body {
                if let ast::Stmt::Import(import_stmt) = &stmt.node {
                    let parent_pkg = &import_stmt.path.node;
                    if let Some(parent_node) = self.node_map.get_mut(parent_pkg) {
                        parent_node.children.insert(new_module.filename.clone());
                    }
                    for pkgpath in pkgpaths {
                        let cur_node = self.node_map.get_mut(pkgpath).unwrap();
                        cur_node.parents.insert(parent_pkg.clone());
                    }
                }
            }
        }
    }

    fn invalidate_module(
        &mut self,
        changed_module: &ast::Module,
    ) -> Result<HashSet<String>, String> {
        let module_file = changed_module.filename.clone();
        let mut invalidated_set = HashSet::new();
        if let Some(pkgpaths) = self.module_map.get(&module_file).cloned() {
            let mut pkg_queue = VecDeque::new();
            for pkgpath in pkgpaths.iter() {
                invalidated_set.insert(pkgpath.clone());
                pkg_queue.push_back(self.node_map.get(pkgpath));
            }

            let mut old_size = 0;
            while old_size < invalidated_set.len() {
                old_size = invalidated_set.len();
                let cur_node = loop {
                    match pkg_queue.pop_front() {
                        Some(cur_node) => match cur_node {
                            None => continue,
                            Some(cur_node) => {
                                if invalidated_set.contains(&cur_node.pkgpath) {
                                    continue;
                                }
                                invalidated_set.insert(cur_node.pkgpath.clone());
                                break Some(cur_node);
                            }
                        },
                        None => break None,
                    }
                };
                if let Some(cur_node) = cur_node {
                    for child in cur_node.children.iter() {
                        if let Some(child_pkgs) = self.module_map.get(child) {
                            for child_pkg in child_pkgs {
                                if invalidated_set.contains(child_pkg) {
                                    continue;
                                }
                                pkg_queue.push_back(self.node_map.get(child_pkg));
                            }
                        }
                    }
                }
            }
        };
        Ok(invalidated_set)
    }

    fn remove_dependency_from_pkg(&mut self, filename: &str) {
        if let Some(pkgpaths) = self.module_map.get(filename).cloned() {
            for pkgpath in pkgpaths {
                if let Some(node) = self.node_map.get(&pkgpath).cloned() {
                    for parent in node.parents {
                        if let Some(parent_node) = self.node_map.get_mut(&parent) {
                            parent_node.children.remove(filename);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct DependencyNode {
    // The package path of the current node.
    pkgpath: String,
    // The pkgpath which is imported by this package.
    parents: HashSet<String>,
    // Files which import this package.
    children: HashSet<String>,
}

impl CachedScope {
    pub fn new(scope: &ProgramScope, program: &ast::Program) -> Self {
        let mut cached_scope = Self {
            program_root: program.root.to_string(),
            scope_map: scope.scope_map.clone(),
            node_ty_map: scope.node_ty_map.clone(),
            dependency_graph: DependencyGraph::default(),
        };
        let invalidated_pkgs = cached_scope.dependency_graph.update(program);
        cached_scope.invalidate_cache(invalidated_pkgs.as_ref());
        cached_scope
    }

    pub fn clear(&mut self) {
        self.scope_map.clear();
        self.node_ty_map.clear();
        self.dependency_graph.clear();
    }

    pub fn invalidate_cache(&mut self, invalidated_pkgs: Result<&HashSet<String>, &String>) {
        match invalidated_pkgs {
            Ok(invalidated_pkgs) => {
                for invalidated_pkg in invalidated_pkgs.iter() {
                    self.scope_map.remove(invalidated_pkg);
                }
            }
            Err(_) => self.clear(),
        }
    }

    pub fn update(&mut self, program: &ast::Program) {
        if self.program_root != program.root {
            self.clear();
            self.program_root = program.root.clone();
        }
        let invalidated_pkgs = self.dependency_graph.update(program);
        self.invalidate_cache(invalidated_pkgs.as_ref());
    }
}
