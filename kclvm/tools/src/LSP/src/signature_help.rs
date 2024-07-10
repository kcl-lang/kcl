use kclvm_error::Position as KCLPos;
use kclvm_sema::core::global_state::GlobalState;
use lsp_types::ParameterInformation;
use lsp_types::SignatureHelp;
use lsp_types::SignatureInformation;

pub fn signature_help(kcl_pos: &KCLPos, gs: &GlobalState) -> Option<SignatureHelp> {
    let help = SignatureHelp {
        signatures: vec![SignatureInformation {
            label: "func1(a: int, b: str, c: int)".to_string(),
            documentation: Some(lsp_types::Documentation::String("123".to_string())),
            parameters: Some(vec![
                ParameterInformation {
                    label: lsp_types::ParameterLabel::Simple("a: int".to_string()),
                    documentation: Some(lsp_types::Documentation::String("intaaa".to_string())),
                },
                ParameterInformation {
                    label: lsp_types::ParameterLabel::Simple("b: str".to_string()),
                    documentation: Some(lsp_types::Documentation::String("str".to_string())),
                },
            ]),
            active_parameter: None,
        }],
        active_signature: None,
        active_parameter: None,
    };
    Some(help)
}
