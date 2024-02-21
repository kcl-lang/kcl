use std::path::{Path, PathBuf};
use std::{env, fs, panic};

use kclvm_config::modfile::get_vendor_home;
use kclvm_config::settings::KeyValuePair;
use kclvm_parser::LoadProgramOptions;
use walkdir::WalkDir;

use crate::arguments::parse_key_value_pair;
use crate::kpm::{fetch_metadata, fill_pkg_maps_for_k_file, update_dependencies};
use crate::lookup_the_nearest_file_dir;
use crate::{canonicalize_input_files, expand_input_files, get_pkg_list};

#[test]
fn test_canonicalize_input_files() {
    let input_files = vec!["file1.k".to_string(), "file2.k".to_string()];
    let work_dir = ".".to_string();
    let expected_files = vec![
        Path::new(".").join("file1.k").to_string_lossy().to_string(),
        Path::new(".").join("file2.k").to_string_lossy().to_string(),
    ];
    assert_eq!(
        canonicalize_input_files(&input_files, work_dir.clone(), false).unwrap(),
        expected_files
    );
    assert!(canonicalize_input_files(&input_files, work_dir, true).is_err());
}

#[test]
fn test_expand_input_files_with_kcl_mod() {
    let path = PathBuf::from("src/test_data/expand_file_pattern");
    let input_files = vec![
        path.join("**").join("main.k").to_string_lossy().to_string(),
        "${KCL_MOD}/src/test_data/expand_file_pattern/KCL_MOD".to_string(),
    ];
    let expected_files = vec![
        path.join("kcl1/kcl2/main.k").to_string_lossy().to_string(),
        path.join("kcl1/kcl4/main.k").to_string_lossy().to_string(),
        path.join("kcl1/main.k").to_string_lossy().to_string(),
        path.join("kcl3/main.k").to_string_lossy().to_string(),
        path.join("main.k").to_string_lossy().to_string(),
        "${KCL_MOD}/src/test_data/expand_file_pattern/KCL_MOD".to_string(),
    ];
    let got_paths: Vec<String> = expand_input_files(&input_files)
        .iter()
        .map(|s| s.replace(['/', '\\'], ""))
        .collect();
    let expect_paths: Vec<String> = expected_files
        .iter()
        .map(|s| s.replace(['/', '\\'], ""))
        .collect();
    assert_eq!(got_paths, expect_paths);
}

#[test]
#[cfg(not(windows))]
fn test_expand_input_files() {
    let input_files = vec!["./src/test_data/expand_file_pattern/**/main.k".to_string()];
    let mut expected_files = vec![
        Path::new("src/test_data/expand_file_pattern/kcl1/kcl2/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/kcl3/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/kcl1/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/kcl1/kcl4/main.k")
            .to_string_lossy()
            .to_string(),
    ];
    expected_files.sort();
    let mut input = expand_input_files(&input_files);
    input.sort();
    assert_eq!(input, expected_files);

    let input_files = vec![
        "./src/test_data/expand_file_pattern/kcl1/main.k".to_string(),
        "./src/test_data/expand_file_pattern/**/main.k".to_string(),
    ];
    let mut expected_files = vec![
        Path::new("src/test_data/expand_file_pattern/kcl1/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/kcl1/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/kcl1/kcl2/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/kcl1/kcl4/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/kcl3/main.k")
            .to_string_lossy()
            .to_string(),
        Path::new("src/test_data/expand_file_pattern/main.k")
            .to_string_lossy()
            .to_string(),
    ];
    expected_files.sort();
    let mut input = expand_input_files(&input_files);
    input.sort();
    assert_eq!(input, expected_files);
}

#[test]
fn test_parse_key_value_pair() {
    let cases = [
        (
            "k=v",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"v\"".into(),
            },
        ),
        (
            "k=1",
            KeyValuePair {
                key: "k".to_string(),
                value: "1".into(),
            },
        ),
        (
            "k=None",
            KeyValuePair {
                key: "k".to_string(),
                value: "null".into(),
            },
        ),
        (
            "k=True",
            KeyValuePair {
                key: "k".to_string(),
                value: "true".into(),
            },
        ),
        (
            "k=true",
            KeyValuePair {
                key: "k".to_string(),
                value: "true".into(),
            },
        ),
        (
            "k={\"key\": \"value\"}",
            KeyValuePair {
                key: "k".to_string(),
                value: "{\"key\": \"value\"}".into(),
            },
        ),
        (
            "k=[1, 2, 3]",
            KeyValuePair {
                key: "k".to_string(),
                value: "[1, 2, 3]".into(),
            },
        ),
    ];
    for (value, pair) in cases {
        let result = parse_key_value_pair(value).unwrap();
        assert_eq!(result.key, pair.key);
        assert_eq!(result.value, pair.value);
    }
}

fn clear_path(path: PathBuf) {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .for_each(|e| {
            fs::remove_file(e.path())
                .or_else(|_| fs::remove_dir(e.path()))
                .ok();
        });
}

