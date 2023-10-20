use std::rc::Rc;

use generational_arena::Arena;
use indexmap::IndexMap;

use kclvm_error::Position;

use super::package::{PackageDB, PackageInfo};
use crate::ty::Type;
use kclvm_ast::ast::AstIndex;

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct SymbolDB {
    pub(crate) fully_qualified_name_map: IndexMap<String, SymbolRef>,
    pub(crate) ast_id_map: IndexMap<AstIndex, SymbolRef>,
    pub(crate) symbol_ty_map: IndexMap<SymbolRef, Rc<Type>>,
}

impl KCLSymbolData {
    pub fn get_symbol_by_ast_index(&self, id: &AstIndex) -> Option<SymbolRef> {
        self.symbols_info.ast_id_map.get(id).cloned()
    }

    pub fn get_symbol_by_fully_qualified_name(&self, fqn: &str) -> Option<SymbolRef> {
        self.symbols_info.fully_qualified_name_map.get(fqn).cloned()
    }

    pub fn get_fully_qualified_name(&self, symbol_ref: SymbolRef) -> Option<String> {
        match symbol_ref.get_kind() {
            SymbolKind::Schema => {
                let schema_symbol = self.schemas.get(symbol_ref.get_id())?;
                let owner_name = self.get_fully_qualified_name(schema_symbol.owner.clone())?;
                Some(owner_name + "." + &schema_symbol.name)
            }
            SymbolKind::Attribute => {
                let attribute_symbol = self.attributes.get(symbol_ref.get_id())?;
                let owner_name = self.get_fully_qualified_name(attribute_symbol.owner)?;
                Some(owner_name + "." + &attribute_symbol.name)
            }
            SymbolKind::TypeAlias => {
                let type_symbol = self.type_aliases.get(symbol_ref.get_id())?;
                let owner_name = self.get_fully_qualified_name(type_symbol.owner)?;
                Some(owner_name + "." + &type_symbol.name)
            }
            SymbolKind::Rule => {
                let rule_symbol = self.rules.get(symbol_ref.get_id())?;
                let owner_name = self.get_fully_qualified_name(rule_symbol.owner)?;
                Some(owner_name + "." + &rule_symbol.name)
            }
            SymbolKind::Value => {
                let value_symbol = self.values.get(symbol_ref.get_id())?;
                let owner_name = self.get_fully_qualified_name(value_symbol.owner)?;
                Some(owner_name + "." + &value_symbol.name)
            }
            SymbolKind::Package => {
                Some(self.packages.get(symbol_ref.get_id()).unwrap().name.clone())
            }
            SymbolKind::Unresolved => None,
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

    fn resolve_symbol(&mut self, symbol_ref: SymbolRef, package_info: &PackageInfo) -> SymbolRef {
        if matches!(symbol_ref.get_kind(), SymbolKind::Unresolved) {
            let unresolved_symbol = self.unresolved.get_mut(symbol_ref.get_id()).unwrap();
            let target_symbol = self
                .symbols_info
                .fully_qualified_name_map
                .get(&unresolved_symbol.get_fully_qualified_name(package_info));

            match target_symbol {
                Some(target_symbol) => {
                    unresolved_symbol.def = Some(*target_symbol);
                    *target_symbol
                }
                None => symbol_ref,
            }
        } else {
            symbol_ref
        }
    }

    pub fn replace_unresolved_symbol(&mut self, package_db: &PackageDB) {
        let mut schema_refs = vec![];
        for (id, _) in self.schemas.iter() {
            schema_refs.push(id)
        }
        for schema_ref in schema_refs {
            let pkg_path = self
                .get_fully_qualified_name(self.schemas.get(schema_ref).unwrap().owner)
                .unwrap();
            let package_info = package_db.get_package_info(&pkg_path).unwrap();

            let parent_schema = self.schemas.get(schema_ref).unwrap().parent_schema.clone();
            self.schemas.get_mut(schema_ref).unwrap().parent_schema =
                parent_schema.map(|symbol_ref| self.resolve_symbol(symbol_ref, package_info));

            let mut mixins = self.schemas.get(schema_ref).unwrap().mixins.clone();

            for mixin_index in 0..mixins.len() {
                let mixin = mixins.get_mut(mixin_index).unwrap();
                *mixin = self.resolve_symbol(mixin.clone(), package_info)
            }
            self.schemas.get_mut(schema_ref).unwrap().mixins = mixins;
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy)]
pub struct SymbolRef {
    id: generational_arena::Index,
    kind: SymbolKind,
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
#[derive(Debug)]
pub struct SchemaSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,

    pub(crate) parent_schema: Option<SymbolRef>,
    pub(crate) mixins: Vec<SymbolRef>,
    pub(crate) attributes: IndexMap<String, SymbolRef>,
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
            mixins: Vec::default(),
            attributes: IndexMap::default(),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct ValueSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}

impl ValueSymbol {
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
#[derive(Debug)]
pub struct AttributeSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
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
#[derive(Debug)]
pub struct PackageSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) members: IndexMap<String, SymbolRef>,
    pub(crate) start: Position,
    pub(crate) end: Position,
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
#[derive(Debug)]
pub struct TypeAliasSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
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
#[derive(Debug)]
pub struct RuleSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}

impl RuleSymbol {
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
#[derive(Debug)]
pub struct UnresolvedSymbol {
    pub(crate) id: Option<SymbolRef>,
    pub(crate) def: Option<SymbolRef>,
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}

impl UnresolvedSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
            id: None,
            def: None,
            name,
            start,
            end,
            owner,
        }
    }

    pub fn get_fully_qualified_name(&self, package_info: &PackageInfo) -> String {
        let names: Vec<_> = self.name.split('.').collect();
        let pkg_path = if names.len() == 1 {
            kclvm_ast::MAIN_PKG.to_string()
        } else {
            let pkg_alias = names.first().unwrap();
            let import_info = package_info.get_import_info(*pkg_alias);
            match import_info {
                Some(info) => info.fully_qualified_name.clone(),
                None => kclvm_ast::MAIN_PKG.to_string(),
            }
        };

        pkg_path + "." + names.last().unwrap()
    }
}
