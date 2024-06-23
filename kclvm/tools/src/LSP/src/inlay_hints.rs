use kclvm_error::Position as KCLPos;
use kclvm_sema::core::{global_state::GlobalState, symbol::KCLSymbol};
use kclvm_sema::ty::TypeKind;
use lsp_types::{InlayHint, InlayHintLabelPart, Position as LspPosition, Range};
use std::convert::TryInto;

pub fn inlay_hints(file: &str, gs: &GlobalState) -> Option<Vec<InlayHint>> {
    let mut inlay_hints: Vec<InlayHint> = vec![];
    let sema_db = gs.get_sema_db();
    if let Some(file_sema) = sema_db.get_file_sema(file) {
        let symbols = file_sema.get_symbols();
        for symbol_ref in symbols {
            if let Some(symbol) = gs.get_symbols().get_symbol(*symbol_ref) {
                let (start, end) = symbol.get_range();
                if let Some(hint) = generate_inlay_hint(symbol, gs, &start, &end) {
                    inlay_hints.push(hint);
                }
            }
        }
    }
    Some(inlay_hints)
}

fn generate_inlay_hint(
    symbol: &KCLSymbol,
    gs: &GlobalState,
    start: &KCLPos,
    end: &KCLPos,
) -> Option<InlayHint> {
    match get_hint_label(symbol, gs) {
        Some(label_parts) => {
            let range = Range {
                start: LspPosition::new(
                    (start.line - 1).try_into().unwrap(),
                    start.column.unwrap_or(0).try_into().unwrap(),
                ),
                end: LspPosition::new(
                    (end.line - 1).try_into().unwrap(),
                    end.column.unwrap_or(0).try_into().unwrap(),
                ),
            };
            Some(InlayHint {
                position: range.end,
                label: lsp_types::InlayHintLabel::LabelParts(label_parts),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: Some(true),
                padding_right: Some(true),
                data: None,
            })
        }
        None => None,
    }
}

fn get_hint_label(symbol: &KCLSymbol, _gs: &GlobalState) -> Option<Vec<InlayHintLabelPart>> {
    if let Some(ty) = &symbol.get_sema_info().ty {
        let mut label_parts = Vec::new();

        match &ty.kind {
            TypeKind::Str
            | TypeKind::Bool
            | TypeKind::Int
            | TypeKind::Float
            | TypeKind::Any
            | TypeKind::None
            | TypeKind::Named(_)
            | TypeKind::NumberMultiplier(_)
            | TypeKind::Union(_)
            | TypeKind::Dict(_)
            | TypeKind::List(_) => {
                label_parts.push(InlayHintLabelPart {
                    value: format!(": {}", ty.ty_str()),
                    ..Default::default()
                });
            }
            TypeKind::Module(module_ty) => {
                label_parts.push(InlayHintLabelPart {
                    value: format!(": {}", module_ty.pkgpath),
                    ..Default::default()
                });
            }
            TypeKind::Function(_) => {
                let symbol_name = symbol.get_name().to_string();
                label_parts.push(InlayHintLabelPart {
                    value: format!("fn {}", symbol_name),
                    ..Default::default()
                });
            }
            TypeKind::Schema(schema_ty) => {
                let fully_qualified_ty_name = format!("schema {}", schema_ty.name);
                label_parts.push(InlayHintLabelPart {
                    value: format!(": {}", fully_qualified_ty_name),
                    ..Default::default()
                });
            }
            _ => {
                return None;
            }
        }
        Some(label_parts)
    } else {
        None
    }
}
