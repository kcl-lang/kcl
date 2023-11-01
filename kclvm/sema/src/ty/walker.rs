use std::sync::Arc;

use super::{Attr, DictType, Type, TypeRef};

/// Walk one type recursively and deal the type using the `walk_fn`
pub fn walk_type(ty: &Type, walk_fn: impl Fn(&Type) -> TypeRef + Copy) -> TypeRef {
    let ty = walk_fn(ty);
    match &ty.kind {
        super::TypeKind::List(item_ty) => Arc::new(Type::list(walk_type(item_ty, walk_fn))),
        super::TypeKind::Dict(DictType {
            key_ty,
            val_ty,
            attrs,
        }) => Arc::new(Type::dict_with_attrs(
            walk_type(key_ty, walk_fn),
            walk_type(val_ty, walk_fn),
            attrs
                .into_iter()
                .map(|(key, attr)| {
                    (
                        key.to_string(),
                        Attr {
                            ty: walk_type(&attr.ty, walk_fn),
                            range: attr.range.clone(),
                        },
                    )
                })
                .collect(),
        )),
        super::TypeKind::Union(types) => Arc::new(Type::union(
            &types
                .iter()
                .map(|ty| walk_type(ty, walk_fn))
                .collect::<Vec<TypeRef>>(),
        )),
        _ => ty,
    }
}
