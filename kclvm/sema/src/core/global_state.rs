use indexmap::IndexMap;
use kclvm_error::Position;

use super::{
    package::{ModuleInfo, PackageDB},
    scope::{Scope, ScopeData, ScopeKind, ScopeRef},
    semantic_information::{FileSemanticInfo, SemanticDB, SymbolLocation},
    symbol::{KCLSymbolData, SymbolKind, SymbolRef},
};

#[derive(Default, Debug)]
pub struct GlobalState {
    symbols: KCLSymbolData,
    packages: PackageDB,
    scopes: ScopeData,
    pub(crate) sema_db: SemanticDB,
}

impl GlobalState {
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
    pub fn build_sema_db(&mut self) {
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
