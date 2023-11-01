use std::sync::Arc;

use super::{Type, TypeFlags, TypeKind};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

/* Type string constants */

pub const INT_TYPE_STR: &str = "int";
pub const FLOAT_TYPE_STR: &str = "float";
pub const STR_TYPE_STR: &str = "str";
pub const BOOL_TYPE_STR: &str = "bool";
pub const ANY_TYPE_STR: &str = "any";
pub const NONE_TYPE_STR: &str = "NoneType";
pub const UNDEFINED_TYPE_STR: &str = "UndefinedType";
pub const FUNCTION_TYPE_STR: &str = "function";
pub const LIST_TYPE_STR: &str = "list";
pub const DICT_TYPE_STR: &str = "dict";
pub const SCHEMA_TYPE_STR: &str = "schema";
pub const NUMBER_MULTIPLIER_TYPE_STR: &str = "number_multiplier";
pub const NUMBER_MULTIPLIER_PKG_TYPE_STR: &str = "units.NumberMultiplier";
pub const NUMBER_MULTIPLIER_REGEX: &str =
    r"^([1-9][0-9]{0,63})(E|P|T|G|M|K|k|m|u|n|Ei|Pi|Ti|Gi|Mi|Ki)$";

pub const ITERABLE_TYPE_STR: &str = "str|{:}|[]";
pub const NUMBER_TYPE_STR: &str = "int|float|bool";
pub const NUM_OR_STR_TYPE_STR: &str = "int|float|bool|str";
pub const RESERVED_TYPE_IDENTIFIERS: [&str; 5] = [
    ANY_TYPE_STR,
    INT_TYPE_STR,
    FLOAT_TYPE_STR,
    STR_TYPE_STR,
    BOOL_TYPE_STR,
];
pub const BUILTIN_TYPES: [&str; 4] = [INT_TYPE_STR, FLOAT_TYPE_STR, STR_TYPE_STR, BOOL_TYPE_STR];

pub const MODULE_TYPE_STR: &str = "module";
pub const NAMED_TYPE_STR: &str = "named";
pub const VOID_TYPE_STR: &str = "void";

pub const NAME_CONSTANT_TRUE: &str = "True";
pub const NAME_CONSTANT_FALSE: &str = "False";
pub const NAME_CONSTANT_NONE: &str = "None";
pub const NAME_CONSTANT_UNDEFINED: &str = "Undefined";
pub const NAME_CONSTANTS: [&str; 4] = [
    NAME_CONSTANT_NONE,
    NAME_CONSTANT_UNDEFINED,
    NAME_CONSTANT_TRUE,
    NAME_CONSTANT_FALSE,
];

pub const TYPES_MAPPING: Lazy<IndexMap<String, Type>> = Lazy::new(|| {
    let mut mapping = IndexMap::default();
    mapping.insert(INT_TYPE_STR.to_string(), Type::INT);
    mapping.insert(FLOAT_TYPE_STR.to_string(), Type::FLOAT);
    mapping.insert(STR_TYPE_STR.to_string(), Type::STR);
    mapping.insert(BOOL_TYPE_STR.to_string(), Type::BOOL);
    mapping.insert(ANY_TYPE_STR.to_string(), Type::ANY);
    mapping.insert("[]".to_string(), Type::list(Arc::new(Type::ANY)));
    mapping.insert("[any]".to_string(), Type::list(Arc::new(Type::ANY)));
    mapping.insert("[str]".to_string(), Type::list(Arc::new(Type::STR)));
    mapping.insert(
        "{:}".to_string(),
        Type::dict(Arc::new(Type::ANY), Arc::new(Type::ANY)),
    );
    mapping.insert(
        "{str:}".to_string(),
        Type::dict(Arc::new(Type::STR), Arc::new(Type::ANY)),
    );
    mapping.insert(
        "{str:any}".to_string(),
        Type::dict(Arc::new(Type::STR), Arc::new(Type::ANY)),
    );
    mapping.insert(
        "{str:str}".to_string(),
        Type::dict(Arc::new(Type::STR), Arc::new(Type::STR)),
    );
    mapping
});
pub const ZERO_LIT_TYPES: Lazy<Vec<Type>> = Lazy::new(|| {
    vec![
        Type::int_lit(0),
        Type::float_lit(0.0),
        Type::bool_lit(false),
    ]
});
pub static SCHEMA_MEMBER_FUNCTIONS: Lazy<Vec<&'static str>> = Lazy::new(|| vec!["instances"]);

impl Type {
    /* Type constant definitions */

    /// Type constant `void`.
    pub const VOID: Type = Type {
        kind: TypeKind::Void,
        flags: TypeFlags::VOID,
        is_type_alias: false,
    };
    /// Type constant `int`.
    pub const INT: Type = Type {
        kind: TypeKind::Int,
        flags: TypeFlags::INT,
        is_type_alias: false,
    };
    /// Type constant `float`.
    pub const FLOAT: Type = Type {
        kind: TypeKind::Float,
        flags: TypeFlags::FLOAT,
        is_type_alias: false,
    };
    /// Type constant `str`.
    pub const STR: Type = Type {
        kind: TypeKind::Str,
        flags: TypeFlags::STR,
        is_type_alias: false,
    };
    /// Type constant `bool`.
    pub const BOOL: Type = Type {
        kind: TypeKind::Bool,
        flags: TypeFlags::BOOL,
        is_type_alias: false,
    };
    /// Type constant `any`.
    pub const ANY: Type = Type {
        kind: TypeKind::Any,
        flags: TypeFlags::ANY,
        is_type_alias: false,
    };
    /// Type constant `NoneType` including the name constants `None` and `Undefined`.
    pub const NONE: Type = Type {
        kind: TypeKind::None,
        flags: TypeFlags::NONE,
        is_type_alias: false,
    };
}
