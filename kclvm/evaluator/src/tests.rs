use crate::Evaluator;
use kclvm_loader::{load_packages, LoadPackageOptions};
use kclvm_parser::LoadProgramOptions;

#[macro_export]
macro_rules! evaluator_snapshot {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            let p = load_packages(&LoadPackageOptions {
                paths: vec!["test.k".to_string()],
                load_opts: Some(LoadProgramOptions {
                    k_code_list: vec![$src.to_string()],
                    ..Default::default()
                }),
                load_builtin: false,
                ..Default::default()
            })
            .unwrap();
            let evaluator = Evaluator::new(&p.program);
            insta::assert_snapshot!(format!("{}", evaluator.run().unwrap().1));
        }
    };
}

evaluator_snapshot! {expr_stmt_0, "1"}
evaluator_snapshot! {expr_stmt_1, "2.0"}
evaluator_snapshot! {expr_stmt_2, "True"}
evaluator_snapshot! {expr_stmt_3, r#"None"#}
evaluator_snapshot! {expr_stmt_4, r#"[1, 2, 3]"#}
evaluator_snapshot! {expr_stmt_5, r#"{k = "v"}"#}

evaluator_snapshot! {assign_stmt_0, "a = 1"}
evaluator_snapshot! {assign_stmt_1, "a = 1 + 1"}
evaluator_snapshot! {assign_stmt_2, "a = (1 + 2)"}
evaluator_snapshot! {assign_stmt_3, r#"a = 1
b = a + 1
"#}
evaluator_snapshot! {assign_stmt_4, r#"a: int = 1
b: int = a + 1
"#}

evaluator_snapshot! {aug_assign_stmt_0, r#"_a = 1
_a += 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_1, r#"_a = 1
_a -= 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_2, r#"_a = 1
_a *= 2
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_3, r#"_a = 2
_a /= 2
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_4, r#"_a = 3
_a %= 2
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_5, r#"_a = 3
_a **= 2
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_6, r#"_a = 3
_a <<= 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_7, r#"_a = 3
_a >>= 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_8, r#"_a = 3
_a |= 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_9, r#"_a = 3
_a ^= 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_10, r#"_a = 3
_a &= 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_11, r#"_a = 3
_a //= 2
a = _a
"#}

evaluator_snapshot! {assert_stmt_0, r#"assert True, "msg"
a = 1
"#}

evaluator_snapshot! {assert_stmt_1, r#"assert False if False, "msg"
a = 1
"#}

evaluator_snapshot! {if_stmt_0, r#"if True:
    a = 1
else:
    b = 2
"#}
evaluator_snapshot! {if_stmt_1, r#"if False:
    a = 1
else:
    b = 2
"#}
evaluator_snapshot! {if_stmt_3, r#"if False:
    a = 1
elif True:
    b = 2
else:
    c = 3
"#}
evaluator_snapshot! {if_stmt_4, r#"if False:
    a = 1
elif False:
    b = 2
else:
    c = 3
"#}
evaluator_snapshot! {if_stmt_5, r#"if False:
    a = 1
else:
    if True:
        b = 2
    else:
        c = 3
"#}

evaluator_snapshot! {import_stmt_0, r#"import math
a = 1
"#}
evaluator_snapshot! {import_stmt_1, r#"import math
import math
b = 2
"#}

evaluator_snapshot! {quant_expr_0, r#"b = all a in [1, 2, 3] {
    a > 0
}
"#}
evaluator_snapshot! {quant_expr_1, r#"b = any a in [1, 2, 3] {
    a > 2
}
"#}
evaluator_snapshot! {quant_expr_2, r#"b = map a in [1, 2, 3] {
    a + 1
}
"#}
evaluator_snapshot! {quant_expr_4, r#"b = filter a in [1, 2, 3] {
    a > 1
}
"#}
