//! Copyright The KCL Authors. All rights reserved.
extern crate chrono;
use super::modfile::KCL_FILE_SUFFIX;
use anyhow::Result;
use fslock::LockFile;
use kclvm_utils::pkgpath::{parse_external_pkg_name, rm_external_pkg_name};
use md5::{Digest, Md5};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::error;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;

use kclvm_version as version;

const LOCK_SUFFIX: &str = ".lock";
const DEFAULT_CACHE_DIR: &str = ".kclvm/cache";
const CACHE_INFO_FILENAME: &str = "info";
const KCL_SUFFIX_PATTERN: &str = "*.k";
pub const KCL_CACHE_PATH_ENV_VAR: &str = "KCL_CACHE_PATH";

pub type CacheInfo = Vec<u8>;
pub type Cache = HashMap<String, CacheInfo>;

#[allow(dead_code)]
pub struct CacheOption {
    cache_dir: String,
}

impl Default for CacheOption {
    fn default() -> Self {
        Self {
            cache_dir: DEFAULT_CACHE_DIR.to_string(),
        }
    }
}

/// Load pkg cache.
pub fn load_pkg_cache<T>(
    root: &str,
    target: &str,
    pkgpath: &str,
    option: CacheOption,
    external_pkgs: &HashMap<String, String>,
) -> Option<T>
where
    T: DeserializeOwned + Default,
{
    if root.is_empty() || pkgpath.is_empty() {
        None
    } else {
        // The cache file path
        let filename = get_cache_filename(root, target, pkgpath, Some(&option.cache_dir));
        if !Path::new(&filename).exists() {
            None
        } else {
            // Compare the md5 using cache
            let real_path = get_pkg_realpath_from_pkgpath(root, pkgpath);
            // If the file exists and it is an internal package or an external package,
            // Check the cache info.
            let pkg_name = parse_external_pkg_name(pkgpath).ok()?;

            // If it is an internal package
            let real_path = if Path::new(&real_path).exists() {
                real_path
                // If it is an external package
            } else if external_pkgs.get(&pkg_name).is_some()
                && Path::new(external_pkgs.get(&pkg_name)?).exists()
            {
                get_pkg_realpath_from_pkgpath(
                    external_pkgs.get(&pkg_name)?,
                    &rm_external_pkg_name(pkgpath).ok()?,
                )
            } else {
                return None;
            };

            // get the cache info from cache file "info"
            let cache_info = read_info_cache(root, target, Some(&option.cache_dir));
            let relative_path = real_path.replacen(root, ".", 1);
            match cache_info.get(&relative_path) {
                Some(path_info_in_cache) => {
                    // calculate the md5 of the file and compare it with the cache info
                    if get_cache_info(&real_path).ne(path_info_in_cache) {
                        return None;
                    }
                }
                None => return None,
            };
            // If the md5 is the same, load the cache file
            load_data_from_file(&filename)
        }
    }
}

/// Save pkg cache.
pub fn save_pkg_cache<T>(
    root: &str,
    target: &str,
    pkgpath: &str,
    data: T,
    option: CacheOption,
    external_pkgs: &HashMap<String, String>,
) -> Result<()>
where
    T: Serialize,
{
    if root.is_empty() || pkgpath.is_empty() {
        return Err(anyhow::anyhow!(
            "failed to save package cache {} to root {}",
            pkgpath,
            root
        ));
    }
    let dst_filename = get_cache_filename(root, target, pkgpath, Some(&option.cache_dir));
    let real_path = get_pkg_realpath_from_pkgpath(root, pkgpath);
    if Path::new(&real_path).exists() {
        write_info_cache(root, target, Some(&option.cache_dir), &real_path).unwrap();
    } else {
        // If the file does not exist, it is an external package.
        let pkg_name = parse_external_pkg_name(pkgpath)?;
        let real_path = get_pkg_realpath_from_pkgpath(
            external_pkgs
                .get(&pkg_name)
                .ok_or(anyhow::anyhow!("failed to save cache"))?,
            &rm_external_pkg_name(pkgpath)?,
        );
        if Path::new(&real_path).exists() {
            write_info_cache(root, target, Some(&option.cache_dir), &real_path).unwrap();
        }
    }
    let cache_dir = get_cache_dir(root, Some(&option.cache_dir));
    create_dir_all(&cache_dir).unwrap();
    let tmp_filename = temp_file(&cache_dir, pkgpath);
    save_data_to_file(&dst_filename, &tmp_filename, data);
    Ok(())
}

#[inline]
fn get_cache_dir(root: &str, cache_dir: Option<&str>) -> String {
    let cache_dir = cache_dir.unwrap_or(DEFAULT_CACHE_DIR);
    let root = std::env::var(KCL_CACHE_PATH_ENV_VAR).unwrap_or(root.to_string());
    Path::new(&root)
        .join(cache_dir)
        .join(format!("{}-{}", version::VERSION, version::CHECK_SUM))
        .display()
        .to_string()
}

