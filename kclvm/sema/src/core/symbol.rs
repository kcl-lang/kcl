use generational_arena::Arena;
use indexmap::IndexMap;

use kclvm_error::Position;

use super::package::{PackageDB, PackageInfo};

#[derive(Default, Debug)]
pub struct KCLSymbolData {
    pub(crate) values: Arena<ValueSymbol>,
    pub(crate) packages: Arena<PackageSymbol>,
    pub(crate) functions: Arena<FunctionSymbol>,
    pub(crate) attributes: Arena<AttributeSymbol>,
    pub(crate) schemas: Arena<SchemaSymbol>,
    pub(crate) type_aliases: Arena<TypeAliasSymbol>,
    pub(crate) unresolved: Arena<UnresolvedSymbol>,
    pub(crate) rules: Arena<RuleSymbol>,

    pub(crate) fully_qualified_name_map: IndexMap<String, SymbolRef>,
}

impl KCLSymbolData {
    pub fn get_symbol_by_fully_qualified_name(&self, fqn: &str) -> Option<SymbolRef> {
        self.fully_qualified_name_map.get(fqn).cloned()
    }

    pub fn get_fully_qualified_name(&self, symbol_ref: SymbolRef) -> String {
        match symbol_ref.get_kind() {
            SymbolKind::Schema => {
                let schema_symbol = self.schemas.get(symbol_ref.get_id()).unwrap();
                let owner_name = self.get_fully_qualified_name(schema_symbol.owner.clone());
                owner_name + "." + &schema_symbol.name
            }
            SymbolKind::Attribute => {
                let attribute_symbol = self.attributes.get(symbol_ref.get_id()).unwrap();
                let owner_name = self.get_fully_qualified_name(attribute_symbol.owner);
                owner_name + "." + &attribute_symbol.name
            }
            SymbolKind::TypeAlias => {
                let type_symbol = self.type_aliases.get(symbol_ref.get_id()).unwrap();
                let owner_name = self.get_fully_qualified_name(type_symbol.owner);
                owner_name + "." + &type_symbol.name
            }
            SymbolKind::Rule => {
                let rule_symbol = self.rules.get(symbol_ref.get_id()).unwrap();
                let owner_name = self.get_fully_qualified_name(rule_symbol.owner);
                owner_name + "." + &rule_symbol.name
            }
            SymbolKind::Package => self.packages.get(symbol_ref.get_id()).unwrap().name.clone(),
            SymbolKind::Function => todo!(),
            SymbolKind::Value => todo!(),
            SymbolKind::Unresolved => todo!(),
            SymbolKind::Dummy => todo!(),
        }
    }

