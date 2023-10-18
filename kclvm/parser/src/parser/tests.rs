use crate::parse_file;
use crate::session::ParseSession;
use expect_test::{expect, Expect};
use regex::Regex;
use std::sync::Arc;

fn check_parsing_file_ast_json(filename: &str, src: &str, expect: Expect) {
    let m = crate::parse_file_with_global_session(
        Arc::new(ParseSession::default()),
        filename,
        Some(src.into()),
    )
    .unwrap();
    let actual = serde_json::ser::to_string(&m).unwrap();
    let actual = format!("{actual}\n");
    expect.assert_eq(&actual)
}

fn check_parsing_module(filename: &str, src: &str, expect: &str) {
    let m = crate::parse_file(filename, Some(src.to_string())).expect(filename);
    let actual = format!("{}\n", serde_json::ser::to_string(&m).unwrap());
    assert_eq!(actual.trim(), expect.trim());
}

#[test]
fn test_parse_schema_stmt() {
    check_parsing_file_ast_json(
        "hello.k",
        r####"
schema TestBool:
    []
    [str    ]: int
    [a: str]: int
    [a: ...str]: int
    [...str]: int
    a: int
    b?: str
    c: int = 0
    d?: str = ""

    [a]
    [a, b, c]
    [
        1
    ]
    [
        a
    ]
    [a for a in [1, 2, 3]]
    [
        a for a in [1, 2, 3]
    ]

    check:
        a > 1, "msg"
        name not None, "we fail here"
        "####,
        expect![[r#"
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Schema":{"doc":"","name":{"node":"TestBool","filename":"hello.k","line":2,"column":7,"end_line":2,"end_column":15},"parent_name":null,"for_host_name":null,"is_mixin":false,"is_protocol":false,"args":null,"mixins":[],"body":[{"node":{"Expr":{"exprs":[{"node":{"List":{"elts":[],"ctx":"Load"}},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":6}]}},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":6},{"node":{"SchemaAttr":{"doc":"","name":{"node":"a","filename":"hello.k","line":8,"column":4,"end_line":8,"end_column":5},"type_str":{"node":"int","filename":"hello.k","line":8,"column":7,"end_line":8,"end_column":10},"op":null,"value":null,"is_optional":false,"decorators":[],"ty":{"node":{"Basic":"Int"},"filename":"hello.k","line":8,"column":7,"end_line":8,"end_column":10}}},"filename":"hello.k","line":8,"column":4,"end_line":8,"end_column":10},{"node":{"SchemaAttr":{"doc":"","name":{"node":"b","filename":"hello.k","line":9,"column":4,"end_line":9,"end_column":5},"type_str":{"node":"str","filename":"hello.k","line":9,"column":8,"end_line":9,"end_column":11},"op":null,"value":null,"is_optional":true,"decorators":[],"ty":{"node":{"Basic":"Str"},"filename":"hello.k","line":9,"column":8,"end_line":9,"end_column":11}}},"filename":"hello.k","line":9,"column":4,"end_line":10,"end_column":0},{"node":{"SchemaAttr":{"doc":"","name":{"node":"c","filename":"hello.k","line":10,"column":4,"end_line":10,"end_column":5},"type_str":{"node":"int","filename":"hello.k","line":10,"column":7,"end_line":10,"end_column":10},"op":{"Aug":"Assign"},"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":0}}},"filename":"hello.k","line":10,"column":13,"end_line":10,"end_column":14},"is_optional":false,"decorators":[],"ty":{"node":{"Basic":"Int"},"filename":"hello.k","line":10,"column":7,"end_line":10,"end_column":10}}},"filename":"hello.k","line":10,"column":4,"end_line":10,"end_column":14},{"node":{"SchemaAttr":{"doc":"","name":{"node":"d","filename":"hello.k","line":11,"column":4,"end_line":11,"end_column":5},"type_str":{"node":"str","filename":"hello.k","line":11,"column":8,"end_line":11,"end_column":11},"op":{"Aug":"Assign"},"value":{"node":{"StringLit":{"is_long_string":false,"raw_value":"\"\"","value":""}},"filename":"hello.k","line":11,"column":14,"end_line":11,"end_column":16},"is_optional":true,"decorators":[],"ty":{"node":{"Basic":"Str"},"filename":"hello.k","line":11,"column":8,"end_line":11,"end_column":11}}},"filename":"hello.k","line":11,"column":4,"end_line":13,"end_column":0},{"node":{"Expr":{"exprs":[{"node":{"List":{"elts":[{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":13,"column":5,"end_line":13,"end_column":6}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":13,"column":5,"end_line":13,"end_column":6}],"ctx":"Load"}},"filename":"hello.k","line":13,"column":4,"end_line":13,"end_column":7}]}},"filename":"hello.k","line":13,"column":4,"end_line":13,"end_column":7},{"node":{"Expr":{"exprs":[{"node":{"List":{"elts":[{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":14,"column":5,"end_line":14,"end_column":6}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":14,"column":5,"end_line":14,"end_column":6},{"node":{"Identifier":{"names":[{"node":"b","filename":"hello.k","line":14,"column":8,"end_line":14,"end_column":9}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":14,"column":8,"end_line":14,"end_column":9},{"node":{"Identifier":{"names":[{"node":"c","filename":"hello.k","line":14,"column":11,"end_line":14,"end_column":12}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":14,"column":11,"end_line":14,"end_column":12}],"ctx":"Load"}},"filename":"hello.k","line":14,"column":4,"end_line":14,"end_column":13}]}},"filename":"hello.k","line":14,"column":4,"end_line":14,"end_column":13},{"node":{"Expr":{"exprs":[{"node":{"List":{"elts":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":16,"column":8,"end_line":16,"end_column":9}],"ctx":"Load"}},"filename":"hello.k","line":15,"column":4,"end_line":17,"end_column":5}]}},"filename":"hello.k","line":15,"column":4,"end_line":17,"end_column":5},{"node":{"Expr":{"exprs":[{"node":{"List":{"elts":[{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":19,"column":8,"end_line":19,"end_column":9}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":19,"column":8,"end_line":19,"end_column":9}],"ctx":"Load"}},"filename":"hello.k","line":18,"column":4,"end_line":20,"end_column":5}]}},"filename":"hello.k","line":18,"column":4,"end_line":20,"end_column":5},{"node":{"Expr":{"exprs":[{"node":{"ListComp":{"elt":{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":21,"column":5,"end_line":21,"end_column":6}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":21,"column":5,"end_line":21,"end_column":6},"generators":[{"node":{"targets":[{"node":{"names":[{"node":"a","filename":"hello.k","line":21,"column":11,"end_line":21,"end_column":12}],"pkgpath":"","ctx":"Load"},"filename":"hello.k","line":21,"column":11,"end_line":21,"end_column":12}],"iter":{"node":{"List":{"elts":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":21,"column":17,"end_line":21,"end_column":18},{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":2}}},"filename":"hello.k","line":21,"column":20,"end_line":21,"end_column":21},{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":3}}},"filename":"hello.k","line":21,"column":23,"end_line":21,"end_column":24}],"ctx":"Load"}},"filename":"hello.k","line":21,"column":16,"end_line":21,"end_column":25},"ifs":[]},"filename":"hello.k","line":21,"column":7,"end_line":21,"end_column":25}]}},"filename":"hello.k","line":21,"column":4,"end_line":21,"end_column":26}]}},"filename":"hello.k","line":21,"column":4,"end_line":21,"end_column":26},{"node":{"Expr":{"exprs":[{"node":{"ListComp":{"elt":{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":23,"column":8,"end_line":23,"end_column":9}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":23,"column":8,"end_line":23,"end_column":9},"generators":[{"node":{"targets":[{"node":{"names":[{"node":"a","filename":"hello.k","line":23,"column":14,"end_line":23,"end_column":15}],"pkgpath":"","ctx":"Load"},"filename":"hello.k","line":23,"column":14,"end_line":23,"end_column":15}],"iter":{"node":{"List":{"elts":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":23,"column":20,"end_line":23,"end_column":21},{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":2}}},"filename":"hello.k","line":23,"column":23,"end_line":23,"end_column":24},{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":3}}},"filename":"hello.k","line":23,"column":26,"end_line":23,"end_column":27}],"ctx":"Load"}},"filename":"hello.k","line":23,"column":19,"end_line":23,"end_column":28},"ifs":[]},"filename":"hello.k","line":23,"column":10,"end_line":24,"end_column":0}]}},"filename":"hello.k","line":22,"column":4,"end_line":24,"end_column":5}]}},"filename":"hello.k","line":22,"column":4,"end_line":24,"end_column":5}],"decorators":[],"checks":[{"node":{"test":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":27,"column":8,"end_line":27,"end_column":9}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":27,"column":8,"end_line":27,"end_column":9},"ops":["Gt"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":27,"column":12,"end_line":27,"end_column":13}]}},"filename":"hello.k","line":27,"column":8,"end_line":27,"end_column":13},"if_cond":null,"msg":{"node":{"StringLit":{"is_long_string":false,"raw_value":"\"msg\"","value":"msg"}},"filename":"hello.k","line":27,"column":15,"end_line":27,"end_column":20}},"filename":"hello.k","line":27,"column":8,"end_line":27,"end_column":20},{"node":{"test":{"node":{"Identifier":{"names":[{"node":"name","filename":"hello.k","line":28,"column":8,"end_line":28,"end_column":12}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":28,"column":8,"end_line":28,"end_column":12},"if_cond":null,"msg":null},"filename":"hello.k","line":28,"column":8,"end_line":28,"end_column":12},{"node":{"test":{"node":{"Unary":{"op":"Not","operand":{"node":{"NameConstantLit":{"value":"None"}},"filename":"hello.k","line":28,"column":17,"end_line":28,"end_column":21}}},"filename":"hello.k","line":28,"column":13,"end_line":28,"end_column":21},"if_cond":null,"msg":{"node":{"StringLit":{"is_long_string":false,"raw_value":"\"we fail here\"","value":"we fail here"}},"filename":"hello.k","line":28,"column":23,"end_line":28,"end_column":37}},"filename":"hello.k","line":28,"column":13,"end_line":28,"end_column":37}],"index_signature":{"node":{"key_name":null,"key_type":{"node":"str","filename":"hello.k","line":7,"column":8,"end_line":7,"end_column":11},"value_type":{"node":"int","filename":"hello.k","line":7,"column":14,"end_line":7,"end_column":17},"value":null,"any_other":true,"value_ty":{"node":{"Basic":"Int"},"filename":"hello.k","line":7,"column":14,"end_line":7,"end_column":17}},"filename":"hello.k","line":7,"column":4,"end_line":8,"end_column":0}}},"filename":"hello.k","line":2,"column":0,"end_line":29,"end_column":8}],"comments":[]}
        "#]],
    );
}

