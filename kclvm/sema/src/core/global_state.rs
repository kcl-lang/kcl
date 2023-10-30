use indexmap::IndexMap;
use kclvm_error::Position;

use super::{
    package::{ModuleInfo, PackageDB},
    scope::{Scope, ScopeData, ScopeKind, ScopeRef},
    semantic_information::{FileSemanticInfo, SemanticDB, SymbolLocation},
    symbol::{KCLSymbolData, SymbolKind, SymbolRef},
};

/// GlobalState is used to store semantic information of KCL source code
#[derive(Default, Debug, Clone)]
pub struct GlobalState {
    // store all allocated symbols
    symbols: KCLSymbolData,
    // store all allocated scopes
    scopes: ScopeData,
    // store package infomation for name mapping
    packages: PackageDB,
    // store semantic information after analysis
    pub(crate) sema_db: SemanticDB,
}

impl GlobalState {
    pub fn get_symbols(&self) -> &KCLSymbolData {
        &self.symbols
    }

    pub fn get_symbols_mut(&mut self) -> &mut KCLSymbolData {
        &mut self.symbols
    }

    pub fn get_scopes(&self) -> &ScopeData {
        &self.scopes
    }

    pub fn get_scopes_mut(&mut self) -> &mut ScopeData {
        &mut self.scopes
    }

    pub fn get_packages(&self) -> &PackageDB {
        &self.packages
    }

    pub fn get_packages_mut(&mut self) -> &mut PackageDB {
        &mut self.packages
    }
}

impl GlobalState {
    /// look up symbol by name within specific scope
    ///
    /// # Parameters
    ///
    ///
    /// `name`: [&str]
    ///     The name of symbol
    ///
    /// `scope_ref`: [ScopeRef]
    ///     the reference of scope which was allocated by [ScopeData]
    ///
    /// `module_info`: [Option<&ModuleInfo>]
    ///     the module import infomation
    ///
    /// # Returns
    ///
    /// result: [Option<SymbolRef>]
    ///     the matched symbol
    pub fn look_up_symbol(
        &self,
        name: &str,
        scope_ref: ScopeRef,
        module_info: Option<&ModuleInfo>,
    ) -> Option<SymbolRef> {
        self.scopes.get_scope(scope_ref)?.look_up_def(
            name,
            &self.scopes,
            &self.symbols,
            module_info,
        )
    }

    /// look up scope by specific position
    ///
    /// # Parameters
    ///
    /// `pos`: [&Position]
    ///     The pos within the scope
    ///
    ///
    /// # Returns
    ///
    /// result: [Option<ScopeRef>]
    ///     the matched scope
    pub fn look_up_scope(&self, pos: &Position) -> Option<ScopeRef> {
        let scopes = &self.scopes;
        for root_ref in scopes.root_map.values() {
            if let Some(root) = scopes.get_scope(*root_ref) {
                if root.contains_pos(pos) {
                    if let Some(inner_ref) = self.look_up_into_scope(root, pos) {
                        return Some(inner_ref);
                    } else {
                        return Some(*root_ref);
                    }
                }
            }
        }
        None
    }

    /// get all definition symbols within specific scope
    ///
    /// # Parameters
    ///
    /// `scope`: [ScopeRef]
    ///     the reference of scope which was allocated by [ScopeData]
    ///
    ///
    /// # Returns
    ///
    /// result: [Option<Vec<SymbolRef>>]
    ///      all definition symbols in the scope
    pub fn get_all_defs_in_scope(&self, scope: ScopeRef) -> Option<Vec<SymbolRef>> {
        let scopes = &self.scopes;
        let scope = scopes.get_scope(scope)?;
        let mut all_defs = scope.get_all_defs(
            scopes,
            &self.symbols,
            self.packages.get_module_info(scope.get_filename()),
        );
        all_defs.sort();
        all_defs.dedup();
        Some(all_defs)
    }

    /// look up closest symbol by specific position, which means  
    /// the specified position is located after the starting position of the returned symbol
    /// and before the starting position of the next symbol
    ///
    /// # Parameters
    ///
    /// `pos`: [&Position]
    ///     The target pos
    ///
    ///
    /// # Returns
    ///
    /// result: [Option<SymbolRef>]
    ///     the closest symbol to the target pos
    pub fn look_up_closest_symbol(&self, pos: &Position) -> Option<SymbolRef> {
        Some(
            self.sema_db
                .file_sema_map
                .get(&pos.filename)?
                .look_up_closest_symbol(&SymbolLocation {
                    line: pos.line,
                    column: pos.column.unwrap_or(0),
                }),
        )
    }

    /// look up exact symbol by specific position, which means  
    /// the specified position is within the range of the returned symbol
    ///
    /// # Parameters
    ///
    /// `pos`: [&Position]
    ///     The target pos
    ///
    ///
    /// # Returns
    ///
    /// result: [Option<SymbolRef>]
    ///     the exact symbol to the target pos
    pub fn look_up_exact_symbol(&self, pos: &Position) -> Option<SymbolRef> {
        let candidate = self
            .sema_db
            .file_sema_map
            .get(&pos.filename)?
            .look_up_closest_symbol(&SymbolLocation {
                line: pos.line,
                column: pos.column.unwrap_or(0),
            });
        let (start, end) = self.symbols.get_symbol(candidate)?.get_range();
        if start.less_equal(pos) && pos.less_equal(&end) {
            Some(candidate)
        } else {
            None
        }
    }

