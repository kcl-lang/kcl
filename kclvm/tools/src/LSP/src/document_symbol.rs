use std::path::Path;

use kclvm_ast::MAIN_PKG;
use kclvm_error::Position;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::core::symbol::KCLSymbol;
use kclvm_sema::core::symbol::SymbolKind as KCLSymbolKind;
use lsp_types::Range;
use lsp_types::{DocumentSymbol, DocumentSymbolResponse, SymbolKind};

use crate::to_lsp::lsp_pos;

pub(crate) fn document_symbol(
    file: &str,
    gs: &GlobalState,
) -> Option<lsp_types::DocumentSymbolResponse> {
    let mut document_symbols: Vec<DocumentSymbol> = vec![];

    let dummy_pos = Position {
        filename: file.to_string(),
        line: 1,
        column: Some(0),
    };
    if let Some(scope) = gs.get_scopes().get_root_scope(MAIN_PKG.to_owned()) {
        if let Some(defs) = gs.get_all_defs_in_scope(scope) {
            for symbol_ref in defs {
                match gs.get_symbols().get_symbol(symbol_ref) {
                    Some(symbol) => {
                        let def = symbol.get_definition();
                        match def {
                            Some(def) => {
                                let symbol_range = symbol.get_range();
                                // filter current file symbols
                                if let Ok(canonicalized_path) =
                                    Path::new(&symbol_range.0.filename).canonicalize()
                                {
                                    if canonicalized_path.eq(Path::new(file)) {
                                        match def.get_kind() {
                                            KCLSymbolKind::Schema => {
                                                match &mut symbol_to_document_symbol(symbol) {
                                                    Some(schema_symbol) => {
                                                        let module_info = gs
                                                            .get_packages()
                                                            .get_module_info(&dummy_pos.filename);
                                                        let attrs = symbol.get_all_attributes(
                                                            gs.get_symbols(),
                                                            module_info,
                                                        );
                                                        let mut children = vec![];

                                                        for attr in attrs {
                                                            match gs.get_symbols().get_symbol(attr)
                                                            {
                                                                Some(attr_symbol) => {
                                                                    match symbol_to_document_symbol(
                                                                        attr_symbol,
                                                                    ) {
                                                                        Some(symbol) => {
                                                                            children.push(symbol)
                                                                        }
                                                                        None => {}
                                                                    }
                                                                }
                                                                None => {}
                                                            }
                                                        }

                                                        schema_symbol.children = Some(children);
                                                        document_symbols
                                                            .push(schema_symbol.clone());
                                                    }
                                                    None => {}
                                                }
                                            }
                                            _ => {
                                                if let Some(symbol) =
                                                    symbol_to_document_symbol(symbol)
                                                {
                                                    document_symbols.push(symbol)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    None => {}
                }
            }
        }
    }
    Some(DocumentSymbolResponse::Nested(document_symbols))
}

fn symbol_to_document_symbol(symbol: &KCLSymbol) -> Option<DocumentSymbol> {
    let sema_info = symbol.get_sema_info();
    let def = symbol.get_definition();
    match def {
        Some(def) => {
            let name = symbol.get_name();
            let symbol_range = symbol.get_range();
            let range = Range {
                start: lsp_pos(&symbol_range.0),
                end: lsp_pos(&symbol_range.1),
            };
            let kind = def.get_kind();
            let kind = symbol_kind_to_document_symbol_kind(kind)?;
            let detail = sema_info.ty.clone().map(|ty| ty.ty_str());

            #[allow(deprecated)]
            Some(DocumentSymbol {
                name,
                kind,
                range,
                selection_range: range,
                detail,
                tags: None,
                children: None,
                deprecated: None,
            })
        }
        None => None,
    }
}

fn symbol_kind_to_document_symbol_kind(kind: KCLSymbolKind) -> Option<SymbolKind> {
    match kind {
        KCLSymbolKind::Schema => Some(SymbolKind::STRUCT),
        KCLSymbolKind::Attribute => Some(SymbolKind::PROPERTY),
        KCLSymbolKind::Value => Some(SymbolKind::VARIABLE),
        KCLSymbolKind::Package => Some(SymbolKind::PACKAGE),
        KCLSymbolKind::TypeAlias => Some(SymbolKind::TYPE_PARAMETER),
        KCLSymbolKind::Unresolved => Some(SymbolKind::NULL),
        KCLSymbolKind::Rule => Some(SymbolKind::FUNCTION),
        KCLSymbolKind::Expression => None,
        KCLSymbolKind::Comment => None,
        KCLSymbolKind::Decorator => None,
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::{DocumentSymbol, DocumentSymbolResponse, Position, Range, SymbolKind};
    use proc_macro_crate::bench_test;

    use crate::{document_symbol::document_symbol, tests::compile_test_file};

    #[allow(deprecated)]
    fn build_document_symbol(
        name: &str,
        kind: SymbolKind,
        range: ((u32, u32), (u32, u32)),
        child: Option<Vec<DocumentSymbol>>,
        detail: Option<String>,
    ) -> DocumentSymbol {
        let range: Range = Range {
            start: Position {
                line: range.0 .0,
                character: range.0 .1,
            },
            end: Position {
                line: range.1 .0,
                character: range.1 .1,
            },
        };
        DocumentSymbol {
            name: name.to_string(),
            detail,
            kind,
            tags: None,
            deprecated: None,
            range,
            selection_range: range,
            children: child,
        }
    }

    #[test]
    #[bench_test]
    fn document_symbol_test() {
        let (file, _, _, gs) = compile_test_file("src/test_data/document_symbol.k");

        let mut res = document_symbol(file.as_str(), &gs).unwrap();
        let mut expect = vec![];
        expect.push(build_document_symbol(
            "Person4",
            SymbolKind::STRUCT,
            ((0, 7), (0, 14)),
            Some(vec![build_document_symbol(
                "name",
                SymbolKind::PROPERTY,
                ((1, 4), (1, 8)),
                None,
                Some("str".to_string()),
            )]),
            Some("Person4".to_string()),
        ));
        expect.push(build_document_symbol(
            "p",
            SymbolKind::VARIABLE,
            ((3, 0), (3, 1)),
            None,
            Some("Person4".to_string()),
        ));

        match &mut res {
            DocumentSymbolResponse::Flat(_) => panic!("test failed"),
            DocumentSymbolResponse::Nested(got) => {
                got.sort_by(|a, b| a.name.cmp(&b.name));
                assert_eq!(got, &expect)
            }
        }
    }
}
