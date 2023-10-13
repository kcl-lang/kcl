use crate::core::global_state::GlobalState;
use crate::core::package::PackageInfo;
use crate::core::symbol::{PackageSymbol, SymbolRef};
use kclvm_ast::ast::Program;
use kclvm_ast::walker::MutSelfTypedResultWalker;
use kclvm_error::Position;
mod node;

struct Namer<'ctx> {
    gs: GlobalState,
    ctx: NamerContext<'ctx>,
}

struct NamerContext<'ctx> {
    pub program: &'ctx Program,
    pub current_package_info: Option<PackageInfo>,
    pub owner_symbols: Vec<SymbolRef>,
}

impl<'ctx> Namer<'ctx> {
    fn new(program: &'ctx Program, gs: GlobalState) -> Self {
        Self {
            ctx: NamerContext {
                program,
                current_package_info: None,
                owner_symbols: Vec::default(),
            },
            gs,
        }
    }

    // serial namer pass
    pub fn find_symbols(program: &'ctx Program, gs: GlobalState) -> GlobalState {
        let mut namer = Self::new(program, gs);

        for (name, modules) in namer.ctx.program.pkgs.iter() {
            {
                if modules.is_empty() {
                    continue;
                }

                let pkg_pos = Position {
                    filename: modules.last().unwrap().filename.clone(),
                    line: 1,
                    column: None,
                };

                let pkg_symbol = PackageSymbol::new(name.clone(), pkg_pos.clone(), pkg_pos);
                let symbol_ref = namer.gs.get_symbols_mut().alloc_package_symbol(pkg_symbol);
                namer.ctx.owner_symbols.push(symbol_ref);

                namer.ctx.current_package_info = Some(PackageInfo::new(name.to_string()));
            }

            for module in modules.iter() {
                namer.walk_module(module);
            }

            namer.ctx.owner_symbols.pop();
            namer
                .gs
                .get_packages_mut()
                .add_package(namer.ctx.current_package_info.take().unwrap())
        }

        namer.define_symbols();

        namer.gs
    }

    pub fn define_symbols(&mut self) {
        self.gs.get_symbols_mut().build_fully_qualified_name_map();

        self.gs.resolve_symbols()
    }
}

#[cfg(test)]
mod tests {
    use super::Namer;
    use crate::core::global_state::GlobalState;
    use crate::core::symbol::SymbolKind;
    use kclvm_parser::ParseSession;
    use kclvm_parser::{load_program, parse_program};
    use std::sync::Arc;

    #[test]
    fn test_find_symbols() {
        let sess = Arc::new(ParseSession::default());
        let program = load_program(
            sess.clone(),
            &["./src/namer/test_data/schema_symbols.k"],
            None,
        )
        .unwrap();
        let gs = GlobalState::default();
        let gs = Namer::find_symbols(&program, gs);

        let symbols = gs.get_symbols();

        let excepts_symbols = vec![
            ("import_test.a", SymbolKind::Package),
            ("import_test.b", SymbolKind::Package),
            ("import_test.c", SymbolKind::Package),
            ("import_test.d", SymbolKind::Package),
            ("import_test.e", SymbolKind::Package),
            ("import_test.f", SymbolKind::Package),
            ("__main__", SymbolKind::Package),
            ("pkg", SymbolKind::Package),
            ("import_test.f.UnionType", SymbolKind::Schema),
            ("import_test.a.Person", SymbolKind::Schema),
            ("import_test.c.TestOfMixin", SymbolKind::Schema),
            ("import_test.d.Parent", SymbolKind::Schema),
            ("import_test.e.UnionType", SymbolKind::Schema),
            ("pkg.Name", SymbolKind::Schema),
            ("pkg.Person", SymbolKind::Schema),
            ("__main__.Main", SymbolKind::Schema),
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
        ];

        assert_eq!(
            symbols.fully_qualified_name_map.len(),
            excepts_symbols.len()
        );

        for (fqn, kind) in excepts_symbols {
            assert!(symbols.fully_qualified_name_map.contains_key(fqn));
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
