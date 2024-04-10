use std::cell::RefCell;
use std::rc::Rc;

use generational_arena::Index;
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::ast::AstIndex;
use kclvm_runtime::ValueRef;

use crate::error as kcl_error;
use crate::error::INTERNAL_ERROR_MSG;
use crate::Evaluator;
pub type LazyEvalScopeRef = Rc<RefCell<LazyEvalScope>>;

#[macro_export]
macro_rules! backtrack_break_here {
    ($ctx: expr, $stmt: expr) => {{
        // If is break, do not execute the stmt and return immediately.
        if let Some(meta) = $ctx.backtrack_meta.borrow().last() {
            if meta.is_break {
                return $ctx.ok_result();
            }
        }
    }};
}

#[macro_export]
macro_rules! backtrack_update_break {
    ($ctx: expr, $stmt: expr) => {{
        // Check whether pass the breakpoint.
        if let Some(meta) = $ctx.backtrack_meta.borrow_mut().last_mut() {
            if let Some(stopped) = &meta.stopped {
                if stopped == &$stmt.id {
                    meta.is_break = true
                }
            }
        }
    }};
}

/// LazyEvalScope represents a scope of sequentially independent calculations, where
/// the calculation of values is lazy and only recursively performed through
/// backtracking when needed.
#[derive(PartialEq, Clone, Default, Debug)]
pub struct LazyEvalScope {
    /// Variable value cache.
    pub cache: IndexMap<String, ValueRef>,
    /// Backtrack levels.
    pub levels: IndexMap<String, usize>,
    /// Variable setter function pointers.
    pub setters: IndexMap<String, Vec<Setter>>,
    /// Calculate times without backtracking.
    pub cal_times: IndexMap<String, usize>,
}

impl LazyEvalScope {
    #[inline]
    pub fn is_backtracking(&self, key: &str) -> bool {
        let level = self.levels.get(key).unwrap_or(&0);
        *level > 0
    }

    #[inline]
    pub fn setter_len(&self, key: &str) -> usize {
        self.setters.get(key).unwrap_or(&vec![]).len()
    }

    #[inline]
    pub fn cal_increment(&mut self, key: &str) -> bool {
        if self.is_backtracking(key) {
            false
        } else {
            let cal_time = *self.cal_times.get(key).unwrap_or(&0);
            let next_cal_time = cal_time + 1;
            self.cal_times.insert(key.to_string(), next_cal_time);
            next_cal_time >= self.setter_len(key)
        }
    }
}

/// Setter function definition.
#[derive(PartialEq, Clone, Default, Debug)]
pub struct Setter {
    /// Schema or body index, none denotes the current schema or body.
    pub index: Option<Index>,
    /// Statement index in the schema or body in the body array.
    pub stmt: usize,
    /// If the statement is a if statement, stop the backtrack process at the stopped statement index.
    pub stopped: Option<AstIndex>,
}

/// Merge setters and set the value with default undefined value.
pub(crate) fn merge_variables_and_setters(
    v: &mut ValueRef,
    s: &mut IndexMap<String, Vec<Setter>>,
    other: &IndexMap<String, Vec<Setter>>,
) {
    for (key, setters) in other {
        if let Some(values) = s.get_mut(key) {
            for setter in setters {
                values.push(setter.clone());
            }
        } else {
            s.insert(key.to_string(), setters.clone());
        }
        // Store a undefined value for the setter initial value to
        // prevent self referencing.
        if v.get_by_key(key).is_none() {
            v.dict_update_key_value(key, ValueRef::undefined());
        }
    }
}

/// Merge setters and set the value with default undefined value.
pub(crate) fn merge_setters(
    s: &mut IndexMap<String, Vec<Setter>>,
    other: &IndexMap<String, Vec<Setter>>,
) {
    for (key, setters) in other {
        if let Some(values) = s.get_mut(key) {
            for setter in setters {
                values.push(setter.clone());
            }
        } else {
            s.insert(key.to_string(), setters.clone());
        }
    }
}

/// Schema or Global internal order independent computation
/// backtracking meta information.
#[derive(Debug, Default)]
pub struct BacktrackMeta {
    pub stopped: Option<AstIndex>,
    pub is_break: bool,
}

impl<'ctx> Evaluator<'ctx> {
    /// Emit setter functions for the AST body.
    /// Separate if statements with the same targets using the backtrack meta, such as
    /// ```no_check
    /// a = 1
    /// if True:
    ///    a = 1
    ///    a += 1 # a = 2 instead of a = 3
    /// ```
    pub(crate) fn emit_setters(
        &self,
        body: &'ctx [Box<ast::Node<ast::Stmt>>],
        index: Option<Index>,
    ) -> IndexMap<String, Vec<Setter>> {
        let mut body_map: IndexMap<String, Vec<Setter>> = IndexMap::new();
        self.emit_setters_with(body, &mut body_map, false, &mut vec![], index);
        body_map
    }

