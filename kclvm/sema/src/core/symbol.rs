use std::sync::Arc;

use generational_arena::Arena;
use indexmap::IndexMap;

use kclvm_error::{diagnostic::Range, Position};

use super::package::ModuleInfo;
use crate::ty::Type;
use kclvm_ast::ast::AstIndex;

pub trait Symbol {
    type SymbolData;
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
    fn has_attribue(
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

#[derive(Default, Debug, Clone)]
pub struct KCLSymbolData {
    pub(crate) values: Arena<ValueSymbol>,
    pub(crate) packages: Arena<PackageSymbol>,
    pub(crate) attributes: Arena<AttributeSymbol>,
    pub(crate) schemas: Arena<SchemaSymbol>,
    pub(crate) type_aliases: Arena<TypeAliasSymbol>,
    pub(crate) unresolved: Arena<UnresolvedSymbol>,
    pub(crate) rules: Arena<RuleSymbol>,

    pub(crate) symbols_info: SymbolDB,
}

#[derive(Default, Debug, Clone)]
pub struct SymbolDB {
    pub(crate) fully_qualified_name_map: IndexMap<String, SymbolRef>,
    pub(crate) ast_id_map: IndexMap<AstIndex, SymbolRef>,
    pub(crate) symbol_ty_map: IndexMap<SymbolRef, Arc<Type>>,
}

impl KCLSymbolData {
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

    pub fn get_attribue_symbol(&self, id: SymbolRef) -> Option<&AttributeSymbol> {
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

    pub fn get_symbol(&self, id: SymbolRef) -> Option<&dyn Symbol<SymbolData = Self>> {
        match id.get_kind() {
            SymbolKind::Schema => self
                .schemas
                .get(id.get_id())
                .map(|symbol| symbol as &dyn Symbol<SymbolData = Self>),
            SymbolKind::Attribute => self
                .attributes
                .get(id.get_id())
                .map(|symbol| symbol as &dyn Symbol<SymbolData = Self>),
            SymbolKind::Value => self
                .values
                .get(id.get_id())
                .map(|symbol| symbol as &dyn Symbol<SymbolData = Self>),
            SymbolKind::Package => self
                .packages
                .get(id.get_id())
                .map(|symbol| symbol as &dyn Symbol<SymbolData = Self>),
            SymbolKind::TypeAlias => self
                .type_aliases
                .get(id.get_id())
                .map(|symbol| symbol as &dyn Symbol<SymbolData = Self>),
            SymbolKind::Unresolved => self
                .unresolved
                .get(id.get_id())
                .map(|symbol| symbol as &dyn Symbol<SymbolData = Self>),
            SymbolKind::Rule => self
                .rules
                .get(id.get_id())
                .map(|symbol| symbol as &dyn Symbol<SymbolData = Self>),
        }
    }

    pub fn get_type_symbol(
        &self,
        ty: &Type,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        match &ty.kind {
            //TODO: builtin ty symbol,now we just return none
            crate::ty::TypeKind::None => None,
            crate::ty::TypeKind::Any => None,
            crate::ty::TypeKind::Void => None,
            crate::ty::TypeKind::Bool => None,
            crate::ty::TypeKind::BoolLit(_) => None,
            crate::ty::TypeKind::Int => None,
            crate::ty::TypeKind::IntLit(_) => None,
            crate::ty::TypeKind::Float => None,
            crate::ty::TypeKind::FloatLit(_) => None,
            crate::ty::TypeKind::Str => None,
            crate::ty::TypeKind::StrLit(_) => None,
            crate::ty::TypeKind::List(_) => None,
            crate::ty::TypeKind::Dict(_) => None,
            crate::ty::TypeKind::NumberMultiplier(_) => None,
            crate::ty::TypeKind::Function(_) => None,
            crate::ty::TypeKind::Union(_) => None,

            crate::ty::TypeKind::Schema(schema_ty) => {
                let fully_qualified_ty_name = schema_ty.pkgpath.clone() + "." + &schema_ty.name;

                self.get_symbol_by_fully_qualified_name(&fully_qualified_ty_name)
            }
            crate::ty::TypeKind::Module(module_ty) => {
                self.get_symbol_by_fully_qualified_name(&module_ty.pkgpath)
            }
            crate::ty::TypeKind::Named(name) => {
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

    pub fn get_type_attribute(
        &self,
        ty: &Type,
        name: &str,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        match &ty.kind {
            //TODO: builtin ty symbol,now we just return none
            crate::ty::TypeKind::None => None,
            crate::ty::TypeKind::Any => None,
            crate::ty::TypeKind::Void => None,
            crate::ty::TypeKind::Bool => None,
            crate::ty::TypeKind::BoolLit(_) => None,
            crate::ty::TypeKind::Int => None,
            crate::ty::TypeKind::IntLit(_) => None,
            crate::ty::TypeKind::Float => None,
            crate::ty::TypeKind::FloatLit(_) => None,
            crate::ty::TypeKind::Str => None,
            crate::ty::TypeKind::StrLit(_) => None,
            crate::ty::TypeKind::List(_) => None,
            crate::ty::TypeKind::Dict(_) => None,
            crate::ty::TypeKind::NumberMultiplier(_) => None,
            crate::ty::TypeKind::Function(_) => None,
            crate::ty::TypeKind::Union(tys) => {
                for ty in tys.iter() {
                    if let Some(symbol_ref) = self.get_type_attribute(ty, name, module_info) {
                        return Some(symbol_ref);
                    }
                }
                None
            }
            crate::ty::TypeKind::Schema(_) => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
            crate::ty::TypeKind::Module(_) => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
            crate::ty::TypeKind::Named(_) => self
                .get_symbol(self.get_type_symbol(ty, module_info)?)?
                .get_attribute(name, self, module_info),
        }
    }

    pub fn add_symbol_info(&mut self, symbol: SymbolRef, ty: Arc<Type>, ast_id: AstIndex) {
        self.symbols_info.ast_id_map.insert(ast_id, symbol);
        self.symbols_info.symbol_ty_map.insert(symbol, ty);
    }

    pub fn get_symbol_by_ast_index(&self, id: &AstIndex) -> Option<SymbolRef> {
        self.symbols_info.ast_id_map.get(id).cloned()
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

    pub fn alloc_schema_symbol(&mut self, schema: SchemaSymbol, ast_id: &AstIndex) -> SymbolRef {
        let symbol_id = self.schemas.insert(schema);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Schema,
        };
        self.symbols_info
            .ast_id_map
            .insert(ast_id.clone(), symbol_ref);
        self.schemas.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_unresolved_symbol(
        &mut self,
        unresolved: UnresolvedSymbol,
        ast_id: &AstIndex,
    ) -> SymbolRef {
        let symbol_id = self.unresolved.insert(unresolved);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Unresolved,
        };
        self.symbols_info
            .ast_id_map
            .insert(ast_id.clone(), symbol_ref);
        self.unresolved.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_type_alias_symbol(
        &mut self,
        alias: TypeAliasSymbol,
        ast_id: &AstIndex,
    ) -> SymbolRef {
        let symbol_id = self.type_aliases.insert(alias);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::TypeAlias,
        };
        self.symbols_info
            .ast_id_map
            .insert(ast_id.clone(), symbol_ref);
        self.type_aliases.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_rule_symbol(&mut self, rule: RuleSymbol, ast_id: &AstIndex) -> SymbolRef {
        let symbol_id = self.rules.insert(rule);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Rule,
        };
        self.symbols_info
            .ast_id_map
            .insert(ast_id.clone(), symbol_ref);
        self.rules.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_attribute_symbol(
        &mut self,
        attribute: AttributeSymbol,
        ast_id: &AstIndex,
    ) -> SymbolRef {
        let symbol_id = self.attributes.insert(attribute);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Attribute,
        };
        self.symbols_info
            .ast_id_map
            .insert(ast_id.clone(), symbol_ref);
        self.attributes.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }

    pub fn alloc_value_symbol(&mut self, value: ValueSymbol, ast_id: &AstIndex) -> SymbolRef {
        let symbol_id = self.values.insert(value);
        let symbol_ref = SymbolRef {
            id: symbol_id,
            kind: SymbolKind::Value,
        };
        self.symbols_info
            .ast_id_map
            .insert(ast_id.clone(), symbol_ref);
        self.values.get_mut(symbol_id).unwrap().id = Some(symbol_ref);
        symbol_ref
    }
}
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolKind {
    Schema,
    Attribute,
    Value,
    Package,
    TypeAlias,
    Unresolved,
    Rule,
}
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolRef {
    pub(crate) id: generational_arena::Index,
    pub(crate) kind: SymbolKind,
}

impl SymbolRef {
    pub fn get_kind(&self) -> SymbolKind {
        self.kind
    }

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

    pub(crate) parent_schema: Option<SymbolRef>,
    pub(crate) for_host: Option<SymbolRef>,
    pub(crate) mixins: Vec<SymbolRef>,
    pub(crate) attributes: IndexMap<String, SymbolRef>,
}

impl Symbol for SchemaSymbol {
    type SymbolData = KCLSymbolData;

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

    fn has_attribue(
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
    pub(crate) is_global: bool,
}

impl Symbol for ValueSymbol {
    type SymbolData = KCLSymbolData;
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
        let ty = data.symbols_info.symbol_ty_map.get(&self.id?)?;
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
        if let Some(ty) = data.symbols_info.symbol_ty_map.get(&self.id.unwrap()) {
            if let Some(symbol_ref) = data.get_type_symbol(ty, module_info) {
                if let Some(symbol) = data.get_symbol(symbol_ref) {
                    result.append(&mut symbol.get_all_attributes(data, module_info))
                }
            }
        }

        result
    }

    fn has_attribue(
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
}

impl Symbol for AttributeSymbol {
    type SymbolData = KCLSymbolData;
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
        let ty = data.symbols_info.symbol_ty_map.get(&self.id?)?;
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
        if let Some(ty) = data.symbols_info.symbol_ty_map.get(&self.id.unwrap()) {
            if let Some(symbol_ref) = data.get_type_symbol(ty, module_info) {
                if let Some(symbol) = data.get_symbol(symbol_ref) {
                    result.append(&mut symbol.get_all_attributes(data, module_info))
                }
            }
        }

        result
    }

    fn has_attribue(
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
}

impl AttributeSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
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
}

impl Symbol for PackageSymbol {
    type SymbolData = KCLSymbolData;
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

    fn has_attribue(
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
}

impl PackageSymbol {
    pub fn new(name: String, start: Position, end: Position) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
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
}

impl Symbol for TypeAliasSymbol {
    type SymbolData = KCLSymbolData;
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
        data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        let ty = data.symbols_info.symbol_ty_map.get(&self.id?)?;
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
        if let Some(ty) = data.symbols_info.symbol_ty_map.get(&self.id.unwrap()) {
            if let Some(symbol_ref) = data.get_type_symbol(ty, module_info) {
                if let Some(symbol) = data.get_symbol(symbol_ref) {
                    result.append(&mut symbol.get_all_attributes(data, module_info))
                }
            }
        }

        result
    }

    fn has_attribue(
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
}

impl TypeAliasSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
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

    pub(crate) parent_rules: Vec<SymbolRef>,
    pub(crate) for_host: Option<SymbolRef>,
}

impl Symbol for RuleSymbol {
    type SymbolData = KCLSymbolData;
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

    fn has_attribue(
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
}

impl RuleSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            name,
            start,
            end,
            owner,
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
}

impl Symbol for UnresolvedSymbol {
    type SymbolData = KCLSymbolData;
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

    fn has_attribue(
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
}

impl UnresolvedSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: Option<SymbolRef>) -> Self {
        Self {
            id: None,
            def: None,
            name,
            start,
            end,
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
