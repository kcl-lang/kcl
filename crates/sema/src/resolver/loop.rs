use crate::resolver::Resolver;
use crate::ty::{DictType, TypeKind, TypeRef, sup};
use kcl_ast::ast;
use kcl_error::diagnostic::Range;

impl<'ctx> Resolver<'ctx> {
    /// Do loop type check including quant and comp for expression.
    pub(crate) fn do_loop_type_check(
        &mut self,
        first_var_name: Option<&ast::Node<String>>,
        second_var_name: Option<&ast::Node<String>>,
        iter_ty: TypeRef,
        iter_range: Range,
    ) {
        let types = match &iter_ty.kind {
            TypeKind::Union(types) => types.clone(),
            _ => vec![iter_ty.clone()],
        };
        let mut first_var_ty = self.void_ty();
        let mut second_var_ty = self.void_ty();
        if let Some(first_var_name) = first_var_name {
            for iter_ty in &types {
                if !(iter_ty.is_iterable() || iter_ty.is_any()) {
                    self.handler.add_compile_error(
                        &format!("'{}' object is not iterable", iter_ty.ty_str()),
                        iter_range.clone(),
                    );
                }
                match &iter_ty.kind {
                    TypeKind::List(item_ty) => {
                        if let Some(second_var_name) = second_var_name {
                            first_var_ty = sup(&[self.int_ty(), first_var_ty.clone()]);
                            second_var_ty = sup(&[item_ty.clone(), second_var_ty.clone()]);
                            self.set_infer_type_to_scope(
                                &first_var_name.node,
                                first_var_ty.clone(),
                                first_var_name,
                            );
                            self.set_infer_type_to_scope(
                                &second_var_name.node,
                                second_var_ty.clone(),
                                second_var_name,
                            );
                        } else {
                            first_var_ty = sup(&[item_ty.clone(), first_var_ty.clone()]);
                            self.set_infer_type_to_scope(
                                &first_var_name.node,
                                first_var_ty.clone(),
                                first_var_name,
                            );
                        }
                    }
                    TypeKind::Dict(DictType { key_ty, val_ty, .. }) => {
                        first_var_ty = sup(&[key_ty.clone(), first_var_ty.clone()]);
                        self.set_infer_type_to_scope(
                            &first_var_name.node,
                            first_var_ty.clone(),
                            first_var_name,
                        );
                        if let Some(second_var_name) = second_var_name {
                            second_var_ty = sup(&[val_ty.clone(), second_var_ty.clone()]);
                            self.set_infer_type_to_scope(
                                &second_var_name.node,
                                second_var_ty.clone(),
                                second_var_name,
                            );
                        }
                    }
                    TypeKind::Schema(schema_ty) => {
                        let (key_ty, val_ty) = (schema_ty.key_ty(), schema_ty.val_ty());
                        first_var_ty = sup(&[key_ty, first_var_ty.clone()]);
                        self.set_infer_type_to_scope(
                            &first_var_name.node,
                            first_var_ty.clone(),
                            first_var_name,
                        );
                        if let Some(second_var_name) = second_var_name {
                            second_var_ty = sup(&[val_ty, second_var_ty.clone()]);
                            self.set_infer_type_to_scope(
                                &second_var_name.node,
                                second_var_ty.clone(),
                                second_var_name,
                            );
                        }
                    }
                    TypeKind::Str | TypeKind::StrLit(_) => {
                        if let Some(second_var_name) = second_var_name {
                            first_var_ty = sup(&[self.int_ty(), first_var_ty.clone()]);
                            second_var_ty = sup(&[self.str_ty(), second_var_ty.clone()]);
                            self.set_infer_type_to_scope(
                                &first_var_name.node,
                                first_var_ty.clone(),
                                first_var_name,
                            );
                            self.set_infer_type_to_scope(
                                &second_var_name.node,
                                second_var_ty.clone(),
                                second_var_name,
                            );
                        } else {
                            first_var_ty = sup(&[self.str_ty(), first_var_ty.clone()]);
                            self.set_infer_type_to_scope(
                                &first_var_name.node,
                                first_var_ty.clone(),
                                first_var_name,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
