use crate::tests::parse_file_ast_json_snapshot;

parse_file_ast_json_snapshot!(
    schema_stmt,
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
        "####
);
parse_file_ast_json_snapshot!(assign_stmt, "hello.k", r####"a=123"####);
parse_file_ast_json_snapshot!(
    if_stmt_0,
    "hello.k",
    r####"
a = 10
b = 12
_condition = 0
if a == 11 or b == 13: _condition = 1
elif a == 10 and b == 12: _condition = 2
condition = _condition
        "####
);
parse_file_ast_json_snapshot!(
    if_stmt_1,
    "hello.k",
    r####"
data2 = {
    **{key = "value1"}
    if a == 123: if b == 456: key = "value2"
}
    "####
);
parse_file_ast_json_snapshot!(
    basic_stmt,
    "hello.k",
    r####"
# comment1
a = 1
# comment22
b = 2
# comment333
c = 3 # comment4444
    "####
);
