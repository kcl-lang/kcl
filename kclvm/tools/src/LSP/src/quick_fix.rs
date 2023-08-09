use std::collections::HashMap;

use kclvm_error::{DiagnosticId, WarningKind};
use lsp_types::{CodeAction, CodeActionKind, CodeActionOrCommand, NumberOrString, TextEdit};

pub(crate) fn quick_fix(
    params: lsp_types::CodeActionParams,
) -> Vec<lsp_types::CodeActionOrCommand> {
    let diags = params.context.diagnostics;
    let mut code_actions: Vec<lsp_types::CodeActionOrCommand> = vec![];
    for diag in diags {
        if let Some(code) = &diag.code {
            if let Some(id) = conver_code_to_kcl_diag_id(code) {
                match id {
                    DiagnosticId::Error(_) => continue,
                    DiagnosticId::Warning(warn) => match warn {
                        WarningKind::UnusedImportWarning => {
                            let mut changes = HashMap::new();
                            changes.insert(
                                params.text_document.uri.clone(),
                                vec![TextEdit {
                                    range: diag.range,
                                    new_text: "".to_string(),
                                }],
                            );
                            code_actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: WarningKind::UnusedImportWarning.name(),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diag.clone()]),
                                edit: Some(lsp_types::WorkspaceEdit {
                                    changes: Some(changes),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }))
                        }
                        WarningKind::ReimportWarning => {
                            let mut changes = HashMap::new();
                            changes.insert(
                                params.text_document.uri.clone(),
                                vec![TextEdit {
                                    range: diag.range,
                                    new_text: "".to_string(),
                                }],
                            );
                            code_actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: WarningKind::ReimportWarning.name(),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diag.clone()]),
                                edit: Some(lsp_types::WorkspaceEdit {
                                    changes: Some(changes),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }))
                        }
                        _ => continue,
                    },
                }
            }
        }
    }
    code_actions
}

pub(crate) fn conver_code_to_kcl_diag_id(code: &NumberOrString) -> Option<DiagnosticId> {
    match code {
        NumberOrString::Number(_) => None,
        NumberOrString::String(code) => match code.as_str() {
            "CompilerWarning" => Some(DiagnosticId::Warning(WarningKind::CompilerWarning)),
            "UnusedImportWarning" => Some(DiagnosticId::Warning(WarningKind::UnusedImportWarning)),
            "ReimportWarning" => Some(DiagnosticId::Warning(WarningKind::ReimportWarning)),
            "ImportPositionWarning" => {
                Some(DiagnosticId::Warning(WarningKind::ImportPositionWarning))
            }
            _ => None,
        },
    }
}
