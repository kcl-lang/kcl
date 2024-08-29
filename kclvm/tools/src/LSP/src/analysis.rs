use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_driver::WorkSpaceKind;
use kclvm_error::Diagnostic;
use kclvm_sema::core::global_state::GlobalState;
use parking_lot::RwLock;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub type DocumentVersion = i32;

#[derive(Default, Clone)]
pub struct OpenFileInfo {
    pub version: DocumentVersion,
    pub workspaces: HashSet<WorkSpaceKind>,
}

/// Analysis holds the analysis mapping (FileId -> AnalysisDatabase)
#[derive(Default)]
pub struct Analysis {
    pub workspaces: Arc<RwLock<HashMap<WorkSpaceKind, DBState>>>,
}

#[derive(Clone)]
pub enum DBState {
    Ready(Arc<AnalysisDatabase>),
    // The previous version of db
    Compiling(Arc<AnalysisDatabase>),
    Init,
    Failed(String),
}

/// AnalysisDatabase holds the result of the compile
#[derive(Default, Clone)]
pub struct AnalysisDatabase {
    pub prog: Program,
    pub gs: GlobalState,
    pub diags: IndexSet<Diagnostic>,
}
