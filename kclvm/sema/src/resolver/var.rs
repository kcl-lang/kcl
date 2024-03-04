use crate::resolver::Resolver;
use indexmap::IndexMap;
use kclvm_ast::pos::GetPos;
use kclvm_error::diagnostic::Range;
use kclvm_error::*;

use super::node::ResolvedResult;
use super::scope::{ScopeObject, ScopeObjectKind};

impl<'ctx> Resolver<'ctx> {
    /// Resolve variables.
    pub fn resolve_var(
        &mut self,
        names: &[String],
        pkgpath: &str,
        range: Range,
    ) -> Vec<ResolvedResult> {
        if !pkgpath.is_empty() && self.ctx.l_value {
            self.handler.add_compile_error(
                "only schema and dict object can be updated attribute",
                range.clone(),
            );
        }
        if names.len() == 1 {
            let name = &names[0];
            let scope_schema_ty = self.ctx.schema.clone();
            if let Some(schema_ty) = &scope_schema_ty {
                let mut schema_ty = schema_ty.borrow_mut();
                // Find attribute type in the schema.
                let ty = schema_ty.get_type_of_attr(name);
                // Load from schema if the variable in schema
                if !self.ctx.l_value {
                    // Find the type from from local and global scope.
                    let scope_ty = self.find_type_in_scope(name);
                    if self.ctx.local_vars.contains(name) {
                        return vec![scope_ty.map_or(self.any_ty(), |ty| ty)];
                    }
                    // If it is a schema attribute, return the attribute type.
                    if let Some(ref ty) = ty {
                        if !ty.is_any() {
                            return vec![ty.clone()];
                        }
                    }
                    // Find from mixin schemas of a non-mixin schema
                    if ty.is_none() && !schema_ty.is_mixin {
                        for mixin in &schema_ty.mixins {
                            if let Some(ty) = mixin.get_type_of_attr(name) {
                                return vec![ty.clone()];
                            }
                        }
                    }
                    // If the variable is not found in a schema but not a schema mixin or rule,
                    // raise an error and return an any type.
                    // At present, retaining certain dynamic characteristics for mixins and rules
                    // requires further consideration of their semantics.
                    if ty.is_none()
                        && scope_ty.is_none()
                        && !schema_ty.is_mixin
                        && !schema_ty.is_rule
                    {
                        vec![self.lookup_type_from_scope(name, range)]
                    } else {
                        vec![scope_ty.map_or(self.any_ty(), |ty| ty)]
                    }
                }
                // Store
                else {
                    if !self.contains_object(name) || ty.is_none() {
                        self.insert_object(
                            name,
                            ScopeObject {
                                name: name.to_string(),
                                start: range.0.clone(),
                                end: range.1.clone(),
                                ty: self.any_ty(),
                                kind: ScopeObjectKind::Variable,
                                doc: None,
                            },
                        );
                        if ty.is_none() {
                            schema_ty.set_type_of_attr(name, self.any_ty())
                        }
                        return vec![self.any_ty()];
                    }
                    vec![ty.map_or(self.lookup_type_from_scope(name, range.clone()), |ty| ty)]
                }
            } else {
                // Load from schema if in schema
                if !self.ctx.l_value {
                    vec![self.lookup_type_from_scope(name, range)]
                }
                // Store
                else {
                    if !self.contains_object(name) && self.ctx.schema.is_none() {
                        self.insert_object(
                            name,
                            ScopeObject {
                                name: name.to_string(),
                                start: range.0.clone(),
                                end: range.1.clone(),
                                ty: self.any_ty(),
                                kind: ScopeObjectKind::Variable,
                                doc: None,
                            },
                        );
                        return vec![self.any_ty()];
                    }
                    vec![self.lookup_type_from_scope(name, range)]
                }
            }
        } else if !names.is_empty() {
            // Lookup pkgpath scope object and record it as "used". When enter child scope, e.g., in a schema scope, cant find module object.
            // It should be recursively search whole scope to lookup scope object, not the current scope.element.
            if !pkgpath.is_empty() {
                if let Some(obj) = self.scope.borrow().lookup(pkgpath) {
                    if let ScopeObjectKind::Module(m) = &mut obj.borrow_mut().kind {
                        for (stmt, used) in m.import_stmts.iter_mut() {
                            if stmt.get_pos().filename == range.0.filename {
                                *used = true;
                            }
                        }
                    }
                }
            }
            // Load type
            let mut tys = self.resolve_var(
                &[if !pkgpath.is_empty() {
                    pkgpath.to_string()
                } else {
                    names[0].clone()
                }],
                pkgpath,
                range.clone(),
            );
            let mut ty = tys[0].clone();

            for name in &names[1..] {
                // Store and config attr check
                if self.ctx.l_value {
                    self.must_check_config_attr(name, &range, &ty);
                }
                ty = self.load_attr(ty, name, range.clone());
                tys.push(ty.clone());
            }
            tys
        } else {
            self.handler
                .add_compile_error("missing variable", range.clone());
            vec![self.any_ty()]
        }
    }

    /// Resolve an unique key in the current package.
    pub(crate) fn resolve_unique_key(&mut self, name: &str, range: &Range) {
        if !self.contains_global_name(name) && self.scope_level == 0 {
            self.insert_global_name(name, range);
        } else {
            let mut msgs = vec![Message {
                range: range.clone(),
                style: Style::LineAndColumn,
                message: format!("Unique key error name '{}'", name),
                note: None,
                suggested_replacement: None,
            }];
            if let Some(pos) = self.get_global_name_pos(name) {
                msgs.push(Message {
                    range: pos.clone(),
                    style: Style::LineAndColumn,
                    message: format!("The variable '{}' is declared here", name),
                    note: None,
                    suggested_replacement: None,
                });
            }
            self.handler.add_error(ErrorKind::UniqueKeyError, &msgs);
        }
    }

    /// Insert global name in the current package.
    pub(crate) fn insert_global_name(&mut self, name: &str, range: &Range) {
        match self.ctx.global_names.get_mut(&self.ctx.pkgpath) {
            Some(mapping) => {
                mapping.insert(name.to_string(), range.clone());
            }
            None => {
                let mut mapping = IndexMap::default();
                mapping.insert(name.to_string(), range.clone());
                self.ctx
                    .global_names
                    .insert(self.ctx.pkgpath.clone(), mapping);
            }
        }
    }

    /// Whether contains global name in the current package.
    pub(crate) fn contains_global_name(&mut self, name: &str) -> bool {
        match self.ctx.global_names.get_mut(&self.ctx.pkgpath) {
            Some(mapping) => mapping.contains_key(name),
            None => false,
        }
    }

    /// Get global name position in the current package.
    pub(crate) fn get_global_name_pos(&mut self, name: &str) -> Option<&Range> {
        match self.ctx.global_names.get_mut(&self.ctx.pkgpath) {
            Some(mapping) => mapping.get(name),
            None => None,
        }
    }
}
