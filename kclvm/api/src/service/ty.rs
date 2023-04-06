use crate::gpyrpc::{Decorator, KclType};
use indexmap::IndexSet;
use kclvm_runtime::SCHEMA_SETTINGS_ATTR_NAME;
use kclvm_sema::ty::{SchemaType, Type};
use std::collections::HashMap;

/// Convert the kcl sematic type to the kcl protobuf type.
pub(crate) fn kcl_ty_to_pb_ty(ty: &Type) -> KclType {
    match &ty.kind {
        kclvm_sema::ty::TypeKind::List(item_ty) => KclType {
            r#type: "list".to_string(),
            item: Some(Box::new(kcl_ty_to_pb_ty(item_ty))),
            ..Default::default()
        },
        kclvm_sema::ty::TypeKind::Dict(key_ty, val_ty) => KclType {
            r#type: "dict".to_string(),
            key: Some(Box::new(kcl_ty_to_pb_ty(key_ty))),
            item: Some(Box::new(kcl_ty_to_pb_ty(val_ty))),
            ..Default::default()
        },
        kclvm_sema::ty::TypeKind::Union(types) => KclType {
            r#type: "union".to_string(),
            union_types: types.iter().map(|ty| kcl_ty_to_pb_ty(ty)).collect(),
            ..Default::default()
        },
        kclvm_sema::ty::TypeKind::Schema(schema_ty) => kcl_schema_ty_to_pb_ty(schema_ty),
        _ => KclType {
            r#type: ty.ty_str(),
            ..Default::default()
        },
    }
}

/// Convert the kcl sematic type to the kcl protobuf type.
pub(crate) fn kcl_schema_ty_to_pb_ty(schema_ty: &SchemaType) -> KclType {
    KclType {
        r#type: "schema".to_string(),
        schema_name: schema_ty.name.clone(),
        schema_doc: schema_ty.doc.clone(),
        properties: get_schema_ty_attributes(schema_ty, &mut 1),
        required: get_schema_ty_required_attributes(schema_ty),
        decorators: schema_ty
            .decorators
            .iter()
            .map(|d| Decorator {
                name: d.name.clone(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

fn get_schema_ty_attributes(schema_ty: &SchemaType, line: &mut i32) -> HashMap<String, KclType> {
    let mut base_type_mapping = if let Some(base) = &schema_ty.base {
        get_schema_ty_attributes(base, line)
    } else {
        HashMap::new()
    };
    let mut type_mapping = HashMap::new();
    for (key, attr) in &schema_ty.attrs {
        if key != SCHEMA_SETTINGS_ATTR_NAME {
            let mut ty = kcl_ty_to_pb_ty(&attr.ty);
            ty.line = *line;
            type_mapping.insert(key.to_string(), ty);
            *line += 1
        }
    }
    for (k, ty) in type_mapping {
        base_type_mapping.insert(k, ty);
    }
    base_type_mapping
}

fn get_schema_ty_required_attributes(schema_ty: &SchemaType) -> Vec<String> {
    let base_attr_set = if let Some(base) = &schema_ty.base {
        get_schema_ty_required_attributes(base)
    } else {
        Vec::new()
    };
    let mut attr_set = IndexSet::new();
    for (key, _) in &schema_ty.attrs {
        if key != SCHEMA_SETTINGS_ATTR_NAME {
            attr_set.insert(key.to_string());
        }
    }
    for k in base_attr_set {
        attr_set.insert(k);
    }
    attr_set.iter().cloned().collect()
}
