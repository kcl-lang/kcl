use std::rc::Rc;

use crate::resolver::Resolver;
use crate::ty::parser::parse_type_str;
use crate::ty::{assignable_to, SchemaType, Type, TypeKind};
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_error::*;

use super::node::ResolvedResult;

fn ty_str_to_pkgpath(ty_str: &str) -> &str {
    let splits: Vec<&str> = ty_str.rsplitn(2, '.').collect();
    let len = splits.len();
    splits[len - 1]
}

pub fn ty_str_replace_pkgpath(ty_str: &str, pkgpath: &str) -> String {
    let pkgpath = format!("@{}", pkgpath);
    if ty_str.contains('.') && ty_str_to_pkgpath(ty_str) == pkgpath {
        ty_str.replacen(&format!("{}.", pkgpath), "", 1)
    } else {
        ty_str.to_string()
    }
}

impl<'ctx> Resolver<'ctx> {
    #[inline]
    pub fn any_ty(&self) -> Rc<Type> {
        self.ctx.ty_ctx.builtin_types.any.clone()
    }
    #[inline]
    pub fn int_ty(&self) -> Rc<Type> {
        self.ctx.ty_ctx.builtin_types.int.clone()
    }
    #[inline]
    pub fn float_ty(&self) -> Rc<Type> {
        self.ctx.ty_ctx.builtin_types.float.clone()
    }
    #[inline]
    pub fn bool_ty(&self) -> Rc<Type> {
        self.ctx.ty_ctx.builtin_types.bool.clone()
    }
    #[inline]
    pub fn str_ty(&self) -> Rc<Type> {
        self.ctx.ty_ctx.builtin_types.str.clone()
    }
    #[inline]
    pub fn none_ty(&self) -> Rc<Type> {
        self.ctx.ty_ctx.builtin_types.none.clone()
    }
    #[inline]
    pub fn void_ty(&self) -> Rc<Type> {
        self.ctx.ty_ctx.builtin_types.void.clone()
    }
    /// Parse the type string with the scope, if parse_ty returns a Named type(schema type or type alias),
    /// found it from the scope.
    pub fn parse_ty_with_scope(&mut self, ty: &ast::Type, pos: Position) -> ResolvedResult {
        let ty: Rc<Type> = Rc::new(ty.clone().into());
        // If a named type, find it from scope to get the specific type
        let ret_ty = self.upgrade_named_ty_with_scope(ty.clone(), &pos);
        self.add_type_alias(
            &ty.into_type_annotation_str(),
            &ret_ty.into_type_annotation_str(),
        );
        ret_ty
    }

    pub fn parse_ty_str_with_scope(&mut self, ty_str: &str, pos: Position) -> ResolvedResult {
        let ty: Rc<Type> = parse_type_str(ty_str);
        // If a named type, find it from scope to get the specific type
        let ret_ty = self.upgrade_named_ty_with_scope(ty, &pos);
        self.add_type_alias(ty_str, &ret_ty.into_type_annotation_str());
        ret_ty
    }

    /// The given expression must be the expected type.
    #[inline]
    pub fn must_be_type(&mut self, expr: &'ctx ast::NodeRef<ast::Expr>, expected_ty: Rc<Type>) {
        let ty = self.expr(expr);
        self.must_assignable_to(ty, expected_ty, expr.get_pos(), None);
    }

    /// Must assignable to the expected type.
    #[inline]
    pub fn must_assignable_to(
        &mut self,
        ty: Rc<Type>,
        expected_ty: Rc<Type>,
        pos: Position,
        expected_pos: Option<Position>,
    ) {
        if !self.check_type(ty.clone(), expected_ty.clone(), &pos) {
            let mut msgs = vec![Message {
                pos,
                style: Style::LineAndColumn,
                message: format!("expected {}, got {}", expected_ty.ty_str(), ty.ty_str(),),
                note: None,
            }];

            if let Some(expected_pos) = expected_pos {
                msgs.push(Message {
                    pos: expected_pos,
                    style: Style::LineAndColumn,
                    message: format!(
                        "variable is defined here, its type is {}, but got {}",
                        expected_ty.ty_str(),
                        ty.ty_str(),
                    ),
                    note: None,
                });
            }
            self.handler.add_error(ErrorKind::TypeError, &msgs);
        }
    }

