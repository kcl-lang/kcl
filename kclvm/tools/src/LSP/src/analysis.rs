use kclvm_ast::ast::Program;
use kclvm_sema::core::global_state::GlobalState;
use parking_lot::RwLock;
use ra_ap_vfs::FileId;
use std::{collections::HashMap, sync::Arc};

pub type DocumentVersion = i32;

/// Analysis holds the analysis mapping (FileId -> AnalysisDatabase)
#[derive(Default)]
pub struct Analysis {
    pub db: Arc<RwLock<HashMap<FileId, Option<Arc<AnalysisDatabase>>>>>,
}

/// AnalysisDatabase holds the result of the compile
#[derive(Default, Clone)]
pub struct AnalysisDatabase {
    pub prog: Program,
    pub gs: GlobalState,
    pub version: DocumentVersion,
}
