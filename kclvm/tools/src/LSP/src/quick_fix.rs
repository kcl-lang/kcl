use std::collections::HashMap;

use kclvm_error::{DiagnosticId, ErrorKind, WarningKind};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, NumberOrString, TextEdit, Url,
};
use serde_json::Value;

pub(crate) fn quick_fix(uri: &Url, diags: &Vec<Diagnostic>) -> Vec<lsp_types::CodeActionOrCommand> {
    let mut code_actions: Vec<lsp_types::CodeActionOrCommand> = vec![];
    for diag in diags {
        if let Some(code) = &diag.code {
            if let Some(id) = convert_code_to_kcl_diag_id(code) {
                match id {
                    DiagnosticId::Error(error) => match error {
                        ErrorKind::CompileError => {
                            let replacement_texts = extract_suggested_replacements(&diag.data);
                            for replacement_text in replacement_texts {
                                let mut changes = HashMap::new();
                                changes.insert(
                                    uri.clone(),
                                    vec![TextEdit {
                                        range: diag.range,
                                        new_text: replacement_text.clone(),
                                    }],
                                );
                                let action_title = format!(
                                    "a local variable with a similar name exists: `{}`",
                                    replacement_text
                                );
                                code_actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                    title: action_title,
                                    kind: Some(CodeActionKind::QUICKFIX),
                                    diagnostics: Some(vec![diag.clone()]),
                                    edit: Some(lsp_types::WorkspaceEdit {
                                        changes: Some(changes),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }));
                            }
                        }
                        _ => continue,
                    },
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

fn extract_suggested_replacements(data: &Option<Value>) -> Vec<String> {
    data.as_ref()
        .and_then(|data| match data {
            Value::Object(obj) => obj.get("suggested_replacement").map(|val| match val {
                Value::String(s) => vec![s.clone()],
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
                _ => vec![],
            }),
            _ => None,
        })
        .unwrap_or_default()
}

pub(crate) fn convert_code_to_kcl_diag_id(code: &NumberOrString) -> Option<DiagnosticId> {
    match code {
        NumberOrString::Number(_) => None,
        NumberOrString::String(code) => match code.as_str() {
            "CompilerWarning" => Some(DiagnosticId::Warning(WarningKind::CompilerWarning)),
            "UnusedImportWarning" => Some(DiagnosticId::Warning(WarningKind::UnusedImportWarning)),
            "ReimportWarning" => Some(DiagnosticId::Warning(WarningKind::ReimportWarning)),
            "CompileError" => Some(DiagnosticId::Error(ErrorKind::CompileError)),
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
        state::KCLVfs,
        to_lsp::kcl_diag_to_lsp_diags,
        util::{compile_with_params, Params},
    };

    #[test]
    #[bench_test]
    fn quick_fix_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut test_file = path.clone();
        test_file.push("src/test_data/quick_fix.k");
        let file = test_file.to_str().unwrap();

        let (_, diags, _) = compile_with_params(Params {
            file: file.to_string(),
            module_cache: None,
            scope_cache: None,
            vfs: Some(KCLVfs::default()),
        })
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
