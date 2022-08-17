use crate::resolver::Resolver;
use crate::ty::TypeKind;
use kclvm_error::*;

use super::node::ResolvedResult;
use super::scope::{ScopeObject, ScopeObjectKind};

impl<'ctx> Resolver<'ctx> {
    /// Resolve variables.
    pub fn resolve_var(
        &mut self,
        names: &[String],
        pkgpath: &str,
        pos: Position,
    ) -> ResolvedResult {
        if !pkgpath.is_empty() && self.ctx.l_value {
            self.handler.add_compile_error(
                "only schema and dict object can be updated attribute",
                pos.clone(),
            );
        }
        if names.len() == 1 {
            let name = &names[0];
            let scope_schema_ty = self.ctx.schema.clone();
            if let Some(schema_ty) = &scope_schema_ty {
                let mut schema_ty = schema_ty.borrow_mut();
                let ty = schema_ty.get_type_of_attr(name);
                // Load from schema if in schema
                if !self.ctx.l_value {
                    let scope_ty = self.find_type_in_scope(name);
                    if self.ctx.local_vars.contains(name) {
                        return scope_ty.map_or(self.any_ty(), |ty| ty);
                    } else if let Some(ref ty) = ty {
                        if !ty.is_any() {
                            return ty.clone();
                        }
                    }
                    scope_ty.map_or(self.any_ty(), |ty| ty)
                }
                // Store
                else {
                    if !self.contains_object(name) || ty.is_none() {
                        self.insert_object(
                            name,
                            ScopeObject {
                                name: name.to_string(),
                                start: pos.clone(),
                                end: pos.clone(),
                                ty: self.any_ty(),
                                kind: ScopeObjectKind::Variable,
                                used: false,
                            },
                        );
                        if ty.is_none() {
                            schema_ty.set_type_of_attr(name, self.any_ty())
                        }
                        return self.any_ty();
                    }
                    // FIXME: self.check_config_attr(name, &pos, &schema_ty);
                    ty.map_or(self.lookup_type_from_scope(name, pos.clone()), |ty| ty)
                }
            } else {
                // Load from schema if in schema
                if !self.ctx.l_value {
                    self.lookup_type_from_scope(name, pos)
                }
                // Store
                else {
                    if !self.contains_object(name) && self.ctx.schema.is_none() {
                        self.insert_object(
                            name,
                            ScopeObject {
                                name: name.to_string(),
                                start: pos.clone(),
                                end: pos.clone(),
                                ty: self.any_ty(),
                                kind: ScopeObjectKind::Variable,
                                used: false,
                            },
                        );
                        return self.any_ty();
                    }
                    self.lookup_type_from_scope(name, pos)
                }
            }
        } else {
            // Lookup pkgpath scope object and record it as "used". When enter child scope, e.g., in a schema scope, cant find module object.
            // It should be recursively search whole scope to lookup scope object, not the current scope.element.
            if !pkgpath.is_empty() {
                if let Some(obj) = self.scope.borrow().lookup(pkgpath) {
                    obj.borrow_mut().used = true;
                }
            }
            // Load type
            let mut ty = self.resolve_var(
                &[if !pkgpath.is_empty() {
                    pkgpath.to_string()
                } else {
                    names[0].clone()
                }],
                pkgpath,
                pos.clone(),
            );
            for name in &names[1..] {
                // Store and config attr check
                if self.ctx.l_value {
                    if let TypeKind::Schema(schema_ty) = &ty.kind {
                        self.check_config_attr(name, &pos, schema_ty);
                    }
                }
                ty = self.load_attr(ty, name, pos.clone())
            }
            ty
        }
    }
}
