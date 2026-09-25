use kcl_utils::path::PathPrefix;
use kcl_version as version;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::{
    cache::{CacheOption, load_pkg_cache, save_pkg_cache},
    modfile::{KCL_PKG_PATH, LockDependency, get_vendor_home},
};

#[test]
fn test_vendor_home() {
    unsafe { env::set_var(KCL_PKG_PATH, "test_vendor_home") };
    assert_eq!(get_vendor_home(), "test_vendor_home");
    unsafe { env::remove_var(KCL_PKG_PATH) };

    #[cfg(target_os = "windows")]
    let root_dir = env::var("USERPROFILE").unwrap();
    #[cfg(not(target_os = "windows"))]
    let root_dir = env::var("HOME").unwrap();

    let kpm_home = PathBuf::from(root_dir)
        .join(".kcl")
        .join("kpm")
        .canonicalize()
        .unwrap();
    assert_eq!(
        get_vendor_home(),
        kpm_home.display().to_string().adjust_canonicalization()
    )
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
        .join(".kcl/cache")
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

/// Regression test for kcl-lang/modules#281: importing an OCI / Git package
/// whose repo or dependency name contains `-` previously failed because
/// `gen_filename` returned the unsanitized last URL segment. KCL identifiers
/// disallow `-`, so the on-disk filename must use `_` instead — same as the
/// `name` fallback branch.
#[test]
fn test_gen_filename_sanitizes_hyphens() {
    // OCI repo with a hyphen in the last segment.
    let dep = LockDependency {
        name: "ignored".to_string(),
        full_name: None,
        version: None,
        sum: None,
        reg: Some("ghcr.io".to_string()),
        repo: Some("oci://ghcr.io/some-org/hello-world".to_string()),
        oci_tag: None,
        url: None,
        branch: None,
        commit: None,
        git_tag: None,
        path: None,
    };
    assert_eq!(dep.gen_filename(), "hello_world");

    // Git URL with a hyphen in the last segment and a `.git` suffix.
    let dep = LockDependency {
        name: "ignored".to_string(),
        full_name: None,
        version: None,
        sum: None,
        reg: None,
        repo: None,
        oci_tag: None,
        url: Some("https://github.com/some-org/hello-world.git".to_string()),
        branch: None,
        commit: None,
        git_tag: None,
        path: None,
    };
    assert_eq!(dep.gen_filename(), "hello_world");

    // No URL/repo → fall back to `name`.
    let dep = LockDependency {
        name: "hello-world".to_string(),
        full_name: None,
        version: None,
        sum: None,
        reg: None,
        repo: None,
        oci_tag: None,
        url: None,
        branch: None,
        commit: None,
        git_tag: None,
        path: None,
    };
    assert_eq!(dep.gen_filename(), "hello_world");

    // Names without hyphens must be unchanged on every branch.
    let dep = LockDependency {
        name: "hello_world".to_string(),
        full_name: None,
        version: None,
        sum: None,
        reg: Some("ghcr.io".to_string()),
        repo: Some("oci://ghcr.io/some-org/hello_world".to_string()),
        oci_tag: None,
        url: None,
        branch: None,
        commit: None,
        git_tag: None,
        path: None,
    };
    assert_eq!(dep.gen_filename(), "hello_world");
}