    pub fn build_fully_qualified_name_map(&mut self) {
        for (id, _) in self.packages.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Package,
            };
            self.fully_qualified_name_map
                .insert(self.get_fully_qualified_name(symbol_ref), symbol_ref);
        }

        for (id, _) in self.schemas.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Schema,
            };
            self.fully_qualified_name_map
                .insert(self.get_fully_qualified_name(symbol_ref), symbol_ref);
        }

        for (id, _) in self.type_aliases.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::TypeAlias,
            };
            self.fully_qualified_name_map
                .insert(self.get_fully_qualified_name(symbol_ref), symbol_ref);
        }

        for (id, _) in self.attributes.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Attribute,
            };
            self.fully_qualified_name_map
                .insert(self.get_fully_qualified_name(symbol_ref), symbol_ref);
        }

        for (id, _) in self.rules.iter() {
            let symbol_ref = SymbolRef {
                id,
                kind: SymbolKind::Rule,
            };
            self.fully_qualified_name_map
                .insert(self.get_fully_qualified_name(symbol_ref), symbol_ref);
        }
    }

    fn resolve_symbol(&self, symbol_ref: SymbolRef, package_info: &PackageInfo) -> SymbolRef {
        if matches!(symbol_ref.get_kind(), SymbolKind::Unresolved) {
            let unresolved_symbol = self.unresolved.get(symbol_ref.get_id()).unwrap();
            let target_symbol = self
                .fully_qualified_name_map
                .get(&unresolved_symbol.get_fully_qualified_name(package_info));
            match target_symbol {
                Some(target_symbol) => *target_symbol,
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
            let pkg_path =
                self.get_fully_qualified_name(self.schemas.get(schema_ref).unwrap().owner);
            let package_info = package_db.get_package_info(&pkg_path).unwrap();
            {
                self.schemas.get_mut(schema_ref).unwrap().parent_schema = self
                    .schemas
                    .get(schema_ref)
                    .unwrap()
                    .parent_schema
                    .map(|symbol_ref| self.resolve_symbol(symbol_ref, package_info));
            }

            self.schemas.get_mut(schema_ref).unwrap().mixins = self
                .schemas
                .get(schema_ref)
                .unwrap()
                .mixins
                .iter()
                .map(|symbol_ref| self.resolve_symbol(*symbol_ref, package_info))
                .collect();
        }
    }

    pub fn alloc_package_symbol(&mut self, pkg: PackageSymbol) -> SymbolRef {
        let id = self.packages.insert(pkg);
        SymbolRef {
            id,
            kind: SymbolKind::Package,
        }
    }

    pub fn alloc_schema_symbol(&mut self, schema: SchemaSymbol) -> SymbolRef {
        let id = self.schemas.insert(schema);
        SymbolRef {
            id,
            kind: SymbolKind::Schema,
        }
    }

    pub fn alloc_unresolved_symbol(&mut self, unresolved: UnresolvedSymbol) -> SymbolRef {
        let id = self.unresolved.insert(unresolved);
        SymbolRef {
            id,
            kind: SymbolKind::Unresolved,
        }
    }

    pub fn alloc_type_alias_symbol(&mut self, alias: TypeAliasSymbol) -> SymbolRef {
        let id = self.type_aliases.insert(alias);
        SymbolRef {
            id,
            kind: SymbolKind::TypeAlias,
        }
    }

    pub fn alloc_rule_symbol(&mut self, rule: RuleSymbol) -> SymbolRef {
        let id = self.rules.insert(rule);
        SymbolRef {
            id,
            kind: SymbolKind::Rule,
        }
    }

    pub fn alloc_attribute_symbol(&mut self, attribute: AttributeSymbol) -> SymbolRef {
        let id = self.attributes.insert(attribute);
        SymbolRef {
            id,
            kind: SymbolKind::Attribute,
        }
    }
}
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolKind {
    Schema,
    Function,
    Attribute,
    Value,
    Package,
    TypeAlias,
    Unresolved,
    Rule,
    Dummy,
}
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct SymbolRef {
    id: generational_arena::Index,
    kind: SymbolKind,
}

impl SymbolRef {
    pub fn dummy_symbol() -> Self {
        Self {
            id: generational_arena::Index::from_raw_parts(0, 0),
            kind: SymbolKind::Dummy,
        }
    }
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

pub struct FunctionSymbol {
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,

    pub(crate) args: IndexMap<String, SymbolRef>,
}
#[allow(unused)]
#[derive(Debug)]
pub struct ValueSymbol {
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}
#[allow(unused)]
#[derive(Debug)]
pub struct AttributeSymbol {
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}

impl AttributeSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
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
    pub(crate) name: String,
    pub(crate) members: IndexMap<String, SymbolRef>,
    pub(crate) start: Position,
    pub(crate) end: Position,
}

impl PackageSymbol {
    pub fn new(name: String, start: Position, end: Position) -> Self {
        Self {
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
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}

impl TypeAliasSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
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
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}

impl RuleSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
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
    pub(crate) name: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) owner: SymbolRef,
}

impl UnresolvedSymbol {
    pub fn new(name: String, start: Position, end: Position, owner: SymbolRef) -> Self {
        Self {
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
