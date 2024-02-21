use std::collections::HashMap;

use indexmap::{IndexMap, IndexSet};
use kclvm_error::Position;
use serde::Serialize;

use crate::core::symbol::SymbolRef;

use super::{package::ModuleInfo, symbol::SymbolData};

pub trait Scope {
    type SymbolData;
    fn get_filename(&self) -> &str;
    fn get_parent(&self) -> Option<ScopeRef>;
    fn get_children(&self) -> Vec<ScopeRef>;

    fn contains_pos(&self, pos: &Position) -> bool;
    fn get_range(&self) -> Option<(Position, Position)>;

    fn get_owner(&self) -> Option<SymbolRef>;
    fn look_up_def(
        &self,
        name: &str,
        scope_data: &ScopeData,
        symbol_data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
        local: bool,
    ) -> Option<SymbolRef>;

    fn get_all_defs(
        &self,
        scope_data: &ScopeData,
        symbol_data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
        recursive: bool,
    ) -> HashMap<String, SymbolRef>;

    fn dump(&self, scope_data: &ScopeData, symbol_data: &Self::SymbolData) -> Option<String>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize)]
pub enum ScopeKind {
    Local,
    Root,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ScopeRef {
    pub(crate) id: generational_arena::Index,
    pub(crate) kind: ScopeKind,
}

impl Serialize for ScopeRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (index, generation) = self.id.into_raw_parts();
        let data = SerializableScopeRef {
            i: index as u64,
            g: generation,
            kind: self.kind.clone(),
        };
        data.serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize)]

struct SerializableScopeRef {
    i: u64,
    g: u64,
    kind: ScopeKind,
}

impl ScopeRef {
    pub fn get_id(&self) -> generational_arena::Index {
        self.id
    }

    pub fn get_kind(&self) -> ScopeKind {
        self.kind
    }
}

#[derive(Default, Debug, Clone)]
pub struct ScopeData {
    /// map pkgpath to root_scope
    pub(crate) root_map: IndexMap<String, ScopeRef>,
    pub(crate) locals: generational_arena::Arena<LocalSymbolScope>,
    pub(crate) roots: generational_arena::Arena<RootSymbolScope>,
}

impl ScopeData {
    #[inline]
    pub fn get_root_scope_map(&self) -> &IndexMap<String, ScopeRef> {
        &self.root_map
    }

    pub fn get_scope(&self, scope: &ScopeRef) -> Option<&dyn Scope<SymbolData = SymbolData>> {
        match scope.get_kind() {
            ScopeKind::Local => {
                Some(self.locals.get(scope.get_id())? as &dyn Scope<SymbolData = SymbolData>)
            }
            ScopeKind::Root => {
                Some(self.roots.get(scope.get_id())? as &dyn Scope<SymbolData = SymbolData>)
            }
        }
    }

    pub fn try_get_local_scope(&self, scope: &ScopeRef) -> Option<&LocalSymbolScope> {
        match scope.get_kind() {
            ScopeKind::Local => Some(self.locals.get(scope.get_id())?),
            ScopeKind::Root => None,
        }
    }

    pub fn get_root_scope(&self, name: String) -> Option<ScopeRef> {
        self.root_map.get(&name).copied()
    }

    pub fn add_def_to_scope(&mut self, scope: ScopeRef, name: String, symbol: SymbolRef) {
        match scope.get_kind() {
            ScopeKind::Local => {
                if let Some(local) = self.locals.get_mut(scope.get_id()) {
                    local.defs.insert(name, symbol);
                }
            }
            ScopeKind::Root => {
                unreachable!("never add symbol to root scope after namer pass")
            }
        }
    }

    pub fn add_ref_to_scope(&mut self, scope: ScopeRef, symbol: SymbolRef) {
        match scope.get_kind() {
            ScopeKind::Local => {
                if let Some(local) = self.locals.get_mut(scope.get_id()) {
                    local.refs.push(symbol);
                }
            }
            ScopeKind::Root => {
                if let Some(root) = self.roots.get_mut(scope.get_id()) {
                    root.refs.push(symbol);
                }
            }
        }
    }

