use std::fmt;

use ra_ap_vfs::VfsPath;

pub(crate) const RETRY_REQUEST: &str = "Retry Request";

#[derive(Debug, Clone)]
pub(crate) enum LSPError {
    Retry,
    FileIdNotFound(VfsPath),
    AnalysisDatabaseNotFound(VfsPath),
}

impl fmt::Display for LSPError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LSPError::Retry => write!(f, "{}", RETRY_REQUEST),
            LSPError::FileIdNotFound(path) => write!(f, "Path {path} fileId not found"),
            LSPError::AnalysisDatabaseNotFound(path) => {
                write!(f, "Path {path} AnalysisDatabase not found")
            }
        }
    }
}

impl std::error::Error for LSPError {}
