use indexmap::IndexMap;
use kclvm_error::Position;

use super::{
    package::{ModuleInfo, PackageDB},
    scope::{ScopeData, ScopeKind, ScopeRef},
    semantic_information::{CachedLocation, CachedRange, FileSemanticInfo, SemanticDB},
    symbol::{SymbolData, SymbolKind, SymbolRef},
};

/// GlobalState is used to store semantic information of KCL source code
#[derive(Default, Debug, Clone)]
pub struct GlobalState {
    // store all allocated symbols
    symbols: SymbolData,
    // store all allocated scopes
    scopes: ScopeData,
    // store package information for name mapping
    packages: PackageDB,
    // store semantic information after analysis
    pub(crate) sema_db: SemanticDB,
}

impl GlobalState {
    pub fn get_symbols(&self) -> &SymbolData {
        &self.symbols
    }

    pub fn get_symbols_mut(&mut self) -> &mut SymbolData {
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

    pub fn get_sema_db(&self) -> &SemanticDB {
        &self.sema_db
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
    ///     the module import information
    /// `local`: [bool]
    ///     look up in current scope
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
        local: bool,
    ) -> Option<SymbolRef> {
        match self.scopes.get_scope(&scope_ref)?.look_up_def(
            name,
            &self.scopes,
            &self.symbols,
            module_info,
            local,
        ) {
            None => self
                .symbols
                .symbols_info
                .global_builtin_symbols
                .get(name)
                .cloned(),
            some => some,
        }
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
            if let Some(root) = scopes.get_scope(root_ref) {
                if root.contains_pos(pos) {
                    if let Some(inner_ref) = self.look_up_into_scope(*root_ref, pos) {
                        return Some(inner_ref);
                    } else {
                        return Some(*root_ref);
                    }
                }
            }
        }
        None
    }

    fn look_up_closest_sub_scope(&self, parent: ScopeRef, pos: &Position) -> Option<ScopeRef> {
        let file_sema_info = self.sema_db.file_sema_map.get(&pos.filename)?;
        let loc = CachedLocation {
            line: pos.line,
            column: pos.column.unwrap_or(0),
        };
        let children = match parent.kind {
            ScopeKind::Local => &self.scopes.locals.get(parent.id)?.children,
            ScopeKind::Root => &self
                .scopes
                .roots
                .get(parent.id)?
                .children
                .get(&pos.filename)?,
        };

        match children.binary_search_by(|scope_ref| {
            file_sema_info
                .local_scope_locs
                .get(scope_ref)
                .unwrap()
                .start
                .cmp(&loc)
        }) {
            Ok(symbol_index) => Some(children[symbol_index]),
            Err(symbol_index) => {
                if symbol_index > 0 {
                    Some(children[symbol_index - 1])
                } else {
                    None
                }
            }
        }
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
        let scope = scopes.get_scope(&scope)?;
        let all_defs: Vec<SymbolRef> = scope
            .get_all_defs(
                scopes,
                &self.symbols,
                self.packages.get_module_info(scope.get_filename()),
                false,
            )
            .values()
            .into_iter()
            .cloned()
            .collect();
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
        let file_sema_info = self.sema_db.file_sema_map.get(&pos.filename)?;
        let candidate = file_sema_info.look_up_closest_symbol(&CachedLocation {
            line: pos.line,
            column: pos.column.unwrap_or(0),
        });
        match self.look_up_scope(pos) {
            Some(parent_scope_ref) => {
                let candidate_symbol = self.symbols.get_symbol(candidate?)?;
                let (start, _) = candidate_symbol.get_range();
                let parent_scope = self.scopes.get_scope(&parent_scope_ref)?;
                if parent_scope.contains_pos(&start) {
                    let barrier_scope = self.look_up_closest_sub_scope(parent_scope_ref, pos);
                    match barrier_scope {
                        Some(barrier_scope) => {
                            let barrier_scope = self.scopes.locals.get(barrier_scope.id)?;
                            // there is no local scope between the candidate and the specified position
                            // the candidate is the answer
                            if barrier_scope.end.less(&candidate_symbol.get_range().0) {
                                candidate
                            }
                            // otherwise, it indicates that the found symbol is shadowed by the local scope.
                            // we just skip the scope and directly look up its start pos
                            else {
                                file_sema_info.look_up_closest_symbol(&CachedLocation {
                                    line: barrier_scope.start.line,
                                    column: barrier_scope.start.column.unwrap_or(0),
                                })
                            }
                        }
                        None => candidate,
                    }
                } else {
                    None
                }
            }
            None => None,
        }
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
            .look_up_closest_symbol(&CachedLocation {
                line: pos.line,
                column: pos.column.unwrap_or(0),
            });
        let (start, end) = self.symbols.get_symbol(candidate?)?.get_range();
        if start.less_equal(pos) && pos.less_equal(&end) {
            candidate
        } else {
            None
        }
    }

