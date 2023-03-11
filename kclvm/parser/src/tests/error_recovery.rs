use crate::tests::parse_expr_snapshot;

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
