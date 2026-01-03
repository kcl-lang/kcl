use std::panic;
use std::path::PathBuf;

use kcl_config::settings::KeyValuePair;

use crate::arguments::parse_key_value_pair;
use crate::toolchain::NativeToolchain;
use crate::toolchain::Toolchain;
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
        // Test scientific notation - should be treated as string
        (
            "k=12e1",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"12e1\"".into(),
            },
        ),
        (
            "k=1.5e-3",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"1.5e-3\"".into(),
            },
        ),
        (
            "k=2E10",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"2E10\"".into(),
            },
        ),
    ];
    for (value, pair) in cases {
        let result = parse_key_value_pair(value).unwrap();
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
