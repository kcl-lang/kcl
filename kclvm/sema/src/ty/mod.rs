mod constants;
mod constructor;
mod context;
mod into;
pub mod parser;
mod unify;
mod walker;

use std::rc::Rc;

pub use constants::*;
pub use context::{TypeContext, TypeInferMethods};
use kclvm_ast::ast;
use kclvm_ast::MAIN_PKG;
use kclvm_error::Position;
pub use unify::*;
pub use walker::walk_type;

use indexmap::IndexMap;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    // The type kind.
    pub kind: TypeKind,
    // Is a type alias.
    pub is_type_alias: bool,
    /// This field provides fast access to information that is
    /// also contained in `kind`.
    flags: TypeFlags,
}

impl Type {
    /// Whether the type contains the flag.
    #[inline]
    pub fn contains_flags(&self, flag: TypeFlags) -> bool {
        self.flags.contains(flag)
    }
    /// Returns the type string used for error handler.
    pub fn ty_str(&self) -> String {
        match &self.kind {
            TypeKind::None => NONE_TYPE_STR.to_string(),
            TypeKind::Any => ANY_TYPE_STR.to_string(),
            TypeKind::Bool => BOOL_TYPE_STR.to_string(),
            TypeKind::BoolLit(v) => format!("{}({})", BOOL_TYPE_STR, v),
            TypeKind::Int => INT_TYPE_STR.to_string(),
            TypeKind::IntLit(v) => format!("{}({})", INT_TYPE_STR, v),
            TypeKind::Float => FLOAT_TYPE_STR.to_string(),
            TypeKind::FloatLit(v) => format!("{}({})", FLOAT_TYPE_STR, v),
            TypeKind::Str => STR_TYPE_STR.to_string(),
            TypeKind::StrLit(v) => format!("{}({})", STR_TYPE_STR, v),
            TypeKind::List(item_ty) => format!("[{}]", item_ty.ty_str()),
            TypeKind::Dict(key_ty, val_ty) => {
                format!("{{{}:{}}}", key_ty.ty_str(), val_ty.ty_str())
            }
            TypeKind::Union(types) => types
                .iter()
                .map(|ty| ty.ty_str())
                .collect::<Vec<String>>()
                .join("|"),
            TypeKind::Schema(schema_ty) => schema_ty.name.to_string(),
            TypeKind::NumberMultiplier(number_multiplier) => number_multiplier.ty_str(),
            TypeKind::Function(_) => FUNCTION_TYPE_STR.to_string(),
            TypeKind::Void => VOID_TYPE_STR.to_string(),
            TypeKind::Module(module_ty) => format!("{} '{}'", MODULE_TYPE_STR, module_ty.pkgpath),
            TypeKind::Named(name) => name.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// A primitive None name constant.
    None,
    /// The any type.
    Any,
    /// The primitive boolean type. Written as `bool`.
    Bool,
    BoolLit(bool),
    /// A primitive integer type. Written as `int`.
    Int,
    /// A primitive integer literal type.
    IntLit(i64),
    /// A primitive float type. Written as `float`.
    Float,
    /// A primitive float literal type.
    FloatLit(f64),
    /// The primitive string type; holds a Unicode scalar value
    /// (a non-surrogate code point). Written as `str`.
    Str,
    /// A primitive string literal type.
    StrLit(String),
    /// The pointer of an array slice. Written as `[T]`.
    List(Rc<Type>),
    /// A map type. Written as `{kT, vT}`.
    Dict(Rc<Type>, Rc<Type>),
    /// A union type. Written as ty1 | ty2 | ... | tyn
    Union(Vec<Rc<Type>>),
    /// A schema type.
    Schema(SchemaType),
    /// A number multiplier type.
    NumberMultiplier(NumberMultiplierType),
    /// The function type.
    Function(FunctionType),
    /// The bottom never type.
    Void,
    /// The module type.
    Module(ModuleType),
    /// A named type alias.
    Named(String),
}

bitflags::bitflags! {
    /// TypeFlags provides fast access to information that is also contained
    /// in `kind`.
    pub struct TypeFlags: u16 {
        const VOID = 1 << 0;
        const INT = 1 << 1;
        const FLOAT = 1 << 2;
        const STR = 1 << 3;
        const BOOL = 1 << 4;
        const ANY = 1 << 5;
        const NONE = 1 << 6;
        const LIST = 1 << 7;
        const DICT = 1 << 8;
        const SCHEMA = 1 << 9;
        const UNION = 1 << 10;
        const LITERAL = 1 << 11;
        const NUMBER_MULTIPLIER = 1 << 12;
        const FUNCTION = 1 << 13;
        const MODULE = 1 << 14;
        const NAMED = 1 << 15;
    }
}

/// The schema type.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaType {
    /// The schema name.
    pub name: String,
    /// The schema definition package path
    pub pkgpath: String,
    /// The schema definition file path.
    pub filename: String,
    /// The schema definition document string.
    pub doc: String,
    /// Indicates whether the schema is a type of a instance or
    /// a type (value). Besides, it is necessary to distinguish
    /// between a type instance and a type value, such as the following code:
    /// ```no_check
    /// # `Person` in `schema Person` is a type and it is not a schema instance.
    /// schema Person:
    ///     name: str
    ///
    /// # `person` is a schema instance.
    /// person = Person {name = "Alice"}
    /// # `person` is a schema instance used in the value expression.
    /// name = person.name
    /// # `Person` in `persons: [Person]` is a type, `Person` in `Person.instances()`
    /// # is a type value, and they are not schema instances.
    /// persons: [Person] = Person.instances()
    /// ```
    pub is_instance: bool,
    /// Indicates whether it is a schema mixin.
    pub is_mixin: bool,
    /// Indicates whether it is a schema protocol.
    pub is_protocol: bool,
    /// Indicates whether it is a rule.
    pub is_rule: bool,
    /// Base schema.
    pub base: Option<Box<SchemaType>>,
    /// Protocol schema.
    pub protocol: Option<Box<SchemaType>>,
    /// Schema Mixins.
    pub mixins: Vec<SchemaType>,
    /// Schema attributes.
    pub attrs: IndexMap<String, SchemaAttr>,
    /// Schema function type.
    pub func: Box<FunctionType>,
    /// Schema index signature.
    pub index_signature: Option<Box<SchemaIndexSignature>>,
    /// Schema decorators including self and attribute decorators.
    pub decorators: Vec<Decorator>,
}

impl SchemaType {
    /// Get the object type string with pkgpath
    pub fn ty_str_with_pkgpath(&self) -> String {
        if self.pkgpath.is_empty() || self.pkgpath == MAIN_PKG {
            self.name.clone()
        } else {
            format!("@{}.{}", self.pkgpath, self.name)
        }
    }
    /// Is `name` a schema member function
    pub fn is_member_functions(&self, name: &str) -> bool {
        !self.is_instance && SCHEMA_MEMBER_FUNCTIONS.contains(&name)
    }