    fn look_up_into_scope(&self, parent: ScopeRef, pos: &Position) -> Option<ScopeRef> {
        let candidate_ref = self.look_up_closest_sub_scope(parent, pos)?;

        let candidate = self.scopes.get_scope(&candidate_ref)?;
        if candidate.contains_pos(pos) {
            if let Some(inner_ref) = self.look_up_into_scope(candidate_ref, pos) {
                return Some(inner_ref);
            } else {
                return Some(candidate_ref);
            }
        }
        None
    }
}

impl GlobalState {
    fn build_sema_db_with_symbols(&self, file_sema_map: &mut IndexMap<String, FileSemanticInfo>) {
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
                CachedLocation {
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
                CachedLocation {
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
                CachedLocation {
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
                CachedLocation {
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
                CachedLocation {
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
                CachedLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }

        for (index, symbol) in self.symbols.exprs.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Expression,
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
                CachedLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }

        for (index, symbol) in self.symbols.comments.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Comment,
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
                CachedLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }

        for (index, symbol) in self.symbols.decorators.iter() {
            let symbol_ref = SymbolRef {
                kind: SymbolKind::Decorator,
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
                CachedLocation {
                    line: symbol.start.line,
                    column: symbol.start.column.unwrap_or(0),
                },
            );
        }

        // remove dummy file
        file_sema_map.remove("");

        for (_, sema_info) in file_sema_map.iter_mut() {
            sema_info
                .symbols
                .sort_by_key(|symbol_ref| sema_info.symbol_locs.get(symbol_ref).unwrap())
        }
    }
    fn build_sema_db_with_scopes(&self, file_sema_map: &mut IndexMap<String, FileSemanticInfo>) {
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
            let file_sema_info = file_sema_map.get_mut(&filename).unwrap();
            file_sema_info.local_scope_locs.insert(
                scope_ref,
                CachedRange {
                    start: CachedLocation {
                        line: scope.start.line,
                        column: scope.start.column.unwrap_or(0),
                    },
                    end: CachedLocation {
                        line: scope.end.line,
                        column: scope.end.column.unwrap_or(0),
                    },
                },
            );
            file_sema_map
                .get_mut(&filename)
                .unwrap()
                .scopes
                .push(scope_ref);
        }
    }

    fn sort_local_scopes(&mut self, file_sema_map: &IndexMap<String, FileSemanticInfo>) {
        // Direct sub scopes do not overlap, so we can directly sort them by start loc
        for (_, root) in self.scopes.roots.iter_mut() {
            for (filename, scopes) in root.children.iter_mut() {
                let file_sema_info = file_sema_map.get(filename).unwrap();
                scopes.sort_by_key(|scope_ref| {
                    &file_sema_info
                        .local_scope_locs
                        .get(scope_ref)
                        .unwrap()
                        .start
                })
            }
        }
        // Direct sub scopes do not overlap, so we can directly sort them by start loc
        for (_, scope) in self.scopes.locals.iter_mut() {
            let file_sema_info = file_sema_map.get(&scope.start.filename).unwrap();
            scope.children.sort_by_key(|scope_ref| {
                &file_sema_info
                    .local_scope_locs
                    .get(scope_ref)
                    .unwrap()
                    .start
            })
        }
    }

    pub(crate) fn build_sema_db(&mut self) {
        let mut file_sema_map = IndexMap::<String, FileSemanticInfo>::default();
        self.build_sema_db_with_symbols(&mut file_sema_map);
        self.build_sema_db_with_scopes(&mut file_sema_map);
        self.sort_local_scopes(&mut file_sema_map);

        self.sema_db.file_sema_map = file_sema_map;
    }
}