    fn look_up_into_scope(
        &self,
        root: &dyn Scope<SymbolData = KCLSymbolData>,
        pos: &Position,
    ) -> Option<ScopeRef> {
        let children = root.get_children();
        for child_ref in children {
            if let Some(child) = self.scopes.get_scope(child_ref) {
                if child.contains_pos(pos) {
                    if let Some(inner_ref) = self.look_up_into_scope(child, pos) {
                        return Some(inner_ref);
                    } else {
                        return Some(child_ref);
                    }
                }
            }
        }
        None
    }
}

impl GlobalState {
    pub(crate) fn build_sema_db(&mut self) {
        let mut file_sema_map = IndexMap::<String, FileSemanticInfo>::default();

        // put symbols
        for (index, symbol) in self.symbols.schemas.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Schema,
                id: index,
            };
            let filename = symbol.start.filename.clone();
            if !file_sema_map.contains_key(&filename) {
                file_sema_map.insert(filename.clone(), FileSemanticInfo::new(filename.clone()));
            }
            let file_sema_info = file_sema_map.get_mut(&filename).unwrap();
            file_sema_info.symbols.push(symbol_ref);
            file_sema_info.symbol_locs.insert(
                symbol_ref,
                SymbolLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }
        for (index, symbol) in self.symbols.type_aliases.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::TypeAlias,
                id: index,
            };
            let filename = symbol.start.filename.clone();
            if !file_sema_map.contains_key(&filename) {
                file_sema_map.insert(filename.clone(), FileSemanticInfo::new(filename.clone()));
            }
            let file_sema_info = file_sema_map.get_mut(&filename).unwrap();
            file_sema_info.symbols.push(symbol_ref);
            file_sema_info.symbol_locs.insert(
                symbol_ref,
                SymbolLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }
        for (index, symbol) in self.symbols.attributes.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Attribute,
                id: index,
            };
            let filename = symbol.start.filename.clone();
            if !file_sema_map.contains_key(&filename) {
                file_sema_map.insert(filename.clone(), FileSemanticInfo::new(filename.clone()));
            }
            let file_sema_info = file_sema_map.get_mut(&filename).unwrap();
            file_sema_info.symbols.push(symbol_ref);
            file_sema_info.symbol_locs.insert(
                symbol_ref,
                SymbolLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }
        for (index, symbol) in self.symbols.rules.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Rule,
                id: index,
            };
            let filename = symbol.start.filename.clone();
            if !file_sema_map.contains_key(&filename) {
                file_sema_map.insert(filename.clone(), FileSemanticInfo::new(filename.clone()));
            }
            let file_sema_info = file_sema_map.get_mut(&filename).unwrap();
            file_sema_info.symbols.push(symbol_ref);
            file_sema_info.symbol_locs.insert(
                symbol_ref,
                SymbolLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }
        for (index, symbol) in self.symbols.values.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Value,
                id: index,
            };
            let filename = symbol.start.filename.clone();
            if !file_sema_map.contains_key(&filename) {
                file_sema_map.insert(filename.clone(), FileSemanticInfo::new(filename.clone()));
            }
            let file_sema_info = file_sema_map.get_mut(&filename).unwrap();
            file_sema_info.symbols.push(symbol_ref);
            file_sema_info.symbol_locs.insert(
                symbol_ref,
                SymbolLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }
        for (index, symbol) in self.symbols.unresolved.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Unresolved,
                id: index,
            };
            let filename = symbol.start.filename.clone();
            if !file_sema_map.contains_key(&filename) {
                file_sema_map.insert(filename.clone(), FileSemanticInfo::new(filename.clone()));
            }
            let file_sema_info = file_sema_map.get_mut(&filename).unwrap();
            file_sema_info.symbols.push(symbol_ref);
            file_sema_info.symbol_locs.insert(
                symbol_ref,
                SymbolLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }

        // put scope
        for (index, scope) in self.scopes.locals.iter() {
            let scope_ref = ScopeRef {
                kind: ScopeKind::Local,
                id: index,
            };
            let filename = scope.start.filename.clone();
            if !file_sema_map.contains_key(&filename) {
                file_sema_map.insert(filename.clone(), FileSemanticInfo::new(filename.clone()));
            }
            file_sema_map
                .get_mut(&filename)
                .unwrap()
                .scopes
                .push(scope_ref);
        }

        for (_, sema_info) in file_sema_map.iter_mut() {
            sema_info
                .symbols
                .sort_by_key(|symbol_ref| sema_info.symbol_locs.get(symbol_ref).unwrap())
        }

        self.sema_db.file_sema_map = file_sema_map;
    }
}
