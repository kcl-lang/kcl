use indexmap::IndexSet;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::core::symbol::{SymbolHint, SymbolHintKind};
use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabelPart, Position as LspPosition, Range, TextEdit,
};
use std::collections::HashMap;
use std::hash::Hash;

use crate::to_lsp::lsp_pos;

#[derive(Clone, Debug)]
struct KCLInlayHint {
    /// The position of this hint.
    pub position: LspPosition,

    /// An inlay hint label part allows for interactive and composite labels
    /// of inlay hints.
    pub part: InlayHintLabelPart,

    pub kind: InlayHintKind,

    /// Optional text edits that are performed when accepting(e.g. double-click in VSCode) this inlay hint.
    pub text_edits: Option<Vec<TextEdit>>,
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
        let mut line_hint: HashMap<u64, SymbolHint> = HashMap::new();
        for hint in file_sema.get_hints() {
            match &hint.kind {
                SymbolHintKind::KeyTypeHint(_) => match line_hint.get(&hint.pos.line) {
                    Some(h) => {
                        if hint.pos.column.unwrap_or_default() > h.pos.column.unwrap_or_default() {
                            line_hint.insert(hint.pos.line, hint.clone());
                        }
                    }
                    None => {
                        line_hint.insert(hint.pos.line, hint.clone());
                    }
                },
                _ => {
                    inlay_hints.insert(generate_inlay_hint(hint));
                }
            }
        }
        inlay_hints.extend(line_hint.values().map(|h| generate_inlay_hint(h)));
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
    let (part, position, kind) = get_hint_label(hint);
    let text_edits = match hint.kind {
        SymbolHintKind::TypeHint(_) => Some(vec![TextEdit {
            range: Range {
                start: position.clone(),
                end: position.clone(),
            },
            new_text: part.value.clone(),
        }]),
        SymbolHintKind::VarHint(_) => None,
        SymbolHintKind::KeyTypeHint(_) => None,
    };
    KCLInlayHint {
        position,
        part,
        kind,
        text_edits,
    }
}

#[inline]
fn into_lsp_inlay_hint(hint: &KCLInlayHint) -> InlayHint {
    InlayHint {
        position: hint.position.clone(),
        label: lsp_types::InlayHintLabel::LabelParts(vec![hint.part.clone()]),
        kind: Some(hint.kind),
        text_edits: hint.text_edits.clone(),
        tooltip: None,
        padding_left: None,
        padding_right: None,
        data: None,
    }
}

fn get_hint_label(hint: &SymbolHint) -> (InlayHintLabelPart, LspPosition, InlayHintKind) {
    match &hint.kind {
        SymbolHintKind::TypeHint(ty) => (
            InlayHintLabelPart {
                value: format!(": {ty}"),
                ..Default::default()
            },
            lsp_pos(&hint.pos),
            InlayHintKind::TYPE,
        ),
        SymbolHintKind::VarHint(var) => (
            InlayHintLabelPart {
                value: format!("{var}: "),
                ..Default::default()
            },
            lsp_pos(&hint.pos),
            InlayHintKind::PARAMETER,
        ),
        SymbolHintKind::KeyTypeHint(ty) => (
            InlayHintLabelPart {
                value: format!(": {ty}"),
                ..Default::default()
            },
            lsp_pos(&hint.pos),
            InlayHintKind::TYPE,
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

    inlay_hints_test_snapshot!(
        test_config_key_ty_1,
        "src/test_data/inlay_hints/config_key1/config_key.k"
    );
}