#[test]
fn test_parse_assign_stmt() {
    check_parsing_file_ast_json(
        "hello.k",
        r####"a=123"####,
        expect![[r#"
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"a","filename":"hello.k","line":1,"column":0,"end_line":1,"end_column":1}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":1,"column":0,"end_line":1,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":123}}},"filename":"hello.k","line":1,"column":2,"end_line":1,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":1,"column":0,"end_line":1,"end_column":5}],"comments":[]}
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
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"a","filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":1}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":10}}},"filename":"hello.k","line":2,"column":4,"end_line":2,"end_column":6},"type_annotation":null,"ty":null}},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":6},{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"b","filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":1}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":12}}},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":6},"type_annotation":null,"ty":null}},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":6},{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"_condition","filename":"hello.k","line":4,"column":0,"end_line":4,"end_column":10}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":4,"column":0,"end_line":4,"end_column":10}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":0}}},"filename":"hello.k","line":4,"column":13,"end_line":4,"end_column":14},"type_annotation":null,"ty":null}},"filename":"hello.k","line":4,"column":0,"end_line":4,"end_column":14},{"node":{"If":{"body":[{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"_condition","filename":"hello.k","line":5,"column":23,"end_line":5,"end_column":33}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":5,"column":23,"end_line":5,"end_column":33}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":5,"column":36,"end_line":5,"end_column":37},"type_annotation":null,"ty":null}},"filename":"hello.k","line":5,"column":23,"end_line":5,"end_column":37}],"cond":{"node":{"Binary":{"left":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":5,"column":3,"end_line":5,"end_column":4}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":5,"column":3,"end_line":5,"end_column":4},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":11}}},"filename":"hello.k","line":5,"column":8,"end_line":5,"end_column":10}]}},"filename":"hello.k","line":5,"column":3,"end_line":5,"end_column":21},"op":{"Bin":"Or"},"right":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":[{"node":"b","filename":"hello.k","line":5,"column":14,"end_line":5,"end_column":15}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":5,"column":14,"end_line":5,"end_column":15},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":13}}},"filename":"hello.k","line":5,"column":19,"end_line":5,"end_column":21}]}},"filename":"hello.k","line":5,"column":14,"end_line":5,"end_column":21}}},"filename":"hello.k","line":5,"column":3,"end_line":5,"end_column":21},"orelse":[{"node":{"If":{"body":[{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"_condition","filename":"hello.k","line":6,"column":26,"end_line":6,"end_column":36}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":6,"column":26,"end_line":6,"end_column":36}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":2}}},"filename":"hello.k","line":6,"column":39,"end_line":6,"end_column":40},"type_annotation":null,"ty":null}},"filename":"hello.k","line":6,"column":26,"end_line":6,"end_column":40}],"cond":{"node":{"Binary":{"left":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":6,"column":5,"end_line":6,"end_column":6}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":6,"column":5,"end_line":6,"end_column":6},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":10}}},"filename":"hello.k","line":6,"column":10,"end_line":6,"end_column":12}]}},"filename":"hello.k","line":6,"column":5,"end_line":6,"end_column":24},"op":{"Bin":"And"},"right":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":[{"node":"b","filename":"hello.k","line":6,"column":17,"end_line":6,"end_column":18}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":6,"column":17,"end_line":6,"end_column":18},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":12}}},"filename":"hello.k","line":6,"column":22,"end_line":6,"end_column":24}]}},"filename":"hello.k","line":6,"column":17,"end_line":6,"end_column":24}}},"filename":"hello.k","line":6,"column":5,"end_line":6,"end_column":24},"orelse":[]}},"filename":"hello.k","line":6,"column":0,"end_line":7,"end_column":0}]}},"filename":"hello.k","line":5,"column":0,"end_line":7,"end_column":0},{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"condition","filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":9}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":9}],"value":{"node":{"Identifier":{"names":[{"node":"_condition","filename":"hello.k","line":7,"column":12,"end_line":7,"end_column":22}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":7,"column":12,"end_line":7,"end_column":22},"type_annotation":null,"ty":null}},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":22}],"comments":[]}
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
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"data2","filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":5}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":5}],"value":{"node":{"Config":{"items":[{"node":{"key":null,"value":{"node":{"Config":{"items":[{"node":{"key":{"node":{"Identifier":{"names":[{"node":"key","filename":"hello.k","line":3,"column":7,"end_line":3,"end_column":10}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":3,"column":7,"end_line":3,"end_column":10},"value":{"node":{"StringLit":{"is_long_string":false,"raw_value":"\"value1\"","value":"value1"}},"filename":"hello.k","line":3,"column":13,"end_line":3,"end_column":21},"operation":"Override","insert_index":-1},"filename":"hello.k","line":3,"column":7,"end_line":3,"end_column":21}]}},"filename":"hello.k","line":3,"column":6,"end_line":3,"end_column":22},"operation":"Union","insert_index":-1},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":22},{"node":{"key":null,"value":{"node":{"ConfigIfEntry":{"if_cond":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":[{"node":"a","filename":"hello.k","line":4,"column":7,"end_line":4,"end_column":8}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":4,"column":7,"end_line":4,"end_column":8},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":123}}},"filename":"hello.k","line":4,"column":12,"end_line":4,"end_column":15}]}},"filename":"hello.k","line":4,"column":7,"end_line":4,"end_column":15},"items":[{"node":{"key":null,"value":{"node":{"ConfigIfEntry":{"if_cond":{"node":{"Compare":{"left":{"node":{"Identifier":{"names":[{"node":"b","filename":"hello.k","line":4,"column":20,"end_line":4,"end_column":21}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":4,"column":20,"end_line":4,"end_column":21},"ops":["Eq"],"comparators":[{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":456}}},"filename":"hello.k","line":4,"column":25,"end_line":4,"end_column":28}]}},"filename":"hello.k","line":4,"column":20,"end_line":4,"end_column":28},"items":[{"node":{"key":{"node":{"Identifier":{"names":[{"node":"key","filename":"hello.k","line":4,"column":30,"end_line":4,"end_column":33}],"pkgpath":"","ctx":"Load"}},"filename":"hello.k","line":4,"column":30,"end_line":4,"end_column":33},"value":{"node":{"StringLit":{"is_long_string":false,"raw_value":"\"value2\"","value":"value2"}},"filename":"hello.k","line":4,"column":36,"end_line":4,"end_column":44},"operation":"Override","insert_index":-1},"filename":"hello.k","line":4,"column":30,"end_line":4,"end_column":44}],"orelse":null}},"filename":"hello.k","line":4,"column":17,"end_line":5,"end_column":0},"operation":"Override","insert_index":-1},"filename":"hello.k","line":4,"column":17,"end_line":5,"end_column":0}],"orelse":null}},"filename":"hello.k","line":4,"column":4,"end_line":5,"end_column":0},"operation":"Union","insert_index":-1},"filename":"hello.k","line":4,"column":4,"end_line":5,"end_column":0}]}},"filename":"hello.k","line":2,"column":8,"end_line":5,"end_column":1},"type_annotation":null,"ty":null}},"filename":"hello.k","line":2,"column":0,"end_line":5,"end_column":1}],"comments":[]}
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
        {"filename":"hello.k","pkg":"__main__","doc":"","name":"__main__","body":[{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"a","filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":1}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":1}}},"filename":"hello.k","line":3,"column":4,"end_line":3,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":3,"column":0,"end_line":3,"end_column":5},{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"b","filename":"hello.k","line":5,"column":0,"end_line":5,"end_column":1}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":5,"column":0,"end_line":5,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":2}}},"filename":"hello.k","line":5,"column":4,"end_line":5,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":5,"column":0,"end_line":5,"end_column":5},{"node":{"Assign":{"targets":[{"node":{"names":[{"node":"c","filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":1}],"pkgpath":"","ctx":"Store"},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":1}],"value":{"node":{"NumberLit":{"binary_suffix":null,"value":{"Int":3}}},"filename":"hello.k","line":7,"column":4,"end_line":7,"end_column":5},"type_annotation":null,"ty":null}},"filename":"hello.k","line":7,"column":0,"end_line":7,"end_column":5}],"comments":[{"node":{"text":"# comment1"},"filename":"hello.k","line":2,"column":0,"end_line":2,"end_column":10},{"node":{"text":"# comment22"},"filename":"hello.k","line":4,"column":0,"end_line":4,"end_column":11},{"node":{"text":"# comment333"},"filename":"hello.k","line":6,"column":0,"end_line":6,"end_column":12},{"node":{"text":"# comment4444"},"filename":"hello.k","line":7,"column":6,"end_line":7,"end_column":19}]}
        "###]],
    );
}

