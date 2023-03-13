use lsp_types::Url;
use ra_ap_vfs::AbsPathBuf;

/// Converts the specified `uri` to an absolute path. Returns an error if the url could not be
/// converted to an absolute path.
pub(crate) fn abs_path(uri: &Url) -> anyhow::Result<AbsPathBuf> {
    uri.to_file_path()
        .ok()
        .and_then(|path| AbsPathBuf::try_from(path).ok())
        .ok_or_else(|| anyhow::anyhow!("invalid uri: {}", uri))
}
