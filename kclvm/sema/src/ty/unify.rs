use std::{collections::HashSet, sync::Arc};

use indexmap::IndexMap;

use super::{SchemaType, Type, TypeKind, TypeRef};

/// The type can be assigned to the expected type.
///
/// For security and performance considerations, dynamic dispatch of
/// types is not supported at this stage.
pub fn subsume(ty_lhs: TypeRef, ty_rhs: TypeRef, check_left_any: bool) -> bool {
    if (check_left_any && ty_lhs.is_any()) || (ty_rhs.is_any() || ty_lhs.is_none()) {
        true
    } else if ty_lhs.is_union() {
        let types = ty_lhs.union_types();
        types
            .iter()
            .all(|ty| subsume(ty.clone(), ty_rhs.clone(), false))
    } else if ty_rhs.is_union() {
        let types = ty_rhs.union_types();
        types
            .iter()
            .any(|ty| subsume(ty_lhs.clone(), ty.clone(), false))
    } else if ty_lhs.is_schema() {
        match &ty_rhs.kind {
            TypeKind::Schema(ty_rhs_schema) => {
                is_sub_schema_of(&ty_lhs.into_schema_type(), ty_rhs_schema)
            }
            _ => false,
        }
    } else if ty_lhs.is_int() && ty_rhs.is_float() {
        true
    } else if ty_lhs.is_number_multiplier() && ty_rhs.is_number_multiplier() {
        let ty_lhs = ty_lhs.into_number_multiplier();
        let ty_rhs = ty_rhs.into_number_multiplier();
        if ty_lhs.is_literal && ty_rhs.is_literal {
            ty_lhs.raw_value == ty_rhs.raw_value && ty_lhs.binary_suffix == ty_rhs.binary_suffix
        } else if ty_lhs.is_literal && !ty_rhs.is_literal {
            true
        } else {
            ty_lhs.is_literal || !ty_rhs.is_literal
        }
    } else if ty_lhs.is_primitive() && ty_rhs.is_primitive() {
        ty_lhs.kind == ty_rhs.kind
    } else if ty_lhs.is_literal() {
        if ty_rhs.is_literal() {
            ty_lhs.kind == ty_rhs.kind
        } else if ty_rhs.is_primitive() {
            // float_lit -> float
            // int_lit -> int
            // bool_lit -> bool
            // str_lit -> str
            // int_lit/bool_lit -> float
            if ty_rhs.is_float() && !ty_lhs.is_str() {
                true
            } else {
                ty_lhs.ty_str().contains(&ty_rhs.ty_str())
            }
        } else {
            false
        }
    } else if ty_lhs.is_list() && ty_rhs.is_list() {
        subsume(ty_lhs.list_item_ty(), ty_rhs.list_item_ty(), check_left_any)
    } else if ty_lhs.is_dict() && ty_rhs.is_dict() {
        let (ty_lhs_key, ty_lhs_val) = ty_lhs.dict_entry_ty();
        let (ty_rhs_key, ty_rhs_val) = ty_rhs.dict_entry_ty();
        subsume(ty_lhs_key, ty_rhs_key, check_left_any)
            && subsume(ty_lhs_val, ty_rhs_val, check_left_any)
    } else if ty_lhs.is_str() && ty_rhs.is_literal() && ty_rhs.is_str() {
        return true;
    } else if ty_lhs.is_func() && ty_rhs.is_func() {
        let ty_lhs_fn_ty = ty_lhs.into_func_type();
        let ty_rhs_fn_ty = ty_rhs.into_func_type();
        let mut is_ok = ty_lhs_fn_ty.params.len() == ty_rhs_fn_ty.params.len();
        for (ty_lhs_param, ty_rhs_param) in
            ty_lhs_fn_ty.params.iter().zip(ty_rhs_fn_ty.params.iter())
        {
            is_ok = is_ok
                && (subsume(
                    ty_rhs_param.ty.clone(),
                    ty_lhs_param.ty.clone(),
                    check_left_any,
                ));
        }
        is_ok = is_ok
            && (subsume(
                ty_lhs_fn_ty.return_ty.clone(),
                ty_rhs_fn_ty.return_ty.clone(),
                check_left_any,
            ));
        is_ok
    } else {
        equal(ty_lhs, ty_rhs)
    }
}

