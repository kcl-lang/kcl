use std::{env, path::PathBuf};

use crate::modfile::{get_vendor_home, KCL_PKG_PATH};

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
