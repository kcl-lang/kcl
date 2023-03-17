use kclvm_error::Position as KCLPos;
use lsp_types::{Position, Url};
use ra_ap_vfs::AbsPathBuf;

/// Converts the specified `uri` to an absolute path. Returns an error if the url could not be
/// converted to an absolute path.
pub(crate) fn abs_path(uri: &Url) -> anyhow::Result<AbsPathBuf> {
    uri.to_file_path()
        .ok()
        .and_then(|path| AbsPathBuf::try_from(path).ok())
        .ok_or_else(|| anyhow::anyhow!("invalid uri: {}", uri))
}

// Convert pos format
// The position in lsp protocol is different with position in ast node whose line number is 1 based.
pub(crate) fn kcl_pos(file: &str, pos: Position) -> KCLPos {
    KCLPos {
        filename: file.to_string(),
        line: (pos.line + 1) as u64,
        column: if pos.character == 0 {
            None
        } else {
            Some(pos.character as u64)
        },
    }
}
