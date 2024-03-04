use crate::tests::{parse_expr_snapshot, parse_module_snapshot};

/* Expr error recovery */

parse_expr_snapshot! { string_literal_recovery_0, "'abc" }
parse_expr_snapshot! { string_literal_recovery_1, "r'abc" }
parse_expr_snapshot! { string_literal_recovery_2, "'''abc" }
parse_expr_snapshot! { string_literal_recovery_3, "r'''abc" }
parse_expr_snapshot! { string_literal_recovery_4, "r''abc'" }
parse_expr_snapshot! { string_literal_recovery_5, "'" }
parse_expr_snapshot! { string_literal_recovery_6, "'''" }
parse_expr_snapshot! { string_literal_recovery_7, "'\n" }
parse_expr_snapshot! { string_literal_recovery_8, "r'abc\n" }
parse_expr_snapshot! { number_literal_recovery_0, "00" }
parse_expr_snapshot! { number_literal_recovery_1, "00a" }
parse_expr_snapshot! { number_literal_recovery_2, "0x112.3" }
parse_expr_snapshot! { number_literal_recovery_3, "0o" }
parse_expr_snapshot! { number_literal_recovery_4, "0oA" }
parse_expr_snapshot! { number_literal_recovery_5, "0x" }
parse_expr_snapshot! { number_literal_recovery_6, "0xH" }
parse_expr_snapshot! { number_literal_recovery_7, "0e0" }
parse_expr_snapshot! { number_literal_recovery_8, "0b333" }
parse_expr_snapshot! { number_literal_recovery_9, "10KI" }
parse_expr_snapshot! { number_literal_recovery_10, "100mm" }
parse_expr_snapshot! { line_continue_recovery_0, "0x\\2\n12" }
parse_expr_snapshot! { line_continue_recovery_1, "'abc\\ \ndef" }
parse_expr_snapshot! { line_continue_recovery_2, r#"'a' + \
'b'
"# }
parse_expr_snapshot! { line_continue_recovery_3, r#"'a' + \1
'b'
"# }
parse_expr_snapshot! { paren_recovery_0, "(a" }
parse_expr_snapshot! { paren_recovery_1, "(a + 1" }
parse_expr_snapshot! { paren_recovery_2, r#"("# }
parse_expr_snapshot! { paren_recovery_3, r#"(]"# }
parse_expr_snapshot! { paren_recovery_4, r#"(a"# }
parse_expr_snapshot! { paren_recovery_5, r#"(a +"# }
parse_expr_snapshot! { list_recovery_0, "[" }
parse_expr_snapshot! { list_recovery_1, "[0" }
parse_expr_snapshot! { list_recovery_2, "[0,1" }
parse_expr_snapshot! { list_recovery_3, "[[0,1]" }
parse_expr_snapshot! { list_recovery_4, "[[0,1" }
parse_expr_snapshot! { list_recovery_5, r#"[
    0,
    1
    "# }
parse_expr_snapshot! { list_recovery_6, "[0 1]" }
parse_expr_snapshot! { list_recovery_7, "[0,, 1" }
parse_expr_snapshot! { list_recovery_8, "[0 ~ 1" }
parse_expr_snapshot! { list_recovery_9, "[*a, **b]" }
parse_expr_snapshot! { list_recovery_10, "[**a, *b" }
parse_expr_snapshot! { list_recovery_11, "[if True: a, b]" }
parse_expr_snapshot! { list_recovery_12, "[if True: **a, b]" }
parse_expr_snapshot! { config_recovery_0, "{" }
parse_expr_snapshot! { config_recovery_1, "{a = 1" }
parse_expr_snapshot! { config_recovery_2, "{a = 1, b = 2" }
parse_expr_snapshot! { config_recovery_3, "{a = {a = 1}" }
parse_expr_snapshot! { config_recovery_4, "{a = {a = 1" }
parse_expr_snapshot! { config_recovery_5, r#"{
    a = 1
    b = 2
    "# }
parse_expr_snapshot! { config_recovery_6, "{a = 1 b = 2}" }
parse_expr_snapshot! { config_recovery_7, "{a = 1,, b = 2}" }
parse_expr_snapshot! { config_recovery_8, "{a = 1 ~ b = 2}" }
parse_expr_snapshot! { config_recovery_9, "{*a, **b}" }
parse_expr_snapshot! { config_recovery_10, "{**a, *b}" }
parse_expr_snapshot! { config_recovery_11, "{if True: a = , b = 2}" }
parse_expr_snapshot! { config_recovery_12, "{if True: *a, b = 2}" }
parse_expr_snapshot! { config_recovery_13, "{if True: key: {}}" }
parse_expr_snapshot! { config_recovery_14, "{if True: key: []}" }
parse_expr_snapshot! { config_recovery_15, "{你好" }
parse_expr_snapshot! { comp_clause_recovery_0, "[i for i in [1,2,3]]" }
parse_expr_snapshot! { comp_clause_recovery_1, "[i, j for i in [1,2,3]]" }
parse_expr_snapshot! { comp_clause_recovery_2, "[for i in [1,2,3]]" }
parse_expr_snapshot! { comp_clause_recovery_3, "{i for i in [1,2,3]}" }
parse_expr_snapshot! { comp_clause_recovery_4, "{i: for i in [1,2,3]}" }
parse_expr_snapshot! { comp_clause_recovery_5, "{i: 1, j for i in [1,2,3]}" }
parse_expr_snapshot! { comp_clause_recovery_6, "{for i in [1,2,3]}" }
parse_expr_snapshot! { unary_recovery_0, r#"!a"# }
parse_expr_snapshot! { unary_recovery_1, r#"!!a"# }
parse_expr_snapshot! { unary_recovery_2, r#"not (!a)"# }
parse_expr_snapshot! { unary_recovery_3, r#"! (not a)"# }
parse_expr_snapshot! { unary_recovery_5, r#"++i"# }
parse_expr_snapshot! { unary_recovery_6, r#"--i"# }
parse_expr_snapshot! { unary_recovery_7, r#"-+i"# }
parse_expr_snapshot! { unary_recovery_8, r#"~~i"# }
parse_expr_snapshot! { binary_recovery_0, r#"a not is b"# }
parse_expr_snapshot! { binary_recovery_1, r#"a is is not b"# }
parse_expr_snapshot! { binary_recovery_2, r#"a not b"# }
parse_expr_snapshot! { binary_recovery_3, r#"a not is in b"# }
parse_expr_snapshot! { binary_recovery_4, r#"a in in b"# }
parse_expr_snapshot! { binary_recovery_5, r#"a ++ b"# }
parse_expr_snapshot! { binary_recovery_6, r#"a -not- b"# }
parse_expr_snapshot! { binary_recovery_7, r#"a +is b"# }
parse_expr_snapshot! { binary_recovery_8, r#"a +=+ b"# }
parse_expr_snapshot! { compare_recovery_0, r#"a <> b"# }
parse_expr_snapshot! { compare_recovery_1, r#"a < !b >!1"# }
parse_expr_snapshot! { compare_recovery_2, r#"a < !b >!1"# }
parse_expr_snapshot! { compare_recovery_3, r#"a <<< b"# }
parse_expr_snapshot! { compare_recovery_4, r#"a <+< b"# }
parse_expr_snapshot! { compare_recovery_5, r#"a >+ b"# }
parse_expr_snapshot! { compare_recovery_6, r#"<a >+ b"# }
parse_expr_snapshot! { if_recovery_0, r#"1 if"# }
parse_expr_snapshot! { if_recovery_1, r#"1 if"# }
parse_expr_snapshot! { if_recovery_2, r#"1 if True"# }
parse_expr_snapshot! { if_recovery_3, r#"1 if True else"# }
parse_expr_snapshot! { if_recovery_4, r#"if True else"# }
parse_expr_snapshot! { subscript_recovery_0, r#"a[b 1]"# }
parse_expr_snapshot! { subscript_recovery_1, r#"a[1,b]"# }
parse_expr_snapshot! { subscript_recovery_2, r#"a[b;;b]"# }
parse_expr_snapshot! { subscript_recovery_3, r#"a[b[b]"# }
parse_expr_snapshot! { subscript_recovery_4, r#"a[:::]"# }
parse_expr_snapshot! { subscript_recovery_5, r#"a[:1:2:]"# }
parse_expr_snapshot! { subscript_recovery_6, r#"[][a:b:c:d]"# }
parse_expr_snapshot! { subscript_recovery_7, r#"[][]"# }
parse_expr_snapshot! { subscript_recovery_8, r#"[][][]"# }
parse_expr_snapshot! { subscript_recovery_9, r#"[]?[]"# }
parse_expr_snapshot! { subscript_recovery_10, r#"[0]?.[0]"# }
parse_expr_snapshot! { subscript_recovery_11, r#"[0]??[0]"# }
parse_expr_snapshot! { subscript_recovery_12, r#"[0].?[0]"# }
parse_expr_snapshot! { select_recovery_0, r#"a."# }
parse_expr_snapshot! { select_recovery_1, r#"a.b."# }
parse_expr_snapshot! { select_recovery_2, r#"a.b.c."# }
parse_expr_snapshot! { select_recovery_3, r#"''."# }
parse_expr_snapshot! { select_recovery_4, r#"''.lower"# }
parse_expr_snapshot! { select_recovery_5, r#"''.lower()."# }
parse_expr_snapshot! { select_recovery_6, r#"a?."# }
parse_expr_snapshot! { select_recovery_7, r#"a?.b?."# }
parse_expr_snapshot! { select_recovery_8, r#"a?.b?.c?."# }
parse_expr_snapshot! { select_recovery_9, r#"a?"# }
parse_expr_snapshot! { select_recovery_10, r#"a?.b?"# }
parse_expr_snapshot! { select_recovery_11, r#"a?.b?.c?"# }
parse_expr_snapshot! { select_recovery_12, r#"a.0"# }
parse_expr_snapshot! { select_recovery_13, r#"a..0"# }
parse_expr_snapshot! { select_recovery_14, r#"a..."# }
parse_expr_snapshot! { call_recovery_0, r#"a("# }
parse_expr_snapshot! { call_recovery_1, r#"a(]"# }
parse_expr_snapshot! { call_recovery_2, r#"a(a,,)"# }
parse_expr_snapshot! { call_recovery_3, r#"a.b(a=1,2)"# }
parse_expr_snapshot! { call_recovery_4, r#"a(a.ba=1,2)"# }
parse_expr_snapshot! { call_recovery_5, r#"a(a.b+a=1,2)"# }
parse_expr_snapshot! { call_recovery_6, r#"a(a-1.b=1)"# }
parse_expr_snapshot! { call_recovery_7, r#"a(type="list", "key")"# }
parse_expr_snapshot! { schema_recovery_0, r#"s {"# }
parse_expr_snapshot! { schema_recovery_1, r#"s {a=1"# }
parse_expr_snapshot! { schema_recovery_2, r#"s.0 {a=1}"# }
parse_expr_snapshot! { schema_recovery_3, r#"s?.a {a=1}"# }
parse_expr_snapshot! { schema_recovery_4, r#"s. {a=1}"# }
parse_expr_snapshot! { schema_recovery_5, r#"s( {a=1}"# }
parse_expr_snapshot! { schema_recovery_6, r#"s(] {a=1}"# }
parse_expr_snapshot! { joined_string_recovery_0, r#"'${}'"# }
parse_expr_snapshot! { joined_string_recovery_1, r#"'${a +}'"# }
parse_expr_snapshot! { joined_string_recovery_2, r#"'${(a +}'"# }
parse_expr_snapshot! { joined_string_recovery_3, r#"'${a'"# }
parse_expr_snapshot! { joined_string_recovery_5, r#"'${a + 1 = }'"# }
parse_expr_snapshot! { joined_string_recovery_6, r#"'${a: json}'"# }
parse_expr_snapshot! { joined_string_recovery_7, r#"'\n${a: #json}'"# }
parse_expr_snapshot! { joined_string_recovery_8, r#"'a\nb${a: #json}\n'"# }
parse_expr_snapshot! { lambda_recovery_0, r#"lambda"# }
parse_expr_snapshot! { lambda_recovery_1, r#"lambda {"# }
parse_expr_snapshot! { lambda_recovery_2, r#"lambda {}"# }
parse_expr_snapshot! { lambda_recovery_3, r#"{lambda}"# }
parse_expr_snapshot! { lambda_recovery_4, r#"{lambda{}"# }
parse_expr_snapshot! { lambda_recovery_5, r#"{lambda a{}"# }

/* Stmt error recovery */

parse_module_snapshot! { assign_stmt_recovery_0, r#"a = "#}
parse_module_snapshot! { assign_stmt_recovery_1, r#" = 1"#}
parse_module_snapshot! { assign_stmt_recovery_2, r#"a: int ="#}
parse_module_snapshot! { assign_stmt_recovery_3, r#"a: a = 1"#}
parse_module_snapshot! { assign_stmt_recovery_4, r#"a:"#}
parse_module_snapshot! { assign_stmt_recovery_5, r#"a = b = "#}
parse_module_snapshot! { assign_stmt_recovery_6, r#"a() = b. = c"#}
parse_module_snapshot! { assign_stmt_recovery_7, r#"a: () = 0"#}
parse_module_snapshot! { assign_stmt_recovery_8, r#"a: () = 0"#}
parse_module_snapshot! { assign_stmt_recovery_9, r#"a ++= 1"#}
parse_module_snapshot! { assign_stmt_recovery_10, r#"a[0] -= 1"#}
parse_module_snapshot! { assert_stmt_recovery_0, r#"assert"#}
parse_module_snapshot! { assert_stmt_recovery_1, r#"assert a."#}
parse_module_snapshot! { assert_stmt_recovery_2, r#"assert True,,, 'msg'"#}
parse_module_snapshot! { assert_stmt_recovery_3, r#"assert True if data else 'msg'"#}
parse_module_snapshot! { import_stmt_recovery_0, r#"import"#}
parse_module_snapshot! { import_stmt_recovery_1, r#"import 'pkg_path'"#}
parse_module_snapshot! { import_stmt_recovery_2, r#"import pkg_path."#}
parse_module_snapshot! { import_stmt_recovery_3, r#"import pkg_path[0]"#}
parse_module_snapshot! { import_stmt_recovery_4, r#"import .pkg_path."#}
parse_module_snapshot! { import_stmt_recovery_5, r#"import pkg_path as "#}
parse_module_snapshot! { import_stmt_recovery_6, r#"import pkg_path as 'data'"#}
parse_module_snapshot! { type_alias_recovery_0, r#"type"#}
parse_module_snapshot! { type_alias_recovery_1, r#"type 'pkg_path'"#}
parse_module_snapshot! { type_alias_recovery_2, r#"type pkg_path."#}
parse_module_snapshot! { type_alias_recovery_3, r#"type pkg_path[0]"#}
parse_module_snapshot! { type_alias_recovery_4, r#"type .pkg_path."#}
parse_module_snapshot! { type_alias_recovery_5, r#"type pkg_path = "#}
parse_module_snapshot! { type_alias_recovery_6, r#"type pkg_path = 'data'"#}
parse_module_snapshot! { if_stmt_recovery_0, r#"if True a = 1"#}
parse_module_snapshot! { if_stmt_recovery_1, r#"if True: a = 1 else if b = 2"#}
parse_module_snapshot! { if_stmt_recovery_2, r#"if : a = 1"#}
parse_module_snapshot! { if_stmt_recovery_3, r#"if True: a = 1 else b = 2"#}
parse_module_snapshot! { if_stmt_recovery_4, r#"if True: else: b = 2"#}
parse_module_snapshot! { if_stmt_recovery_5, r#"if"#}
parse_module_snapshot! { if_stmt_recovery_6, r#"if else"#}
parse_module_snapshot! { if_stmt_recovery_7, r#"if True:"#}
parse_module_snapshot! { if_stmt_recovery_8, r#"if True: a = 1
else if False: b = 1"#}
parse_module_snapshot! { if_stmt_recovery_9, r#"if True: a = 1
else False: b = 1"#}
parse_module_snapshot! { schema_stmt_recovery_0, r#"schema"#}
parse_module_snapshot! { schema_stmt_recovery_1, r#"schema A"#}
parse_module_snapshot! { schema_stmt_recovery_2, r#"schema A["#}
parse_module_snapshot! { schema_stmt_recovery_3, r#"schema A::"#}
parse_module_snapshot! { schema_stmt_recovery_4, r#"schema A:B"#}
parse_module_snapshot! { schema_stmt_recovery_5, r#"schema A(:"#}
parse_module_snapshot! { schema_stmt_recovery_6, r#"schema A():"#}
parse_module_snapshot! { schema_stmt_recovery_7, r#"schema A:
a:: int"#}
parse_module_snapshot! { schema_stmt_recovery_8, r#"schema A:
a: int ="#}
parse_module_snapshot! { schema_stmt_recovery_9, r#"schema A:
[]: []"#}
parse_module_snapshot! { schema_stmt_recovery_10, r#"schema A:
[str:]: []"#}
parse_module_snapshot! { schema_stmt_recovery_11, r#"schema A:
[str]: str = "#}
parse_module_snapshot! { schema_stmt_recovery_12, r#"schema A:
[str]: = "#}
parse_module_snapshot! { schema_stmt_recovery_13, r#"schema A:
[str]: ''= "#}
parse_module_snapshot! { schema_stmt_recovery_14, r#"schema A:
a??: int "#}
parse_module_snapshot! { schema_stmt_recovery_15, r#"schema A:
a!: int "#}
parse_module_snapshot! { schema_stmt_recovery_16, r#"schema A:
a!!: int "#}
parse_module_snapshot! { schema_stmt_recovery_17, r#"schema A:
a: "#}
parse_module_snapshot! { schema_stmt_recovery_19, r#"@deprecated
schema A:
    a: "#}
parse_module_snapshot! { schema_stmt_recovery_20, r#"@deprecated(
schema A:
    a: "#}
parse_module_snapshot! { schema_stmt_recovery_21, r#"@deprecated(
schema A:
    a: "#}
parse_module_snapshot! { schema_stmt_recovery_22, r#"@deprecated(a
schema A:
    a: "#}
parse_module_snapshot! { schema_stmt_recovery_23, r#"@deprecated(a,
schema A:
    a: "#}
parse_module_snapshot! { schema_stmt_recovery_24, r#"@deprecated((),
schema A:
    a: "#}
parse_module_snapshot! { schema_stmt_recovery_25, r#"
schema A:
    check: "#}
parse_module_snapshot! { schema_stmt_recovery_26, r#"
schema A:
    check: 
        @"#}
parse_module_snapshot! { schema_stmt_recovery_27, r#"
schema A:
    [.str]: str "#}
parse_module_snapshot! { schema_stmt_recovery_28, r#"
schema A:
    [....str]: str "#}
parse_module_snapshot! { schema_stmt_recovery_29, r#"
schema A:
    @"#}
parse_module_snapshot! { schema_stmt_recovery_30, r#"
schema A:
    ."#}
parse_module_snapshot! { schema_stmt_recovery_31, r#"
schema A:
    [str]: str
    [str]: int"#}
parse_module_snapshot! { schema_stmt_recovery_32, r#"
schema A:
    "attr": str"#}
parse_module_snapshot! { schema_stmt_recovery_33, r#"
schema A:
    """Schema Doc"""
    "attr": str"#}
parse_module_snapshot! { schema_stmt_recovery_34, r#"
schema A:
    "attr: str"#}
parse_module_snapshot! { schema_stmt_recovery_35, r#"
schema A:
    "attr":"#}
parse_module_snapshot! { rule_stmt_recovery_0, r#"rule"#}
parse_module_snapshot! { rule_stmt_recovery_1, r#"rule A"#}
parse_module_snapshot! { rule_stmt_recovery_2, r#"rule A["#}
parse_module_snapshot! { rule_stmt_recovery_3, r#"rule A::"#}
parse_module_snapshot! { rule_stmt_recovery_4, r#"rule A:B"#}
parse_module_snapshot! { rule_stmt_recovery_5, r#"rule A(:"#}
parse_module_snapshot! { rule_stmt_recovery_6, r#"
rule A:
    True "#}
parse_module_snapshot! { rule_stmt_recovery_7, r#"
rule A:
    @
"#}
parse_module_snapshot! { fn_ty_annotation_recovery_0, r#"a:("#}
parse_module_snapshot! { fn_ty_annotation_recovery_1, r#"a:(i"#}
parse_module_snapshot! { fn_ty_annotation_recovery_2, r#"a:(int"#}
parse_module_snapshot! { fn_ty_annotation_recovery_3, r#"a:i)"#}
parse_module_snapshot! { fn_ty_annotation_recovery_4, r#"a:([i"#}
parse_module_snapshot! { fn_ty_annotation_recovery_5, r#"a:([i:"#}
parse_module_snapshot! { fn_ty_annotation_recovery_6, r#"a:([i]"#}
parse_module_snapshot! { fn_ty_annotation_recovery_7, r#"a:([int]"#}
parse_module_snapshot! { fn_ty_annotation_recovery_8, r#"a:([int"#}
parse_module_snapshot! { fn_ty_annotation_recovery_9, r#"a:({}"#}
parse_module_snapshot! { fn_ty_annotation_recovery_10, r#"a:({"#}
parse_module_snapshot! { fn_ty_annotation_recovery_11, r#"a:({i"#}
parse_module_snapshot! { fn_ty_annotation_recovery_12, r#"a:({i:"#}
parse_module_snapshot! { fn_ty_annotation_recovery_13, r#"a:({i:i"#}
parse_module_snapshot! { fn_ty_annotation_recovery_14, r#"a:({i:int"#}
parse_module_snapshot! { fn_ty_annotation_recovery_15, r#"a:({i:int]"#}
parse_module_snapshot! { fn_ty_annotation_recovery_16, r#"a:({str:int]"#}
parse_module_snapshot! { fn_ty_annotation_recovery_17, r#"a:({str:int}"#}
parse_module_snapshot! { fn_ty_annotation_recovery_18, r#"a:({str:int} ->"#}
parse_module_snapshot! { fn_ty_annotation_recovery_19, r#"a:({str:int}) -> i"#}
parse_module_snapshot! { fn_ty_annotation_recovery_20, r#"a:(str|int) -> i"#}
parse_module_snapshot! { fn_ty_annotation_recovery_21, r#"a:(str|int, int) -> i"#}
parse_module_snapshot! { fn_ty_annotation_recovery_22, r#"a:(str|int, int|"#}
parse_module_snapshot! { fn_ty_annotation_recovery_23, r#"a:(str|int, int|) ->"#}
parse_module_snapshot! { import_recovery_0, r#"import json as j.a"#}
