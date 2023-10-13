use super::{package::PackageDB, symbol::KCLSymbolData};

#[derive(Default, Debug)]
pub struct GlobalState {
    symbols: KCLSymbolData,
    packages: PackageDB,
}

impl GlobalState {
    pub fn get_symbols(&self) -> &KCLSymbolData {
        &self.symbols
    }

    pub fn get_symbols_mut(&mut self) -> &mut KCLSymbolData {
        &mut self.symbols
    }

    pub fn get_packages(&self) -> &PackageDB {
        &self.packages
    }

    pub fn get_packages_mut(&mut self) -> &mut PackageDB {
        &mut self.packages
    }

    pub fn resolve_symbols(&mut self) {
        self.symbols.replace_unresolved_symbol(&self.packages)
    }
}
