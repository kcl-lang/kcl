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
evaluator_snapshot! {quant_expr_2, r#"b = all a in [1, 2, 3] {
    a > 5
}
"#}
evaluator_snapshot! {quant_expr_3, r#"b = any a in [1, 2, 3] {
    a > 5
}
"#}
evaluator_snapshot! {quant_expr_4, r#"b = map a in [1, 2, 3] {
    a + 1
}
"#}
evaluator_snapshot! {quant_expr_5, r#"b = filter a in [1, 2, 3] {
    a > 1
}
"#}
evaluator_snapshot! {quant_expr_6, r#"b = map a in [1, 2, 3] {
    a ** 2
}
"#}
evaluator_snapshot! {quant_expr_7, r#"b = filter a in [1, 2, 3] {
    a == 0
}
"#}

evaluator_snapshot! {if_expr_0, r#"a = 1 if True else 0"#}
evaluator_snapshot! {if_expr_1, r#"a = 1 if False else 0"#}
evaluator_snapshot! {if_expr_2, r#"a = 1 if False else 0 if False else 2"#}

evaluator_snapshot! {unary_expr_0, r#"a = +1"#}
evaluator_snapshot! {unary_expr_1, r#"a = -1"#}
evaluator_snapshot! {unary_expr_2, r#"a = ~1"#}
evaluator_snapshot! {unary_expr_3, r#"a = not None"#}

evaluator_snapshot! {binary_expr_0, r#"a = 1 + 1 * 2 - 4"#}
evaluator_snapshot! {binary_expr_1, r#"a = None or {}
b = [] and {}
"#}

evaluator_snapshot! {selector_expr_0, r#"a = {k = "v"}.k
b = {k = "v"}?.k
c = None?.k
"#}
evaluator_snapshot! {selector_expr_1, r#"a = [1, 2, 3]?[::-1]
b = a?[-1]
c = a?[0]
d = None?[1]
"#}

evaluator_snapshot! {subscript_expr_0, r#"a = [1, 2, 3][::-1]
b = a[-1]
c = a[0]
"#}
evaluator_snapshot! {subscript_expr_1, r#"a = [1, 2, 3]?[::-1]
b = a?[-1]
c = a?[0]
d = None?[1]
"#}

evaluator_snapshot! {compare_expr_0, r#"a = 1 < 10
b = 1 < 10 < 100
c = 1 > 10 > 100
d = 1 < 10 > 100
"#}

evaluator_snapshot! {paren_expr_0, r#"a = 2 * (1 + 1)
b = (((1 + 1))) * 2
"#}

evaluator_snapshot! {list_expr_0, r#"a = [1, 2, 3]
b = [1, if True: 2, 3]
c = [1, if False: 2, 3]
d = [1, *[2, 3]]
"#}

evaluator_snapshot! {dict_expr_0, r#"a = {k1 = "v1", k2 = "v2"}
b = {k1 = "v1", if True: k2 = "v2"}
c = {k1 = "v1", if False: k2 = "v2"}
d = {k1 = "v1", **{k2 = "v2"}}
"#}

evaluator_snapshot! {loop_0, r#"a = [i ** 2 for i in [1, 2, 3]]"#}
evaluator_snapshot! {loop_1, r#"a = [i + j for i in [1, 2, 3] for j in [1, 2, 3] if i < j]"#}

evaluator_snapshot! {literal_0, r#"longStringStartWithNewline = """\
This is the second line
This is the third line
"""
"#}
evaluator_snapshot! {literal_1, r#"a = {k = "v"}
b = "${a: #json}"
"#}
evaluator_snapshot! {literal_2, r#"a = 1Mi
b = 2K
"#}

evaluator_snapshot! {lambda_0, r#"f = lambda x {x * 2}
a = f(1)
b = f(2)
"#}
evaluator_snapshot! {lambda_1, r#"a = lambda x {x * 2}(1)
b = lambda x {x * 2}(2)
"#}
evaluator_snapshot! {lambda_2, r#"import math
a = math.log(10)
b = len("abc")
c = len([1, 2])
"#}
evaluator_snapshot! {lambda_3, r#"
x = lambda {
    a = 1
    lambda {
        a + 1
    }()
}()
"#}
evaluator_snapshot! {lambda_4, r#"
x = lambda {
    a = 1
    b = 2
    lambda x {
        a + b + x
    }(3)
}()
"#}
evaluator_snapshot! {lambda_5, r#"
func = lambda config: {str:} {
    x = 1
    lambda {
        y = 1
        lambda {
            z = 1
            lambda {
                {value = x + y + z + config.key}
            }()
        }()
    }()
}

x = func({key = 1})
"#}

evaluator_snapshot! {schema_0, r#"
schema Person:
    name: str = "Alice"
    age: int = 10

alice = Person {}
bob = Person {
    name = "Bob"
    age = 18
}
"#}
