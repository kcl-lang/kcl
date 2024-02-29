use std::sync::Arc;

use generational_arena::Arena;
use indexmap::{IndexMap, IndexSet};

use kclvm_error::{diagnostic::Range, Position};
use serde::Serialize;

use super::package::ModuleInfo;
use crate::{
    resolver::scope::NodeKey,
    ty::{Type, TypeKind, TypeRef},
};

pub trait Symbol {
    type SymbolData;
    type SemanticInfo;
    fn get_sema_info(&self) -> &Self::SemanticInfo;
    fn is_global(&self) -> bool;
    fn get_range(&self) -> Range;
    fn get_owner(&self) -> Option<SymbolRef>;
    fn get_definition(&self) -> Option<SymbolRef>;
    fn get_name(&self) -> String;
    fn get_id(&self) -> Option<SymbolRef>;
    fn get_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef>;
    fn has_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> bool;

    fn get_all_attributes(
        &self,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef>;

    fn simple_dump(&self) -> String;

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String>;
}

pub type KCLSymbol = dyn Symbol<SymbolData = SymbolData, SemanticInfo = KCLSymbolSemanticInfo>;
#[derive(Debug, Clone, Default)]
pub struct KCLSymbolSemanticInfo {
    pub ty: Option<Arc<Type>>,
    pub doc: Option<String>,
}

pub(crate) const BUILTIN_STR_PACKAGE: &'static str = "@str";

#[derive(Default, Debug, Clone)]
pub struct SymbolData {
    pub(crate) values: Arena<ValueSymbol>,
    pub(crate) packages: Arena<PackageSymbol>,
    pub(crate) attributes: Arena<AttributeSymbol>,
    pub(crate) schemas: Arena<SchemaSymbol>,
    pub(crate) type_aliases: Arena<TypeAliasSymbol>,
    pub(crate) unresolved: Arena<UnresolvedSymbol>,
    pub(crate) rules: Arena<RuleSymbol>,
    pub(crate) exprs: Arena<ExpressionSymbol>,
    pub(crate) comments: Arena<CommentSymbol>,
    pub(crate) decorators: Arena<DecoratorSymbol>,

    pub(crate) symbols_info: SymbolDB,
}

#[derive(Default, Debug, Clone)]
pub struct SymbolDB {
    pub(crate) symbol_pos_set: IndexSet<Position>,
    pub(crate) global_builtin_symbols: IndexMap<String, SymbolRef>,
    pub(crate) fully_qualified_name_map: IndexMap<String, SymbolRef>,
    pub(crate) schema_builtin_symbols: IndexMap<SymbolRef, IndexMap<String, SymbolRef>>,
    pub(crate) node_symbol_map: IndexMap<NodeKey, SymbolRef>,
    pub(crate) symbol_node_map: IndexMap<SymbolRef, NodeKey>,
}

impl SymbolData {
    pub fn get_package_symbol(&self, id: SymbolRef) -> Option<&PackageSymbol> {
        if matches!(id.get_kind(), SymbolKind::Package) {
            self.packages.get(id.get_id())
        } else {
            None
        }
    }

    pub fn get_value_symbol(&self, id: SymbolRef) -> Option<&ValueSymbol> {
        if matches!(id.get_kind(), SymbolKind::Value) {
            self.values.get(id.get_id())
        } else {
            None
        }
    }

    pub fn get_attribute_symbol(&self, id: SymbolRef) -> Option<&AttributeSymbol> {
        if matches!(id.get_kind(), SymbolKind::Attribute) {
            self.attributes.get(id.get_id())
        } else {
            None
        }
    }

    pub fn get_type_alias_symbol(&self, id: SymbolRef) -> Option<&TypeAliasSymbol> {
        if matches!(id.get_kind(), SymbolKind::TypeAlias) {
            self.type_aliases.get(id.get_id())
        } else {
            None
        }
    }

    pub fn get_schema_symbol(&self, id: SymbolRef) -> Option<&SchemaSymbol> {
        if matches!(id.get_kind(), SymbolKind::Schema) {
            self.schemas.get(id.get_id())
        } else {
            None
        }
    }

    pub fn get_rule_symbol(&self, id: SymbolRef) -> Option<&RuleSymbol> {
        if matches!(id.get_kind(), SymbolKind::Rule) {
            self.rules.get(id.get_id())
        } else {
            None
        }
    }

