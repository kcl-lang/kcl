use indexmap::IndexMap;
use kclvm_ast::ast::AstIndex;
use std::sync::Arc;

use super::{scope::ScopeRef, symbol::SymbolRef};
use crate::ty::Type;
#[allow(unused)]
#[derive(Debug, Default, Clone)]
pub struct SemanticDB {
    pub(crate) tys: IndexMap<AstIndex, Arc<Type>>,
    pub(crate) file_sema_map: IndexMap<String, FileSemanticInfo>,
}

impl SemanticDB {
    pub fn get_file_sema(&self, file: &str) -> Option<&FileSemanticInfo> {
        self.file_sema_map.get(file)
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct FileSemanticInfo {
    pub(crate) filename: String,
    pub(crate) symbols: Vec<SymbolRef>,
    pub(crate) scopes: Vec<ScopeRef>,
    pub(crate) symbol_locs: IndexMap<SymbolRef, CachedLocation>,
    pub(crate) local_scope_locs: IndexMap<ScopeRef, CachedRange>,
}

impl FileSemanticInfo {
    pub fn new(filename: String) -> Self {
        Self {
            filename,
            symbols: vec![],
            scopes: vec![],
            symbol_locs: IndexMap::default(),
            local_scope_locs: IndexMap::default(),
        }
    }

    pub fn look_up_closest_symbol(&self, loc: &CachedLocation) -> Option<SymbolRef> {
        match self
            .symbols
            .binary_search_by(|symbol_ref| self.symbol_locs.get(symbol_ref).unwrap().cmp(loc))
        {
            Ok(symbol_index) => Some(self.symbols[symbol_index]),
            Err(symbol_index) => {
                if symbol_index > 0 {
                    Some(self.symbols[symbol_index - 1])
                } else {
                    None
                }
            }
        }
    }

    pub fn get_symbols(&self) -> &Vec<SymbolRef> {
        &self.symbols
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CachedLocation {
    pub(crate) line: u64,
    pub(crate) column: u64,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CachedRange {
    pub(crate) start: CachedLocation,
    pub(crate) end: CachedLocation,
}

impl Ord for CachedLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            core::cmp::Ordering::Equal => self.column.cmp(&other.column),
            ord => return ord,
        }
    }
}

impl PartialOrd for CachedLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.line.partial_cmp(&other.line) {
            Some(core::cmp::Ordering::Equal) => self.column.partial_cmp(&other.column),
            ord => return ord,
        }
    }
}
