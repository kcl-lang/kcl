use std::panic::{catch_unwind, set_hook};

use crate::*;

use core::any::Any;
use expect_test::{expect, Expect};

fn check_parsing_file_ast_json(filename: &str, src: &str, expect: Expect) {
    let m = parse_file(filename, Some(src.into())).unwrap();
    let actual = serde_json::ser::to_string(&m).unwrap();
    let actual = format!("{}\n", actual);
    expect.assert_eq(&actual)
}

#[test]
fn test_parse_file() {
    check_parsing_file_ast_json(
        "hello.k",
        r####"a=123"####,
        expect![[r#"
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":["a"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":1,"column":0,"end_line":1,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":123}}},"filename":"hello.k","line":1,"column":2,"end_line":1,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":1,"column":0,"end_line":1,"end_column":5}],"comments":[]}
        "#]],
    );
}

#[test]
fn test_parse_if_stmt() {
    check_parsing_file_ast_json(
        "hello.k",
        r####"
a = 10
b = 12
_condition = 0
if a == 11 or b == 13: _condition = 1
elif a == 10 and b == 12: _condition = 2
condition = _condition
        "####,
        expect![[r#"
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":["a"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":10}}},"filename":"hello.k","line":2,"column":4,"end_line":2,"end_column":6},"type_annotation":null,"ty":null}},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":6},{"node":{"Assign":{"targets":[{"node":{"names":["b"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":12}}},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":6},"type_annotation":null,"ty":null}},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":6},{"node":{"Assign":{"targets":[{"node":{"names":["_condition"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":4,"column":0,"end_line":4,"end_column":10}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":0}}},"filename":"hello.k","line":4,"column":13,"end_line":4,"end_column":14},"type_annotation":null,"ty":null}},"filename":"hello.k","line":4,"column":0,"end_line":4,"end_column":14},{"node":{"If":{"body":[{"node":{"Assign":{"targets":[{"node":{"names":["_condition"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":5,"column":23,"end_line":5,"end_column":33}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":5,"column":36,"end_line":5,"end_column":37},"type_annotation":null,"ty":null}},"filename":"hello.k","line":5,"column":23,"end_line":5,"end_column":37}],"cond":{"node":{"Binary":{"left":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":["a"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":5,"column":3,"end_line":5,"end_column":4},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":11}}},"filename":"hello.k","line":5,"column":8,"end_line":5,"end_column":10}]}},"filename":"hello.k","line":5,"column":3,"end_line":5,"end_column":21},"op":{"Bin":"Or"},"right":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":["b"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":5,"column":14,"end_line":5,"end_column":15},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":13}}},"filename":"hello.k","line":5,"column":19,"end_line":5,"end_column":21}]}},"filename":"hello.k","line":5,"column":14,"end_line":5,"end_column":21}}},"filename":"hello.k","line":5,"column":3,"end_line":5,"end_column":21},"orelse":[{"node":{"If":{"body":[{"node":{"Assign":{"targets":[{"node":{"names":["_condition"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":6,"column":26,"end_line":6,"end_column":36}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":2}}},"filename":"hello.k","line":6,"column":39,"end_line":6,"end_column":40},"type_annotation":null,"ty":null}},"filename":"hello.k","line":6,"column":26,"end_line":6,"end_column":40}],"cond":{"node":{"Binary":{"left":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":["a"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":6,"column":5,"end_line":6,"end_column":6},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":10}}},"filename":"hello.k","line":6,"column":10,"end_line":6,"end_column":12}]}},"filename":"hello.k","line":6,"column":5,"end_line":6,"end_column":24},"op":{"Bin":"And"},"right":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":["b"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":6,"column":17,"end_line":6,"end_column":18},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":12}}},"filename":"hello.k","line":6,"column":22,"end_line":6,"end_column":24}]}},"filename":"hello.k","line":6,"column":17,"end_line":6,"end_column":24}}},"filename":"hello.k","line":6,"column":5,"end_line":6,"end_column":24},"orelse":[]}},"filename":"hello.k","line":6,"column":0,"end_line":7,"end_column":0}]}},"filename":"hello.k","line":5,"column":0,"end_line":7,"end_column":0},{"node":{"Assign":{"targets":[{"node":{"names":["condition"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":9}],"value":{"node":{"Identifier":{"names":["_condition"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":7,"column":12,"end_line":7,"end_column":22},"type_annotation":null,"ty":null}},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":22}],"comments":[]}
        "#]],
    );

    check_parsing_file_ast_json(
        "hello.k",
        r####"
data2 = {
    **{key = "value1"}
    if a == 123: if b == 456: key = "value2"
}
        "####,
        expect![[r#"
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":["data2"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":5}],"value":{"node":{"Config":{"items":[{"node":{"key":null,"value":{"node":{"Config":{"items":[{"node":{"key":{"node":{"Identifier":{"names":["key"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":3,"column":7,"end_line":3,"end_column":10},"value":{"node":{"StringLit":{"is_long_string":false,"raw_value":"\"value1\"","value":"value1"}},"filename":"hello.k","line":3,"column":13,"end_line":3,"end_column":21},"operation":"Override","insert_index":-1},"filename":"hello.k","line":3,"column":7,"end_line":3,"end_column":21}]}},"filename":"hello.k","line":3,"column":6,"end_line":3,"end_column":22},"operation":"Union","insert_index":-1},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":22},{"node":{"key":null,"value":{"node":{"ConfigIfEntry":{"if_cond":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":["a"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":4,"column":7,"end_line":4,"end_column":8},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":123}}},"filename":"hello.k","line":4,"column":12,"end_line":4,"end_column":15}]}},"filename":"hello.k","line":4,"column":7,"end_line":4,"end_column":15},"items":[{"node":{"key":null,"value":{"node":{"ConfigIfEntry":{"if_cond":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":["b"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":4,"column":20,"end_line":4,"end_column":21},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":456}}},"filename":"hello.k","line":4,"column":25,"end_line":4,"end_column":28}]}},"filename":"hello.k","line":4,"column":20,"end_line":4,"end_column":28},"items":[{"node":{"key":{"node":{"Identifier":{"names":["key"],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":4,"column":30,"end_line":4,"end_column":33},"value":{"node":{"StringLit":{"is_long_string":false,"raw_value":"\"value2\"","value":"value2"}},"filename":"hello.k","line":4,"column":36,"end_line":4,"end_column":44},"operation":"Override","insert_index":-1},"filename":"hello.k","line":4,"column":36,"end_line":4,"end_column":44}],"orelse":null}},"filename":"hello.k","line":4,"column":30,"end_line":5,"end_column":0},"operation":"Override","insert_index":-1},"filename":"hello.k","line":4,"column":30,"end_line":5,"end_column":0}],"orelse":null}},"filename":"hello.k","line":4,"column":17,"end_line":5,"end_column":0},"operation":"Union","insert_index":-1},"filename":"hello.k","line":4,"column":4,"end_line":5,"end_column":0}]}},"filename":"hello.k","line":2,"column":8,"end_line":5,"end_column":1},"type_annotation":null,"ty":null}},"filename":"hello.k","line":2,"column":0,"end_line":5,"end_column":1}],"comments":[]}
        "#]],
    );

    check_parsing_file_ast_json(
        "hello.k",
        r####"
# comment1
a = 1
# comment22
b = 2
# comment333
c = 3 # comment4444
        "####,
        expect![[r###"
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":["a"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":5},{"node":{"Assign":{"targets":[{"node":{"names":["b"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":5,"column":0,"end_line":5,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":2}}},"filename":"hello.k","line":5,"column":4,"end_line":5,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":5,"column":0,"end_line":5,"end_column":5},{"node":{"Assign":{"targets":[{"node":{"names":["c"],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":3}}},"filename":"hello.k","line":7,"column":4,"end_line":7,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":5}],"comments":[{"node":{"text":"# comment1"},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":10},{"node":{"text":"# comment22"},"filename":"hello.k","line":4,"column":0,"end_line":4,"end_column":11},{"node":{"text":"# comment333"},"filename":"hello.k","line":6,"column":0,"end_line":6,"end_column":12},{"node":{"text":"# comment4444"},"filename":"hello.k","line":7,"column":6,"end_line":7,"end_column":19}]}
        "###]],
    );
}

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

const PARSE_EXPR_INVALID_TEST_CASES: &[&str] = &["fs1_i1re1~s", "fh==-h==-", "8_________i"];

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
    "a: int",                // No initial value error
    "a -",                   // Invalid binary expression error
    "a?: int",               // Invalid optional annotation error
    "a: () = 1",             // Type annotation error
    "if a not is b: a = 1",  // Logic operator error
    "if True:\n  a=1\n b=2", // Indent error
    "a[1::::]",              // List slice error
    "a[1 a]",                // List index error
    "{a ++ 1}",              // Config attribute operator error
    "func(a=1,b)",           // Call argument error
    "'${}'",                 // Empty string interpolation error
    "'${a: jso}'",           // Invalid string interpolation format spec error
];

#[test]
pub fn test_parse_file_invalid() {
    for case in PARSE_FILE_INVALID_TEST_CASES {
        let result = parse_file("test.k", Some((&case).to_string()));
        assert!(result.is_err(), "case: {}, result {:?}", case, result)
    }
}