    pub fn get_symbol(&self, id: SymbolRef) -> Option<&KCLSymbol> {
        match id.get_kind() {
            SymbolKind::Schema => self
                .schemas
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Attribute => self
                .attributes
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Value => self
                .values
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Package => self
                .packages
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::TypeAlias => self
                .type_aliases
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Unresolved => self
                .unresolved
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Rule => self
                .rules
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Expression => self
                .exprs
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Comment => self
                .comments
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
            SymbolKind::Decorator => self
                .decorators
                .get(id.get_id())
                .map(|symbol| symbol as &KCLSymbol),
        }
    }

    pub fn set_symbol_type(&mut self, id: SymbolRef, ty: TypeRef) {
        match id.get_kind() {
            SymbolKind::Schema => {
                self.schemas.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Attribute => {
                self.attributes.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Value => {
                self.values.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Package => {
                self.packages.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::TypeAlias => {
                self.type_aliases.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Unresolved => {
                self.unresolved.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Rule => {
                self.rules.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Expression => {
                self.exprs.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Comment => {
                self.comments.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
            SymbolKind::Decorator => {
                self.decorators.get_mut(id.get_id()).map(|symbol| {
                    symbol.sema_info.ty = Some(ty);
                    symbol
                });
            }
        }
    }

    pub fn get_type_symbol(
        &self,
        ty: &Type,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        match &ty.kind {
            //TODO: builtin ty symbol,now we just return none
            TypeKind::None => None,
            TypeKind::Any => None,
            TypeKind::Void => None,
            TypeKind::Bool => None,
            TypeKind::BoolLit(_) => None,
            TypeKind::Int => None,
            TypeKind::IntLit(_) => None,
            TypeKind::Float => None,
            TypeKind::FloatLit(_) => None,
            TypeKind::Str => self.get_symbol_by_fully_qualified_name(BUILTIN_STR_PACKAGE),
            TypeKind::StrLit(_) => self.get_symbol_by_fully_qualified_name(BUILTIN_STR_PACKAGE),
            TypeKind::List(_) => None,
            TypeKind::Dict(_) => None,
            TypeKind::NumberMultiplier(_) => None,
            TypeKind::Function(_) => None,
            TypeKind::Union(_) => None,

            TypeKind::Schema(schema_ty) => {
                let fully_qualified_ty_name = schema_ty.pkgpath.clone() + "." + &schema_ty.name;

                self.get_symbol_by_fully_qualified_name(&fully_qualified_ty_name)
            }
            TypeKind::Module(module_ty) => {
                self.get_symbol_by_fully_qualified_name(&module_ty.pkgpath)
            }
            TypeKind::Named(name) => {
                let splits: Vec<&str> = name.rsplitn(2, '.').collect();
                let len = splits.len();
                let pkgname = splits[len - 1];

                let pkgpath: &String = &module_info?.get_import_info(pkgname)?.fully_qualified_name;
                let fully_qualified_ty_name = if name.contains('.') {
                    name.replacen(&pkgname, pkgpath, 1)
                } else {
                    kclvm_ast::MAIN_PKG.to_string() + name
                };

                self.get_symbol_by_fully_qualified_name(&fully_qualified_ty_name)
            }
        }
    }

    pub fn get_type_all_attribute(
        &self,
        ty: &Type,
        name: &str,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        match &ty.kind {
            //TODO: builtin ty symbol,now we just return none
            TypeKind::None => vec![],
            TypeKind::Any => vec![],
            TypeKind::Void => vec![],
            TypeKind::Bool => vec![],
            TypeKind::BoolLit(_) => vec![],
            TypeKind::Int => vec![],
            TypeKind::IntLit(_) => vec![],
            TypeKind::Float => vec![],
            TypeKind::FloatLit(_) => vec![],
            TypeKind::Str => {
                let mut result = vec![];
                if let Some(symbol_ref) = self.get_type_symbol(ty, module_info) {
                    if let Some(symbol) = self.get_symbol(symbol_ref) {
                        result = symbol.get_all_attributes(self, module_info);
                    }
                }
                result
            }
            TypeKind::StrLit(_) => vec![],
            TypeKind::List(_) => vec![],
            TypeKind::Dict(_) => vec![],
            TypeKind::NumberMultiplier(_) => vec![],
            TypeKind::Function(_) => vec![],
            TypeKind::Union(tys) => {
                let mut result = vec![];
                for ty in tys.iter() {
                    result.append(&mut self.get_type_all_attribute(ty, name, module_info));
                }
                result
            }
            TypeKind::Schema(_) => {
                let mut result = vec![];
                if let Some(symbol_ref) = self.get_type_symbol(ty, module_info) {
                    if let Some(symbol) = self.get_symbol(symbol_ref) {
                        result = symbol.get_all_attributes(self, module_info);
                    }
                }
                result
            }
            TypeKind::Module(_) => {
                let mut result = vec![];
                if let Some(symbol_ref) = self.get_type_symbol(ty, module_info) {
                    if let Some(symbol) = self.get_symbol(symbol_ref) {
                        result = symbol.get_all_attributes(self, module_info);
                    }
                }
                result
            }
            TypeKind::Named(_) => {
                let mut result = vec![];
                if let Some(symbol_ref) = self.get_type_symbol(ty, module_info) {
                    if let Some(symbol) = self.get_symbol(symbol_ref) {
                        result = symbol.get_all_attributes(self, module_info);
                    }
                }
                result
            }
        }
    }

    pub fn get_type_attribute(
        &self,
        ty: &Type,
        name: &str,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        match &ty.kind {
            TypeKind::None => None,
            TypeKind::Any => None,
            TypeKind::Void => None,
            TypeKind::Bool => None,
            TypeKind::BoolLit(_) => None,
            TypeKind::Int => None,
            TypeKind::IntLit(_) => None,
            TypeKind::Float => None,
            TypeKind::FloatLit(_) => None,
            TypeKind::Str => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
            TypeKind::StrLit(_) => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
            TypeKind::List(_) => None,
            TypeKind::Dict(_) => None,
            TypeKind::NumberMultiplier(_) => None,
            TypeKind::Function(_) => None,
            TypeKind::Union(tys) => {
                for ty in tys.iter() {
                    if let Some(symbol_ref) = self.get_type_attribute(ty, name, module_info) {
                        return Some(symbol_ref);
                    }
                }
                None
            }
            TypeKind::Schema(_) => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
            TypeKind::Module(_) => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
            TypeKind::Named(_) => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
        }
    }

    pub fn get_symbol_by_fully_qualified_name(&self, fqn: &str) -> Option<SymbolRef> {
        self.symbols_info.fully_qualified_name_map.get(fqn).cloned()
    }

    pub fn get_fully_qualified_name(&self, symbol_ref: SymbolRef) -> Option<String> {
        match symbol_ref.get_kind() {
            SymbolKind::Unresolved => None,
            _ => {
                let symbol = self.get_symbol(symbol_ref)?;
                let owner = symbol.get_owner();
                if let Some(owner) = owner {
                    Some(self.get_fully_qualified_name(owner)? + "." + &symbol.get_name())
                } else {
                    Some(symbol.get_name())
                }
            }
        }
    }

    pub fn build_fully_qualified_name_map(&mut self) {
        for (id, _) in self.packages.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Package,
            };
            self.symbols_info.fully_qualified_name_map.insert(
                self.get_fully_qualified_name(symbol_ref).unwrap(),
                symbol_ref,
            );
        }

        for (id, _) in self.schemas.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Schema,
            };
            self.symbols_info.fully_qualified_name_map.insert(
                self.get_fully_qualified_name(symbol_ref).unwrap(),
                symbol_ref,
            );
        }

        for (id, _) in self.type_aliases.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::TypeAlias,
            };
            self.symbols_info.fully_qualified_name_map.insert(
                self.get_fully_qualified_name(symbol_ref).unwrap(),
                symbol_ref,
            );
        }

        for (id, _) in self.attributes.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Attribute,
            };
            self.symbols_info.fully_qualified_name_map.insert(
                self.get_fully_qualified_name(symbol_ref).unwrap(),
                symbol_ref,
            );
        }

        for (id, _) in self.rules.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Rule,
            };
            self.symbols_info.fully_qualified_name_map.insert(
                self.get_fully_qualified_name(symbol_ref).unwrap(),
                symbol_ref,
            );
        }