/// Are the two types exactly equal.
#[inline]
pub fn equal(ty_lhs: TypeRef, ty_rhs: TypeRef) -> bool {
    ty_lhs.kind == ty_rhs.kind
}

/// Whether the schema is sub schema of another schema.
pub fn is_sub_schema_of(schema_ty_lhs: &SchemaType, schema_ty_rhs: &SchemaType) -> bool {
    if schema_ty_lhs.ty_str_with_pkgpath() == schema_ty_rhs.ty_str_with_pkgpath() {
        true
    } else {
        match &schema_ty_lhs.base {
            Some(base) => is_sub_schema_of(base, schema_ty_rhs),
            None => false,
        }
    }
}

/// The type can be assigned to the expected type.
#[inline]
pub fn assignable_to(ty: TypeRef, expected_ty: TypeRef) -> bool {
    if !ty.is_assignable_type() {
        return false;
    }
    subsume(ty, expected_ty, true)
}

/// Whether `lhs_ty` is the upper bound of the `rhs_ty`
#[inline]
pub fn is_upper_bound(lhs_ty: TypeRef, rhs_ty: TypeRef) -> bool {
    subsume(rhs_ty, lhs_ty, false)
}

/// Whether the type list contains the `any` type.
#[inline]
pub fn has_any_type(types: &[TypeRef]) -> bool {
    types.iter().any(|ty| ty.is_any())
}

/// The sup function returns the minimum supremum of all types in an array of types.
#[inline]
pub fn sup(types: &[TypeRef]) -> TypeRef {
    r#typeof(types, true)
}

/// Typeof types
pub fn r#typeof(types: &[TypeRef], should_remove_sub_types: bool) -> TypeRef {
    // 1. Initialize an ordered set to store the type array
    let mut type_set: IndexMap<*const Type, TypeRef> = IndexMap::default();
    // 2. Add the type array to the ordered set for sorting by the type id and de-duplication.
    add_types_to_type_set(&mut type_set, types);
    // 3. Remove sub types according to partial order relation rules e.g. sub schema types.
    if should_remove_sub_types {
        let mut remove_index_set = HashSet::new();
        for (i, (source_addr, source)) in type_set.iter().enumerate() {
            for j in i + 1..type_set.len() {
                let (target_addr, target) = type_set.get_index(j).unwrap();
                if subsume(source.clone(), target.clone(), false) {
                    remove_index_set.insert(*source_addr);
                } else if subsume(target.clone(), source.clone(), false) {
                    remove_index_set.insert(*target_addr);
                }
            }
        }
        for i in remove_index_set {
            type_set.remove(&i);
        }
    }
    if type_set.is_empty() {
        Arc::new(Type::ANY)
    } else if type_set.len() == 1 {
        type_set[0].clone()
    } else {
        Arc::new(Type::union(
            &type_set.values().cloned().collect::<Vec<TypeRef>>(),
        ))
    }
}

fn add_types_to_type_set(type_set: &mut IndexMap<*const Type, TypeRef>, types: &[TypeRef]) {
    for ty in types {
        add_type_to_type_set(type_set, ty.clone());
    }
}

fn add_type_to_type_set(type_set: &mut IndexMap<*const Type, TypeRef>, ty: TypeRef) {
    match &ty.kind {
        TypeKind::Union(types) => {
            add_types_to_type_set(type_set, types);
        }
        _ => {
            let ref_addr = ty.as_ref() as *const Type;
            // Remove the bottom type.
            if !ty.is_void() && !type_set.contains_key(&ref_addr) {
                type_set.insert(ref_addr, ty.clone());
            }
        }
    }
}
