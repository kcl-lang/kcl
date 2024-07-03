use kclvm_sema::core::symbol::SymbolHint;
use kclvm_sema::core::{global_state::GlobalState, symbol::KCLSymbol};
use lsp_types::{InlayHint, InlayHintLabelPart, Position as LspPosition};
use std::convert::TryInto;

pub fn inlay_hints(file: &str, gs: &GlobalState) -> Option<Vec<InlayHint>> {
    let mut inlay_hints: Vec<InlayHint> = vec![];
    let sema_db = gs.get_sema_db();
    if let Some(file_sema) = sema_db.get_file_sema(file) {
        let symbols = file_sema.get_symbols();
        for symbol_ref in symbols {
            if let Some(symbol) = gs.get_symbols().get_symbol(*symbol_ref) {
                if let Some(hint) = symbol.get_hint() {
                    inlay_hints.push(generate_inlay_hint(symbol, hint));
                }
            }
        }
    }
    Some(inlay_hints)
}

#[inline]
fn generate_inlay_hint(symbol: &KCLSymbol, hint: &SymbolHint) -> InlayHint {
    let (part, position) = get_hint_label(symbol, &hint);
    InlayHint {
        position,
        label: lsp_types::InlayHintLabel::LabelParts(vec![part]),
        kind: None,
        text_edits: None,
        tooltip: None,
        padding_left: Some(true),
        padding_right: Some(true),
        data: None,
    }
}

fn get_hint_label(symbol: &KCLSymbol, hint: &SymbolHint) -> (InlayHintLabelPart, LspPosition) {
    let (start, end) = symbol.get_range();
    match hint {
        SymbolHint::TypeHint(ty) => (
            InlayHintLabelPart {
                value: format!(": {ty}"),
                ..Default::default()
            },
            LspPosition::new(
                (end.line - 1).try_into().unwrap(),
                end.column.unwrap_or(0).try_into().unwrap(),
            ),
        ),
        SymbolHint::VarHint(var) => (
            InlayHintLabelPart {
                value: format!("{var}: "),
                ..Default::default()
            },
            LspPosition::new(
                (start.line - 1).try_into().unwrap(),
                start.column.unwrap_or(0).try_into().unwrap(),
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::inlay_hints;
    use crate::tests::compile_test_file;

    #[macro_export]
    macro_rules! inlay_hints_test_snapshot {
        ($name:ident, $file:expr) => {
            #[test]
            fn $name() {
                let (file, _, _, gs) = compile_test_file($file);
                let res = inlay_hints(&file, &gs);
                insta::assert_snapshot!(format!("{:#?}", res));
            }
        };
    }

    inlay_hints_test_snapshot!(
        test_assign_stmt_type_hint,
        "src/test_data/inlay_hints/assign_stmt_type_hint/assign_stmt_type_hint.k"
    );
}
