//! Copyright The KCL Authors. All rights reserved.

#[cfg(unix)]
pub fn open_lock_file(path: &str) -> Result<fslock::LockFile, fslock::Error> {
    return fslock::LockFile::open(path);
}

#[cfg(windows)]
pub fn open_lock_file(path: &str) -> Result<fslock::LockFile, fslock::Error> {
    return fslock::LockFile::open(path);
}

#[cfg(target_arch = "wasm32")]
pub fn open_lock_file(_path: &str) -> Result<LockFile, std::io::Error> {
    Ok(LockFile { _fd: 0 })
}

#[cfg(target_arch = "wasm32")]
pub struct LockFile {
    _fd: i32,
}

#[cfg(target_arch = "wasm32")]
impl LockFile {
    pub fn lock(&mut self) -> Result<(), std::io::Error> {
        Ok(()) // TODO: support wasm32
    }
    pub fn unlock(&mut self) -> Result<(), std::io::Error> {
        Ok(()) // TODO: support wasm32
    }
}
