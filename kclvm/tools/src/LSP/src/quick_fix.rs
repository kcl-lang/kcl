use std::collections::HashMap;

use kclvm_error::{DiagnosticId, WarningKind};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, NumberOrString, TextEdit, Url,
};

pub(crate) fn quick_fix(uri: &Url, diags: &Vec<Diagnostic>) -> Vec<lsp_types::CodeActionOrCommand> {
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
                                uri.clone(),
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
                                uri.clone(),
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
                    DiagnosticId::Suggestions => continue,
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

#[cfg(test)]
mod tests {

    use lsp_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Position, Range, TextEdit,
        Url, WorkspaceEdit,
    };
    use proc_macro_crate::bench_test;
    use std::path::PathBuf;

    use super::quick_fix;
    use crate::{
        to_lsp::kcl_diag_to_lsp_diags,
        util::{parse_param_and_compile, Param},
    };
    use parking_lot::RwLock;
    use std::sync::Arc;

    #[test]
    #[bench_test]
    fn quick_fix_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut test_file = path.clone();
        test_file.push("src/test_data/quick_fix.k");
        let file = test_file.to_str().unwrap();

        let (_, _, diags, _) = parse_param_and_compile(
            Param {
                file: file.to_string(),
                module_cache: None,
            },
            Some(Arc::new(RwLock::new(Default::default()))),
        )
        .unwrap();

        let diagnostics = diags
            .iter()
            .flat_map(|diag| kcl_diag_to_lsp_diags(diag, file))
            .collect::<Vec<Diagnostic>>();

        let uri = Url::from_file_path(file).unwrap();
        let code_actions = quick_fix(&uri, &diagnostics);

        let expected = vec![
            CodeActionOrCommand::CodeAction(CodeAction {
                title: "ReimportWarning".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostics[0].clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(
                        vec![(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position {
                                        line: 1,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 1,
                                        character: 20,
                                    },
                                },
                                new_text: "".to_string(),
                            }],
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            CodeActionOrCommand::CodeAction(CodeAction {
                title: "UnusedImportWarning".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostics[1].clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(
                        vec![(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 20,
                                    },
                                },
                                new_text: "".to_string(),
                            }],
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        ];

        for (get, expected) in code_actions.iter().zip(expected.iter()) {
            assert_eq!(get, expected)
        }

        assert_eq!(expected[0], code_actions[0]);
        assert_eq!(expected[1], code_actions[1]);
    }
}
