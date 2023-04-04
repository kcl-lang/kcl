use kclvm_error::Diagnostic as KCLDiagnostic;
use kclvm_error::Level;
use kclvm_error::Message;
use kclvm_error::Position as KCLPos;
use lsp_types::*;
use ra_ap_vfs::FileId;

use crate::state::LanguageServerSnapshot;
use std::{
    path::{Component, Path, Prefix},
    str::FromStr,
};

/// Convert pos format
/// The position in lsp protocol is different with position in ast node whose line number is 1 based.
pub fn lsp_pos(pos: &KCLPos) -> Position {
    Position {
        line: pos.line as u32 - 1,
        character: pos.column.unwrap_or(0) as u32,
    }
}

/// Convert KCL Message to LSP Diagnostic
fn kcl_msg_to_lsp_diags(msg: &Message, severity: DiagnosticSeverity) -> Diagnostic {
    let kcl_pos = msg.pos.clone();
    let start_position = lsp_pos(&kcl_pos);
    let end_position = lsp_pos(&kcl_pos);

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

/// Returns the `Url` associated with the specified `FileId`.
pub(crate) fn url(snapshot: &LanguageServerSnapshot, file_id: FileId) -> anyhow::Result<Url> {
    let vfs = snapshot.vfs.read();
    if let Some(path) = vfs.file_path(FileId(file_id.0)).as_path() {
        Ok(url_from_path_with_drive_lowercasing(path)?)
    } else {
        Err(anyhow::anyhow!(
            "{} isn't on the file system.",
            vfs.file_path(FileId(file_id.0))
        ))
    }
}

/// Returns a `Url` object from a given path, will lowercase drive letters if present.
/// This will only happen when processing Windows paths.
///
/// When processing non-windows path, this is essentially do the same as `Url::from_file_path`.
pub(crate) fn url_from_path_with_drive_lowercasing(path: impl AsRef<Path>) -> anyhow::Result<Url> {
    let component_has_windows_drive = path.as_ref().components().any(|comp| {
        if let Component::Prefix(c) = comp {
            match c.kind() {
                Prefix::Disk(_) | Prefix::VerbatimDisk(_) => return true,
                _ => return false,
            }
        }
        false
    });

    // VSCode expects drive letters to be lowercased, whereas rust will uppercase the drive letters.
    if component_has_windows_drive {
        let url_original = Url::from_file_path(&path).map_err(|_| {
            anyhow::anyhow!("can't convert path to url: {}", path.as_ref().display())
        })?;

        let drive_partition: Vec<&str> = url_original.as_str().rsplitn(2, ':').collect();

        // There is a drive partition, but we never found a colon.
        // This should not happen, but in this case we just pass it through.
        if drive_partition.len() == 1 {
            return Ok(url_original);
        }

        let joined = drive_partition[1].to_ascii_lowercase() + ":" + drive_partition[0];
        let url = Url::from_str(&joined)
            .map_err(|e| anyhow::anyhow!("Url from str ParseError: {}", e))?;
        Ok(url)
    } else {
        Ok(Url::from_file_path(&path).map_err(|_| {
            anyhow::anyhow!("can't convert path to url: {}", path.as_ref().display())
        })?)
    }
}
