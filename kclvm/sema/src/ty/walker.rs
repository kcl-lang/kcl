use std::rc::Rc;

use super::Type;

/// Walk one type recursively and deal the type using the `walk_fn`
pub fn walk_type(ty: &Type, walk_fn: impl Fn(&Type) -> Rc<Type> + Copy) -> Rc<Type> {
    let ty = walk_fn(ty);
    match &ty.kind {
        super::TypeKind::List(item_ty) => Rc::new(Type::list(walk_type(item_ty, walk_fn))),
        super::TypeKind::Dict(key_ty, val_ty) => Rc::new(Type::dict(
            walk_type(key_ty, walk_fn),
            walk_type(val_ty, walk_fn),
        )),
        super::TypeKind::Union(types) => Rc::new(Type::union(
            &types
                .iter()
                .map(|ty| walk_type(ty, walk_fn))
                .collect::<Vec<Rc<Type>>>(),
        )),
        _ => ty,
    }
}
