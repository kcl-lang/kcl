use indexmap::IndexMap;

#[derive(Default, Debug)]
pub struct PackageDB {
    pub(crate) package_info: IndexMap<String, PackageInfo>,
}

impl PackageDB {
    pub fn add_package(&mut self, info: PackageInfo) {
        self.package_info
            .insert(info.fully_qualified_name.clone(), info);
    }

    pub fn remove_package_info(&mut self, name: &str) {
        self.package_info.remove(name);
    }

    pub fn get_package_info(&self, name: &str) -> Option<&PackageInfo> {
        self.package_info.get(name)
    }

    pub fn get_package_info_mut(&mut self, name: &str) -> Option<&mut PackageInfo> {
        self.package_info.get_mut(name)
    }
}
#[derive(Debug)]
pub struct PackageInfo {
    pub(crate) fully_qualified_name: String,
    pub(crate) modules: IndexMap<String, ModuleInfo>,
    pub(crate) imports: IndexMap<String, ImportInfo>,
}

impl PackageInfo {
    pub fn new(fully_qualified_name: String) -> Self {
        Self {
            fully_qualified_name,
            modules: IndexMap::default(),
            imports: IndexMap::default(),
        }
    }

    pub fn add_import_info(&mut self, info: ImportInfo) {
        self.imports.insert(info.fully_qualified_name.clone(), info);
    }

    pub fn remove_import_info(&mut self, name: &str) {
        self.imports.remove(name);
    }

    pub fn get_import_info(&self, name: &str) -> Option<&ImportInfo> {
        self.imports.get(name)
    }

    pub fn add_module_info(&mut self, info: ModuleInfo) {
        self.modules.insert(info.filename.clone(), info);
    }

    pub fn remove_module_info(&mut self, name: &str) {
        self.modules.remove(name);
    }

    pub fn get_module_info(&self, name: &str) -> Option<&ModuleInfo> {
        self.modules.get(name)
    }
}
#[derive(Debug)]
pub struct ImportInfo {
    pub(crate) unqualified_name: String,
    pub(crate) fully_qualified_name: String,
}

impl ImportInfo {
    pub fn new(unqualified_name: String, fully_qualified_name: String) -> Self {
        Self {
            unqualified_name,
            fully_qualified_name,
        }
    }
}
#[derive(Debug)]
pub struct ModuleInfo {
    pub(crate) filename: String,
}

impl ModuleInfo {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}
