use crate::goto_def::find_def;
use kclvm_error::Position as KCLPos;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::core::scope::Scope;
use lsp_types::ParameterInformation;
use lsp_types::SignatureHelp;
use lsp_types::SignatureInformation;

pub fn signature_help(
    pos: &KCLPos,
    gs: &GlobalState,
    trigger_character: Option<String>,
) -> Option<SignatureHelp> {
    if trigger_character.is_none() {
        return None;
    }
    match trigger_character.unwrap().as_str() {
        // func<cursor>
        "(" => {
            if let Some(def) = find_def(pos, gs, false) {
                match def.get_kind() {
                    kclvm_sema::core::symbol::SymbolKind::Value => {
                        if let Some(symbol) = gs.get_symbols().get_symbol(def) {
                            if let Some(ty) = &symbol.get_sema_info().ty {
                                match &ty.kind {
                                    kclvm_sema::ty::TypeKind::Function(func_ty) => {
                                        let label = func_ty.func_signature_str(&symbol.get_name());
                                        let documentation = match &symbol.get_sema_info().doc {
                                            Some(s) => {
                                                Some(lsp_types::Documentation::String(s.clone()))
                                            }
                                            None => None,
                                        };
                                        let parameters = Some(
                                            func_ty
                                                .params
                                                .iter()
                                                .map(|param| ParameterInformation {
                                                    label: lsp_types::ParameterLabel::Simple(
                                                        format!(
                                                            "{}: {}",
                                                            param.name,
                                                            param.ty.ty_str()
                                                        ),
                                                    ),
                                                    documentation: None,
                                                })
                                                .collect::<Vec<ParameterInformation>>(),
                                        );

                                        return Some(SignatureHelp {
                                            signatures: vec![SignatureInformation {
                                                label,
                                                documentation,
                                                parameters,
                                                active_parameter: Some(0),
                                            }],
                                            active_signature: None,
                                            active_parameter: None,
                                        });
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            None
        }
        // func(arg1,<cursor>)
        "," => {
            if let Some(scope) = gs.look_up_scope(pos) {
                match scope.get_kind() {
                    kclvm_sema::core::scope::ScopeKind::Local => {
                        if let Some(local_scope) = gs.get_scopes().try_get_local_scope(&scope) {
                            match local_scope.get_kind() {
                                kclvm_sema::core::scope::LocalSymbolScopeKind::Callable => {
                                    if let Some(func_symbol) = local_scope.get_owner() {
                                        if let Some(symbol) =
                                            gs.get_symbols().get_symbol(func_symbol)
                                        {
                                            if let Some(ty) = &symbol.get_sema_info().ty {
                                                match &ty.kind {
                                                    kclvm_sema::ty::TypeKind::Function(func_ty) => {
                                                        let label = func_ty
                                                            .func_signature_str(&symbol.get_name());
                                                        let documentation = match &symbol
                                                            .get_sema_info()
                                                            .doc
                                                        {
                                                            Some(s) => Some(
                                                                lsp_types::Documentation::String(
                                                                    s.clone(),
                                                                ),
                                                            ),
                                                            None => None,
                                                        };
                                                        let parameters = Some(
                                                            func_ty
                                                                .params
                                                                .iter()
                                                                .map(|param| ParameterInformation {
                                                                    label:
                                                                        lsp_types::ParameterLabel::Simple(
                                                                            format!(
                                                                                "{}: {}",
                                                                                param.name,
                                                                                param.ty.ty_str()
                                                                            ),
                                                                        ),
                                                                    documentation: None,
                                                                })
                                                                .collect::<Vec<ParameterInformation>>(),
                                                        );

                                                        return Some(SignatureHelp {
                                                            signatures: vec![
                                                                SignatureInformation {
                                                                    label,
                                                                    documentation,
                                                                    parameters,
                                                                    active_parameter: None,
                                                                },
                                                            ],
                                                            active_signature: None,
                                                            active_parameter: None,
                                                        });
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    kclvm_sema::core::scope::ScopeKind::Root => {}
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::signature_help;

    use crate::tests::compile_test_file;
    use kclvm_error::Position as KCLPos;

    #[test]
    fn aaaa() {
        let (file, _program, _, gs) =
            compile_test_file("src/test_data/signature_help/signature_help.k");

        let pos = KCLPos {
            filename: file.clone(),
            line: 5,
            column: Some(9),
        };
        let got = signature_help(&pos, &gs, Some("(".to_string())).unwrap();
        println!("{:?}", got);

        let pos = KCLPos {
            filename: file.clone(),
            line: 6,
            column: Some(10),
        };
        let got = signature_help(&pos, &gs, Some(",".to_string())).unwrap();
        println!("{:?}", got);
    }
}