    /// The check type main function, returns a boolean result.
    #[inline]
    pub fn check_type(&mut self, ty: Rc<Type>, expected_ty: Rc<Type>, pos: &Position) -> bool {
        match (&ty.kind, &expected_ty.kind) {
            (TypeKind::List(item_ty), TypeKind::List(expected_item_ty)) => {
                self.check_type(item_ty.clone(), expected_item_ty.clone(), pos)
            }
            (TypeKind::Dict(key_ty, val_ty), TypeKind::Dict(expected_key_ty, expected_val_ty)) => {
                self.check_type(key_ty.clone(), expected_key_ty.clone(), pos)
                    && self.check_type(val_ty.clone(), expected_val_ty.clone(), pos)
            }
            (TypeKind::Dict(key_ty, val_ty), TypeKind::Schema(schema_ty)) => {
                self.dict_assignable_to_schema(key_ty.clone(), val_ty.clone(), schema_ty, pos)
            }
            (TypeKind::Union(types), _) => types
                .iter()
                .all(|ty| self.check_type(ty.clone(), expected_ty.clone(), pos)),
            (_, TypeKind::Union(types)) => types
                .iter()
                .any(|expected_ty| self.check_type(ty.clone(), expected_ty.clone(), pos)),
            _ => assignable_to(ty, expected_ty),
        }
    }

    /// Judge a dict can be converted to schema in compile time
    /// Do relaxed schema check key and value type check.
    pub fn dict_assignable_to_schema(
        &mut self,
        key_ty: Rc<Type>,
        val_ty: Rc<Type>,
        schema_ty: &SchemaType,
        pos: &Position,
    ) -> bool {
        if let Some(index_signature) = &schema_ty.index_signature {
            if !assignable_to(val_ty.clone(), index_signature.val_ty.clone()) {
                self.handler.add_type_error(
                    &format!(
                        "expected schema index signature value type {}, got {}",
                        index_signature.val_ty.ty_str(),
                        val_ty.ty_str()
                    ),
                    pos.clone(),
                );
            }
            if index_signature.any_other {
                return assignable_to(key_ty, index_signature.key_ty.clone())
                    && assignable_to(val_ty, index_signature.val_ty.clone());
            }
            true
        } else {
            true
        }
    }

    fn upgrade_named_ty_with_scope(&mut self, ty: Rc<Type>, pos: &Position) -> ResolvedResult {
        match &ty.kind {
            TypeKind::List(item_ty) => {
                Type::list_ref(self.upgrade_named_ty_with_scope(item_ty.clone(), pos))
            }
            TypeKind::Dict(key_ty, val_ty) => Type::dict_ref(
                self.upgrade_named_ty_with_scope(key_ty.clone(), pos),
                self.upgrade_named_ty_with_scope(val_ty.clone(), pos),
            ),
            TypeKind::Union(types) => Type::union_ref(
                &types
                    .iter()
                    .map(|ty| self.upgrade_named_ty_with_scope(ty.clone(), pos))
                    .collect::<Vec<Rc<Type>>>(),
            ),
            TypeKind::Named(ty_str) => {
                let ty_str = ty_str_replace_pkgpath(ty_str, &self.ctx.pkgpath);
                let names: Vec<&str> = if ty_str.starts_with('@') {
                    let names: Vec<&str> = ty_str.rsplitn(2, '.').collect();
                    names.iter().rev().cloned().collect()
                } else {
                    ty_str.split('.').collect()
                };
                if names.is_empty() {
                    self.handler
                        .add_compile_error("missing type annotation", pos.clone());
                    return self.any_ty();
                }
                let mut pkgpath = "".to_string();
                let name = names[0];
                if names.len() > 1 && !self.ctx.local_vars.contains(&name.to_string()) {
                    if let Some(mapping) = self.ctx.import_names.get(&self.ctx.filename) {
                        pkgpath = mapping
                            .get(name)
                            .map_or("".to_string(), |pkgpath| pkgpath.to_string());
                    }
                }
                self.ctx.l_value = false;
                self.resolve_var(
                    &names.iter().map(|n| n.to_string()).collect::<Vec<String>>(),
                    &pkgpath,
                    pos.clone(),
                )
            }
            _ => ty.clone(),
        }
    }

    pub fn add_type_alias(&mut self, name: &str, alias: &str) {
        if alias.starts_with('@') {
            if name == &alias[1..] {
                return;
            }
        } else if name == alias {
            return;
        }
        match self.ctx.type_alias_mapping.get_mut(&self.ctx.pkgpath) {
            Some(mapping) => {
                mapping.insert(name.to_string(), alias.to_string());
            }
            None => {
                let mut mapping = IndexMap::default();
                mapping.insert(name.to_string(), alias.to_string());
                self.ctx
                    .type_alias_mapping
                    .insert(self.ctx.pkgpath.clone(), mapping);
            }
        }
    }
}
