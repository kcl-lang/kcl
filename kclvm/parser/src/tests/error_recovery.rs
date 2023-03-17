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
