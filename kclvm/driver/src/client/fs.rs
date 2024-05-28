use std::path::Path;

#[inline]
pub(crate) fn directory_is_not_empty<P: AsRef<Path>>(path: P) -> bool {
    std::fs::read_dir(path)
        .map(|mut entries| entries.next().is_some())
        .is_ok()
}
