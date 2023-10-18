use crate::tests::parse_expr_snapshot;

parse_expr_snapshot!(smoke_test_parsing_expr_0, "1\n");
parse_expr_snapshot!(smoke_test_parsing_expr_1, "\"1\"\n");
parse_expr_snapshot!(named_literal_expr_0, "Undefined");
parse_expr_snapshot!(named_literal_expr_1, "None");
parse_expr_snapshot!(named_literal_expr_2, "True");
parse_expr_snapshot!(named_literal_expr_3, "False");
parse_expr_snapshot!(nonstring_literal_expr, r####"1234"####);
parse_expr_snapshot!(string_literal_expr_0, r####"'1234'"####);
parse_expr_snapshot!(string_literal_expr_1, r####""1234""####);
parse_expr_snapshot!(string_literal_expr_2, r####""1234\n""####);
parse_expr_snapshot!(number_bin_suffix_expr, r####"1234Ki"####);
parse_expr_snapshot!(unary_expr, r####"+1"####);
parse_expr_snapshot!(binary_expr_0, r####"1+2+3"####);
parse_expr_snapshot!(binary_expr_1, r####"1+2*3-4"####);
parse_expr_snapshot!(binary_expr_2, r####"1+2*3/4"####);
parse_expr_snapshot!(binary_expr_3, r####"a or b"####);
parse_expr_snapshot!(binary_expr_4, r####"x == a or b"####);
parse_expr_snapshot!(binary_expr_5, r####"22 > 11 and 111 < 222"####);
parse_expr_snapshot!(binary_expr_6, r####"int(e.value) > 1 and i == 0"####);
parse_expr_snapshot!(binary_expr_7, r####"key in ['key']"####);
parse_expr_snapshot!(binary_expr_8, r####"key not in ['key']"####);
parse_expr_snapshot!(binary_expr_9, r####"1 is 1 and 11 is not 22"####);
parse_expr_snapshot!(binary_expr_10, r####"1 + a and b"####);
parse_expr_snapshot!(binary_expr_with_paren, r####"1*(2+3)-4"####);
parse_expr_snapshot!(logic_expr_0, r####"0 < a < 100"####);
parse_expr_snapshot!(logic_expr_1, r####"0 < a < 100 + a"####);
parse_expr_snapshot!(logic_expr_2, r####"100 > a > 0"####);
parse_expr_snapshot!(logic_expr_3, r####"100 + a > a > 0"####);
parse_expr_snapshot!(logic_expr_4, r####"a is b"####);
parse_expr_snapshot!(logic_expr_5, r####"a is not True"####);
parse_expr_snapshot!(logic_expr_6, r####"not False or a > 0 and b is True"####);
parse_expr_snapshot!(if_expr, r####"1 if true else 2"####);
parse_expr_snapshot!(primary_expr_0, r####"a.b.c"####);
parse_expr_snapshot!(primary_expr_1, r####"'{}'.format(1)"####);
parse_expr_snapshot!(primary_expr_2, r####"str(1).isdigit()"####);
parse_expr_snapshot!(list_expr_0, r####"[1, 2, 3]"####);
parse_expr_snapshot!(list_expr_1, r####"[1, if True: 2, 3]"####);
parse_expr_snapshot!(list_comp_expr_0, r####"[x ** 2 for x in [1, 2, 3]]"####);
parse_expr_snapshot!(list_comp_expr_1, r####"[i for i in [1, 2, 3] if i > 2]"####);
parse_expr_snapshot!(dict_expr, r####"{k0=v0, k1=v1}"####);
parse_expr_snapshot!(
    dict_comp_expr,
    r####"{k: v + 1 for k, v in {k1 = 1, k2 = 2}}"####
);
parse_expr_snapshot!(subscript_expr_0, r####"a[0]"####);
parse_expr_snapshot!(subscript_expr_1, r####"b["k"]"####);
parse_expr_snapshot!(subscript_expr_2, r####"c?[1]"####);
parse_expr_snapshot!(subscript_expr_3, r####"a[1:]"####);
parse_expr_snapshot!(subscript_expr_4, r####"a[:-1]"####);
parse_expr_snapshot!(subscript_expr_5, r####"a[1:len]"####);
parse_expr_snapshot!(subscript_expr_6, r####"a[0:-1]"####);
parse_expr_snapshot!(subscript_expr_7, r####"a[::]"####);
parse_expr_snapshot!(subscript_expr_8, r####"a[1::]"####);
parse_expr_snapshot!(subscript_expr_9, r####"a[:0:]"####);
parse_expr_snapshot!(subscript_expr_10, r####"a[::-1]"####);
parse_expr_snapshot!(subscript_expr_11, r####"a[1::2]"####);
parse_expr_snapshot!(subscript_expr_12, r####"a[:2:1]"####);
parse_expr_snapshot!(subscript_expr_13, r####"a[1:2:]"####);
parse_expr_snapshot!(subscript_expr_14, r####"a[1:3:1]"####);
parse_expr_snapshot!(call_expr_0, r####"func0()"####);
parse_expr_snapshot!(call_expr_1, r####"func1(1)"####);
parse_expr_snapshot!(call_expr_2, r####"func2(x=2)"####);
parse_expr_snapshot!(call_expr_3, r####"func3(1,x=2)"####);
parse_expr_snapshot!(quant_expr_0, r####"all x in collection {x > 0}"####);
parse_expr_snapshot!(quant_expr_1, r####"any y in collection {y < 0}"####);
parse_expr_snapshot!(quant_expr_2, r####"map x in collection {x + 1}"####);
parse_expr_snapshot!(quant_expr_3, r####"filter x in collection {x > 1}"####);
parse_expr_snapshot!(quant_expr_4, r####"filter x in collection {x > 1}"####);
parse_expr_snapshot!(
    quant_expr_5,
    r####"map i, e in [{k1 = "v1", k2 = "v2"}] { e }"####
);
parse_expr_snapshot!(
    quant_expr_6,
    r####"map i, e in [{k1 = "v1", k2 = "v2"}] { e if i > 0 }"####
);
parse_expr_snapshot!(lambda_expr_0, r####"lambda {}"####);
parse_expr_snapshot!(lambda_expr_1, r####"lambda x {}"####);
parse_expr_snapshot!(lambda_expr_2, r####"lambda x: int -> int {x}"####);
parse_expr_snapshot!(
    lambda_expr_3,
    r####"lambda {
    if True:
        _a = 1
    else:
        _a = 2
    _a
}"####
);
parse_expr_snapshot!(
    config_expr_0,
    r####"{
    "name" = {
        "name": "alice"
    },
    "gender" = "female"
}"####
);
parse_expr_snapshot!(
    config_expr_1,
    r####"{
    "name" = {
        "name": "alice"
    }
    "gender" = "female",
}"####
);
parse_expr_snapshot!(
    config_expr_2,
    r####"{
    "name" = {
        "name": "alice",
    }
    "gender" = "female"
}"####
);
parse_expr_snapshot!(
    config_if_expr_0,
    r####"{
    if True:
        a = 1
}"####
);
parse_expr_snapshot!(
    config_if_expr_1,
    r####"{
    if True:
        a = 1
    else:
        a = 2
}"####
);
parse_expr_snapshot!(
    config_if_expr_2,
    r####"{
    if True:
        a = 1
    elif x > 1:
        a = 2
    else:
        a = 3
}"####
);
parse_expr_snapshot!(
    config_if_expr_3,
    r####"{
    if True:
        if False:
            a = 1
}"####
);
parse_expr_snapshot!(schema_expr_0, r####"Schema {}"####);
parse_expr_snapshot!(schema_expr_1, r####"Schema {k=v}"####);
parse_expr_snapshot!(schema_expr_2, r####"Schema () {k=v}"####);
parse_expr_snapshot!(schema_expr_3, r####"Schema (1, 2) {k=v}"####);
parse_expr_snapshot!(
    schema_expr_4,
    r####"Schema (1, 2) {
    k=v
}"####
);
parse_expr_snapshot!(
    line_continue,
    r####"1 + \
2
"####
);
parse_expr_snapshot!(parse_joined_string_0, r####"'${123+200}'"####);
parse_expr_snapshot!(parse_joined_string_1, r####"'abc${a+1}cde'"####);
parse_expr_snapshot!(expr_with_paren_0, r####"(2+3)"####);
parse_expr_snapshot!(expr_with_paren_1, r####"((2+3)"####);
parse_expr_snapshot!(expr_with_paren_2, r####"(2+3))"####);
parse_expr_snapshot!(expr_with_bracket_0, r####"[2,3]"####);
parse_expr_snapshot!(expr_with_bracket_1, r####"[[2,3]"####);
parse_expr_snapshot!(expr_with_bracket_2, r####"[2,3]]"####);
parse_expr_snapshot!(expr_with_bracket_3, r####"[2,3"####);
parse_expr_snapshot!(expr_with_bracket_4, r####"["####);
parse_expr_snapshot!(
    expr_with_bracket_5,
    r####"[
    1
    2,
]
        "####
);
parse_expr_snapshot!(
    expr_with_bracket_6,
    r####"[
    1,2,
]
        "####
);
parse_expr_snapshot!(
    expr_with_bracket_7,
    r####"[
    1,2,

        "####
);
parse_expr_snapshot!(expr_with_brace_0, r####"{a=2}"####);
parse_expr_snapshot!(expr_with_brace_1, r####"{a=2}}"####);
parse_expr_snapshot!(expr_with_delim_0, r####"({a=2}"####);
parse_expr_snapshot!(expr_with_delim_1, r####"({a=(2}"####);
parse_expr_snapshot!(expr_with_delim_2, r####"{a=[2]"####);
parse_expr_snapshot!(expr_with_delim_3, r####"[{a=2}"####);
parse_expr_snapshot!(expr_with_delim_4, r####"({a=[2}"####);
parse_expr_snapshot!(expr_with_delim_5, r####"{"####);
parse_expr_snapshot!(
    expr_with_delim_6,
    r####"{
    a = 1
}"####
);
parse_expr_snapshot!(
    expr_with_delim_7,
    r####"{
    a = 1
"####
);
