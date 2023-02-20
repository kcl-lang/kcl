use kclvm_error::Diagnostic as KCLDiagnostic;
use kclvm_error::Level;
use kclvm_error::Message;
use kclvm_error::Position as KCLPos;
use tower_lsp::lsp_types::*;

/// Convert pos format
/// The position in lsp protocol is different with position in ast node whose line number is 1 based.
pub fn kcl_pos_to_lsp_pos(pos: &KCLPos) -> Position {
    Position {
        line: pos.line as u32 - 1,
        character: pos.column.unwrap_or(0) as u32,
    }
}

/// Convert KCL Message to LSP Diagnostic
fn kcl_msg_to_lsp_diags(msg: &Message, severity: DiagnosticSeverity) -> Diagnostic {
    let kcl_pos = msg.pos.clone();
    let start_position = kcl_pos_to_lsp_pos(&kcl_pos);
    let end_position = kcl_pos_to_lsp_pos(&kcl_pos);
    Diagnostic {
        range: Range::new(start_position, end_position),
        severity: Some(severity),
        code: None,
        code_description: None,
        source: None,
        message: msg.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn kcl_err_level_to_severity(level: Level) -> DiagnosticSeverity {
    match level {
        Level::Error => DiagnosticSeverity::ERROR,
        Level::Warning => DiagnosticSeverity::WARNING,
        Level::Note => DiagnosticSeverity::HINT,
    }
}

/// Convert KCL Diagnostic to LSP Diagnostics.
/// Because the diagnostic of KCL contains multiple messages, and each messages corresponds to a diagnostic of LSP, the return value is a vec
pub fn kcl_diag_to_lsp_diags(diag: &KCLDiagnostic, file_name: &str) -> Vec<Diagnostic> {
    diag.messages
        .iter()
        .filter(|msg| msg.pos.filename == file_name)
        .map(|msg| kcl_msg_to_lsp_diags(msg, kcl_err_level_to_severity(diag.level)))
        .collect()
}
