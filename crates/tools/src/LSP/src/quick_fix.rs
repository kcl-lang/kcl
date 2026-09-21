use std::collections::HashMap;

use kcl_config::modfile::get_pkg_root;
use kcl_error::{DiagnosticId, ErrorKind, WarningKind};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, NumberOrString, TextEdit,
    Url,
};
use serde_json::Value;

use crate::mod_update::UPDATE_DEPENDENCIES_COMMAND;

pub fn quick_fix(uri: &Url, diags: &[Diagnostic]) -> Vec<lsp_types::CodeActionOrCommand> {
    let mut code_actions: Vec<lsp_types::CodeActionOrCommand> = vec![];
    for diag in diags {
        if let Some(code) = &diag.code
            && let Some(id) = convert_code_to_kcl_diag_id(code)
        {
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

                            let action_title = if replacement_text.is_empty() {
                                "Consider removing the problematic code".to_string()
                            } else {
                                format!(
                                    "A local variable with a similar name exists: `{}`",
                                    replacement_text
                                )
                            };

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
                    ErrorKind::InvalidSyntax => {
                        let replacement_texts = extract_suggested_replacements(&diag.data);
                        for replacement_text in replacement_texts {
                            let title = "Consider fix the problematic code".to_string();
                            let mut changes = HashMap::new();
                            changes.insert(
                                uri.clone(),
                                vec![TextEdit {
                                    range: diag.range,
                                    new_text: replacement_text.clone(),
                                }],
                            );
                            code_actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title,
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
                    ErrorKind::CannotFindModule => {
                        // The imported package may not be downloaded yet —
                        // offer to update the dependencies through `kcl mod update`.
                        let mod_dir = crate::from_lsp::abs_path(uri).ok().and_then(|path| {
                            let std_path: &std::path::Path = path.as_ref();
                            std_path
                                .to_str()
                                .and_then(get_pkg_root)
                                .map(std::path::PathBuf::from)
                        });
                        if let Some(dir) = mod_dir {
                            code_actions.push(CodeActionOrCommand::Command(Command {
                                title: "Update dependencies (kcl mod update)".to_string(),
                                command: UPDATE_DEPENDENCIES_COMMAND.to_string(),
                                arguments: Some(vec![Value::String(dir.display().to_string())]),
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
            "InvalidSyntax" => Some(DiagnosticId::Error(ErrorKind::InvalidSyntax)),
            "CannotFindModule" => Some(DiagnosticId::Error(ErrorKind::CannotFindModule)),
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
        CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, DiagnosticSeverity,
        NumberOrString, Position, Range, TextEdit, Url, WorkspaceEdit,
    };
    use proc_macro_crate::bench_test;
    use std::path::PathBuf;

    use kcl_utils::path::PathPrefix;

    use super::{UPDATE_DEPENDENCIES_COMMAND, quick_fix};
    use crate::{
        compile::{Params, compile_with_params},
        state::KCLVfs,
        to_lsp::kcl_diag_to_lsp_diags_by_file,
    };

    #[test]
    #[bench_test]
    fn quick_fix_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let test_file = path.clone();
        let test_file = test_file
            .join("src")
            .join("test_data")
            .join("code_action")
            .join("quick_fix")
            .join("quick_fix.k");
        let file = test_file.to_str().unwrap();

        let diags = compile_with_params(Params {
            file: Some(file.to_string()),
            module_cache: None,
            scope_cache: None,
            vfs: Some(KCLVfs::default()),
            gs_cache: None,
        })
        .0;

        let diagnostics = diags
            .iter()
            .flat_map(|diag| kcl_diag_to_lsp_diags_by_file(diag, file))
            .collect::<Vec<Diagnostic>>();

        let uri = Url::from_file_path(file).unwrap();
        let code_actions = quick_fix(&uri, &diagnostics);

        let expected = [
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

    #[test]
    #[bench_test]
    fn cannot_find_module_quick_fix_test() {
        let dir = std::env::temp_dir().join(format!(
            "kcl-lsp-quick-fix-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("kcl.mod"),
            "[package]\nname = \"quick_fix_test\"\n",
        )
        .unwrap();
        let test_file = dir.join("main.k");
        std::fs::write(&test_file, "import nonexistent_pkg\n").unwrap();

        let diag = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("CannotFindModule".to_string())),
            code_description: None,
            source: Some("kcl".to_string()),
            message: "Cannot find the module nonexistent_pkg".to_string(),
            related_information: None,
            tags: None,
            data: None,
        };

        let uri = Url::from_file_path(&test_file).unwrap();
        let code_actions = quick_fix(&uri, &[diag]);

        assert_eq!(code_actions.len(), 1);
        match &code_actions[0] {
            CodeActionOrCommand::Command(command) => {
                assert_eq!(command.title, "Update dependencies (kcl mod update)");
                assert_eq!(command.command, UPDATE_DEPENDENCIES_COMMAND);
                let arguments = command.arguments.as_ref().unwrap();
                // The argument comes from `get_pkg_root`, which normalizes
                // the canonicalized path (strips the `\\?\` UNC prefix on
                // Windows).
                assert_eq!(
                    PathBuf::from(arguments[0].as_str().unwrap()),
                    PathBuf::from(dir.canonicalize().unwrap().adjust_canonicalization())
                );
            }
            _ => panic!("expected a command quick fix, got {:?}", code_actions[0]),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
