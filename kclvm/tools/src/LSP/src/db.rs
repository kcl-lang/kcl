// #[salsa::database(
//     SourceDatabaseStorage,
//     DefDatabaseStorage,
//     AstDatabaseStorage,
//     InternDatabaseStorage
// )]
#[derive(Default)]
pub(crate) struct AnalysisDatabase {
    // storage: salsa::Storage<Self>,
}