    pub fn set_owner_to_scope(&mut self, scope: ScopeRef, owner: SymbolRef) {
        match scope.get_kind() {
            ScopeKind::Local => {
                if let Some(local) = self.locals.get_mut(scope.get_id()) {
                    local.owner = Some(owner);
                }
            }
            ScopeKind::Root => {
                if let Some(root) = self.roots.get_mut(scope.get_id()) {
                    root.owner = owner;
                }
            }
        }
    }

    pub fn alloc_root_scope(&mut self, root: RootSymbolScope) -> ScopeRef {
        let filepath = root.pkgpath.clone();
        let id = self.roots.insert(root);
        let scope_ref = ScopeRef {
            id,
            kind: ScopeKind::Root,
        };
        self.root_map.insert(filepath, scope_ref);
        scope_ref
    }

    pub fn alloc_local_scope(&mut self, local: LocalSymbolScope) -> ScopeRef {
        let id = self.locals.insert(local);
        ScopeRef {
            id,
            kind: ScopeKind::Local,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RootSymbolScope {
    pub(crate) pkgpath: String,

    pub(crate) filename: String,

    pub(crate) kfile_path: IndexSet<String>,

    /// PackageSymbol of this scope
    pub(crate) owner: SymbolRef,

    /// map filepath to children
    pub(crate) children: IndexMap<String, Vec<ScopeRef>>,

    pub(crate) refs: Vec<SymbolRef>,
}

impl Scope for RootSymbolScope {
    type SymbolData = SymbolData;
    fn get_filename(&self) -> &str {
        &self.filename
    }

    fn get_children(&self) -> Vec<ScopeRef> {
        let mut children = vec![];
        for scopes in self.children.values() {
            children.append(&mut scopes.clone())
        }
        children
    }

    fn get_parent(&self) -> Option<ScopeRef> {
        None
    }

    fn contains_pos(&self, pos: &Position) -> bool {
        self.kfile_path.contains(&pos.filename)
    }
    fn get_owner(&self) -> Option<SymbolRef> {
        Some(self.owner)
    }

    fn look_up_def(
        &self,
        name: &str,
        _scope_data: &ScopeData,
        symbol_data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
        _local: bool,
    ) -> Option<SymbolRef> {
        let package_symbol = symbol_data.get_symbol(self.owner)?;

        package_symbol.get_attribute(name, symbol_data, module_info)
    }

    fn get_all_defs(
        &self,
        _scope_data: &ScopeData,
        symbol_data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
        _recursive: bool,
    ) -> HashMap<String, SymbolRef> {
        let mut all_defs_map = HashMap::new();
        if let Some(owner) = symbol_data.get_symbol(self.owner) {
            let all_defs = owner.get_all_attributes(symbol_data, module_info);

            for def_ref in all_defs {
                if let Some(def) = symbol_data.get_symbol(def_ref) {
                    all_defs_map.insert(def.get_name(), def_ref);
                }
            }
        }
        all_defs_map
    }

    fn dump(&self, scope_data: &ScopeData, symbol_data: &Self::SymbolData) -> Option<String> {
        let mut output = String::from("");
        output.push_str("{\n\"scope_kind\": \"Root\",\n");
        output.push_str(&format!("\n\"pkgpath\": \"{}\",\n", self.pkgpath));
        let owner_symbol = symbol_data.get_symbol(self.owner)?;
        output.push_str(&format!(
            "\"owner\": {},\n",
            owner_symbol.full_dump(symbol_data)?
        ));
        output.push_str("\"refs\": [\n");
        for (index, symbol) in self.refs.iter().enumerate() {
            let symbol = symbol_data.get_symbol(*symbol)?;
            output.push_str(&format!("{}", symbol.full_dump(symbol_data)?));
            if index + 1 < self.refs.len() {
                output.push_str(",\n")
            }
        }
        output.push_str("\n],\n");
        output.push_str("\"children\": {\n");
        for (index, (key, scopes)) in self.children.iter().enumerate() {
            output.push_str(&format!("\"{}\": [\n", key));
            for (index, scope) in scopes.iter().enumerate() {
                let scope = scope_data.get_scope(scope)?;
                output.push_str(&format!("{}", scope.dump(scope_data, symbol_data)?));
                if index + 1 < scopes.len() {
                    output.push_str(",\n");
                }
            }
            output.push_str("\n]");
            if index + 1 < self.children.len() {
                output.push_str(",\n");
            }
        }
        output.push_str("\n}\n}");

        let val: serde_json::Value = serde_json::from_str(&output).unwrap();
        Some(serde_json::to_string_pretty(&val).ok()?)
    }

    fn get_range(&self) -> Option<(Position, Position)> {
        None
    }
}

impl RootSymbolScope {
    pub fn new(
        pkgpath: String,
        filename: String,
        owner: SymbolRef,
        kfile_path: IndexSet<String>,
    ) -> Self {
        Self {
            pkgpath,
            kfile_path,
            filename,
            owner,
            children: IndexMap::default(),
            refs: vec![],
        }
    }

    pub fn add_child(&mut self, filepath: &str, child: ScopeRef) {
        if self.children.contains_key(filepath) {
            self.children.get_mut(filepath).unwrap().push(child);
        } else {
            self.children.insert(filepath.to_string(), vec![child]);
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalSymbolScope {
    pub(crate) parent: ScopeRef,
    pub(crate) owner: Option<SymbolRef>,
    pub(crate) children: Vec<ScopeRef>,
    pub(crate) defs: IndexMap<String, SymbolRef>,
    pub(crate) refs: Vec<SymbolRef>,

    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) kind: LocalSymbolScopeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalSymbolScopeKind {
    List,
    Dict,
    Quant,
    Lambda,
    SchemaDef,
    SchemaConfig,
    Value,
    Check,
}

impl Scope for LocalSymbolScope {
    type SymbolData = SymbolData;

    fn get_filename(&self) -> &str {
        &self.start.filename
    }

    fn get_children(&self) -> Vec<ScopeRef> {
        self.children.clone()
    }

    fn get_parent(&self) -> Option<ScopeRef> {
        Some(self.parent)
    }

    fn contains_pos(&self, pos: &Position) -> bool {
        self.start.filename == pos.filename
            && self.start.less_equal(pos)
            && pos.less_equal(&self.end)
    }

    fn get_owner(&self) -> Option<SymbolRef> {
        self.owner.clone()
    }

    fn look_up_def(
        &self,
        name: &str,
        scope_data: &ScopeData,
        symbol_data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
        local: bool,
    ) -> Option<SymbolRef> {
        match self.defs.get(name) {
            Some(symbol_ref) => return Some(*symbol_ref),
            None => {
                if let Some(owner) = self.owner.as_ref() {
                    let owner_symbol = symbol_data.get_symbol(*owner)?;
                    if let Some(symbol_ref) =
                        owner_symbol.get_attribute(name, symbol_data, module_info)
                    {
                        return Some(symbol_ref);
                    }
                };

                if local {
                    None
                } else {
                    let parent = scope_data.get_scope(&self.parent)?;
                    parent.look_up_def(name, scope_data, symbol_data, module_info, false)
                }
            }
        }
    }

    fn get_all_defs(
        &self,
        scope_data: &ScopeData,
        symbol_data: &Self::SymbolData,
        module_info: Option<&ModuleInfo>,
        recursive: bool,
    ) -> HashMap<String, SymbolRef> {
        let mut all_defs_map = HashMap::new();
        if let Some(owner) = self.owner {
            if let Some(owner) = symbol_data.get_symbol(owner) {
                for def_ref in owner.get_all_attributes(symbol_data, module_info) {
                    if let Some(def) = symbol_data.get_symbol(def_ref) {
                        let name = def.get_name();
                        if !all_defs_map.contains_key(&name) {
                            all_defs_map.insert(name, def_ref);
                        }
                    }
                }
            }
        }
        // In SchemaConfig, available definitions only contain keys of schema attr，i.e., `left` values in schema expr.
        // but in child scope, i.e., right value in schema expr, available definitions contain all parent definitions.
        // ```
        // b = "bar"
        // foo = Foo{
        //   bar: b
        // }
        // ````
        // and scope range is(use `#kind[]` to represent the range of the scope`)
        // ```
        // #Root[
        // b = "bar"
        // foo = Foo #SchemaConfig[{
        //   bar: #Value[b]
        // }]
        // ]
        // ````
        // At position of `bar`, the scope kind is SchemaConfig, only get the definition of bar.
        // At position of seconde `b`, the scope is the child scope of SchemaConfig, need to recursively find the definition of `b`` at a higher level
        if self.kind == LocalSymbolScopeKind::SchemaConfig && !recursive {
            return all_defs_map;
        } else {
            for def_ref in self.defs.values() {
                if let Some(def) = symbol_data.get_symbol(*def_ref) {
                    all_defs_map.insert(def.get_name(), *def_ref);
                }
            }

            if let Some(parent) = scope_data.get_scope(&self.parent) {
                for (name, def_ref) in
                    parent.get_all_defs(scope_data, symbol_data, module_info, true)
                {
                    if !all_defs_map.contains_key(&name) {
                        all_defs_map.insert(name, def_ref);
                    }
                }
            }
        }
        all_defs_map
    }

    fn dump(&self, scope_data: &ScopeData, symbol_data: &Self::SymbolData) -> Option<String> {
        let mut output = String::from("");
        output.push_str("{\n\"scope_kind\": \"Local\",\n");
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
        output.push_str("\",\n");
        if let Some(owner) = self.owner.as_ref() {
            let owner_symbol = symbol_data.get_symbol(*owner)?;
            output.push_str(&format!(
                "\"owner\": {},\n",
                owner_symbol.full_dump(symbol_data)?
            ));
        }
        output.push_str("\"defs\": {\n");
        for (index, (key, symbol)) in self.defs.iter().enumerate() {
            let symbol = symbol_data.get_symbol(*symbol)?;
            output.push_str(&format!("\"{}\": {}", key, symbol.full_dump(symbol_data)?));
            if index + 1 < self.defs.len() {
                output.push_str(",\n")
            }
        }
        output.push_str("\n},\n");
        output.push_str("\"refs\": [\n");
        for (index, symbol) in self.refs.iter().enumerate() {
            let symbol = symbol_data.get_symbol(*symbol)?;
            output.push_str(&format!("{}", symbol.full_dump(symbol_data)?));
            if index + 1 < self.refs.len() {
                output.push_str(",\n")
            }
        }
        output.push_str("\n],");
        output.push_str("\n\"children\": [\n");
        for (index, scope) in self.children.iter().enumerate() {
            let scope = scope_data.get_scope(scope)?;
            output.push_str(&format!("{}", scope.dump(scope_data, symbol_data)?));
            if index + 1 < self.children.len() {
                output.push_str(",\n")
            }
        }
        output.push_str("\n]\n}");
        Some(output)
    }

    fn get_range(&self) -> Option<(Position, Position)> {
        Some((self.start.clone(), self.end.clone()))
    }
}

impl LocalSymbolScope {
    pub fn new(
        parent: ScopeRef,
        start: Position,
        end: Position,
        kind: LocalSymbolScopeKind,
    ) -> Self {
        Self {
            parent,
            owner: None,
            children: vec![],
            defs: IndexMap::default(),
            refs: vec![],
            start,
            end,
            kind,
        }
    }

    #[inline]
    pub fn get_kind(&self) -> &LocalSymbolScopeKind {
        &self.kind
    }

    #[inline]
    pub fn add_child(&mut self, child: ScopeRef) {
        self.children.push(child)
    }

    #[inline]
    pub fn set_owner(&mut self, owner: SymbolRef) {
        self.owner = Some(owner)
    }
}