        for (id, _) in self.values.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Value,
            };
            self.symbols_info.fully_qualified_name_map.insert(
                self.get_fully_qualified_name(symbol_ref).unwrap(),
                symbol_ref,
            );
        }
    }

    pub fn alloc_package_symbol(&mut self, pkg: PackageSymbol) -> SymbolRef {
        let symbol_id = self.packages.insert(pkg);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Package,
        };
        self.packages.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_schema_symbol(&mut self, schema: SchemaSymbol, node_key: NodeKey) -> SymbolRef {
        self.symbols_info.symbol_pos_set.insert(schema.end.clone());
        let symbol_id = self.schemas.insert(schema);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Schema,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.schemas.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_unresolved_symbol(
        &mut self,
        unresolved: UnresolvedSymbol,
        node_key: NodeKey,
    ) -> SymbolRef {
        self.symbols_info
            .symbol_pos_set
            .insert(unresolved.end.clone());
        let symbol_id = self.unresolved.insert(unresolved);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Unresolved,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.unresolved.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_type_alias_symbol(
        &mut self,
        alias: TypeAliasSymbol,
        node_key: NodeKey,
    ) -> SymbolRef {
        self.symbols_info.symbol_pos_set.insert(alias.end.clone());
        let symbol_id = self.type_aliases.insert(alias);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::TypeAlias,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.type_aliases.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_rule_symbol(&mut self, rule: RuleSymbol, node_key: NodeKey) -> SymbolRef {
        self.symbols_info.symbol_pos_set.insert(rule.end.clone());
        let symbol_id = self.rules.insert(rule);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Rule,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.rules.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_attribute_symbol(
        &mut self,
        attribute: AttributeSymbol,
        node_key: NodeKey,
    ) -> SymbolRef {
        self.symbols_info
            .symbol_pos_set
            .insert(attribute.end.clone());
        let symbol_id = self.attributes.insert(attribute);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Attribute,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.attributes.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_value_symbol(&mut self, value: ValueSymbol, node_key: NodeKey) -> SymbolRef {
        self.symbols_info.symbol_pos_set.insert(value.end.clone());
        let symbol_id = self.values.insert(value);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Value,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.values.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_expression_symbol(
        &mut self,
        expr: ExpressionSymbol,
        node_key: NodeKey,
    ) -> Option<SymbolRef> {
        if self.symbols_info.symbol_pos_set.contains(&expr.end) {
            return None;
        }
        self.symbols_info.symbol_pos_set.insert(expr.end.clone());
        let symbol_id = self.exprs.insert(expr);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Expression,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.exprs.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        Some(symbol_ref)
    }

    pub fn alloc_comment_symbol(
        &mut self,
        comment: CommentSymbol,
        node_key: NodeKey,
    ) -> Option<SymbolRef> {
        let symbol_id = self.comments.insert(comment);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Comment,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.comments.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        Some(symbol_ref)
    }

    pub fn alloc_decorator_symbol(
        &mut self,
        decorator: DecoratorSymbol,
        node_key: NodeKey,
    ) -> Option<SymbolRef> {
        let symbol_id = self.decorators.insert(decorator);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Decorator,
        };
        self.symbols_info
            .node_symbol_map
            .insert(node_key.clone(), symbol_ref);
        self.symbols_info
            .symbol_node_map
            .insert(symbol_ref, node_key);
        self.decorators.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        Some(symbol_ref)
    }

    #[inline]
    pub fn get_node_symbol_map(&self) -> &IndexMap<NodeKey, SymbolRef> {
        &self.symbols_info.node_symbol_map
    }

    #[inline]
    pub fn get_symbol_node_map(&self) -> &IndexMap<SymbolRef, NodeKey> {
        &self.symbols_info.symbol_node_map
    }

    #[inline]
    pub fn get_fully_qualified_name_map(&self) -> &IndexMap<String, SymbolRef> {
        &self.symbols_info.fully_qualified_name_map
    }

    #[inline]
    pub fn get_builtin_symbols(&self) -> &IndexMap<String, SymbolRef> {
        &self.symbols_info.global_builtin_symbols
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum SymbolKind {
    Schema,
    Attribute,
    Value,
    Package,
    TypeAlias,
    Unresolved,
    Rule,
    Expression,
    Comment,
    Decorator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolRef {
    pub(crate) id: generational_arena::Index,
    pub(crate) kind: SymbolKind,
}

impl Serialize for SymbolRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (index, generation) = self.id.into_raw_parts();
        let data = SerializableSymbolRef {
            i: index as u64,
            g: generation,
            kind: self.kind.clone(),
        };
        data.serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize)]

struct SerializableSymbolRef {
    i: u64,
    g: u64,
    kind: SymbolKind,
}

impl SymbolRef {
    #[inline]
    pub fn get_kind(&self) -> SymbolKind {
        self.kind
    }
    #[inline]
    pub fn get_id(&self) -> generational_arena::Index {
        self.id
    }
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct SchemaSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
    pub(crate) sema_info: KCLSymbolSemanticInfo,

    pub(crate) parent_schema: Option<SymbolRef>,
    pub(crate) for_host: Option<SymbolRef>,
    pub(crate) mixins: Vec<SymbolRef>,
    pub(crate) attributes: IndexMap<String, SymbolRef>,
}

impl Symbol for SchemaSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        true
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        Some(self.owner)
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        match self.attributes.get(name) {
            Some(attribute) => Some(*attribute),
            None => {
                if let Some(parent_schema) = self.parent_schema {
                    if let Some(attribute) =
                        data.get_symbol(parent_schema)?
                            .get_attribute(name, data, module_info)
                    {
                        return Some(attribute);
                    }
                }

                if let Some(for_host) = self.for_host {
                    if let Some(attribute) =
                        data.get_symbol(for_host)?
                            .get_attribute(name, data, module_info)
                    {
                        return Some(attribute);
                    }
                }

                for mixin in self.mixins.iter() {
                    if let Some(attribute) =
                        data.get_symbol(*mixin)?
                            .get_attribute(name, data, module_info)
                    {
                        return Some(attribute);
                    }
                }

                None
            }
        }
    }

    fn get_all_attributes(
        &self,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        let mut result = vec![];
        for attribute in self.attributes.values() {
            result.push(*attribute);
        }
        if let Some(parent_schema) = self.parent_schema {
            if let Some(parent) = data.get_symbol(parent_schema) {
                result.append(&mut parent.get_all_attributes(data, module_info))
            }
        }

        if let Some(for_host) = self.for_host {
            if let Some(for_host) = data.get_symbol(for_host) {
                result.append(&mut for_host.get_all_attributes(data, module_info))
            }
        }
        for mixin in self.mixins.iter() {
            if let Some(mixin) = data.get_symbol(*mixin) {
                result.append(&mut mixin.get_all_attributes(data, module_info))
            }
        }
        result
    }

    fn has_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> bool {
        self.get_attribute(name, data, module_info).is_some()
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"SchemaSymbol\",\n");
        output.push_str(&format!("\"name\":\"{}\",\n", self.name));
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        let owner_symbol = data.get_symbol(self.owner)?;
        output.push_str(&format!("\"owner\": {},\n", owner_symbol.simple_dump()));
        if let Some(parent_schema) = self.parent_schema.as_ref() {
            let parent_schema_symbol = data.get_symbol(*parent_schema)?;
            output.push_str(&format!(
                "\"parent_schema\": {},\n",
                parent_schema_symbol.simple_dump()
            ));
        }
        if let Some(parent_schema) = self.for_host.as_ref() {
            let host_symbol = data.get_symbol(*parent_schema)?;
            output.push_str(&format!("\"for_host\": {},\n", host_symbol.simple_dump()));
        }
        output.push_str("\"mixins\": [\n");
        for (index, mixin) in self.mixins.iter().enumerate() {
            let mixin_symbol = data.get_symbol(*mixin)?;
            output.push_str(&format!("{}", mixin_symbol.simple_dump()));
            if index + 1 < self.mixins.len() {
                output.push_str(",\n")
            }
        }
        output.push_str("\n],\n");
        output.push_str("\"attributes\": {\n");
        for (index, (key, attribute)) in self.attributes.iter().enumerate() {
            let attribute_symbol = data.get_symbol(*attribute)?;
            output.push_str(&format!("\"{}\": {}", key, attribute_symbol.simple_dump()));
            if index + 1 < self.attributes.len() {
                output.push_str(",\n")
            }
        }
        output.push_str("\n}\n}\n}");
        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl SchemaSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            owner,
            parent_schema: None,
            for_host: None,
            sema_info: KCLSymbolSemanticInfo::default(),
            mixins: Vec::default(),
            attributes: IndexMap::default(),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ValueSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: Option<SymbolRef>,
    pub(crate) sema_info: KCLSymbolSemanticInfo,

    pub(crate) is_global: bool,
}

impl Symbol for ValueSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        self.is_global
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        self.owner.clone()
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        data.get_type_attribute(self.sema_info.ty.as_ref()?, name, module_info)
    }

    fn get_all_attributes(
        &self,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        let mut result = vec![];
        if let Some(ty) = self.sema_info.ty.as_ref() {
            if let Some(symbol_ref) = data.get_type_symbol(ty, module_info) {
                if let Some(symbol) = data.get_symbol(symbol_ref) {
                    result.append(&mut symbol.get_all_attributes(data, module_info))
                }
            }
        }

        result
    }

    fn has_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> bool {
        self.get_attribute(name, data, module_info).is_some()
    }
    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"ValueSymbol\",\n");
        output.push_str(&format!("\"name\":\"{}\",\n", self.name));
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        if let Some(owner) = self.owner.as_ref() {
            let owner_symbol = data.get_symbol(*owner)?;
            output.push_str(&format!("\"owner\": {}\n", owner_symbol.simple_dump()));
        }
        output.push_str("\n}\n}");
        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl ValueSymbol {
    pub fn new(
        name: String,
        start: Position,
        end: Position,
        owner: Option<SymbolRef>,
        is_global: bool,
    ) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            owner,
            sema_info: KCLSymbolSemanticInfo::default(),
            is_global,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct AttributeSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
    pub(crate) sema_info: KCLSymbolSemanticInfo,
}

impl Symbol for AttributeSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        true
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        Some(self.owner)
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        let ty = self.sema_info.ty.as_ref()?;
        data.get_type_attribute(ty, name, module_info)
    }

    fn get_all_attributes(
        &self,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        let mut result = vec![];
        if module_info.is_none() {
            return result;
        }
        if let Some(ty) = self.sema_info.ty.as_ref() {
            if let Some(symbol_ref) = data.get_type_symbol(ty, module_info) {
                if let Some(symbol) = data.get_symbol(symbol_ref) {
                    result.append(&mut symbol.get_all_attributes(data, module_info))
                }
            }
        }

        result
    }

    fn has_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> bool {
        self.get_attribute(name, data, module_info).is_some()
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"AttributeSymbol\",\n");
        output.push_str(&format!("\"name\":\"{}\",\n", self.name));
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        let owner_symbol = data.get_symbol(self.owner)?;
        output.push_str(&format!("\"owner\": {}\n", owner_symbol.simple_dump()));
        output.push_str("\n}\n}");
        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl AttributeSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            sema_info: KCLSymbolSemanticInfo::default(),
            owner,
        }
    }
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct PackageSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) members: IndexMap<String, SymbolRef>,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) sema_info: KCLSymbolSemanticInfo,
}

