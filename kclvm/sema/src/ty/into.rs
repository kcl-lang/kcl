use super::*;

impl Type {
    /// Downcast ty into the list type.
    #[inline]
    pub fn list_item_ty(&self) -> Rc<Type> {
        match &self.kind {
            TypeKind::List(item_ty) => item_ty.clone(),
            _ => bug!("invalid list type {}", self.ty_str()),
        }
    }
    /// Downcast ty into the dict entry type.
    #[inline]
    pub fn dict_entry_ty(&self) -> (Rc<Type>, Rc<Type>) {
        match &self.kind {
            TypeKind::Dict(key_ty, val_ty) => (key_ty.clone(), val_ty.clone()),
            _ => bug!("invalid dict type {}", self.ty_str()),
        }
    }
    /// Downcast ty into the config key type.
    #[inline]
    pub fn config_key_ty(&self) -> Rc<Type> {
        match &self.kind {
            TypeKind::Dict(key_ty, _) => key_ty.clone(),
            TypeKind::Schema(schema_ty) => schema_ty.key_ty(),
            _ => bug!("invalid config type {}", self.ty_str()),
        }
    }
    /// Downcast ty into the config value type.
    #[inline]
    pub fn config_val_ty(&self) -> Rc<Type> {
        match &self.kind {
            TypeKind::Dict(_, val_ty) => val_ty.clone(),
            TypeKind::Schema(schema_ty) => schema_ty.val_ty(),
            _ => bug!("invalid config type {}", self.ty_str()),
        }
    }
    /// Get types from the union type.
    #[inline]
    pub fn union_types(&self) -> Vec<Rc<Type>> {
        match &self.kind {
            TypeKind::Union(types) => types.clone(),
            _ => bug!("invalid {} into union type", self.ty_str()),
        }
    }
    /// Into schema type.
    #[inline]
    pub fn into_schema_type(&self) -> SchemaType {
        match &self.kind {
            TypeKind::Schema(schema_ty) => schema_ty.clone(),
            _ => bug!("invalid type {} into schema type", self.ty_str()),
        }
    }
    /// Into number multiplier type.
    #[inline]
    pub fn into_number_multiplier(&self) -> NumberMultiplierType {
        match &self.kind {
            TypeKind::NumberMultiplier(number_multiplier) => number_multiplier.clone(),
            _ => bug!("invalid type {} into number multiplier type", self.ty_str()),
        }
    }
    /// Get the type string.
    pub fn into_type_annotation_str(&self) -> String {
        match &self.kind {
            TypeKind::None => NAME_CONSTANT_NONE.to_string(),
            TypeKind::BoolLit(v) => (if *v {
                NAME_CONSTANT_TRUE
            } else {
                NAME_CONSTANT_FALSE
            })
            .to_string(),
            TypeKind::IntLit(v) => v.to_string(),
            TypeKind::FloatLit(v) => {
                let mut float_str = v.to_string();
                if !float_str.contains('.') {
                    float_str.push_str(".0");
                }
                float_str
            }
            TypeKind::StrLit(v) => format!("\"{}\"", v.replace('"', "\\\"")),
            TypeKind::List(item_ty) => format!("[{}]", item_ty.into_type_annotation_str()),
            TypeKind::Dict(key_ty, val_ty) => {
                format!(
                    "{{{}:{}}}",
                    key_ty.into_type_annotation_str(),
                    val_ty.into_type_annotation_str()
                )
            }
            TypeKind::Union(types) => types
                .iter()
                .map(|ty| ty.into_type_annotation_str())
                .collect::<Vec<String>>()
                .join("|"),
            TypeKind::Schema(schema_ty) => schema_ty.ty_str_with_pkgpath(),
            TypeKind::NumberMultiplier(number_multiplier) => {
                if number_multiplier.is_literal {
                    format!(
                        "{}({}{})",
                        NUMBER_MULTIPLIER_TYPE_STR,
                        number_multiplier.raw_value,
                        number_multiplier.binary_suffix
                    )
                } else {
                    NUMBER_MULTIPLIER_PKG_TYPE_STR.to_string()
                }
            }
            _ => self.ty_str(),
        }
    }
}

impl From<ast::Type> for Type {
    fn from(ty: ast::Type) -> Type {
        match ty {
            ast::Type::Any => Type::ANY,
            ast::Type::Basic(basic_ty) => match basic_ty {
                ast::BasicType::Bool => Type::BOOL,
                ast::BasicType::Int => Type::INT,
                ast::BasicType::Float => Type::FLOAT,
                ast::BasicType::Str => Type::STR,
            },
            ast::Type::Named(identifier) => Type::named(&identifier.get_name()),
            ast::Type::List(list_ty) => Type::list(
                list_ty
                    .inner_type
                    .as_ref()
                    .map_or(Rc::new(Type::ANY), |ty| Rc::new(ty.node.clone().into())),
            ),
            ast::Type::Dict(dict_ty) => Type::dict(
                dict_ty
                    .key_type
                    .as_ref()
                    .map_or(Rc::new(Type::ANY), |ty| Rc::new(ty.node.clone().into())),
                dict_ty
                    .value_type
                    .as_ref()
                    .map_or(Rc::new(Type::ANY), |ty| Rc::new(ty.node.clone().into())),
            ),
            ast::Type::Union(union_ty) => Type::union(
                &union_ty
                    .type_elements
                    .iter()
                    .map(|ty| Rc::new(ty.node.clone().into()))
                    .collect::<Vec<Rc<Type>>>(),
            ),
            ast::Type::Literal(literal_ty) => match literal_ty {
                ast::LiteralType::Bool(v) => Type::bool_lit(v),
                ast::LiteralType::Int(v, suffix_option) => match suffix_option {
                    Some(suffix) => Type::number_multiplier(
                        kclvm::cal_num(v, &suffix.value()),
                        v,
                        &suffix.value(),
                    ),
                    None => Type::int_lit(v),
                },
                ast::LiteralType::Float(v) => Type::float_lit(v),
                ast::LiteralType::Str(v) => Type::str_lit(&v),
            },
        }
    }
}
