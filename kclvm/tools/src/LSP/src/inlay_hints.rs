use kclvm_ast::ast::{self, Program};
use kclvm_ast::pos::GetPos;
use kclvm_error::Position as KCLPos;
use kclvm_sema::core::{global_state::GlobalState, symbol::KCLSymbol};
use kclvm_sema::ty::TypeKind;
use lsp_types::{InlayHint, InlayHintLabelPart, Position as LspPosition, Range};
use std::convert::TryInto;

pub fn inlay_hints(file: &str, gs: &GlobalState, program: &Program) -> Option<Vec<InlayHint>> {
    let mut inlay_hints: Vec<InlayHint> = vec![];
    let sema_db = gs.get_sema_db();
    if let Some(file_sema) = sema_db.get_file_sema(file) {
        let symbols = file_sema.get_symbols();
        for symbol_ref in symbols {
            if let Some(symbol) = gs.get_symbols().get_symbol(*symbol_ref) {
                let (start, end) = symbol.get_range();
                if has_type_assignment(program, &start) {
                    if let Some(hint) = generate_inlay_hint(symbol, gs, &start, &end) {
                        inlay_hints.push(hint);
                    }
                }
            }
        }
    }
    Some(inlay_hints)
}

fn has_type_assignment(program: &Program, start: &KCLPos) -> bool {
    if let Some(stmt_node) = program.pos_to_stmt(start) {
        if let ast::Stmt::Assign(assign_stmt) = stmt_node.node {
            if assign_stmt
                .targets
                .iter()
                .any(|target| target.get_pos() == *start)
                && assign_stmt.ty.is_none()
            {
                return true;
            }
        }
    }
    false
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
            TypeKind::Str | TypeKind::Bool | TypeKind::Int | TypeKind::Float | TypeKind::Any => {
                label_parts.push(InlayHintLabelPart {
                    value: format!("[: {}]", ty.ty_str()),
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