impl Symbol for PackageSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        true
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        None
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        self.members.get(name).cloned()
    }

    fn get_all_attributes(
        &self,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        let mut result = vec![];
        for member in self.members.values() {
            result.push(*member);
        }
        result
    }

    fn has_attribute(
        &self,
        name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> bool {
        self.members.contains_key(name)
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"PackageSymbol\",\n");
        output.push_str(&format!("\"name\":\"{}\",\n", self.name));
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        output.push_str("\"members\": {\n");
        for (index, (key, member)) in self.members.iter().enumerate() {
            let member_symbol = data.get_symbol(*member)?;
            output.push_str(&format!("\"{}\": {}", key, member_symbol.simple_dump()));
            if index + 1 < self.members.len() {
                output.push_str(",\n");
            }
        }
        output.push_str("\n}\n}\n}");
        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl PackageSymbol {
    pub fn new(name: String, start: Position, end: Position) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            sema_info: KCLSymbolSemanticInfo::default(),
            members: IndexMap::default(),
        }
    }
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct TypeAliasSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
    pub(crate) sema_info: KCLSymbolSemanticInfo,
}

impl Symbol for TypeAliasSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        true
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        Some(self.owner)
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        let ty = self.sema_info.ty.as_ref()?;
        data.get_type_attribute(ty, name, module_info)
    }

    fn get_all_attributes(
        &self,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        let mut result = vec![];
        if let Some(ty) = self.sema_info.ty.as_ref() {
            if let Some(symbol_ref) = data.get_type_symbol(ty, module_info) {
                if let Some(symbol) = data.get_symbol(symbol_ref) {
                    result.append(&mut symbol.get_all_attributes(data, module_info))
                }
            }
        }
        result
    }

    fn has_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> bool {
        self.get_attribute(name, data, module_info).is_some()
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"TypeAliasSymbol\",\n");
        output.push_str(&format!("\"name\":\"{}\",\n", self.name));
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        let owner_symbol = data.get_symbol(self.owner)?;
        output.push_str(&format!(
            "\"owner\": {}\n}}\n}}",
            owner_symbol.simple_dump()
        ));
        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl TypeAliasSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            sema_info: KCLSymbolSemanticInfo::default(),
            owner,
        }
    }
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct RuleSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
    pub(crate) sema_info: KCLSymbolSemanticInfo,

    pub(crate) parent_rules: Vec<SymbolRef>,
    pub(crate) for_host: Option<SymbolRef>,
}

