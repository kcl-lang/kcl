use std::{fs, path::Path};

pub(crate) fn copy_directory(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(&dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let new_src = entry.path();
        let new_dst = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_directory(&new_src, &new_dst)?;
        } else if file_type.is_file() {
            fs::copy(&new_src, &new_dst)?;
        }
    }
    Ok(())
}
