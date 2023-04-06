use kclvm_config::settings::KeyValuePair;

use crate::arguments::parse_key_value_pair;

#[test]
fn test_parse_key_value_pair() {
    let cases = [
        (
            "k=v",
            KeyValuePair {
                key: "k".to_string(),
                value: "\"v\"".to_string(),
            },
        ),
        (
            "k=1",
            KeyValuePair {
                key: "k".to_string(),
                value: "1".to_string(),
            },
        ),
        (
            "k=None",
            KeyValuePair {
                key: "k".to_string(),
                value: "null".to_string(),
            },
        ),
        (
            "k=True",
            KeyValuePair {
                key: "k".to_string(),
                value: "true".to_string(),
            },
        ),
        (
            "k=true",
            KeyValuePair {
                key: "k".to_string(),
                value: "true".to_string(),
            },
        ),
        (
            "k={\"key\": \"value\"}",
            KeyValuePair {
                key: "k".to_string(),
                value: "{\"key\": \"value\"}".to_string(),
            },
        ),
        (
            "k=[1, 2, 3]",
            KeyValuePair {
                key: "k".to_string(),
                value: "[1, 2, 3]".to_string(),
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
