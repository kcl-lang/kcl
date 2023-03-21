use crate::tests::parse_expr_snapshot;

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
parse_expr_snapshot! { config_recovery_0, "{" }
parse_expr_snapshot! { config_recovery_1, "{a = 1" }
parse_expr_snapshot! { config_recovery_2, "{a = 1, b = 2" }
parse_expr_snapshot! { config_recovery_3, "{a = {a = 1}" }
parse_expr_snapshot! { config_recovery_4, "{a = {a = 1" }
parse_expr_snapshot! { config_recovery_5, r#"{
    a = 1
    b = 2
    "# }
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
