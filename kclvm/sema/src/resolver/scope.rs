use anyhow::bail;
use compiler_base_session::Session;
use indexmap::IndexMap;
use kclvm_ast::{ast, MAIN_PKG};
use kclvm_error::{Handler, Level};
use std::sync::Arc;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::resolver::Resolver;
use crate::ty::Type;
use crate::{builtin::BUILTIN_FUNCTIONS, ty::TypeInferMethods};
use kclvm_ast::pos::ContainsPos;
use kclvm_error::Position;

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
    pub ty: Rc<Type>,
    /// The scope object kind.
    pub kind: ScopeObjectKind,
    /// Record whether has been used, for check unused imported module and var definition
    pub used: bool,
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

#[derive(PartialEq, Clone, Debug)]
pub enum ScopeObjectKind {
    Variable,
    Attribute,
    Definition,
    Parameter,
    TypeAlias,
    Module,
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

    /// Set a type by name to existed object, return true if found.
    pub fn set_ty(&mut self, name: &str, ty: Rc<Type>) -> bool {
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
        self.start.less_equal(pos) && pos.less_equal(&self.end)
    }
}

#[derive(Clone, Debug)]
pub enum ScopeKind {
    /// Package scope.
    Package,
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
#[derive(Clone, Debug)]
pub struct ProgramScope {
    pub scope_map: IndexMap<String, Rc<RefCell<Scope>>>,
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    pub handler: Handler,
}

unsafe impl Send for ProgramScope {}

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
    pub fn emit_diagnostics_to_string(&self, sess: Arc<Session>) -> Result<(), String> {
        let emit_error = || -> anyhow::Result<()> {
            // Add resolve errors into the session
            for diag in &self.handler.diagnostics {
                if matches!(diag.level, Level::Error) {
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
}

/// Construct a builtin scope
pub(crate) fn builtin_scope() -> Scope {
    let mut elems = IndexMap::default();
    for (name, builtin_func) in BUILTIN_FUNCTIONS.iter() {
        elems.insert(
            name.to_string(),
            Rc::new(RefCell::new(ScopeObject {
                name: name.to_string(),
                start: Position::dummy_pos(),
                end: Position::dummy_pos(),
                ty: Rc::new(builtin_func.clone()),
                kind: ScopeObjectKind::Definition,
                used: false,
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
    pub fn find_type_in_scope(&mut self, name: &str) -> Option<Rc<Type>> {
        self.scope
            .borrow()
            .lookup(name)
            .map(|obj| obj.borrow().ty.clone())
    }

    /// Lookup type from the scope by name, if not found, emit a compile error and
    /// return the any type.
    pub fn lookup_type_from_scope(&mut self, name: &str, pos: Position) -> Rc<Type> {
        match self.find_type_in_scope(name) {
            Some(ty) => ty,
            None => {
                self.handler.add_compile_error(
                    &format!("name '{}' is not defined", name.replace('@', "")),
                    pos,
                );
                self.any_ty()
            }
        }
    }

    /// Set type to the scope exited object, if not found, emit a compile error.
    pub fn set_type_to_scope(&mut self, name: &str, ty: Rc<Type>, pos: Position) {
        let mut scope = self.scope.borrow_mut();
        match scope.elems.get_mut(name) {
            Some(obj) => {
                let mut obj = obj.borrow_mut();
                obj.ty = self.ctx.ty_ctx.infer_to_variable_type(ty);
            }
            None => {
                self.handler.add_compile_error(
                    &format!("name '{}' is not defined", name.replace('@', "")),
                    pos,
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
}
