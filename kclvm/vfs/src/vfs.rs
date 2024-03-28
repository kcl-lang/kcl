use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use parking_lot::RwLock;

use std::fs;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::sourcemap::SourceMapVfs;
use lazy_static::lazy_static;
use std::sync::Mutex;

pub trait VFS {
    // TODO: More actions to be get SourceFile directly
    fn write(&self, path: String, contents: Option<Vec<u8>>) -> Result<()>;
    fn read(&self, path: String) -> Result<Vec<u8>>;
}

pub trait VFSPath {
    fn write(&self, vfs: &dyn VFS, contents: Option<Vec<u8>>) -> Result<()>;
    fn read(&self, vfs: &dyn VFS) -> Result<Vec<u8>>;
    fn exists_in(&self, vfs: &dyn VFS) -> bool;
    fn standardized(&self) -> String;
}

impl<P> VFSPath for P
where
    P: AsRef<Path>,
{
    fn write(&self, vfs: &dyn VFS, contents: Option<Vec<u8>>) -> Result<()> {
        vfs.write(self.as_ref().display().to_string(), contents)
    }

    fn read(&self, vfs: &dyn VFS) -> Result<Vec<u8>> {
        vfs.read(self.as_ref().display().to_string())
    }

    fn exists_in(&self, vfs: &dyn VFS) -> bool {
        vfs.read(self.as_ref().display().to_string()).is_ok()
    }

    fn standardized(&self) -> String {
        use regex::Regex;
        let re = Regex::new(r"([a-zA-Z]):\\").unwrap();
        let path = re
            .replace_all(&self.as_ref().display().to_string(), "$1/")
            .to_string();
        path.replace("\\", "/").replace("//", "/")
    }
}

pub trait PkgPath {
    fn abs(&self, root_path: Option<String>, start_path: Option<String>) -> String;
    fn to_pkgpath(&self) -> String;
}

impl<P> PkgPath for P
where
    P: AsRef<Path>,
{
    fn abs(&self, root_path: Option<String>, start_path: Option<String>) -> String {
        // relpath: import .sub
        // FixImportPath(root, "path/to/app/file.k", ".sub")        => path.to.app.sub
        // FixImportPath(root, "path/to/app/file.k", "..sub")       => path.to.sub
        // FixImportPath(root, "path/to/app/file.k", "...sub")      => path.sub
        // FixImportPath(root, "path/to/app/file.k", "....sub")     => sub
        // FixImportPath(root, "path/to/app/file.k", ".....sub")    => ""
        //
        // abspath: import path.to.sub
        // FixImportPath(root, "path/to/app/file.k", "path.to.sub") => path.to.sub

        let import_path = self.as_ref().display().to_string();
        let root = root_path.unwrap_or("".to_string());
        let filepath = start_path.unwrap_or("".to_string());

        if !import_path.starts_with('.') {
            return import_path.to_string();
        }

        // Filepath to pkgpath
        let pkgpath = {
            let base = Path::new(&root);
            let dirpath = std::path::Path::new(&filepath).parent().unwrap();

            let pkgpath = if let Some(x) = pathdiff::diff_paths(dirpath, base) {
                x.to_str().unwrap().to_string()
            } else {
                dirpath.to_str().unwrap().to_string()
            };

            let pkgpath = pkgpath.replace(['/', '\\'], ".");
            pkgpath.trim_end_matches('.').to_string()
        };

        let mut leading_dot_count = import_path.len();
        for (i, c) in import_path.chars().enumerate() {
            if c != '.' {
                leading_dot_count = i;
                break;
            }
        }

        // The pkgpath is the current root path
        if pkgpath.is_empty() {
            if leading_dot_count <= 1 {
                return import_path.trim_matches('.').to_string();
            } else {
                return "".to_string();
            }
        }

        if leading_dot_count == 1 {
            return pkgpath + &import_path;
        }

        let ss = pkgpath.split('.').collect::<Vec<&str>>();

        if (leading_dot_count - 1) < ss.len() {
            let prefix = ss[..(ss.len() - leading_dot_count + 1)].join(".");
            let suffix = import_path[leading_dot_count..].to_string();

            return format!("{}.{}", prefix, suffix);
        }

        if leading_dot_count - 1 == ss.len() {
            return import_path[leading_dot_count..].to_string();
        }

        "".to_string()
    }

    fn to_pkgpath(&self) -> String {
        let std_path = self.standardized();
        return std_path.replace("/", ".");
    }
}
