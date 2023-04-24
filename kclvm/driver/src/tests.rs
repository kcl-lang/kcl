use std::path::Path;

use kclvm_config::settings::KeyValuePair;

use crate::arguments::parse_key_value_pair;
use crate::canonicalize_input_files;

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
