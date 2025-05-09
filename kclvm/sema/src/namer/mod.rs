/*
  ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
  │                                              namer                                              │
  ├─────────────────────────────────────────────────────────────────────────────────────────────────┤
  │       ┌─────────────────┐             ┌─────────────────┐             ┌─────────────────┐       │
  │       │ ast::Expression │             │ ast::Expression │             │ ast::Expression │       │
  │       └─────────────────┘             └─────────────────┘             └─────────────────┘       │
  │                │                               │                               │                │
  │                │ find_symbols                  │ find_symbols                  │ find_symbols   │
  │                ▼                               ▼                               ▼                │
  │   ┌─────────────────────────┐     ┌─────────────────────────┐     ┌─────────────────────────┐   │
  │   │      core::SymbolRef    │     │     core::SymbolRef     │     │     core::SymbolRef     │   │
  │   └─────────────────────────┘     └─────────────────────────┘     └─────────────────────────┘   │
  │                │                               │                               │                │
  │                │                               │                               │                │
  │                └───────────────────────────────┼───────────────────────────────┘                │
  │                                                │                                                │
  │                                                │ merge findSymbols results                      │
  │                                                ▼                                                │
  │                                   ┌─────────────────────────┐                                   │
  │                                   │     core::SymbolRef     │                                   │
  │                                   └─────────────────────────┘                                   │
  │                                                │                                                │
  │                                                │ define_symbols(FQN)                            │
  │                                                ■                                                │
  │                                                  (mutates GlobalState)                          |
  │                                                                                                 │
  └─────────────────────────────────────────────────────────────────────────────────────────────────┘

   The early stage of the namer will be based on file level , which collects global symbols defined in the file,
   and then merges the symbols based on FQN to obtain a unique GlobalState

   Based on file level, it means that we can easily perform incremental compilation in the future

   Now we just run namer pass serially

*/

use std::path::Path;
use std::sync::Arc;

use crate::builtin::{
    get_system_member_function_ty, get_system_module_members, BUILTIN_FUNCTIONS,
    STANDARD_SYSTEM_MODULES, STRING_MEMBER_FUNCTIONS,
};
use crate::core::global_state::GlobalState;
use crate::core::package::{ModuleInfo, PackageInfo};
use crate::core::symbol::{
    FunctionSymbol, PackageSymbol, SymbolRef, BUILTIN_FUNCTION_PACKAGE, BUILTIN_STR_PACKAGE,
};
use crate::resolver::scope::NodeKey;
use kclvm_ast::ast::AstIndex;
use kclvm_ast::ast::Program;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::Position;
use kclvm_primitives::IndexSet;
mod node;

pub const BUILTIN_SYMBOL_PKG_PATH: &str = "@builtin";

pub struct Namer<'ctx> {
    gs: &'ctx mut GlobalState,
    ctx: NamerContext<'ctx>,
}

struct NamerContext<'ctx> {
    pub program: &'ctx Program,
    pub current_package_info: Option<PackageInfo>,
    pub current_module_info: Option<ModuleInfo>,
    pub owner_symbols: Vec<SymbolRef>,
    pub value_fully_qualified_name_set: IndexSet<String>,
}

impl<'ctx> NamerContext<'ctx> {
    pub fn get_node_key(&self, id: &AstIndex) -> NodeKey {
        NodeKey {
            pkgpath: self
                .current_package_info
                .clone()
                .unwrap()
                .fully_qualified_name,
            id: id.clone(),
        }
    }
}

impl<'ctx> Namer<'ctx> {
    fn new(program: &'ctx Program, gs: &'ctx mut GlobalState) -> Self {
        Self {
            ctx: NamerContext {
                program,
                current_package_info: None,
                current_module_info: None,
                owner_symbols: Vec::default(),
                value_fully_qualified_name_set: IndexSet::default(),
            },
            gs,
        }
    }

    // serial namer pass
    pub fn find_symbols(program: &'ctx Program, gs: &'ctx mut GlobalState) {
        let has_init_builtin = gs.ctx.has_init_builtin;
        gs.ctx.has_init_builtin = true;
        let mut namer = Self::new(program, gs);
        namer.ctx.current_package_info = Some(PackageInfo::new(
            BUILTIN_SYMBOL_PKG_PATH.to_string(),
            "".to_string(),
            true,
        ));
        if !has_init_builtin {
            namer.init_builtin_symbols();
        }

        namer
            .gs
            .get_packages_mut()
            .add_package(namer.ctx.current_package_info.take().unwrap());

        for (name, modules) in namer.ctx.program.pkgs.iter() {
            namer.walk_pkg(name, modules);
        }

        namer.define_symbols();
    }