#[test]
fn test_parse_key_value_pair_fail() {
    let cases = ["=v", "k=", "="];
    for case in cases {
        assert!(parse_key_value_pair(case).is_err());
    }
}

fn test_fill_pkg_maps_for_k_file_with_line() {
    let root_path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_metadata_with_line");

    let main_pkg_path = root_path.join("main_pkg").join("main.k");
    let dep_with_line_path = root_path.join("dep-with-line");

    let mut opts = LoadProgramOptions::default();
    assert_eq!(format!("{:?}", opts.package_maps), "{}");

    let res = fill_pkg_maps_for_k_file(main_pkg_path.clone(), &mut opts);
    assert!(res.is_ok());

    let pkg_maps = opts.package_maps.clone();
    assert_eq!(pkg_maps.len(), 1);
    assert!(pkg_maps.get("dep_with_line").is_some());

    assert_eq!(
        PathBuf::from(pkg_maps.get("dep_with_line").unwrap().clone())
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        dep_with_line_path
            .canonicalize()
            .unwrap()
            .display()
            .to_string()
    );
}

fn test_fill_pkg_maps_for_k_file() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_metadata")
        .join("subdir")
        .join("main.k");

    let vendor_path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("test_vendor");

    env::set_var(
        "KCL_PKG_PATH",
        vendor_path.canonicalize().unwrap().display().to_string(),
    );

    let mut opts = LoadProgramOptions::default();
    assert_eq!(format!("{:?}", opts.package_maps), "{}");

    let res = fill_pkg_maps_for_k_file(path.clone(), &mut opts);
    assert!(res.is_ok());
    let vendor_home = get_vendor_home();

    let pkg_maps = opts.package_maps.clone();
    assert_eq!(pkg_maps.len(), 1);
    assert!(pkg_maps.get("kcl4").is_some());
    assert_eq!(
        PathBuf::from(pkg_maps.get("kcl4").unwrap().clone())
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        PathBuf::from(vendor_home)
            .join("kcl4_v0.0.1")
            .canonicalize()
            .unwrap()
            .display()
            .to_string()
    );

    clear_path(vendor_path.join(".kpm"))
}

#[test]
fn test_lookup_the_nearest_file_dir() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_metadata");
    let result = lookup_the_nearest_file_dir(path.clone(), "kcl.mod");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().display().to_string(),
        path.canonicalize().unwrap().display().to_string()
    );

    let main_path = path.join("subdir").join("main.k");
    let result = lookup_the_nearest_file_dir(main_path, "kcl.mod");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().display().to_string(),
        path.canonicalize().unwrap().display().to_string()
    );
}

#[test]
fn test_fetch_metadata_in_order() {
    test_fetch_metadata();
    println!("test_fetch_metadata() passed");
    test_fill_pkg_maps_for_k_file();
    println!("test_fill_pkg_maps_for_k_file() passed");
    test_fill_pkg_maps_for_k_file_with_line();
    println!("test_fill_pkg_maps_for_k_file_with_line() passed");
    test_update_dependencies();
    println!("test_update_dependencies() passed");
}

fn test_fetch_metadata() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_metadata");

    let vendor_path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("test_vendor");

    env::set_var(
        "KCL_PKG_PATH",
        vendor_path.canonicalize().unwrap().display().to_string(),
    );
    let vendor_home = get_vendor_home();

    let metadata = fetch_metadata(path.clone());
    // Show more information when the test fails.
    println!("{:?}", metadata);
    assert!(metadata.is_ok());
    let pkgs = metadata.unwrap().packages.clone();
    assert_eq!(pkgs.len(), 1);
    assert!(pkgs.get("kcl4").is_some());
    assert_eq!(pkgs.get("kcl4").unwrap().name, "kcl4");
    assert_eq!(
        pkgs.get("kcl4")
            .unwrap()
            .manifest_path
            .clone()
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        PathBuf::from(vendor_home)
            .join("kcl4_v0.0.1")
            .canonicalize()
            .unwrap()
            .display()
            .to_string()
    );
    clear_path(vendor_path.join(".kpm"))
}

#[test]
fn test_fetch_metadata_invalid() {
    let result = panic::catch_unwind(|| {
        let result = fetch_metadata("invalid_path".to_string().into());
        match result {
            Ok(_) => {
                panic!("The method should not return Ok")
            }
            Err(_) => {
                println!("return with an error.")
            }
        }
    });

    match result {
        Ok(_) => println!("no panic"),
        Err(e) => panic!("The method should not panic forever.: {:?}", e),
    }
}

#[test]
fn test_get_pkg_list() {
    assert_eq!(get_pkg_list("./src/test_data/pkg_list/").unwrap().len(), 1);
    assert_eq!(
        get_pkg_list("./src/test_data/pkg_list/...").unwrap().len(),
        3
    );
}

fn test_update_dependencies() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_update");

    let update_mod = update_dependencies(path.clone());
    // Show more information when the test fails.
    println!("{:?}", update_mod);
    assert!(update_mod.is_ok());
}
