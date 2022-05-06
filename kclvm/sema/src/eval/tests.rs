use super::*;

#[test]
fn test_str_literal_eval() {
    let cases = [
        // true cases
        (("'1'", false, false), Some("1".to_string())),
        (("\"1\"", false, false), Some("1".to_string())),
        (("\"1\\n2\"", false, false), Some("1\n2".to_string())),
        (("\"1\\n2\"", false, true), Some("1\\n2".to_string())),
        (("\"1\\2\"", false, true), Some("1\\2".to_string())),
        (("'''1'''", false, false), Some("1".to_string())),
        (("\"\"\"1\"\"\"", false, false), Some("1".to_string())),
        (("\"\"\"1\n2\"\"\"", false, false), Some("1\n2".to_string())),
        // false cases
        (("", false, false), None),
        (("'", false, false), None),
        (("'1", false, false), None),
        (("\"1", false, false), None),
        (("\"1\\\"", false, false), None),
        (("'''1''", false, false), None),
        (("\"\"\"1\"\"", false, false), None),
    ];
    for ((input, is_bytes, is_raw), expected) in cases {
        assert_eq!(
            str_literal_eval(input, is_bytes, is_raw),
            expected,
            "test failed, input: {}",
            input
        )
    }
}
