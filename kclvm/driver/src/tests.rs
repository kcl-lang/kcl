use std::path::{Path, PathBuf};
use std::{env, panic};

use kclvm_config::modfile::get_vendor_home;
use kclvm_config::settings::KeyValuePair;
use kclvm_parser::LoadProgramOptions;

use crate::arguments::parse_key_value_pair;
use crate::canonicalize_input_files;
use crate::kpm_metadata::{fetch_metadata, fill_pkg_maps_for_k_file, lookup_the_nearest_file_dir};

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
        let result = parse_key_value_pair(&value).unwrap();
        assert_eq!(result.key, pair.key);
        assert_eq!(result.value, pair.value);
    }
}

#[test]
fn test_parse_key_value_pair_fail() {
    let cases = ["=v", "k=", "="];
    for case in cases {
        assert!(parse_key_value_pair(case).is_err());
    }
}

#[test]
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
}

#[test]
fn test_lookup_the_nearest_file_dir() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_metadata");
    let result = lookup_the_nearest_file_dir(path.clone(), "kcl.mod");
    assert_eq!(result.is_some(), true);
    assert_eq!(
        result.unwrap().display().to_string(),
        path.canonicalize().unwrap().display().to_string()
    );

    let main_path = path.join("subdir").join("main.k");
    let result = lookup_the_nearest_file_dir(main_path, "kcl.mod");
    assert_eq!(result.is_some(), true);
    assert_eq!(
        result.unwrap().display().to_string(),
        path.canonicalize().unwrap().display().to_string()
    );
}

#[test]
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
    assert_eq!(metadata.is_err(), false);
    let pkgs = metadata.unwrap().packages.clone();
    assert_eq!(pkgs.len(), 1);
    assert!(pkgs.get("kcl4").is_some());
    assert_eq!(pkgs.get("kcl4").clone().unwrap().name, "kcl4");
    assert_eq!(
        PathBuf::from(pkgs.get("kcl4").unwrap().manifest_path.clone())
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
