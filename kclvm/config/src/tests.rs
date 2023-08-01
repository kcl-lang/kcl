use std::{env, path::PathBuf};

use crate::modfile::{get_compile_entries_from_paths, get_vendor_home, KCL_PKG_PATH};

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
fn test_get_compile_entries_from_paths() {
    let testpath = PathBuf::from("./src/testdata/multimods")
        .canonicalize()
        .unwrap();

    // [`kcl1_path`] is a normal path of the package [`kcl1`] root directory.
    // It looks like `/xxx/xxx/xxx`.
    let kcl1_path = testpath.join("kcl1");

    // [`kcl2_path`] is a mod relative path of the packege [`kcl2`] root directory.
    // It looks like `${kcl2:KCL_MOD}/xxx/xxx`
    let kcl2_path = PathBuf::from("${kcl2:KCL_MOD}/main.k");

    // [`kcl3_path`] is a mod relative path of the [`__main__`] packege.
    let kcl3_path = PathBuf::from("${KCL_MOD}/main.k");

    // [`external_pkgs`] is a map to show the real path of the mod relative path [`kcl2`].
    let mut external_pkgs = std::collections::HashMap::<String, String>::new();
    external_pkgs.insert(
        "kcl2".to_string(),
        testpath.join("kcl2").to_str().unwrap().to_string(),
    );

    // [`get_compile_entries_from_paths`] will return the map of package name to package root real path.
    let entries = get_compile_entries_from_paths(
        &[
            kcl1_path.to_str().unwrap().to_string(),
            kcl2_path.display().to_string(),
            kcl3_path.display().to_string(),
        ],
        external_pkgs,
    )
    .unwrap();

    assert_eq!(entries.len(), 3);

    assert_eq!(entries.get_nth_entry(0).unwrap().name(), "__main__");
    assert_eq!(
        PathBuf::from(entries.get_nth_entry(0).unwrap().path())
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        kcl1_path.canonicalize().unwrap().to_str().unwrap()
    );

    assert_eq!(entries.get_nth_entry(1).unwrap().name(), "kcl2");
    assert_eq!(
        PathBuf::from(entries.get_nth_entry(1).unwrap().path())
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        testpath
            .join("kcl2")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );

    assert_eq!(entries.get_nth_entry(2).unwrap().name(), "__main__");
    assert_eq!(
        PathBuf::from(entries.get_nth_entry(2).unwrap().path())
            .display()
            .to_string(),
        kcl1_path
            .join("main.k")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
}