    /// Emit setter functions for the AST body.
    pub(crate) fn emit_setters_with(
        &self,
        body: &'ctx [Box<ast::Node<ast::Stmt>>],
        body_map: &mut IndexMap<String, Vec<Setter>>,
        is_in_if: bool,
        in_if_names: &mut Vec<(String, AstIndex)>,
        index: Option<Index>,
    ) {
        let add_stmt = |name: &str,
                        i: usize,
                        stopped: Option<AstIndex>,
                        body_map: &mut IndexMap<String, Vec<Setter>>| {
            if !body_map.contains_key(name) {
                body_map.insert(name.to_string(), vec![]);
            }
            let body_vec = body_map.get_mut(name).expect(kcl_error::INTERNAL_ERROR_MSG);
            body_vec.push(Setter {
                index,
                stmt: i,
                stopped,
            });
        };
        for (i, stmt) in body.iter().enumerate() {
            match &stmt.node {
                ast::Stmt::Unification(unification_stmt) => {
                    let name = &unification_stmt.target.node.names[0].node;
                    if is_in_if {
                        in_if_names.push((name.to_string(), stmt.id.clone()));
                    } else {
                        add_stmt(name, i, None, body_map);
                    }
                }
                ast::Stmt::Assign(assign_stmt) => {
                    for target in &assign_stmt.targets {
                        let name = &target.node.names[0].node;
                        if is_in_if {
                            in_if_names.push((name.to_string(), stmt.id.clone()));
                        } else {
                            add_stmt(name, i, None, body_map);
                        }
                    }
                }
                ast::Stmt::AugAssign(aug_assign_stmt) => {
                    let target = &aug_assign_stmt.target;
                    let name = &target.node.names[0].node;
                    if is_in_if {
                        in_if_names.push((name.to_string(), stmt.id.clone()));
                    } else {
                        add_stmt(name, i, None, body_map);
                    }
                }
                ast::Stmt::If(if_stmt) => {
                    let mut names: Vec<(String, AstIndex)> = vec![];
                    self.emit_setters_with(&if_stmt.body, body_map, true, &mut names, index);
                    if is_in_if {
                        for (name, id) in &names {
                            in_if_names.push((name.to_string(), id.clone()));
                        }
                    } else {
                        for (name, id) in &names {
                            add_stmt(name, i, Some(id.clone()), body_map);
                        }
                        names.clear();
                    }
                    self.emit_setters_with(&if_stmt.orelse, body_map, true, &mut names, index);
                    if is_in_if {
                        for (name, id) in &names {
                            in_if_names.push((name.to_string(), id.clone()));
                        }
                    } else {
                        for (name, id) in &names {
                            add_stmt(name, i, Some(id.clone()), body_map);
                        }
                        names.clear();
                    }
                }
                ast::Stmt::SchemaAttr(schema_attr) => {
                    let name = schema_attr.name.node.as_str();
                    if is_in_if {
                        in_if_names.push((name.to_string(), stmt.id.clone()));
                    } else {
                        add_stmt(name, i, None, body_map);
                    }
                }
                _ => {}
            }
        }
    }

    /// Get the value from the context.
    pub(crate) fn get_value_from_lazy_scope(
        &self,
        pkgpath: &str,
        key: &str,
        target: &str,
        default: ValueRef,
    ) -> ValueRef {
        // Deal in-place modify and return it self immediately.
        if key == target && {
            let lazy_scopes = self.lazy_scopes.borrow();
            let scope = lazy_scopes.get(pkgpath).expect(INTERNAL_ERROR_MSG);
            !scope.is_backtracking(key) || scope.setter_len(key) <= 1
        } {
            default
        } else {
            let cached_value = {
                let lazy_scopes = self.lazy_scopes.borrow();
                let scope = lazy_scopes.get(pkgpath).expect(INTERNAL_ERROR_MSG);
                scope.cache.get(key).cloned()
            };
            match cached_value {
                Some(value) => value.clone(),
                None => {
                    let setters = {
                        let lazy_scopes = self.lazy_scopes.borrow();
                        let scope = lazy_scopes.get(pkgpath).expect(INTERNAL_ERROR_MSG);
                        scope.setters.get(key).cloned()
                    };
                    match &setters {
                        Some(setters) if !setters.is_empty() => {
                            // Call all setters function to calculate the value recursively.
                            let level = {
                                let lazy_scopes = self.lazy_scopes.borrow();
                                let scope = lazy_scopes.get(pkgpath).expect(INTERNAL_ERROR_MSG);
                                *scope.levels.get(key).unwrap_or(&0)
                            };
                            let next_level = level + 1;
                            {
                                let mut lazy_scopes = self.lazy_scopes.borrow_mut();
                                let scope = lazy_scopes.get_mut(pkgpath).expect(INTERNAL_ERROR_MSG);
                                scope.levels.insert(key.to_string(), next_level);
                            }
                            let n = setters.len();
                            let index = n - next_level;
                            if index >= n {
                                default
                            } else {
                                // Call setter function.
                                self.walk_stmts_with_setter(&setters[index]);
                                // Store cache value.
                                {
                                    let value = self.get_variable_in_pkgpath(key, pkgpath);
                                    let mut lazy_scopes = self.lazy_scopes.borrow_mut();
                                    let scope =
                                        lazy_scopes.get_mut(pkgpath).expect(INTERNAL_ERROR_MSG);
                                    scope.levels.insert(key.to_string(), level);
                                    scope.cache.insert(key.to_string(), value.clone());
                                    value
                                }
                            }
                        }
                        _ => default,
                    }
                }
            }
        }
    }

    /// Set value to the context.
    #[inline]
    pub(crate) fn set_value_to_lazy_scope(&self, pkgpath: &str, key: &str, value: &ValueRef) {
        let mut lazy_scopes = self.lazy_scopes.borrow_mut();
        let scope = lazy_scopes.get_mut(pkgpath).expect(INTERNAL_ERROR_MSG);
        if scope.cal_increment(key) && scope.cache.get(key).is_none() {
            scope.cache.insert(key.to_string(), value.clone());
        }
    }
}
