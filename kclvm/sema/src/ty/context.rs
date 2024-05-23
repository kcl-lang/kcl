use std::collections::HashMap;
use std::sync::Arc;

use super::{sup, DictType, Type, TypeFlags, TypeKind, TypeRef};
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};

/// TypeContext responsible for type generation, calculation,
/// and equality and subtype judgment between types.
#[derive(Debug)]
pub struct TypeContext {
    pub dep_graph: DiGraph<String, ()>,
    pub builtin_types: BuiltinTypes,
    node_index_map: HashMap<String, NodeIndex>,
}

#[derive(Debug)]
pub struct BuiltinTypes {
    pub any: TypeRef,
    pub bool: TypeRef,
    pub int: TypeRef,
    pub float: TypeRef,
    pub str: TypeRef,
    pub void: TypeRef,
    pub none: TypeRef,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    /// New a type context.
    pub fn new() -> Self {
        TypeContext {
            dep_graph: DiGraph::new(),
            builtin_types: BuiltinTypes {
                any: Arc::new(Type::ANY),
                bool: Arc::new(Type::BOOL),
                int: Arc::new(Type::INT),
                float: Arc::new(Type::FLOAT),
                str: Arc::new(Type::STR),
                void: Arc::new(Type::VOID),
                none: Arc::new(Type::NONE),
            },
            node_index_map: HashMap::new(),
        }
    }

    /// Return true if the dep graph contains a cycle.
    #[inline]
    pub fn is_cyclic(&self) -> bool {
        is_cyclic_directed(&self.dep_graph)
    }

    /// Add dependencies between "from" and "to".
    pub fn add_dependencies(&mut self, from: &str, to: &str) {
        let from_idx = self.get_or_insert_node_index(from);
        let to_idx = self.get_or_insert_node_index(to);
        self.dep_graph.add_edge(from_idx, to_idx, ());
    }

    /// Get the node index from the node index map or insert it into the dependency graph.
    #[inline]
    fn get_or_insert_node_index(&mut self, name: &str) -> NodeIndex {
        match self.node_index_map.get(name) {
            Some(idx) => *idx,
            None => {
                let idx = self.dep_graph.add_node(name.to_string());
                self.node_index_map.insert(name.to_string(), idx);
                idx
            }
        }
    }

    /// Convert the literal union type to its variable type
    /// e.g., 1|2 -> int, 's'|'ss' -> str.
    pub fn literal_union_type_to_variable_type(&self, ty: TypeRef) -> TypeRef {
        if ty.is_union() {
            self.infer_to_variable_type(ty)
        } else {
            ty
        }
    }

    /// Judge a type kind in the type kind list or the union
    /// type kinds are all in the type kind.
    pub fn is_kind_type_or_kind_union_type(&self, ty: TypeRef, flags: &[TypeFlags]) -> bool {
        match &ty.kind {
            TypeKind::Union(types) => types
                .iter()
                .all(|ty| flags.iter().any(|flag| ty.contains_flags(*flag))),
            _ => flags.iter().any(|flag| ty.contains_flags(*flag)),
        }
    }

    #[inline]
    pub fn is_number_type_or_number_union_type(&self, ty: TypeRef) -> bool {
        self.is_kind_type_or_kind_union_type(ty, &[TypeFlags::INT, TypeFlags::FLOAT])
    }

    #[inline]
    pub fn is_number_bool_type_or_number_bool_union_type(&self, ty: TypeRef) -> bool {
        self.is_kind_type_or_kind_union_type(
            ty,
            &[TypeFlags::INT, TypeFlags::FLOAT, TypeFlags::BOOL],
        )
    }

    #[inline]
    pub fn is_config_type_or_config_union_type(&self, ty: TypeRef) -> bool {
        self.is_kind_type_or_kind_union_type(ty, &[TypeFlags::DICT, TypeFlags::SCHEMA])
    }

    #[inline]
    pub fn is_str_type_or_str_union_type(&self, ty: TypeRef) -> bool {
        self.is_kind_type_or_kind_union_type(ty, &[TypeFlags::STR])
    }

    #[inline]
    pub fn is_primitive_type_or_primitive_union_type(&self, ty: TypeRef) -> bool {
        self.is_kind_type_or_kind_union_type(
            ty,
            &[
                TypeFlags::INT,
                TypeFlags::FLOAT,
                TypeFlags::BOOL,
                TypeFlags::STR,
            ],
        )
    }

    #[inline]
    pub fn is_mul_val_type_or_mul_val_union_type(&self, ty: TypeRef) -> bool {
        self.is_kind_type_or_kind_union_type(
            ty,
            &[
                TypeFlags::INT,
                TypeFlags::FLOAT,
                TypeFlags::STR,
                TypeFlags::LIST,
            ],
        )
    }

    /// Convert type to the real type annotation
    #[inline]
    pub fn into_type_annotation_str(&self, ty: TypeRef) -> String {
        ty.into_type_annotation_str()
    }
}

pub trait TypeInferMethods {
    /// Infer the value type to the variable type"
    fn infer_to_variable_type(&self, ty: TypeRef) -> TypeRef;
}

impl TypeInferMethods for TypeContext {
    /// Infer the value type to the variable type"
    fn infer_to_variable_type(&self, ty: TypeRef) -> TypeRef {
        match &ty.kind {
            // None/Undefined type to any type e.g., None -> any
            TypeKind::None => self.builtin_types.any.clone(),
            // Literal type to its named type e.g., 1 -> int, "s" -> str
            TypeKind::BoolLit(_) => self.builtin_types.bool.clone(),
            TypeKind::IntLit(_) => self.builtin_types.int.clone(),
            TypeKind::FloatLit(_) => self.builtin_types.float.clone(),
            TypeKind::StrLit(_) => self.builtin_types.str.clone(),
            TypeKind::List(item_ty) => Type::list_ref(self.infer_to_variable_type(item_ty.clone())),
            // Dict type e.g., {str:1|2} -> {str:int}
            TypeKind::Dict(DictType {
                key_ty,
                val_ty,
                attrs,
            }) => Type::dict_ref_with_attrs(
                self.infer_to_variable_type(key_ty.clone()),
                self.infer_to_variable_type(val_ty.clone()),
                attrs.clone(),
            ),
            // Union type e.g., 1|2|"s" -> int|str
            TypeKind::Union(types) => sup(&types
                .iter()
                .map(|ty| self.infer_to_variable_type(ty.clone()))
                .collect::<Vec<TypeRef>>()),
            _ => ty.clone(),
        }
    }
}