    pub fn set_type_of_attr(&mut self, attr: &str, ty: Rc<Type>) {
        match self.attrs.get_mut(attr) {
            Some(attr) => attr.ty = ty,
            None => {
                let schema_attr = SchemaAttr {
                    is_optional: true,
                    has_default: false,
                    ty,
                    pos: Position::dummy_pos(),
                    doc: None,
                };
                self.attrs.insert(attr.to_string(), schema_attr);
            }
        }
    }

    #[inline]
    pub fn get_type_of_attr(&self, attr: &str) -> Option<Rc<Type>> {
        self.get_obj_of_attr(attr).map(|attr| attr.ty.clone())
    }

    #[inline]
    pub fn get_obj_of_attr(&self, attr: &str) -> Option<&SchemaAttr> {
        match self.attrs.get(attr) {
            Some(attr) => Some(attr),
            None => self.base.as_ref().map_or(
                self.protocol
                    .as_ref()
                    .and_then(|protocol| protocol.get_obj_of_attr(attr)),
                |base| base.get_obj_of_attr(attr),
            ),
        }
    }

    pub fn key_ty(&self) -> Rc<Type> {
        Rc::new(Type::STR)
    }

    pub fn val_ty(&self) -> Rc<Type> {
        if let Some(index_signature) = &self.index_signature {
            index_signature.val_ty.clone()
        } else {
            Rc::new(Type::ANY)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaAttr {
    pub is_optional: bool,
    pub has_default: bool,
    pub ty: Rc<Type>,
    pub pos: Position,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaIndexSignature {
    pub key_name: Option<String>,
    pub key_ty: Rc<Type>,
    pub val_ty: Rc<Type>,
    pub any_other: bool,
}

impl SchemaIndexSignature {
    pub fn ty_str(&self) -> String {
        let key_name_str = match &self.key_name {
            Some(name) => format!("[{}: ", name),
            None => "[".to_string(),
        };
        let any_other_str = if self.any_other { "..." } else { "" };
        key_name_str
            + any_other_str
            + &format!("{}]: {}", self.key_ty.ty_str(), self.val_ty.ty_str())
    }
}

/// The module type.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleType {
    pub pkgpath: String,
    pub imported: Vec<String>,
    pub kind: ModuleKind,
}

/// The module kind.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleKind {
    User,
    System,
    Plugin,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decorator {
    pub target: DecoratorTarget,
    /// The decorator name.
    pub name: String,
    /// The schema or attribute name of decorator dimension
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecoratorTarget {
    Schema,
    Attribute,
}

/// The number multiplier type.
#[derive(Debug, Clone, PartialEq)]
pub struct NumberMultiplierType {
    pub value: f64,
    pub raw_value: i64,
    pub binary_suffix: String,
    pub is_literal: bool,
}

impl NumberMultiplierType {
    pub fn ty_str(&self) -> String {
        if self.is_literal {
            format!(
                "{}({}{})",
                NUMBER_MULTIPLIER_TYPE_STR, self.raw_value, self.binary_suffix
            )
        } else {
            NUMBER_MULTIPLIER_TYPE_STR.to_string()
        }
    }
}

/// The function type.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub doc: String,
    pub params: Vec<Parameter>,
    pub self_ty: Option<Rc<Type>>,
    pub return_ty: Rc<Type>,
    pub is_variadic: bool,
    pub kw_only_index: Option<usize>,
}

/// The function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Rc<Type>,
    pub has_default: bool,
}
