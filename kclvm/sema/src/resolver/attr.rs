use std::rc::Rc;

use crate::builtin::system_module::{get_system_module_members, UNITS, UNITS_NUMBER_MULTIPLIER};
use crate::builtin::{get_system_member_function_ty, STRING_MEMBER_FUNCTIONS};
use crate::resolver::Resolver;
use crate::ty::{DictType, ModuleKind, Type, TypeKind};
use kclvm_error::diagnostic::Range;
use kclvm_error::*;

use super::node::ResolvedResult;

impl<'ctx> Resolver<'ctx> {
    pub fn check_attr_ty(&mut self, attr_ty: &Type, range: Range) {
        if !attr_ty.is_any() && !attr_ty.is_key() {
            self.handler.add_error(
                ErrorKind::IllegalAttributeError,
                &[Message {
                    range,
                    style: Style::LineAndColumn,
                    message: format!(
                        "A attribute must be string type, got '{}'",
                        attr_ty.ty_str()
                    ),
                    note: None,
                }],
            );
        }
    }

    pub fn load_attr(&mut self, obj: Rc<Type>, attr: &str, range: Range) -> ResolvedResult {
        let (result, return_ty) = match &obj.kind {
            TypeKind::Any => (true, self.any_ty()),
            TypeKind::None
            | TypeKind::Bool
            | TypeKind::BoolLit(_)
            | TypeKind::Int
            | TypeKind::IntLit(_)
            | TypeKind::Float
            | TypeKind::FloatLit(_)
            | TypeKind::List(_)
            | TypeKind::NumberMultiplier(_)
            | TypeKind::Function(_)
            | TypeKind::Named(_)
            | TypeKind::Void => (false, self.any_ty()),
            TypeKind::Str | TypeKind::StrLit(_) => match STRING_MEMBER_FUNCTIONS.get(attr) {
                Some(ty) => (true, Rc::new(ty.clone())),
                None => (false, self.any_ty()),
            },
            TypeKind::Dict(DictType {
                key_ty: _,
                val_ty,
                attrs,
            }) => (
                true,
                attrs
                    .get(attr)
                    .map(|attr| attr.ty.clone())
                    .unwrap_or(Rc::new(val_ty.as_ref().clone())),
            ),
            // union type load attr based the type guard. e.g, a: str|int; if a is str: xxx; if a is int: xxx;
            // return sup([self.load_attr_type(t, attr, filename, line, column) for t in obj.types])
            TypeKind::Union(_) => (true, self.any_ty()),
            TypeKind::Schema(schema_ty) => {
                let (result, schema_attr_ty) = self.schema_load_attr(schema_ty, attr);
                if result {
                    (result, schema_attr_ty)
                } else if schema_ty.is_member_functions(attr) {
                    (
                        true,
                        Rc::new(Type::function(
                            Some(obj.clone()),
                            Type::list_ref(self.any_ty()),
                            &[],
                            "",
                            false,
                            None,
                        )),
                    )
                } else {
                    (false, self.any_ty())
                }
            }
            TypeKind::Module(module_ty) => {
                match &module_ty.kind {
                    crate::ty::ModuleKind::User => match self.scope_map.get(&module_ty.pkgpath) {
                        Some(scope) => match scope.borrow().elems.get(attr) {
                            Some(v) => {
                                if v.borrow().ty.is_module() {
                                    self.handler
                                            .add_compile_error(&format!("can not import the attribute '{}' from the module '{}'", attr, module_ty.pkgpath), range.clone());
                                }
                                (true, v.borrow().ty.clone())
                            }
                            None => (false, self.any_ty()),
                        },
                        None => (false, self.any_ty()),
                    },
                    ModuleKind::System => {
                        if module_ty.pkgpath == UNITS && attr == UNITS_NUMBER_MULTIPLIER {
                            (true, Rc::new(Type::number_multiplier_non_lit_ty()))
                        } else {
                            let members = get_system_module_members(&module_ty.pkgpath);
                            (
                                members.contains(&attr),
                                get_system_member_function_ty(&module_ty.pkgpath, attr),
                            )
                        }
                    }
                    ModuleKind::Plugin => (true, self.any_ty()),
                }
            }
        };
        if !result {
            self.handler.add_type_error(
                &format!(
                    "{} has no attribute {}",
                    obj.ty_str(),
                    if attr.is_empty() {
                        "[missing name]"
                    } else {
                        attr
                    }
                ),
                range,
            );
        }
        return_ty
    }
}
