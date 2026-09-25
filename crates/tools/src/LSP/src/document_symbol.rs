use kcl_ast::MAIN_PKG;
use kcl_error::Position;
use kcl_sema::core::global_state::GlobalState;
use kcl_sema::core::symbol::KCLSymbol;
use kcl_sema::core::symbol::SymbolKind as KCLSymbolKind;
use lsp_types::Range;
use lsp_types::{DocumentSymbol, DocumentSymbolResponse, SymbolKind};

use crate::to_lsp::lsp_pos;

pub fn document_symbol(file: &str, gs: &GlobalState) -> Option<lsp_types::DocumentSymbolResponse> {
    let mut document_symbols: Vec<DocumentSymbol> = vec![];

    let dummy_pos = Position {
        filename: file.to_string(),
        line: 1,
        column: Some(0),
    };
    if let Some(scope) = gs.get_scopes().get_root_scope(MAIN_PKG.to_owned())
        && let Some(defs) = gs.get_all_defs_in_scope(scope, &dummy_pos)
    {
        for symbol_ref in defs {
            if let Some(symbol) = gs.get_symbols().get_symbol(symbol_ref) {
                let def = symbol.get_definition();
                if let Some(def) = def {
                    let symbol_range = symbol.get_range();
                    // filter current file symbols
                    if symbol_range.0.filename == file {
                        match def.get_kind() {
                            KCLSymbolKind::Schema => {
                                if let Some(schema_symbol) = &mut symbol_to_document_symbol(symbol)
                                {
                                    let module_info =
                                        gs.get_packages().get_module_info(&dummy_pos.filename);
                                    let attrs =
                                        symbol.get_all_attributes(gs.get_symbols(), module_info);
                                    let mut children = vec![];

                                    for attr in attrs {
                                        if let Some(attr_symbol) = gs.get_symbols().get_symbol(attr)
                                            && let Some(symbol) =
                                                symbol_to_document_symbol(attr_symbol)
                                        {
                                            children.push(symbol)
                                        }
                                    }

                                    schema_symbol.children = Some(children);
                                    schema_symbol.name = format!("schema {}", schema_symbol.name);
                                    document_symbols.push(schema_symbol.clone());
                                }
                            }
                            _ => {
                                if let Some(symbol) = symbol_to_document_symbol(symbol) {
                                    document_symbols.push(symbol)
                                }
                            }
                        }
                    }
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
        KCLSymbolKind::Function => Some(SymbolKind::FUNCTION),
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
    use lsp_types::DocumentSymbolResponse;

    use crate::{document_symbol::document_symbol, tests::compile_test_file};

    /// Snapshot helper for `document_symbol` tests. Compiles the fixture,
    /// runs the function, normalises the response so `DocumentSymbol`s are
    /// sorted by name (the order of the result is not semantically meaningful),
    /// and asserts against an `insta` snapshot. The macro mirrors the
    /// `*_test_snapshot!` pattern used by `goto_def`, `hover`, etc. so future
    /// tests can be added with a single invocation.
    #[macro_export]
    macro_rules! document_symbol_test_snapshot {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                let (file, _program, _, gs, _) = compile_test_file($file);
                let mut res = document_symbol(file.as_str(), &gs).unwrap();
                // Symbol order is not part of the contract, so sort by name to
                // make the snapshot deterministic.
                let normalised = match &mut res {
                    DocumentSymbolResponse::Flat(_) => panic!("unexpected flat response"),
                    DocumentSymbolResponse::Nested(got) => {
                        got.sort_by(|a, b| a.name.cmp(&b.name));
                        for s in got.iter_mut() {
                            if let Some(children) = s.children.as_mut() {
                                children.sort_by(|a, b| a.name.cmp(&b.name));
                            }
                        }
                        got.clone()
                    }
                };
                insta::assert_snapshot!(format!("{:#?}", normalised));
            }
        };
    }

    document_symbol_test_snapshot!(
        document_symbol_test,
        "src/test_data/document_symbol/document_symbol.k"
    );
}
