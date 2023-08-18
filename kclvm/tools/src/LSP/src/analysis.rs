use crate::db::AnalysisDatabase;
use ra_ap_vfs::FileId;
use std::collections::HashMap;

#[derive(Default)]
pub struct Analysis {
    pub db: HashMap<FileId, AnalysisDatabase>,
}

impl Analysis {
    pub fn set_db(&mut self, id: FileId, db: AnalysisDatabase) {
        self.db.insert(id, db);
    }
}