#[test]
fn test_parse_file() {
    let filenames = vec![
        "testdata/assert-01.k",
        "testdata/assert-02.k",
        "testdata/assert-03.k",
        "testdata/assert-if-0.k",
        "testdata/assert-if-1.k",
        "testdata/assert-if-2.k",
        "testdata/assign-01.k",
        "testdata/config_expr-01.k",
        "testdata/config_expr-02.k",
        "testdata/config_expr-03.k",
        "testdata/config_expr-04.k",
        "testdata/import-01.k",
        "testdata/if-01.k",
        "testdata/if-02.k",
        "testdata/if-03.k",
        "testdata/type-01.k",
        "testdata/hello_win.k",
    ];
    for filename in filenames {
        let code = std::fs::read_to_string(filename).unwrap();
        let expect = std::fs::read_to_string(filename.to_string() + ".json").unwrap();
        check_parsing_module(
            filename.trim_start_matches("testdata/"),
            code.as_str(),
            expect.as_str(),
        );
    }
}

#[test]
fn test_parse_file_not_found() {
    match parse_file("The file path is invalid", None) {
        Ok(_) => {
            panic!("unreachable")
        }
        Err(err_msg) => {
            assert!(
                Regex::new(r"^Failed to load KCL file 'The file path is invalid'. Because.*")
                    .unwrap()
                    .is_match(&err_msg)
            );
        }
    }
}
