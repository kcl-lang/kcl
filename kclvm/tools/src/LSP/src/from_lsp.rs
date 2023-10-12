use std::ops::Range;

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
        column: Some(pos.character as u64),
    }
}

/// Converts the given lsp range to `Range`
pub(crate) fn text_range(text: &str, range: lsp_types::Range) -> Range<usize> {
    let mut lines_length = vec![];
    let lines_text: Vec<&str> = text.split('\n').collect();
    let mut pre_total_length = 0;
    // range line base-zeror
    for i in 0..range.end.line + 1 {
        let i = i as usize;
        if i < lines_text.len() {
            let line = lines_text.get(i).unwrap();
            lines_length.push(pre_total_length);
            pre_total_length += line.len() + "\n".len();
        } else {
            lines_length.push(pre_total_length);
        }
    }

    let start =
        lines_length.get(range.start.line as usize).unwrap() + range.start.character as usize;
    let end = lines_length.get(range.end.line as usize).unwrap() + range.end.character as usize;

    Range { start, end }
}

/// Converts the specified `url` to a utf8 encoded file path string. Returns an error if the url could not be
/// converted to a valid utf8 encoded file path string.
pub(crate) fn file_path_from_url(url: &Url) -> anyhow::Result<String> {
    url.to_file_path()
        .ok()
        .and_then(|path| path.to_str().map(|p| p.to_string()))
        .ok_or_else(|| anyhow::anyhow!("can't convert url to file path: {}", url))
}
