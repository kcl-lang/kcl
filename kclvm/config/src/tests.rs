use kclvm_version as version;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::{
    cache::{load_pkg_cache, save_pkg_cache, CacheOption},
    modfile::{get_vendor_home, KCL_PKG_PATH},
};

#[test]
fn test_vendor_home() {
    env::set_var(KCL_PKG_PATH, "test_vendor_home");
    assert_eq!(get_vendor_home(), "test_vendor_home");
    env::remove_var(KCL_PKG_PATH);

    #[cfg(target_os = "windows")]
    let root_dir = env::var("USERPROFILE").unwrap();
    #[cfg(not(target_os = "windows"))]
    let root_dir = env::var("HOME").unwrap();

    let kpm_home = PathBuf::from(root_dir)
        .join(".kcl")
        .join("kpm")
        .canonicalize()
        .unwrap();
    assert_eq!(get_vendor_home(), kpm_home.display().to_string())
}

#[test]
fn test_pkg_cache() {
    let root = PathBuf::from("./src/testdata/test_cache/")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let mut external_pkgs = HashMap::new();
    external_pkgs.insert(
        "test_vendor".to_string(),
        "./src/testdata/test_vendor".to_string(),
    );

    let lock_path = Path::new(&root)
        .join(".kclvm/cache")
        .join(format!("{}-{}", version::VERSION, version::CHECK_SUM))
        .join("test_target");

    fs::create_dir_all(lock_path.clone()).unwrap();
    File::create(lock_path.join("test_vendor.lock")).unwrap();

    save_pkg_cache(
        &root,
        "test_target",
        "test_vendor",
        "test_data",
        CacheOption::default(),
        &external_pkgs,
    )
    .unwrap();

    assert_eq!(
        load_pkg_cache(
            &root,
            "test_target",
            "test_vendor",
            CacheOption::default(),
            &external_pkgs,
        ),
        Some("test_data".to_string())
    )
}
