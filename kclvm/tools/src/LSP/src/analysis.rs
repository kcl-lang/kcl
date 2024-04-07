use crate::db::AnalysisDatabase;
use parking_lot::RwLock;
use ra_ap_vfs::FileId;
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub struct Analysis {
    pub db: Arc<RwLock<HashMap<FileId, Arc<AnalysisDatabase>>>>,
}
