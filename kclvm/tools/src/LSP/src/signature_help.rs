use crate::goto_def::find_def;
use kclvm_error::Position as KCLPos;
use kclvm_sema::core::global_state::GlobalState;
use kclvm_sema::core::scope::Scope;
use kclvm_sema::core::symbol::SymbolKind;
use kclvm_sema::core::symbol::SymbolRef;
use kclvm_sema::ty::FunctionType;
use lsp_types::ParameterInformation;
use lsp_types::SignatureHelp;
use lsp_types::SignatureInformation;

pub fn signature_help(
    pos: &KCLPos,
    gs: &GlobalState,
    trigger_character: Option<String>,
) -> Option<SignatureHelp> {
    trigger_character.as_ref()?;
    match trigger_character.unwrap().as_str() {
        // func<cursor>
        "(" => {
            let def = find_def(pos, gs, false)?;
            match def.get_kind() {
                SymbolKind::Value | SymbolKind::Function => {
                    let symbol = gs.get_symbols().get_symbol(def)?;
                    let ty = &symbol.get_sema_info().ty.clone()?;
                    if let kclvm_sema::ty::TypeKind::Function(func_ty) = &ty.kind {
                        let (label, parameters) =
                            function_signatue_help(&symbol.get_name(), func_ty);
                        let documentation = symbol
                            .get_sema_info()
                            .doc
                            .clone()
                            .map(lsp_types::Documentation::String);

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
                }
                _ => {}
            }
            None
        }
        // func(arg1<cursor>)
        "," => {
            let scope = gs.look_up_scope(pos)?;
            if let kclvm_sema::core::scope::ScopeKind::Local = scope.get_kind() {
                let local_scope = gs.get_scopes().try_get_local_scope(&scope)?;
                if let kclvm_sema::core::scope::LocalSymbolScopeKind::Callable =
                    local_scope.get_kind()
                {
                    let func_symbol = local_scope.get_owner()?;
                    let symbol = gs.get_symbols().get_symbol(func_symbol)?;
                    let ty = &symbol.get_sema_info().ty.clone()?;
                    if let kclvm_sema::ty::TypeKind::Function(func_ty) = &ty.kind {
                        let (label, parameters) =
                            function_signatue_help(&symbol.get_name(), func_ty);
                        let documentation = symbol
                            .get_sema_info()
                            .doc
                            .clone()
                            .map(lsp_types::Documentation::String);

                        // highlight parameter's index
                        // if None, it will highlight first param(maybe default)
                        // if index >= param len, it will be no highlight
                        let active_parameter = match gs.get_scope_symbols(scope) {
                            Some(arg_symbols) => {
                                // func(a, 1, z = 3)
                                // Unresolved => variable ref: a
                                // Expression => 1, 3,
                                // filter kw symbol `z`
                                let actually_symbol: Vec<SymbolRef> = arg_symbols
                                    .into_iter()
                                    .filter(|symbol| {
                                        matches!(
                                            symbol.get_kind(),
                                            SymbolKind::Unresolved | SymbolKind::Expression
                                        )
                                    })
                                    .collect();
                                let mut index: usize = 0;
                                for (i, symbol) in actually_symbol.iter().enumerate() {
                                    let s = gs.get_symbols().get_symbol(*symbol).unwrap();
                                    let start = s.get_range().0;
                                    if pos.less_equal(&start) {
                                        index = i;
                                        break;
                                    }
                                }
                                Some(index as u32)
                            }
                            None => None,
                        };

                        return Some(SignatureHelp {
                            signatures: vec![SignatureInformation {
                                label,
                                documentation,
                                parameters,
                                active_parameter,
                            }],
                            active_signature: None,
                            active_parameter: None,
                        });
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn function_signatue_help(
    name: &String,
    func_ty: &FunctionType,
) -> (String, Option<Vec<ParameterInformation>>) {
    let label = func_ty.func_signature_str(name);
    let parameters = Some(
        func_ty
            .params
            .iter()
            .map(|param| ParameterInformation {
                label: lsp_types::ParameterLabel::Simple(format!(
                    "{}: {}",
                    param.name,
                    param.ty.ty_str()
                )),
                documentation: None,
            })
            .collect::<Vec<ParameterInformation>>(),
    );
    (label, parameters)
}

#[cfg(test)]
mod tests {
    use super::signature_help;

    use crate::tests::compile_test_file;
    use kclvm_error::Position as KCLPos;

    #[macro_export]
    macro_rules! signature_help_test_snapshot {
        ($name:ident, $file:expr, $line:expr, $column: expr,  $trigger_character: expr) => {
            #[test]
            fn $name() {
                let (file, _program, _, gs) = compile_test_file($file);

                let pos = KCLPos {
                    filename: file.clone(),
                    line: $line,
                    column: Some($column),
                };
                let res = signature_help(&pos, &gs, $trigger_character).unwrap();
                insta::assert_snapshot!(format!("{:#?}", res));
            }
        };
    }

    signature_help_test_snapshot!(
        lambda_signatue_help_test_0,
        "src/test_data/signature_help/lambda_signature_help/lambda_signature_help.k",
        5,
        9,
        Some("(".to_string())
    );

    signature_help_test_snapshot!(
        lambda_signatue_help_test_1,
        "src/test_data/signature_help/lambda_signature_help/lambda_signature_help.k",
        6,
        11,
        Some(",".to_string())
    );

    signature_help_test_snapshot!(
        lambda_signatue_help_test_2,
        "src/test_data/signature_help/lambda_signature_help/lambda_signature_help.k",
        7,
        14,
        Some(",".to_string())
    );

    signature_help_test_snapshot!(
        lambda_signatue_help_test_3,
        "src/test_data/signature_help/lambda_signature_help/lambda_signature_help.k",
        8,
        21,
        Some(",".to_string())
    );

    signature_help_test_snapshot!(
        builtin_function_signature_help_test_0,
        "src/test_data/signature_help/builtin_function_signature_help/builtin_function_signature_help.k",
        1,
        4,
        Some("(".to_string())
    );

    signature_help_test_snapshot!(
        builtin_function_signature_help_test_1,
        "src/test_data/signature_help/builtin_function_signature_help/builtin_function_signature_help.k",
        2,
        6,
        Some(",".to_string())
    );

    signature_help_test_snapshot!(
        pkg_function_signature_help_test_0,
        "src/test_data/signature_help/pkg_function_signature_help/pkg_function_signature_help.k",
        3,
        9,
        Some("(".to_string())
    );

    signature_help_test_snapshot!(
        pkg_function_signature_help_test_1,
        "src/test_data/signature_help/pkg_function_signature_help/pkg_function_signature_help.k",
        4,
        11,
        Some(",".to_string())
    );
}
