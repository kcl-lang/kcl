use crate::resolver::Resolver;
use crate::ty::{sup, DictType, TypeKind, TypeRef};
use kclvm_ast::ast;
use kclvm_error::diagnostic::Range;

impl<'ctx> Resolver<'ctx> {
    /// Do loop type check including quant and comp for expression.
    pub(crate) fn do_loop_type_check(
        &mut self,
        first_node: &'ctx ast::NodeRef<ast::Identifier>,
        second_node: Option<&'ctx ast::NodeRef<ast::Identifier>>,
        target_node: &'ctx ast::NodeRef<ast::Identifier>,
        first_var_name: Option<String>,
        second_var_name: Option<String>,
        iter_ty: TypeRef,
        iter_range: Range,
    ) {
        let types = match &iter_ty.kind {
            TypeKind::Union(types) => types.clone(),
            _ => vec![iter_ty.clone()],
        };
        let mut first_var_ty = self.void_ty();
        let mut second_var_ty = self.void_ty();
        for iter_ty in &types {
            if !(iter_ty.is_iterable() || iter_ty.is_any()) {
                self.handler.add_compile_error(
                    &format!("'{}' object is not iterable", iter_ty.ty_str()),
                    iter_range.clone(),
                );
            }
            match &iter_ty.kind {
                TypeKind::List(item_ty) => {
                    if second_var_name.is_some() {
                        first_var_ty = sup(&[self.int_ty(), first_var_ty.clone()]);
                        second_var_ty = sup(&[item_ty.clone(), second_var_ty.clone()]);
                        self.set_type_to_scope(
                            first_var_name.as_ref().unwrap(),
                            first_var_ty.clone(),
                            &target_node,
                        );
                        self.set_type_to_scope(
                            second_var_name.as_ref().unwrap(),
                            second_var_ty.clone(),
                            &target_node,
                        );
                    } else {
                        first_var_ty = sup(&[item_ty.clone(), first_var_ty.clone()]);
                        self.set_type_to_scope(
                            first_var_name.as_ref().unwrap(),
                            first_var_ty.clone(),
                            &target_node,
                        );
                    }
                }
                TypeKind::Dict(DictType { key_ty, val_ty, .. }) => {
                    first_var_ty = sup(&[key_ty.clone(), first_var_ty.clone()]);
                    self.set_type_to_scope(
                        first_var_name.as_ref().unwrap(),
                        first_var_ty.clone(),
                        &target_node,
                    );
                    if second_var_name.is_some() {
                        second_var_ty = sup(&[val_ty.clone(), second_var_ty.clone()]);
                        self.set_type_to_scope(
                            second_var_name.as_ref().unwrap(),
                            second_var_ty.clone(),
                            &target_node,
                        );
                    }
                }
                TypeKind::Schema(schema_ty) => {
                    let (key_ty, val_ty) = (schema_ty.key_ty(), schema_ty.val_ty());
                    first_var_ty = sup(&[key_ty, first_var_ty.clone()]);
                    self.set_type_to_scope(
                        first_var_name.as_ref().unwrap(),
                        first_var_ty.clone(),
                        &target_node,
                    );
                    if second_var_name.is_some() {
                        second_var_ty = sup(&[val_ty, second_var_ty.clone()]);
                        self.set_type_to_scope(
                            second_var_name.as_ref().unwrap(),
                            second_var_ty.clone(),
                            &target_node,
                        );
                    }
                }
                TypeKind::Str | TypeKind::StrLit(_) => {
                    if second_var_name.is_some() {
                        first_var_ty = sup(&[self.int_ty(), first_var_ty.clone()]);
                        second_var_ty = sup(&[self.str_ty(), second_var_ty.clone()]);
                        self.set_type_to_scope(
                            first_var_name.as_ref().unwrap(),
                            first_var_ty.clone(),
                            &target_node,
                        );
                        self.set_type_to_scope(
                            second_var_name.as_ref().unwrap(),
                            second_var_ty.clone(),
                            &target_node,
                        );
                    } else {
                        first_var_ty = sup(&[self.str_ty(), first_var_ty.clone()]);
                        self.set_type_to_scope(
                            first_var_name.as_ref().unwrap(),
                            first_var_ty.clone(),
                            &target_node,
                        );
                    }
                }
                _ => {}
            }
            self.node_ty_map
                .insert(first_node.id.clone(), first_var_ty.clone());
            if let Some(second_node) = second_node {
                self.node_ty_map
                    .insert(second_node.id.clone(), second_var_ty.clone());
            }
        }
    }
}