impl Symbol for RuleSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        true
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        Some(self.owner)
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        _name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        None
    }

    fn get_all_attributes(
        &self,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        vec![]
    }

    fn has_attribute(
        &self,
        _name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> bool {
        false
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"RuleSymbol\",\n");
        output.push_str(&format!("\"name\":\"{}\",\n", self.name));
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        let owner_symbol = data.get_symbol(self.owner)?;
        output.push_str(&format!("\"owner\": {},\n", owner_symbol.simple_dump()));

        if let Some(parent_schema) = self.for_host.as_ref() {
            let host_symbol = data.get_symbol(*parent_schema)?;
            output.push_str(&format!("\"for_host\": {},\n", host_symbol.simple_dump()));
        }
        output.push_str("\"parent_rules\": [\n");
        for (index, parent_rule) in self.parent_rules.iter().enumerate() {
            let parent_symbol = data.get_symbol(*parent_rule)?;
            output.push_str(&format!("{}", parent_symbol.simple_dump()));
            if index + 1 < self.parent_rules.len() {
                output.push_str(",\n")
            }
        }
        output.push_str("\n]\n}\n}");

        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl RuleSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            owner,
            sema_info: KCLSymbolSemanticInfo::default(),
            parent_rules: vec![],
            for_host: None,
        }
    }
}
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct UnresolvedSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) def: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: Option<SymbolRef>,
    pub(crate) sema_info: KCLSymbolSemanticInfo,
}

