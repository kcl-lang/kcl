use indexmap::IndexSet;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::core::symbol::{SymbolHint, SymbolHintKind};
use lsp_types::{InlayHint, InlayHintLabelPart, Position as LspPosition};
use std::hash::Hash;

use crate::to_lsp::lsp_pos;

#[derive(Clone, Debug)]
struct KCLInlayHint {
    /// The position of this hint.
    pub position: LspPosition,

    /// An inlay hint label part allows for interactive and composite labels
    /// of inlay hints.
    pub part: InlayHintLabelPart,
}

impl Hash for KCLInlayHint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.line.hash(state);
        self.position.character.hash(state);
        self.part.value.hash(state);
    }
}

impl PartialEq for KCLInlayHint {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.part.value == other.part.value
    }
}

impl Eq for KCLInlayHint {}

pub fn inlay_hints(file: &str, gs: &GlobalState) -> Option<Vec<InlayHint>> {
    let mut inlay_hints: IndexSet<KCLInlayHint> = Default::default();
    let sema_db = gs.get_sema_db();
    if let Some(file_sema) = sema_db.get_file_sema(file) {
        let symbols = file_sema.get_symbols();
        for symbol_ref in symbols {
            if let Some(symbol) = gs.get_symbols().get_symbol(*symbol_ref) {
                if let Some(hint) = symbol.get_hint() {
                    inlay_hints.insert(generate_inlay_hint(hint));
                }
            }
        }
    }
    Some(
        inlay_hints
            .into_iter()
            .map(|h| into_lsp_inlay_hint(&h))
            .collect(),
    )
}

#[inline]
fn generate_inlay_hint(hint: &SymbolHint) -> KCLInlayHint {
    let (part, position) = get_hint_label(hint);
    KCLInlayHint { position, part }
}

#[inline]
fn into_lsp_inlay_hint(hint: &KCLInlayHint) -> InlayHint {
    InlayHint {
        position: hint.position,
        label: lsp_types::InlayHintLabel::LabelParts(vec![hint.part.clone()]),
        kind: None,
        text_edits: None,
        tooltip: None,
        padding_left: None,
        padding_right: None,
        data: None,
    }
}

fn get_hint_label(hint: &SymbolHint) -> (InlayHintLabelPart, LspPosition) {
    match &hint.kind {
        SymbolHintKind::TypeHint(ty) => (
            InlayHintLabelPart {
                value: format!(": {ty}"),
                ..Default::default()
            },
            lsp_pos(&hint.pos),
        ),
        SymbolHintKind::VarHint(var) => (
            InlayHintLabelPart {
                value: format!("{var}: "),
                ..Default::default()
            },
            lsp_pos(&hint.pos),
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

    inlay_hints_test_snapshot!(
        test_function_call_arg_hint,
        "src/test_data/inlay_hints/function_call/function_call.k"
    );

    inlay_hints_test_snapshot!(
        test_schema_arg_hint,
        "src/test_data/inlay_hints/schema_args/schema_args_hint.k"
    );

    inlay_hints_test_snapshot!(
        test_config_key_ty,
        "src/test_data/inlay_hints/config_key/config_key.k"
    );
}
