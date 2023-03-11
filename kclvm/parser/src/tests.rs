use std::panic::{catch_unwind, set_hook};

use crate::*;

use core::any::Any;

pub fn check_result_panic_info(result: Result<(), Box<dyn Any + Send>>) {
    match result {
        Err(e) => match e.downcast::<String>() {
            Ok(_v) => {
                let got = _v.to_string();
                let _u: PanicInfo = serde_json::from_str(&got).unwrap();
            }
            _ => unreachable!(),
        },
        _ => {}
    };
}

const PARSE_EXPR_INVALID_TEST_CASES: &[&str] =
    &["fs1_i1re1~s", "fh==-h==-", "8_________i", "1MM", "0x00x"];

#[test]
pub fn test_parse_expr_invalid() {
    for case in PARSE_EXPR_INVALID_TEST_CASES {
        set_hook(Box::new(|_| {}));
        let result = catch_unwind(|| {
            parse_expr(&case);
        });
        check_result_panic_info(result);
    }
}

const PARSE_FILE_INVALID_TEST_CASES: &[&str] = &[
    "a: int",                   // No initial value error
    "a -",                      // Invalid binary expression error
    "a?: int",                  // Invalid optional annotation error
    "a: () = 1",                // Type annotation error
    "if a not is not b: a = 1", // Logic operator error
    "if True:\n  a=1\n b=2",    // Indent error
    "a[1::::]",                 // List slice error
    "a[1 a]",                   // List index error
    "{a ++ 1}",                 // Config attribute operator error
    "func(a=1,b)",              // Call argument error
    "'${}'",                    // Empty string interpolation error
    "'${a: jso}'",              // Invalid string interpolation format spec error
];

#[test]
pub fn test_parse_file_invalid() {
    for case in PARSE_FILE_INVALID_TEST_CASES {
        let result = parse_file("test.k", Some((&case).to_string()));
        assert!(result.is_err(), "case: {}, result {:?}", case, result)
    }
}