impl Symbol for UnresolvedSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        false
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        self.owner.clone()
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.def.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        data.get_symbol(self.def?)?
            .get_attribute(name, data, module_info)
    }

    fn get_all_attributes(
        &self,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        if let Some(def) = self.def {
            if let Some(def_symbol) = data.get_symbol(def) {
                return def_symbol.get_all_attributes(data, module_info);
            }
        }
        vec![]
    }

    fn has_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> bool {
        self.get_attribute(name, data, module_info).is_some()
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"UnresolvedSymbol\",\n");
        output.push_str(&format!("\"name\":\"{}\",\n", self.name));
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }
    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        if let Some(def) = self.def.as_ref() {
            let def_symbol = data.get_symbol(*def)?;
            output.push_str(&format!("\"def\": {}\n", def_symbol.simple_dump()));
        }
        output.push_str("\n}\n}");
        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl UnresolvedSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: Option<SymbolRef>) -> Self {
        Self {
            id: None,
            def: None,
            name,
            start,
            end,
            sema_info: KCLSymbolSemanticInfo::default(),
            owner,
        }
    }

    pub fn get_fully_qualified_name(&self, module_info: &ModuleInfo) -> String {
        let names: Vec<_> = self.name.split('.').collect();
        let pkg_path = if names.len() == 1 {
            kclvm_ast::MAIN_PKG.to_string()
        } else {
            let pkg_alias = names.first().unwrap();
            let import_info = module_info.get_import_info(*pkg_alias);
            match import_info {
                Some(info) => info.fully_qualified_name.clone(),
                None => kclvm_ast::MAIN_PKG.to_string(),
            }
        };

        pkg_path + "." + names.last().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: Option<SymbolRef>,
    pub(crate) name: String,

    pub(crate) sema_info: KCLSymbolSemanticInfo,
}