    fn walk_pkg(&mut self, name: &String, modules: &Vec<String>) {
        // new pkgs or invalidate pkg
        if self.gs.get_packages().get_package_info(name).is_some()
            && !self.gs.new_or_invalidate_pkgs.contains(name)
        {
            return;
        }

        // add new pkgs to invalidate pkgs
        self.gs.new_or_invalidate_pkgs.insert(name.clone());

        {
            if modules.is_empty() {
                return;
            }
            self.ctx.value_fully_qualified_name_set.clear();
            let mut real_path = Path::new(&self.ctx.program.root)
                .join(name.replace('.', &std::path::MAIN_SEPARATOR.to_string()))
                .to_str()
                .unwrap()
                .to_string();
            if name == kclvm_ast::MAIN_PKG {
                real_path = self.ctx.program.root.clone()
            }
            let pkg_pos = Position {
                filename: real_path.clone(),
                line: 0,
                column: None,
            };

            let pkg_symbol = PackageSymbol::new(name.clone(), pkg_pos.clone(), pkg_pos);
            let symbol_ref = self
                .gs
                .get_symbols_mut()
                .alloc_package_symbol(pkg_symbol, name.to_string());
            self.ctx.owner_symbols.push(symbol_ref);

            self.ctx.current_package_info =
                Some(PackageInfo::new(name.to_string(), real_path, false));
        }

        let modules = self.ctx.program.get_modules_for_pkg(name);
        for module in modules.iter() {
            let module = module.read().expect("Failed to acquire module lock");
            self.ctx
                .current_package_info
                .as_mut()
                .unwrap()
                .kfile_paths
                .insert(module.filename.clone());
            self.ctx.current_module_info =
                Some(ModuleInfo::new(module.filename.clone(), name.to_string()));
            self.walk_module(&module);
            self.gs
                .get_packages_mut()
                .add_module_info(self.ctx.current_module_info.take().unwrap());
        }

        self.ctx.owner_symbols.pop();
        self.gs
            .get_packages_mut()
            .add_package(self.ctx.current_package_info.take().unwrap())
    }

    fn init_builtin_symbols(&mut self) {
        //add global built functions
        for (name, builtin_func) in BUILTIN_FUNCTIONS.iter() {
            let mut func_symbol = FunctionSymbol::new(
                name.to_string(),
                Position::dummy_pos(),
                Position::dummy_pos(),
                None,
                true,
            );

            func_symbol.sema_info.ty = Some(Arc::new(builtin_func.clone()));
            func_symbol.sema_info.doc = builtin_func.ty_doc();
            let symbol_ref = self.gs.get_symbols_mut().alloc_function_symbol(
                func_symbol,
                self.ctx.get_node_key(&AstIndex::default()),
                BUILTIN_FUNCTION_PACKAGE.to_string(),
            );
            self.gs
                .get_symbols_mut()
                .symbols_info
                .global_builtin_symbols
                .insert(name.to_string(), symbol_ref);
        }

        //add system modules
        for system_pkg_name in STANDARD_SYSTEM_MODULES {
            let package_symbol_ref = self.gs.get_symbols_mut().alloc_package_symbol(
                PackageSymbol::new(
                    system_pkg_name.to_string(),
                    Position::dummy_pos(),
                    Position::dummy_pos(),
                ),
                system_pkg_name.to_string(),
            );
            for func_name in get_system_module_members(system_pkg_name) {
                let func_ty = get_system_member_function_ty(*system_pkg_name, func_name);
                let mut func_symbol = FunctionSymbol::new(
                    func_name.to_string(),
                    Position::dummy_pos(),
                    Position::dummy_pos(),
                    Some(package_symbol_ref),
                    false,
                );

                func_symbol.sema_info.ty = Some(func_ty.clone());
                func_symbol.sema_info.doc = func_ty.ty_doc();
                let func_symbol_ref = self.gs.get_symbols_mut().alloc_function_symbol(
                    func_symbol,
                    self.ctx.get_node_key(&AstIndex::default()),
                    system_pkg_name.to_string(),
                );
                self.gs
                    .get_symbols_mut()
                    .packages
                    .get_mut(package_symbol_ref.get_id())
                    .unwrap()
                    .members
                    .insert(func_name.to_string(), func_symbol_ref);
            }
        }

        //add string builtin function
        let package_symbol_ref = self.gs.get_symbols_mut().alloc_package_symbol(
            PackageSymbol::new(
                BUILTIN_STR_PACKAGE.to_string(),
                Position::dummy_pos(),
                Position::dummy_pos(),
            ),
            BUILTIN_STR_PACKAGE.to_string(),
        );
        for (name, builtin_func) in STRING_MEMBER_FUNCTIONS.iter() {
            let mut func_symbol = FunctionSymbol::new(
                name.to_string(),
                Position::dummy_pos(),
                Position::dummy_pos(),
                Some(package_symbol_ref),
                true,
            );

            func_symbol.sema_info.ty = Some(Arc::new(builtin_func.clone()));
            func_symbol.sema_info.doc = builtin_func.ty_doc();
            let symbol_ref = self.gs.get_symbols_mut().alloc_function_symbol(
                func_symbol,
                self.ctx.get_node_key(&AstIndex::default()),
                BUILTIN_STR_PACKAGE.to_string(),
            );
            self.gs
                .get_symbols_mut()
                .packages
                .get_mut(package_symbol_ref.get_id())
                .unwrap()
                .members
                .insert(name.to_string(), symbol_ref);
        }
    }

