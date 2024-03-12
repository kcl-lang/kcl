use super::*;

impl Type {
    /// Construct an int type reference.
    #[inline]
    pub fn int_ref() -> TypeRef {
        Arc::new(Type::INT)
    }
    /// Construct a float type reference.
    #[inline]
    pub fn float_ref() -> TypeRef {
        Arc::new(Type::FLOAT)
    }
    /// Construct a bool type reference.
    #[inline]
    pub fn bool_ref() -> TypeRef {
        Arc::new(Type::BOOL)
    }
    /// Construct a str type reference.
    #[inline]
    pub fn str_ref() -> TypeRef {
        Arc::new(Type::STR)
    }
    /// Construct a any type reference.
    #[inline]
    pub fn any_ref() -> TypeRef {
        Arc::new(Type::ANY)
    }
    /// Construct a union type
    #[inline]
    pub fn union(types: &[TypeRef]) -> Type {
        Type {
            kind: TypeKind::Union(types.to_owned()),
            flags: TypeFlags::UNION,
            is_type_alias: false,
        }
    }
    /// Construct an union type reference.
    #[inline]
    pub fn union_ref(types: &[TypeRef]) -> TypeRef {
        Arc::new(Self::union(types))
    }
    /// Construct a list type
    #[inline]
    pub fn list(item_ty: TypeRef) -> Type {
        Type {
            kind: TypeKind::List(item_ty),
            flags: TypeFlags::LIST,
            is_type_alias: false,
        }
    }
    /// Construct a list type ref
    #[inline]
    pub fn list_ref(item_ty: TypeRef) -> TypeRef {
        Arc::new(Self::list(item_ty))
    }
    /// Construct a dict type
    #[inline]
    pub fn dict(key_ty: TypeRef, val_ty: TypeRef) -> Type {
        Type {
            kind: TypeKind::Dict(DictType {
                key_ty,
                val_ty,
                attrs: IndexMap::new(),
            }),
            flags: TypeFlags::DICT,
            is_type_alias: false,
        }
    }
    /// Construct a dict type ref
    #[inline]
    pub fn dict_ref(key_ty: TypeRef, val_ty: TypeRef) -> TypeRef {
        Arc::new(Self::dict(key_ty, val_ty))
    }
    /// Construct a dict type with attrs
    #[inline]
    pub fn dict_with_attrs(
        key_ty: TypeRef,
        val_ty: TypeRef,
        attrs: IndexMap<String, Attr>,
    ) -> Type {
        Type {
            kind: TypeKind::Dict(DictType {
                key_ty,
                val_ty,
                attrs,
            }),
            flags: TypeFlags::DICT,
            is_type_alias: false,
        }
    }
    /// Construct a dict type reference with attrs
    #[inline]
    pub fn dict_ref_with_attrs(
        key_ty: TypeRef,
        val_ty: TypeRef,
        attrs: IndexMap<String, Attr>,
    ) -> TypeRef {
        Arc::new(Self::dict_with_attrs(key_ty, val_ty, attrs))
    }
    /// Construct a bool literal type.
    #[inline]
    pub fn bool_lit(val: bool) -> Type {
        Type {
            kind: TypeKind::BoolLit(val),
            flags: TypeFlags::BOOL | TypeFlags::LITERAL,
            is_type_alias: false,
        }
    }
    /// Construct a int literal type.
    #[inline]
    pub fn int_lit(num: i64) -> Type {
        Type {
            kind: TypeKind::IntLit(num),
            flags: TypeFlags::INT | TypeFlags::LITERAL,
            is_type_alias: false,
        }
    }
    /// Construct a float literal type.
    #[inline]
    pub fn float_lit(num: f64) -> Type {
        Type {
            kind: TypeKind::FloatLit(num),
            flags: TypeFlags::FLOAT | TypeFlags::LITERAL,
            is_type_alias: false,
        }
    }
    /// Construct a float literal type.
    #[inline]
    pub fn str_lit(val: &str) -> Type {
        Type {
            kind: TypeKind::StrLit(val.to_string()),
            flags: TypeFlags::STR | TypeFlags::LITERAL,
            is_type_alias: false,
        }
    }
    /// Construct a named type.
    #[inline]
    pub fn named(val: &str) -> Type {
        Type {
            kind: TypeKind::Named(val.to_string()),
            flags: TypeFlags::NAMED,
            is_type_alias: false,
        }
    }
    /// Construct a number multiplier type.
    #[inline]
    pub fn number_multiplier(value: f64, raw_value: i64, binary_suffix: &str) -> Type {
        Type {
            kind: TypeKind::NumberMultiplier(NumberMultiplierType {
                value,
                raw_value,
                binary_suffix: binary_suffix.to_string(),
                is_literal: true,
            }),
            flags: TypeFlags::NUMBER_MULTIPLIER,
            is_type_alias: false,
        }
    }
    #[inline]
    pub fn number_multiplier_non_lit_ty() -> Type {
        Type {
            kind: TypeKind::NumberMultiplier(NumberMultiplierType {
                value: 0.0,
                raw_value: 0,
                binary_suffix: "".to_string(),
                is_literal: false,
            }),
            flags: TypeFlags::NUMBER_MULTIPLIER,
            is_type_alias: false,
        }
    }
    /// Construct a function type.
    #[inline]
    pub fn function(
        self_ty: Option<TypeRef>,
        return_ty: TypeRef,
        params: &[Parameter],
        doc: &str,
        is_variadic: bool,
        kw_only_index: Option<usize>,
    ) -> Type {
        Type {
            kind: TypeKind::Function(FunctionType {
                doc: doc.to_string(),
                params: params.to_owned(),
                self_ty,
                return_ty,
                is_variadic,
                kw_only_index,
            }),
            flags: TypeFlags::FUNCTION,
            is_type_alias: false,
        }
    }
    /// Construct a module type.
    pub fn module(pkgpath: &str, imported: &[String], kind: ModuleKind) -> Type {
        Type {
            kind: TypeKind::Module(ModuleType {
                pkgpath: pkgpath.to_string(),
                imported: imported.to_owned(),
                kind,
            }),
            flags: TypeFlags::MODULE,
            is_type_alias: false,
        }
    }
    /// Construct a schema type.
    pub fn schema(schema_ty: SchemaType) -> Type {
        Type {
            kind: TypeKind::Schema(schema_ty),
            flags: TypeFlags::SCHEMA,
            is_type_alias: false,
        }
    }
    /// Construct a iterable type
    #[inline]
    pub fn iterable() -> TypeRef {
        Arc::new(Type::union(&[
            Arc::new(Type::STR),
            Arc::new(Type::dict(Arc::new(Type::ANY), Arc::new(Type::ANY))),
            Arc::new(Type::list(Arc::new(Type::ANY))),
        ]))
    }
    /// Construct a number type.
    #[inline]
    pub fn number() -> TypeRef {
        Type::union_ref(&[Type::int_ref(), Type::float_ref()])
    }
    /// Whether is a any type.
    #[inline]
    pub fn is_any(&self) -> bool {
        self.flags.contains(TypeFlags::ANY)
    }
    /// Whether is a int type.
    #[inline]
    pub fn is_int(&self) -> bool {
        self.flags.contains(TypeFlags::INT)
    }
    /// Whether is a float type.
    #[inline]
    pub fn is_float(&self) -> bool {
        self.flags.contains(TypeFlags::FLOAT)
    }
    /// Whether is a bool type.
    #[inline]
    pub fn is_bool(&self) -> bool {
        self.flags.contains(TypeFlags::BOOL)
    }
    /// Whether is a string type.
    #[inline]
    pub fn is_str(&self) -> bool {
        self.flags.contains(TypeFlags::STR)
    }
    /// Whether is a literal type.
    #[inline]
    pub fn is_literal(&self) -> bool {
        self.flags.contains(TypeFlags::LITERAL)
    }
    /// Whether is a key type.
    #[inline]
    pub fn is_key(&self) -> bool {
        match &self.kind {
            TypeKind::Str | TypeKind::StrLit(_) => true,
            TypeKind::Union(types) => types.iter().all(|ty| ty.is_key()),
            _ => false,
        }
    }
    /// Whether is a primitive type.
    #[inline]
    pub fn is_primitive(&self) -> bool {
        matches!(
            &self.kind,
            TypeKind::Bool | TypeKind::Int | TypeKind::Float | TypeKind::Str
        )
    }
    /// Whether is a None type.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.flags.contains(TypeFlags::NONE)
    }
    /// Whether is a None or any type.
    #[inline]
    pub fn is_none_or_any(&self) -> bool {
        self.is_none() || self.is_any()
    }
    /// Whether is a number type.
    #[inline]
    pub fn is_number(&self) -> bool {
        self.flags.contains(TypeFlags::INT) || self.flags.contains(TypeFlags::FLOAT)
    }
    /// Whether is a void type.
    #[inline]
    pub fn is_void(&self) -> bool {
        self.flags.contains(TypeFlags::VOID)
    }
    /// Whether is a list type.
    #[inline]
    pub fn is_list(&self) -> bool {
        self.flags.contains(TypeFlags::LIST)
    }
    /// Whether is a dict type.
    #[inline]
    pub fn is_dict(&self) -> bool {
        self.flags.contains(TypeFlags::DICT)
    }
    /// Whether is a schema type.
    #[inline]
    pub fn is_schema(&self) -> bool {
        self.flags.contains(TypeFlags::SCHEMA)
    }
    #[inline]
    pub fn is_schema_def(&self) -> bool {
        match &self.kind {
            TypeKind::Schema(schema_ty) => !schema_ty.is_instance,
            _ => false,
        }
    }
    /// Whether is a schema type.
    #[inline]
    pub fn is_dict_or_schema(&self) -> bool {
        self.is_dict() || self.is_schema()
    }
    /// Whether is a union type.
    #[inline]
    pub fn is_union(&self) -> bool {
        self.flags.contains(TypeFlags::UNION)
    }
    /// Whether is a iterable type.
    #[inline]
    pub fn is_iterable(&self) -> bool {
        self.is_str() || self.is_list() || self.is_dict() || self.is_schema()
    }
    /// Whether is a function type.
    #[inline]
    pub fn is_func(&self) -> bool {
        self.flags.contains(TypeFlags::FUNCTION)
    }
    /// Whether is a number multiplier type.
    #[inline]
    pub fn is_number_multiplier(&self) -> bool {
        self.flags.contains(TypeFlags::NUMBER_MULTIPLIER)
    }
    /// Whether is a module type.
    #[inline]
    pub fn is_module(&self) -> bool {
        self.flags.contains(TypeFlags::MODULE)
    }
    /// Whether is an assignable type.
    #[inline]
    pub fn is_assignable_type(&self) -> bool {
        match &self.kind {
            TypeKind::None
            | TypeKind::Any
            | TypeKind::Bool
            | TypeKind::BoolLit(_)
            | TypeKind::Int
            | TypeKind::IntLit(_)
            | TypeKind::Float
            | TypeKind::FloatLit(_)
            | TypeKind::Str
            | TypeKind::StrLit(_)
            | TypeKind::List(_)
            | TypeKind::Dict(DictType { .. })
            | TypeKind::Union(_)
            | TypeKind::Schema(_)
            | TypeKind::NumberMultiplier(_)
            | TypeKind::Function(_) => true,
            TypeKind::Void | TypeKind::Module(_) | TypeKind::Named(_) => false,
        }
    }
}