impl Symbol for ExpressionSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn is_global(&self) -> bool {
        false
    }
    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        self.owner.clone()
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        data.get_type_attribute(self.sema_info.ty.as_ref()?, name, module_info)
    }

    fn get_all_attributes(
        &self,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        let mut result = vec![];
        if let Some(ty) = self.sema_info.ty.as_ref() {
            if let Some(symbol_ref) = data.get_type_symbol(ty, module_info) {
                if let Some(symbol) = data.get_symbol(symbol_ref) {
                    result.append(&mut symbol.get_all_attributes(data, module_info))
                }
            }
        }

        result
    }

    fn has_attribute(
        &self,
        name: &str,
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> bool {
        self.get_attribute(name, data, module_info).is_some()
    }
    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"ExpressionSymbol\",\n");
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, data: &Self::SymbolData) -> Option<String> {
        let mut output = format!("{{\n\"simple_info\": {},\n", self.simple_dump());
        output.push_str("\"additional_info\": {\n");
        if let Some(owner) = self.owner.as_ref() {
            let owner_symbol = data.get_symbol(*owner)?;
            output.push_str(&format!("\"owner\": {}\n", owner_symbol.simple_dump()));
        }
        output.push_str("\n}\n}");
        Some(output)
    }

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }
}

impl ExpressionSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: Option<SymbolRef>) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            sema_info: KCLSymbolSemanticInfo::default(),
            owner,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommentSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) content: String,
    pub(crate) sema_info: KCLSymbolSemanticInfo,
}