    fn define_symbols(&mut self) {
        self.gs.get_symbols_mut().build_fully_qualified_name_map();
    }
}

#[cfg(test)]
mod tests {
    use super::Namer;
    use crate::core::global_state::GlobalState;
    use crate::core::symbol::SymbolKind;
    use kclvm_parser::load_program;
    use kclvm_parser::ParseSession;
    use std::sync::Arc;

    #[test]
    fn test_find_symbols() {
        let sess = Arc::new(ParseSession::default());
        let program = load_program(
            sess.clone(),
            &["./src/namer/test_data/schema_symbols.k"],
            None,
            None,
        )
        .unwrap()
        .program;
        let mut gs = GlobalState::default();
        Namer::find_symbols(&program, &mut gs);

        let symbols = gs.get_symbols();

        let excepts_symbols = vec![
            // package
            ("import_test.a", SymbolKind::Package),
            ("import_test.b", SymbolKind::Package),
            ("import_test.c", SymbolKind::Package),
            ("import_test.d", SymbolKind::Package),
            ("import_test.e", SymbolKind::Package),
            ("import_test.f", SymbolKind::Package),
            ("__main__", SymbolKind::Package),
            ("pkg", SymbolKind::Package),
            // schema
            ("import_test.f.UnionType", SymbolKind::Schema),
            ("import_test.a.Person", SymbolKind::Schema),
            ("import_test.c.TestOfMixin", SymbolKind::Schema),
            ("import_test.d.Parent", SymbolKind::Schema),
            ("import_test.e.UnionType", SymbolKind::Schema),
            ("pkg.Name", SymbolKind::Schema),
            ("pkg.Person", SymbolKind::Schema),
            ("__main__.Main", SymbolKind::Schema),
            // attribute
            ("import_test.f.UnionType.b", SymbolKind::Attribute),
            ("import_test.a.Person.name", SymbolKind::Attribute),
            ("import_test.a.Person.age", SymbolKind::Attribute),
            ("pkg.Name.name", SymbolKind::Attribute),
            ("pkg.Person.name", SymbolKind::Attribute),
            ("import_test.c.TestOfMixin.age", SymbolKind::Attribute),
            ("import_test.d.Parent.age1", SymbolKind::Attribute),
            ("import_test.e.UnionType.a", SymbolKind::Attribute),
            ("__main__.Main.name", SymbolKind::Attribute),
            ("__main__.Main.age", SymbolKind::Attribute),
            ("__main__.Main.person", SymbolKind::Attribute),
            ("__main__.Main.list_union_type", SymbolKind::Attribute),
            ("__main__.Main.dict_union_type", SymbolKind::Attribute),
            // value
            ("__main__.p", SymbolKind::Value),
            ("__main__.person", SymbolKind::Value),
            ("__main__._c", SymbolKind::Value),
            ("import_test.a._a", SymbolKind::Value),
            ("import_test.b._b", SymbolKind::Value),
        ];

        for (fqn, kind) in excepts_symbols {
            assert!(symbols
                .symbols_info
                .fully_qualified_name_map
                .contains_key(fqn));
            assert_eq!(
                symbols
                    .get_symbol_by_fully_qualified_name(fqn)
                    .unwrap()
                    .get_kind(),
                kind
            );
        }
    }
}
