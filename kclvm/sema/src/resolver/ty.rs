use std::sync::Arc;

use crate::resolver::Resolver;
use crate::ty::parser::parse_type_str;
use crate::ty::{
    assignable_to, is_upper_bound, Attr, DictType, Parameter, SchemaType, Type, TypeKind, TypeRef,
};
use indexmap::IndexMap;
use kclvm_ast::ast;
use kclvm_ast::pos::GetPos;
use kclvm_error::diagnostic::Range;
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
    pub fn any_ty(&self) -> TypeRef {
        self.ctx.ty_ctx.builtin_types.any.clone()
    }
    #[inline]
    pub fn int_ty(&self) -> TypeRef {
        self.ctx.ty_ctx.builtin_types.int.clone()
    }
    #[inline]
    pub fn float_ty(&self) -> TypeRef {
        self.ctx.ty_ctx.builtin_types.float.clone()
    }
    #[inline]
    pub fn bool_ty(&self) -> TypeRef {
        self.ctx.ty_ctx.builtin_types.bool.clone()
    }
    #[inline]
    pub fn str_ty(&self) -> TypeRef {
        self.ctx.ty_ctx.builtin_types.str.clone()
    }
    #[inline]
    pub fn none_ty(&self) -> TypeRef {
        self.ctx.ty_ctx.builtin_types.none.clone()
    }
    #[inline]
    pub fn void_ty(&self) -> TypeRef {
        self.ctx.ty_ctx.builtin_types.void.clone()
    }
    /// Parse the type string with the scope, if parse_ty returns a Named type(schema type or type alias),
    /// found it from the scope.
    pub fn parse_ty_with_scope(
        &mut self,
        ty_node: Option<&ast::Node<ast::Type>>,
        range: Range,
    ) -> ResolvedResult {
        let ty: TypeRef = if let Some(ty) = ty_node {
            Arc::new(ty.node.clone().into())
        } else {
            Arc::new(Type::ANY)
        };
        // If a named type, find it from scope to get the specific type
        let ret_ty = self.upgrade_named_ty_with_scope(ty.clone(), &range, ty_node);
        self.add_type_alias(
            &ty.into_type_annotation_str(),
            &ret_ty.into_type_annotation_str(),
        );
        if let Some(ty) = ty_node {
            self.node_ty_map
                .insert(self.get_node_key(ty.id.clone()), ret_ty.clone());
        };
        ret_ty
    }

    pub fn parse_ty_str_with_scope(&mut self, ty_str: &str, range: Range) -> ResolvedResult {
        let ty: TypeRef = parse_type_str(ty_str);
        // If a named type, find it from scope to get the specific type
        let ret_ty = self.upgrade_named_ty_with_scope(ty, &range, None);
        self.add_type_alias(ty_str, &ret_ty.into_type_annotation_str());
        ret_ty
    }

    /// The given expression must be the expected type.
    #[inline]
    pub fn must_be_type(&mut self, expr: &'ctx ast::NodeRef<ast::Expr>, expected_ty: TypeRef) {
        let ty = self.expr(expr);
        self.must_assignable_to(ty, expected_ty, expr.get_span_pos(), None);
    }

    /// Must assignable to the expected type.
    #[inline]
    pub fn must_assignable_to(
        &mut self,
        ty: TypeRef,
        expected_ty: TypeRef,
        range: Range,
        expected_pos: Option<Range>,
    ) {
        if !self.check_type(ty.clone(), expected_ty.clone(), &range) {
            let mut msgs = vec![Message {
                range,
                style: Style::LineAndColumn,
                message: format!("expected {}, got {}", expected_ty.ty_str(), ty.ty_str(),),
                note: None,
                suggested_replacement: None,
            }];

            if let Some(expected_pos) = expected_pos {
                msgs.push(Message {
                    range: expected_pos,
                    style: Style::LineAndColumn,
                    message: format!(
                        "variable is defined here, its type is {}, but got {}",
                        expected_ty.ty_str(),
                        ty.ty_str(),
                    ),
                    note: None,
                    suggested_replacement: None,
                });
            }
            self.handler.add_error(ErrorKind::TypeError, &msgs);
        }
    }

    /// Check the type assignment statement between type annotation and target.
    pub fn check_assignment_type_annotation(
        &mut self,
        assign_stmt: &kclvm_ast::ast::AssignStmt,
        value_ty: TypeRef,
    ) {
        if assign_stmt.ty.is_none() {
            return;
        }
        for target in &assign_stmt.targets {
            if target.node.names.is_empty() {
                continue;
            }
            let name = &target.node.names[0].node;
            // If the assignment statement has type annotation, check the type of value and the type annotation of target

            if let Some(ty_annotation) = &assign_stmt.ty {
                let annotation_ty =
                    self.parse_ty_with_scope(Some(&ty_annotation), ty_annotation.get_span_pos());
                // If the target defined in the scope, check the type of value and the type annotation of target
                let target_ty = if let Some(obj) = self.scope.borrow().elems.get(name) {
                    let obj = obj.borrow();
                    if obj.ty.is_any() {
                        annotation_ty
                    } else {
                        if !is_upper_bound(annotation_ty.clone(), obj.ty.clone()) {
                            self.handler.add_error(
                                ErrorKind::TypeError,
                                &[
                                    Message {
                                        range: target.get_span_pos(),
                                        style: Style::LineAndColumn,
                                        message: format!(
                                            "can not change the type of '{}' to {}",
                                            name,
                                            annotation_ty.ty_str()
                                        ),
                                        note: None,
                                        suggested_replacement: None,
                                    },
                                    Message {
                                        range: obj.get_span_pos(),
                                        style: Style::LineAndColumn,
                                        message: format!("expected {}", obj.ty.ty_str()),
                                        note: None,
                                        suggested_replacement: None,
                                    },
                                ],
                            );
                        }
                        obj.ty.clone()
                    }
                } else {
                    annotation_ty
                };

                self.set_type_to_scope(name, target_ty.clone(), &target.node.names[0]);

                // Check the type of value and the type annotation of target
                self.must_assignable_to(value_ty.clone(), target_ty, target.get_span_pos(), None)
            }
        }
    }

    /// The check type main function, returns a boolean result.
    #[inline]
    pub fn check_type(&mut self, ty: TypeRef, expected_ty: TypeRef, range: &Range) -> bool {
        match (&ty.kind, &expected_ty.kind) {
            (TypeKind::List(item_ty), TypeKind::List(expected_item_ty)) => {
                self.check_type(item_ty.clone(), expected_item_ty.clone(), range)
            }
            (
                TypeKind::Dict(DictType { key_ty, val_ty, .. }),
                TypeKind::Dict(DictType {
                    key_ty: expected_key_ty,
                    val_ty: expected_val_ty,
                    ..
                }),
            ) => {
                self.check_type(key_ty.clone(), expected_key_ty.clone(), range)
                    && self.check_type(val_ty.clone(), expected_val_ty.clone(), range)
            }
            (TypeKind::Dict(DictType { key_ty, val_ty, .. }), TypeKind::Schema(schema_ty)) => {
                self.dict_assignable_to_schema(key_ty.clone(), val_ty.clone(), schema_ty, range)
            }
            (TypeKind::Union(types), _) => types
                .iter()
                .all(|ty| self.check_type(ty.clone(), expected_ty.clone(), range)),
            (_, TypeKind::Union(types)) => types
                .iter()
                .any(|expected_ty| self.check_type(ty.clone(), expected_ty.clone(), range)),
            _ => assignable_to(ty, expected_ty),
        }
    }

    /// Judge a dict can be converted to schema in compile time
    /// Do relaxed schema check key and value type check.
    pub fn dict_assignable_to_schema(
        &mut self,
        key_ty: TypeRef,
        val_ty: TypeRef,
        schema_ty: &SchemaType,
        range: &Range,
    ) -> bool {
        if let Some(index_signature) = &schema_ty.index_signature {
            if !self.check_type(val_ty.clone(), index_signature.val_ty.clone(), range) {
                self.handler.add_type_error(
                    &format!(
                        "expected schema index signature value type {}, got {}",
                        index_signature.val_ty.ty_str(),
                        val_ty.ty_str()
                    ),
                    range.clone(),
                );
            }
            if index_signature.any_other {
                return self.check_type(key_ty, index_signature.key_ty.clone(), range)
                    && self.check_type(val_ty, index_signature.val_ty.clone(), range);
            }
            true
        } else {
            // When assigning a dict type to an instance of a schema type,
            // check whether the type of key value pair in dict matches the attribute type in the schema.
            if let TypeKind::StrLit(key_name) = &key_ty.kind {
                if let Some(attr_obj) = schema_ty.attrs.get(key_name) {
                    self.must_assignable_to(
                        val_ty.clone(),
                        attr_obj.ty.clone(),
                        range.clone(),
                        Some(attr_obj.range.clone()),
                    );
                    return true;
                }
            }
            true
        }
    }

    fn upgrade_named_ty_with_scope(
        &mut self,
        ty: TypeRef,
        range: &Range,
        ty_node: Option<&ast::Node<ast::Type>>,
    ) -> ResolvedResult {
        match &ty.kind {
            TypeKind::List(item_ty) => {
                let mut inner_node = None;
                if let Some(ty_node) = ty_node {
                    if let ast::Type::List(list_type) = &ty_node.node {
                        inner_node = list_type.inner_type.as_ref().map(|ty| ty.as_ref())
                    }
                };
                Type::list_ref(self.upgrade_named_ty_with_scope(item_ty.clone(), range, inner_node))
            }
            TypeKind::Dict(DictType {
                key_ty,
                val_ty,
                attrs,
            }) => {
                let mut key_node = None;
                let mut value_node = None;
                if let Some(ty_node) = ty_node {
                    if let ast::Type::Dict(dict_type) = &ty_node.node {
                        key_node = dict_type.key_type.as_ref().map(|ty| ty.as_ref());
                        value_node = dict_type.value_type.as_ref().map(|ty| ty.as_ref());
                    }
                };
                Type::dict_ref_with_attrs(
                    self.upgrade_named_ty_with_scope(key_ty.clone(), range, key_node),
                    self.upgrade_named_ty_with_scope(val_ty.clone(), range, value_node),
                    attrs
                        .into_iter()
                        .map(|(key, attr)| {
                            (
                                key.to_string(),
                                Attr {
                                    ty: self.upgrade_named_ty_with_scope(
                                        val_ty.clone(),
                                        range,
                                        None,
                                    ),
                                    range: attr.range.clone(),
                                },
                            )
                        })
                        .collect(),
                )
            }
            TypeKind::Union(types) => Type::union_ref(
                &types
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| {
                        let mut elem_node = None;
                        if let Some(ty_node) = ty_node {
                            if let ast::Type::Union(union_type) = &ty_node.node {
                                elem_node =
                                    union_type.type_elements.get(index).map(|ty| ty.as_ref())
                            }
                        };
                        self.upgrade_named_ty_with_scope(ty.clone(), range, elem_node)
                    })
                    .collect::<Vec<TypeRef>>(),
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
                        .add_compile_error("missing type annotation", range.clone());
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
                let tys = self.resolve_var(
                    &names.iter().map(|n| n.to_string()).collect::<Vec<String>>(),
                    &pkgpath,
                    range.clone(),
                );

                if let Some(ty_node) = ty_node {
                    if let ast::Type::Named(identifier) = &ty_node.node {
                        for (index, name) in identifier.names.iter().enumerate() {
                            self.node_ty_map
                                .insert(self.get_node_key(name.id.clone()), tys[index].clone());
                        }
                        let ident_ty = tys.last().unwrap().clone();
                        self.node_ty_map
                            .insert(self.get_node_key(ty_node.id.clone()), ident_ty.clone());
                    }
                };
                tys.last().unwrap().clone()
            }
            TypeKind::Function(fn_ty) => {
                // Replace the type 'Named' to the real type in function params and return type
                let mut params_ty = vec![];
                let mut ret_ty = Type::any_ref();
                if let Some(ty_node) = ty_node {
                    if let ast::Type::Function(fn_ast_type) = &ty_node.node {
                        if let Some(params_ast_ty) = fn_ast_type.params_ty.as_ref() {
                            for (ast_ty, ty) in params_ast_ty.iter().zip(fn_ty.params.iter()) {
                                params_ty.push(Parameter {
                                    name: ty.name.clone(),
                                    ty: self.upgrade_named_ty_with_scope(
                                        ty.ty.clone(),
                                        range,
                                        Some(ast_ty.as_ref()),
                                    ),
                                    has_default: ty.has_default,
                                });
                            }
                        }

                        ret_ty = if let Some(ret_ast_ty) = fn_ast_type.ret_ty.as_ref() {
                            self.upgrade_named_ty_with_scope(
                                fn_ty.return_ty.clone(),
                                range,
                                Some(ret_ast_ty.as_ref()),
                            )
                        } else {
                            Type::any_ref()
                        };
                    }
                };

                Arc::new(Type::function(
                    fn_ty.self_ty.clone(),
                    ret_ty,
                    params_ty.as_slice(),
                    &fn_ty.doc,
                    fn_ty.is_variadic,
                    fn_ty.kw_only_index,
                ))
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
