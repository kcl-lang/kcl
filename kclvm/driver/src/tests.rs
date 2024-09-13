use std::path::PathBuf;
use std::{env, fs, panic};

use kclvm_config::modfile::get_vendor_home;
use kclvm_config::settings::KeyValuePair;
use kclvm_parser::LoadProgramOptions;
use walkdir::WalkDir;

use crate::arguments::parse_key_value_pair;
use crate::toolchain::Toolchain;
use crate::toolchain::{fill_pkg_maps_for_k_file, CommandToolchain, NativeToolchain};
use crate::{get_pkg_list, lookup_the_nearest_file_dir, toolchain};

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

    let res = fill_pkg_maps_for_k_file(&toolchain::default(), main_pkg_path.clone(), &mut opts);
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

#[test]
fn test_native_fill_pkg_maps_for_k_file_with_line() {
    let root_path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_metadata_with_line");

    let main_pkg_path = root_path.join("main_pkg").join("main.k");
    let dep_with_line_path = root_path.join("dep-with-line");

    let mut opts = LoadProgramOptions::default();
    assert_eq!(format!("{:?}", opts.package_maps), "{}");

    let res = fill_pkg_maps_for_k_file(
        &NativeToolchain::default(),
        main_pkg_path.clone(),
        &mut opts,
    );
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

    let res = fill_pkg_maps_for_k_file(&toolchain::default(), path.clone(), &mut opts);
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
    test_cmd_tool_fetch_metadata();
    println!("test_cmd_tool_fetch_metadata() passed");
    test_native_tool_fetch_metadata();
    println!("test_native_tool_fetch_metadata() passed");
    test_fill_pkg_maps_for_k_file();
    println!("test_fill_pkg_maps_for_k_file() passed");
    test_native_fill_pkg_maps_for_k_file_with_line();
    println!("test_native_fill_pkg_maps_for_k_file_with_line() passed");
    test_fill_pkg_maps_for_k_file_with_line();
    println!("test_fill_pkg_maps_for_k_file_with_line() passed");
    test_native_update_dependencies();
    println!("test_native_update_dependencies() passed");
    test_update_dependencies();
    println!("test_update_dependencies() passed");
}

fn test_cmd_tool_fetch_metadata() {
    test_tool_fetch_metadata(CommandToolchain::default())
}

fn test_native_tool_fetch_metadata() {
    test_tool_fetch_metadata(NativeToolchain::default())
}

fn test_tool_fetch_metadata(tool: impl Toolchain) {
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
    let metadata = tool.fetch_metadata(path.clone());
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
        let tool = toolchain::default();
        let result = tool.fetch_metadata("invalid_path".to_string().into());
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
fn test_native_fetch_metadata_invalid() {
    let result = panic::catch_unwind(|| {
        let tool = NativeToolchain::default();
        let result = tool.fetch_metadata("invalid_path".to_string().into());
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

    let tool = toolchain::default();
    let update_mod = tool.update_dependencies(path.clone());
    // Show more information when the test fails.
    println!("{:?}", update_mod);
    assert!(update_mod.is_ok());
}

fn test_native_update_dependencies() {
    let path = PathBuf::from(".")
        .join("src")
        .join("test_data")
        .join("kpm_update");

    let tool = NativeToolchain::default();
    tool.update_dependencies(path.clone()).unwrap();
}