#[inline]
#[allow(dead_code)]
fn get_cache_filename(root: &str, target: &str, pkgpath: &str, cache_dir: Option<&str>) -> String {
    let cache_dir = cache_dir.unwrap_or(DEFAULT_CACHE_DIR);
    let root = std::env::var(KCL_CACHE_PATH_ENV_VAR).unwrap_or(root.to_string());
    Path::new(&root)
        .join(cache_dir)
        .join(format!("{}-{}", version::VERSION, version::CHECK_SUM))
        .join(target)
        .join(pkgpath)
        .display()
        .to_string()
}

#[inline]
fn get_cache_info_filename(root: &str, target: &str, cache_dir: Option<&str>) -> String {
    let cache_dir = cache_dir.unwrap_or(DEFAULT_CACHE_DIR);
    let root = std::env::var(KCL_CACHE_PATH_ENV_VAR).unwrap_or(root.to_string());
    Path::new(&root)
        .join(cache_dir)
        .join(format!("{}-{}", version::VERSION, version::CHECK_SUM))
        .join(target)
        .join(CACHE_INFO_FILENAME)
        .display()
        .to_string()
}

/// Read the cache if it exists and is well formed.
/// If it is not well formed, the call to write_info_cache later should resolve the issue.
pub fn read_info_cache(root: &str, target: &str, cache_dir: Option<&str>) -> Cache {
    let cache_file = get_cache_info_filename(root, target, cache_dir);
    if !Path::new(&cache_file).exists() {
        return Cache::default();
    }
    let file = File::open(cache_file).unwrap();
    match ron::de::from_reader(file) {
        Ok(cache) => cache,
        Err(_) => HashMap::new(),
    }
}

/// Update the cache info file.
pub fn write_info_cache(
    root: &str,
    target: &str,
    cache_name: Option<&str>,
    filepath: &str,
) -> Result<(), Box<dyn error::Error>> {
    let dst_filename = get_cache_info_filename(root, target, cache_name);
    let cache_dir = get_cache_dir(root, cache_name);
    let path = Path::new(&cache_dir);
    create_dir_all(path).unwrap();
    let relative_path = filepath.replacen(root, ".", 1);
    let cache_info = get_cache_info(filepath);
    let tmp_filename = temp_file(&cache_dir, "");
    let mut lock_file = LockFile::open(&format!("{}{}", dst_filename, LOCK_SUFFIX)).unwrap();
    lock_file.lock().unwrap();
    let mut cache = read_info_cache(root, target, cache_name);
    cache.insert(relative_path, cache_info);
    let mut file = File::create(&tmp_filename).unwrap();
    file.write_all(ron::ser::to_string(&cache).unwrap().as_bytes())
        .unwrap();
    std::fs::rename(&tmp_filename, &dst_filename).unwrap();
    lock_file.unlock().unwrap();
    Ok(())
}

/// Return the information used to check if a file or path is already changed or not.
fn get_cache_info(path_str: &str) -> CacheInfo {
    let path = Path::new(path_str);
    let mut md5 = Md5::new();
    if path.is_file() {
        let mut file = File::open(path_str).unwrap();
        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf).unwrap();
        md5.input(buf.as_slice());
    } else {
        let pattern = Path::new(path_str)
            .join(KCL_SUFFIX_PATTERN)
            .display()
            .to_string();
        for file in glob::glob(&pattern).unwrap().flatten() {
            let mut file = File::open(file).unwrap();
            let mut buf: Vec<u8> = vec![];
            file.read_to_end(&mut buf).unwrap();
            md5.input(buf.as_slice());
        }
    }
    md5.result().to_vec()
}

pub fn get_pkg_realpath_from_pkgpath(root: &str, pkgpath: &str) -> String {
    let filepath = format!("{}/{}", root, pkgpath.replace('.', "/"));
    let filepath_with_suffix = format!("{}{}", filepath, KCL_FILE_SUFFIX);
    if Path::new(&filepath_with_suffix).is_file() {
        filepath_with_suffix
    } else {
        filepath
    }
}

pub fn load_data_from_file<T>(filename: &str) -> Option<T>
where
    T: DeserializeOwned + Default,
{
    let file = File::open(filename);
    if let Ok(file) = file {
        ron::de::from_reader(file).ok()
    } else {
        None
    }
}

pub fn save_data_to_file<T>(dst_filename: &str, tmp_filename: &str, data: T)
where
    T: Serialize,
{
    let mut lock_file = LockFile::open(&format!("{}{}", dst_filename, LOCK_SUFFIX)).unwrap();
    lock_file.lock().unwrap();
    let file = File::create(tmp_filename).unwrap();
    ron::ser::to_writer(file, &data).unwrap();
    std::fs::rename(tmp_filename, dst_filename).unwrap();
    lock_file.unlock().unwrap();
}

#[inline]
fn temp_file(cache_dir: &str, pkgpath: &str) -> String {
    let timestamp = chrono::Local::now().timestamp_nanos();
    let id = std::process::id();
    Path::new(cache_dir)
        .join(format!("{}.{}.{}.tmp", pkgpath, id, timestamp))
        .display()
        .to_string()
}
