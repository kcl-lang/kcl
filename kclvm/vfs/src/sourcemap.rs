use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Ok;
use kclvm_span::FilePathMapping;
use parking_lot::RwLock;

use crate::vfs::VFS;

pub struct SourceFile {
    sf_inner: Arc<RwLock<kclvm_span::SourceFile>>,
}

pub struct SourceMapVfs {
    sm_inner: Arc<RwLock<kclvm_span::SourceMap>>,
}

impl SourceMapVfs {
    pub fn new() -> Self {
        SourceMapVfs {
            sm_inner: Arc::new(RwLock::new(kclvm_span::SourceMap::new(
                FilePathMapping::empty(),
            ))),
        }
    }
}

impl VFS for SourceMapVfs {
    fn write(&self, path: String, contents: Option<Vec<u8>>) -> anyhow::Result<()> {
        let contents = if let Some(contents) = contents {
            contents
        } else {
            fs::read(&path)?
        };

        let _ = self
            .sm_inner
            .write()
            .new_source_file(PathBuf::from(&path).into(), String::from_utf8(contents)?);
        Ok(())
    }

    fn read(&self, path: String) -> anyhow::Result<Vec<u8>> {
        let sf = self
            .sm_inner
            .read()
            .get_source_file(&PathBuf::from(&path).into());

        if sf.is_none() {
            return Err(anyhow::anyhow!("Source file {} not found", path));
        }

        let binding = sf.unwrap();
        let src_from_sf = match binding.src.as_ref() {
            Some(src) => src,
            None => {
                return Err(anyhow::anyhow!("Source file {} not found", path));
            }
        };

        Ok(src_from_sf.to_string().into_bytes())
    }
}