impl Symbol for CommentSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }

    fn is_global(&self) -> bool {
        true
    }

    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        None
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id
    }

    fn get_name(&self) -> String {
        self.name()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        _name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        None
    }

    fn has_attribute(
        &self,
        _name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> bool {
        false
    }

    fn get_all_attributes(
        &self,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        vec![]
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"CommentSymbol\",\n");
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str(&format!("content :{}", self.name()));
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, _data: &Self::SymbolData) -> Option<String> {
        Some(self.simple_dump())
    }
}

impl CommentSymbol {
    pub fn new(start: Position, end: Position, content: String) -> Self {
        Self {
            id: None,
            start,
            end,
            content,
            sema_info: KCLSymbolSemanticInfo::default(),
        }
    }

    pub fn name(&self) -> String {
        format!("# {}", self.content)
    }
}

#[derive(Debug, Clone)]
pub struct DecoratorSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) name: String,
    pub(crate) sema_info: KCLSymbolSemanticInfo,
}

impl Symbol for DecoratorSymbol {
    type SymbolData = SymbolData;
    type SemanticInfo = KCLSymbolSemanticInfo;

    fn get_sema_info(&self) -> &Self::SemanticInfo {
        &self.sema_info
    }

    fn is_global(&self) -> bool {
        true
    }

    fn get_range(&self) -> Range {
        (self.start.clone(), self.end.clone())
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        None
    }

    fn get_definition(&self) -> Option<SymbolRef> {
        self.id
    }

    fn get_name(&self) -> String {
        self.name()
    }

    fn get_id(&self) -> Option<SymbolRef> {
        self.id.clone()
    }

    fn get_attribute(
        &self,
        _name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        None
    }

    fn has_attribute(
        &self,
        _name: &str,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> bool {
        false
    }

    fn get_all_attributes(
        &self,
        _data: &Self::SymbolData,
        _module_info: Option<&ModuleInfo>,
    ) -> Vec<SymbolRef> {
        vec![]
    }

    fn simple_dump(&self) -> String {
        let mut output = "{\n".to_string();
        output.push_str("\"kind\": \"CommentSymbol\",\n");
        output.push_str(&format!(
            "\"range\": \"{}:{}",
            self.start.filename, self.start.line
        ));
        if let Some(start_col) = self.start.column {
            output.push_str(&format!(":{}", start_col));
        }

        output.push_str(&format!(" to {}", self.end.line));
        if let Some(end_col) = self.end.column {
            output.push_str(&format!(":{}", end_col));
        }
        output.push_str(&format!("name :{}", self.name()));
        output.push_str("\"\n}");
        output
    }

    fn full_dump(&self, _data: &Self::SymbolData) -> Option<String> {
        Some(self.simple_dump())
    }
}

impl DecoratorSymbol {
    pub fn new(start: Position, end: Position, name: String) -> Self {
        Self {
            id: None,
            start,
            end,
            name,
            sema_info: KCLSymbolSemanticInfo::default(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}
