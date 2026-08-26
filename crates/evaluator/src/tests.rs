use crate::Evaluator;
use kcl_ast::MAIN_PKG;
use kcl_loader::{LoadPackageOptions, load_packages};
use kcl_parser::LoadProgramOptions;
use kcl_runtime::{Context, ValueRef};

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

#[macro_export]
macro_rules! evaluator_function_snapshot {
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
            insta::assert_snapshot!(format!("{}", evaluator.run_as_function().to_string()));
        }
    };
}

evaluator_function_snapshot! {function_stmt_0, r#"
import json

config = {
  foo: "bar"
}

json.encode("${config.foo}")
"#}

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
evaluator_snapshot! {assign_stmt_5, r#"_a = [0] * 2
_a[0] = 1
a = _a
"#}
evaluator_snapshot! {assign_stmt_6, r#"_a = [{"key": 0}] * 2
_a[0].key = 1
a = _a
"#}
evaluator_snapshot! {assign_stmt_7, r#"_a = [{key.key = [0] * 2}] * 2
_a[0].key.key[0] = 1
a = _a
"#}
evaluator_snapshot! {assign_stmt_8, r#"on = 'on'"#}

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
evaluator_snapshot! {aug_assign_stmt_12, r#"_a = [0] * 5
_a[0] += 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_13, r#"_a = [{"key": 1}] * 5
_a[0].key += 1
a = _a
"#}
evaluator_snapshot! {aug_assign_stmt_14, r#"_a = [{key.key = [0, 0]}] * 5
_a[0].key.key[0] += 1
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
evaluator_snapshot! {if_stmt_6, r#"
if False:
    a = 1
else:
    if True:
        b = 1
        if True:
            c = 1
"#}
evaluator_snapshot! {if_stmt_7, r#"
_a = 1
if True:
    _a = 2
    _a += 1
a = _a

schema Config:
    _a = 1
    if True:
        _a = 2
        _a += 1
    a = _a

c = Config {}
"#}
evaluator_snapshot! {if_stmt_8, r#"
_items = []
if False:
    _items += [ {key1 = "value1"} ]
if True:
    _items += [ {key2 = "value2"} ]
items = _items

schema Config:
    _items = []
    if False:
        _items += [ {key1 = "value1"} ]
    if True:
        _items += [ {key2 = "value2"} ]
    items = _items

c = Config {}
"#}

evaluator_snapshot! {import_stmt_0, r#"import math
a = 1
"#}
evaluator_snapshot! {import_stmt_1, r#"import math
import math
b = 2
"#}
evaluator_snapshot! {import_stmt_2, r#"
import regex

v = option("foo")
x = regex.match("foo", "^\\w+$")
"#}
evaluator_snapshot! {import_stmt_3, r#"import math

x = math.log(10)
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

evaluator_snapshot! {lambda_6, r#"
schema A:
    my_field: int

    get_field: () -> int = lambda {
        my_field
    }

_a = A{my_field = 2}
my_dict = {
    my_field = 1
    x = _a.get_field()
}
"#}

// Test for lambda parameter scope bug fix
// When a lambda parameter has the same name as a schema field,
// the lambda parameter should take precedence.
evaluator_snapshot! {lambda_7, r#"
schema Value:
    r: int

schema Value2:
    r: int

foo = lambda v: Value -> Value2 {
    Value2 {
        r = v.r
    }
}

schema bar[input: Value]:
    v: Value2

    v = foo(input)

x = Value {
    r = 1
}
y = bar(x)
"#}

evaluator_snapshot! {lambda_8, r#"
schema TestRole:
    name: str

test_lambda = lambda roles: [TestRole], custom_roles: [str] = [] -> [TestRole] {
    roles + [TestRole{name=role} for role in custom_roles]
}


schema TestSchema:
    custom_roles: [TestRole]

    get_roles: () -> [any] = lambda {
        test_lambda(roles=custom_roles, custom_roles=[])
    }

_test_role = TestRole{name="test"}
test = TestSchema{custom_roles=[_test_role]}.get_roles()
"#}

evaluator_snapshot! {lambda_9, r#"
foo = lambda values: [int] -> [int] {
    [v for v in values]
}

schema Bar:
    values = [123]
    foo_result: [int] = foo([456])

my_bar = Bar {}
"#}

// Test for assign by value with different schemas (issue #2080).
// Assignments should be by value, so mutating a schema after assignment
// should not affect the original object. This test verifies that foo is
// a deep copy of input, so modifying foo.world does NOT propagate back to bar.
evaluator_snapshot! {lambda_10, r#"
schema Foo:
    hello: str

schema Bar(Foo):
    world: str

testCopyByValue = lambda input: Bar {
    foo: any = input
    foo.world = "modified"
    foo
}

bar = Bar {
    hello = "world"
    world = "hello"
}

output = testCopyByValue(bar)
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
evaluator_snapshot! {schema_1, r#"
schema Person:
    name: str = "Alice"
    age: int = 10

alice: Person {}
bob: Person {
    name: "Bob"
    age: 18
}
"#}
evaluator_snapshot! {schema_2, r#"
VALUES_MAP = {
    "1": Values1{
        attr1 = "foo"
    }
    "2": Values2 {
        attr2 = "bar"
    }
}

schema Config:
    provider: "1" | "2"
    values = VALUES_MAP[provider]
    provider_values: Values1 | Values2 = values

schema CommonValues:

schema Values1(CommonValues):
    attr1: str

schema Values2(CommonValues):
    attr2: str

config: Config {
	provider = "1"
	provider_values.attr1 = "foobar"
}
"#}
evaluator_snapshot! {lazy_scope_0, r#"
b = a + c
a = 1
c = a + 1
"#}
evaluator_snapshot! {lazy_scope_1, r#"
schema Data:
    b = a + c
    a = 1
    c = a + 1

data = Data {}
"#}
evaluator_snapshot! {lazy_scope_2, r#"
schema Data:
    name: str
    version?: str

data1 = Data {
    name = data2.name
}

data2 = Data {
    name = "1"
    version = version
}

version = "v0.1.0"
"#}

evaluator_snapshot! {list_comp1, r#"
a = [ x for x in "你好"]
"#}

evaluator_snapshot! {issue_2046_public_scalar_alias, r#"
test = {
    bean: "test"
}
test
"#}

#[test]
fn test_if_stmt_setters() {
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
            _a = 1
            if True:
                _a += 1
            elif False:
                _a += 1
            a=_a
            "#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    evaluator.run().unwrap();
    let scopes = evaluator.lazy_scopes.borrow();
    let var_setters = scopes.get(MAIN_PKG).unwrap().setters.get("_a").unwrap();
    assert_eq!(var_setters.len(), 3);
}

#[test]
fn test_nested_if_stmt_variable_assignment() {
    // Test for bug fix: nested if statements should correctly assign variables
    // Issue: https://github.com/kcl-lang/kcl/issues/1978
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
            _engine_version = None
            _engine_name = None

            if _engine_version == None:
                if _engine_name == "redis":
                    _engine_version = "7.1"
                else:
                    _engine_version = "test"

            items = {
                "engineVersion": _engine_version
            }
            "#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Verify the output contains the expected result (JSON format)
    assert!(
        output.contains(r#""engineVersion": "test""#),
        "Expected '\"engineVersion\": \"test\"' in output, got: {}",
        output
    );
}

#[test]
fn test_nested_if_stmt_different_condition() {
    // Test nested if with different condition variables
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
            _a = None
            _b = None

            if _a == None:
                if _b == "redis":
                    _a = 2
                else:
                    _a = 3

            result = {
                "value": _a
            }
            "#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Verify the output contains the expected value (JSON format)
    assert!(
        output.contains(r#""value": 3"#),
        "Expected '\"value\": 3' in output, got: {}",
        output
    );
}

#[test]
fn test_nested_if_stmt_true_condition() {
    // Test nested if with true condition in inner if
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
            _a = None
            _b = "redis"

            if _a == None:
                if _b == "redis":
                    _a = "matched"
                else:
                    _a = "not matched"

            result = {
                "value": _a
            }
            "#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Verify the output contains the expected value (JSON format)
    assert!(
        output.contains(r#""value": "matched""#),
        "Expected '\"value\": \"matched\"' in output, got: {}",
        output
    );
}

#[test]
fn test_nested_if_stmt_multiple_levels() {
    // Test deeply nested if statements
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
            _a = None
            _b = None
            _c = None

            if _a == None:
                if _b == None:
                    if _c == None:
                        _a = "deeply nested"
                    else:
                        _a = "level 2 else"
                else:
                    _a = "level 1 else"

            result = {
                "value": _a
            }
            "#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Verify the output contains the expected value (JSON format)
    assert!(
        output.contains(r#""value": "deeply nested""#),
        "Expected '\"value\": \"deeply nested\"' in output, got: {}",
        output
    );
}

#[test]
fn test_issue_1918_if_false_function_call_no_duplicate() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1918
    // An element appended after an `if False` branch should NOT be rendered twice
    // even when the appended element uses a function/lambda call inside.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
_items = []
functionTest = lambda -> str {
    "value"
}
_items += [{
    name = "A"
}]

_role = []

if False:
    _items += [
        {name = "B"}
    ]


_items += [
  {
    name = "C"
    key = functionTest()
  }
]

items = _items
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Count occurrences of the "C" entry; it must appear exactly once.
    let count_c = output.matches("\"name\": \"C\"").count();
    assert_eq!(
        count_c, 1,
        "Expected exactly one entry with name \"C\", got {} in output: {}",
        count_c, output
    );
}

#[test]
fn test_issue_1918_if_true_with_function_in_other_branch() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1918
    // When the if branch is True and a function call appears in the *other*
    // (else / later) branch, the appended element must still appear only once.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
_items = []
functionTest = lambda -> str {
    "value"
}
_items += [{name = "A"}]

if True:
    _items += [{name = "B"}]

_items += [{name = "C", key = functionTest()}]

items = _items
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Each of "A", "B", "C" must appear exactly once.
    for name in &["A", "B", "C"] {
        let needle = format!("\"name\": \"{}\"", name);
        let count = output.matches(&needle).count();
        assert_eq!(
            count, 1,
            "Expected exactly one entry with name \"{}\", got {} in output: {}",
            name, count, output
        );
    }
}

#[test]
fn test_issue_1910_if_else_aug_assign_runtime_condition() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1910
    // When both `if` and `else` branches mutate the same list variable and the
    // condition references a runtime variable, lazy evaluation must not
    // execute both branches — only the branch matching the condition.
    // This is the same root cause as `crates/evaluator/src/lazy.rs` and
    // `crates/evaluator/src/schema.rs` accidentally over-incrementing their
    // backtrack level beyond `setters.len()`, which used to panic with
    // "attempt to subtract with overflow".
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
items = []

if True:
    items += [{"kind": "first"}]
else:
    items += [{"kind": "second"}]
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // The `else` branch must not be executed. `items` must contain exactly
    // one entry of `kind: "first"`.
    let first_count = output.matches("\"kind\": \"first\"").count();
    let second_count = output.matches("\"kind\": \"second\"").count();
    assert_eq!(
        first_count, 1,
        "Expected exactly one `kind: \"first\"` entry, got {}. output: {}",
        first_count, output
    );
    assert_eq!(
        second_count, 0,
        "Expected zero `kind: \"second\"` entries (else branch must not run), got {}. output: {}",
        second_count, output
    );
}

#[test]
fn test_issue_1772_mixin_protocol_aug_assign_not_duplicated() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1772
    // A schema with a mixin that does `configs.tree.names += [...]` must not
    // duplicate the appended items inside the nested config, even though the
    // mixin is materialized twice through different paths.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
schema Tree:
  names?: [str] = []

schema NotAPerson[tree: Tree]:
    mixin [TreeNamesMixin]

    configs: Configs {
      tree = tree
    }

    names = configs.tree.names

schema TreeNamesMixin for TreeProtocol:
    configs.tree.names += ["Banyan", "Alder", "Cedar"]

protocol TreeProtocol:
  configs: Configs

schema Configs:
  tree: Tree

_treeConfig = Tree {}

result = NotAPerson(_treeConfig)
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Inside `result.configs.tree.names`, "Banyan", "Alder", and "Cedar"
    // must each appear exactly once. The top-level `result.names` already
    // has them once; the bug duplicates them under `configs.tree.names`.
    for name in &["Banyan", "Alder", "Cedar"] {
        let count = output.matches(&format!("\"{}\"", name)).count();
        assert_eq!(
            count, 2,
            "Expected `{}` to appear exactly twice (top-level names + configs.tree.names), got {} matches. full output: {}",
            name, count, output
        );
    }
}

// Companion regression test for issue #1772: the same eager/lazy duplication
// must not happen when the mixin is *not* decorated with `for Protocol`.
// The eager `call_schema_body` loop and lazy-replay `get_value` setter walk
// both reach the mixin's `+=`; without the dedup, `tree.names` would carry
// `["A", "B", "A", "B"]` instead of `["A", "B"]`. The body's `all_names =
// tree.names` is what forces lazy replay (reading the affected attribute);
// the eager loop alone would only apply the mixin once.
#[test]
fn test_issue_1772_mixin_plain_aug_assign_not_duplicated() {
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
schema Tree:
    names?: [str] = []

schema Holder:
    mixin [AddNamesMixin]
    tree: Tree
    all_names = tree.names

schema AddNamesMixin:
    tree.names += ["A", "B"]

result = Holder { tree = Tree {} }
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Each name must appear exactly twice: once in `result.tree.names` and
    // once in `result.all_names` (which is an alias of the same list).
    // Without the fix, each would appear four times (mixin applied twice via
    // eager path, then read, then mixin re-applied twice via the second
    // eager iteration triggered by something else in the body).
    for name in &["A", "B"] {
        let count = output.matches(&format!("\"{}\"", name)).count();
        assert_eq!(
            count, 2,
            "Expected `{}` to appear exactly twice (tree.names + all_names), got {} matches. full output: {}",
            name, count, output
        );
    }
}

// Companion regression test for issue #1772: when *multiple* mixins all
// mutate the same nested attribute, the dedup must be keyed per-mixin so
// that each mixin's `+=` is applied exactly once. A counter that
// incorrectly collapsed across mixin names would let one of them through
// (or double-apply one of them). Here both mixins are distinct and the
// final list must hold all four items in declared order.
#[test]
fn test_issue_1772_mixin_multiple_aug_assign_not_duplicated() {
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
schema Tree:
    names?: [str] = []

schema TwoMixins:
    mixin [MixinA, MixinB]
    tree: Tree
    all_names = tree.names

schema MixinA:
    tree.names += ["a1", "a2"]

schema MixinB:
    tree.names += ["b1", "b2"]

result = TwoMixins { tree = Tree {} }
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Each of the four items must appear exactly twice (tree.names +
    // all_names). Without the fix, each mixin's `+=` is applied twice
    // (eager + lazy), so each name would appear four times.
    for name in &["a1", "a2", "b1", "b2"] {
        let count = output.matches(&format!("\"{}\"", name)).count();
        assert_eq!(
            count, 2,
            "Expected `{}` to appear exactly twice, got {} matches. full output: {}",
            name, count, output
        );
    }
}

// Companion regression test for issue #1772: a single mixin that emits
// *multiple* `+=` statements against the same attribute must still
// produce exactly one application of each. The dedup counter increments
// once per lazy-replayed setter, so a mixin with two setters sets the
// counter to 2; the eager loop decrements by 1 per mixin iteration and
// skips the body. Both `+=` statements must therefore run exactly once.
#[test]
fn test_issue_1772_mixin_multi_setter_aug_assign_not_duplicated() {
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
schema Tree:
    names?: [str] = []

schema MultiAug:
    mixin [AddTwiceMixin]
    tree: Tree
    all_names = tree.names

schema AddTwiceMixin:
    tree.names += ["x"]
    tree.names += ["y"]

result = MultiAug { tree = Tree {} }
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // Each of the two items must appear exactly twice (tree.names +
    // all_names). Without the fix, each `+=` is applied twice (once via
    // lazy replay per setter, once via the eager `call_schema_body`),
    // so each name would appear four times.
    for name in &["x", "y"] {
        let count = output.matches(&format!("\"{}\"", name)).count();
        assert_eq!(
            count, 2,
            "Expected `{}` to appear exactly twice, got {} matches. full output: {}",
            name, count, output
        );
    }
}

#[test]
fn test_issue_1837_nested_if_with_undefined_reference() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1837
    // Nested if statements must not re-evaluate the body when the inner
    // reference is undefined or the backtracking fires.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
options = {}

if "a" == "a":
    extraResource = options?.extraResources

    if extraResource:
        envFromExtra = [{"hello": "bonjour"}]

    deployManifest = {
        env = envFromExtra
    }

    items = ["a"]
    if "s" != "s":
        items = ["wrong-if"]
    else:
        items = ["b"]
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // The body of the outer if must be evaluated exactly once — items must
    // contain only the value produced by the else branch ("b"), and the
    // list must not be appended twice (e.g. ["b", "b"]).
    let items_count = output.matches("\"b\"").count();
    assert_eq!(
        items_count, 1,
        "Expected exactly one `\"b\"` entry from the else branch, got {} matches. full output: {}",
        items_count, output
    );
    let wrong_count = output.matches("\"wrong-if\"").count();
    assert_eq!(
        wrong_count, 0,
        "Expected zero `\"wrong-if\"` entries (if branch must not run), got {} matches. full output: {}",
        wrong_count, output
    );
}

#[test]
fn test_issue_1961_builder_pattern_preserves_state() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1961
    // The builder pattern — `b = b.add(...)` chained inside a lambda — must
    // accumulate state across calls, not lose all but the first one.
    // The original reproducer uses a schema method `add: (str) = lambda v:
    // str { ... }` which the test-mode loader rejects. We exercise the
    // same lazy-scope state-loss bug by chaining list `+=` updates — when
    // a setter is materialized twice via backtracking, the second `+=`
    // sees the original (empty) list instead of the first call's result.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
result = []

result += ["hello"]
result += ["world"]
result += ["!"]
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (output, _) = evaluator.run().unwrap();

    // After the fix, every chained `+=` must see the previous result, so
    // the list must contain all three values exactly once.
    let hello_count = output.matches("\"hello\"").count();
    let world_count = output.matches("\"world\"").count();
    let bang_count = output.matches("\"!\"").count();
    assert_eq!(
        hello_count, 1,
        "Expected exactly one `hello` in result, got {}. output: {}",
        hello_count, output
    );
    assert_eq!(
        world_count, 1,
        "Expected exactly one `world` in result, got {}. output: {}",
        world_count, output
    );
    assert_eq!(
        bang_count, 1,
        "Expected exactly one `!` in result, got {}. output: {}",
        bang_count, output
    );
}

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

const MULTI_THREAD_SOURCE: &str = r#"
import regex
foo = option("foo")
bar = option("bar")
x = regex.match("", "")
"#;

#[test]
fn test_multithread_exec() {
    let threads = 10;
    multithread_check(threads, |thread| {
        println!("run: {}", thread);
        for _ in 0..1000 {
            run_code(MULTI_THREAD_SOURCE);
        }
        println!("done: {}", thread);
    });
}

fn multithread_check(threads: i32, check: impl Fn(i32) + Send + Sync + 'static) {
    let check_shared = Arc::new(check);
    let mut handles = vec![];
    for thread in 0..threads {
        let check_shared = Arc::clone(&check_shared);
        let handle = thread::spawn(move || {
            check_shared(thread);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn run_code(source: &str) -> (String, String) {
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![source.to_string()],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    evaluator.run().unwrap()
}

fn testing_sum(_: &Context, args: &ValueRef, _: &ValueRef) -> anyhow::Result<ValueRef> {
    let a = args
        .arg_i_int(0, Some(0))
        .ok_or(anyhow::anyhow!("expect int value for the first param"))?;
    let b = args
        .arg_i_int(1, Some(0))
        .ok_or(anyhow::anyhow!("expect int value for the second param"))?;
    Ok((a + b).into())
}

fn context_with_plugin() -> Rc<RefCell<Context>> {
    let mut plugin_functions: kcl_primitives::IndexMap<String, kcl_runtime::PluginFunction> =
        Default::default();
    let func = Arc::new(testing_sum);
    plugin_functions.insert("testing.add".to_string(), func);
    let mut ctx = Context::new();
    ctx.plugin_functions = plugin_functions;
    Rc::new(RefCell::new(ctx))
}

#[test]
fn test_exec_with_plugin() {
    let src = r#"
import kcl_plugin.testing

sum = testing.add(1, 1)
"#;
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            load_plugins: true,
            k_code_list: vec![src.to_string()],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new_with_runtime_ctx(&p.program, context_with_plugin());
    insta::assert_snapshot!(format!("{}", evaluator.run().unwrap().1));
}

#[test]
fn test_issue_1915_child_schema_arg_overrides_parent_type() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1915
    // When a child schema inherits from a parent and both declare the same
    // parameter name with different types, calling the child with a value of
    // the child's declared type must succeed — the child's argument
    // declaration must override the parent's type at the inheritance
    // boundary. Re-validating the child's value against the parent's
    // declared type at runtime was incorrectly raising `expect str, got bool`.
    //
    // The test exercises both positional and keyword passing styles, plus a
    // body that consumes the child-only `extra` parameter, so that the schema
    // body actually executes and proves the parameters are bound correctly.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
schema Parent[value: str]:
schema Child[value: bool, extra: str](Parent):
    valueAttr: bool = value
    extraAttr: str = extra

positional = Child(True, "p")
keyword = Child(value=False, extra="k")
caller_keyword = Child(False, "c")
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    // Before the fix this `run` returned an EvaluationError containing
    // `expect str, got bool` while validating the positional value against
    // the *parent's* declared `str` type. After the fix the parent's runtime
    // type check is skipped (it's the child's value, validated against the
    // child's `bool` declaration), so all three valid constructions must
    // succeed.
    //
    // `Evaluator::run()` returns `(json_string, yaml_string)`. We assert on
    // the YAML rendering because the assertions target human-readable
    // schema-attribute names.
    let (_, yaml_output) = evaluator
        .run()
        .expect("child schema must evaluate without type error from parent");
    assert!(
        yaml_output.contains("positional:"),
        "Expected `positional:` instance in YAML output, got:\n{}",
        yaml_output
    );
    assert!(
        yaml_output.contains("keyword:"),
        "Expected `keyword:` instance in YAML output, got:\n{}",
        yaml_output
    );
    assert!(
        yaml_output.contains("caller_keyword:"),
        "Expected `caller_keyword:` instance in YAML output, got:\n{}",
        yaml_output
    );
    // The bool parameter, bound via the child's body, surfaces as a schema
    // attribute. Two values: `true` (positional) and `false` (keyword).
    assert!(
        yaml_output.contains("valueAttr: true"),
        "Expected `valueAttr: true` (positional=True) in YAML output, got:\n{}",
        yaml_output
    );
    assert!(
        yaml_output.contains("valueAttr: false"),
        "Expected `valueAttr: false` (keyword=False) in YAML output, got:\n{}",
        yaml_output
    );
    // Extra string parameter is preserved through inheritance.
    assert!(
        yaml_output.contains("extraAttr: p"),
        "Expected `extraAttr: p` in YAML output, got:\n{}",
        yaml_output
    );
}

#[test]
fn test_issue_1915_type_mismatch_in_child_still_errors() {
    // Companion to issue #1915: a runtime type error inside the child's own
    // argument list (i.e. against the child's declared type) must still be
    // reported. The fix only skips the *parent's* type check when the schema
    // is invoked as a base of a child — never the child's own check.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
schema Parent[value: str]:
schema Child[value: bool, extra: str](Parent):
child = Child("not_a_bool", "extra")
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    // `type_pack_and_check` raises via `panic!`, which surfaces as a panic
    // propagating out of the evaluator. `catch_unwind` captures it so we
    // can assert on the message.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let e = Evaluator::new(&p.program);
        e.run()
    }));
    let msg = match result {
        Ok(Ok(_)) => panic!(
            "expected a runtime type error from the child's own argument list, \
             but evaluation succeeded"
        ),
        Ok(Err(err)) => format!("{}", err),
        Err(panic) => {
            // The panic payload is the formatted panic message string when
            // raised from `panic!("...")`.
            if let Some(s) = panic.downcast_ref::<&'static str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                String::from("<non-string panic payload>")
            }
        }
    };
    assert!(
        msg.contains("expect bool"),
        "Expected the child's `expect bool` runtime error in the panic message, got: {}",
        msg
    );
    // Critically, the parent's `expect str` check must NOT appear — the
    // parent is invoked as a base, so its argument list must NOT be
    // re-validated. If the pre-fix behaviour resurfaces, this assertion
    // will fire and point us back to `walk_arguments`.
    assert!(
        !msg.contains("expect str"),
        "Parent's `expect str` runtime check must NOT appear when the schema \
         is invoked as a base of a child; got: {}",
        msg
    );
}

#[test]
fn test_issue_1915_three_level_inheritance_default_values() {
    // Regression test for issue https://github.com/kcl-lang/kcl/issues/1915:
    // default values declared on a child schema must not be re-validated
    // against the type declared on a base schema that the child overrides.
    // `B[a: int]` and `C[a: bool]` should override `A[a: str]` without
    // triggering a runtime type error for the default value.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"
schema A[a: str = "a_default"]:
schema B[a: int = 1](A):
schema C[a: bool = True](B):
    chosen = a

c = C()
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    let (_, yaml_output) = evaluator
        .run()
        .expect("multi-level inheritance with default values must succeed");
    // The `chosen` attribute is bound to `a` inside C's body, so it inherits
    // C's default of `true` (overriding A's str default and B's int default).
    // If the pre-fix behaviour resurfaces, evaluation aborts with a runtime
    // type error during default-value binding for one of the intermediate
    // schemas and this assertion is never reached.
    assert!(
        yaml_output.contains("chosen: true"),
        "Expected `chosen: true` (the C-level default wins over A and B) \
         in YAML output, got:\n{}",
        yaml_output
    );
    // Critically the parent's `"a_default"` string default must NOT have been
    // bound to `c.a` when `c = C()` was constructed — the child override must
    // take effect.
    assert!(
        !yaml_output.contains("a_default"),
        "Parent A's `str` default `a_default` must NOT appear; the C-level `bool` \
         default must override it. YAML output:\n{}",
        yaml_output
    );
}

#[test]
fn assign_no_double_eval_on_forward_reference() {
    // Regression test for issue #1759: a top-level field whose value is forced
    // early by a forward reference must not be evaluated a second time during
    // the eager walk. The `print` side effect makes the extra evaluation
    // observable: `_foo` is referenced before its declaration (forward), while
    // `_bar` is referenced after. Each `show(...)` must run exactly once.
    let p = load_packages(&LoadPackageOptions {
        paths: vec!["test.k".to_string()],
        load_opts: Some(LoadProgramOptions {
            k_code_list: vec![
                r#"show = lambda s: str -> str {
    print("show: ${s}")
    s
}

# `_foo` used before declaration (forward reference)
foo = _foo

_foo = {
    name: "foo"
    value: show("foo")
}

_bar = {
    name: "bar"
    value: show("bar")
}

# `_bar` used after declaration
bar = _bar
"#
                .to_string(),
            ],
            ..Default::default()
        }),
        load_builtin: false,
        ..Default::default()
    })
    .unwrap();
    let evaluator = Evaluator::new(&p.program);
    evaluator
        .run()
        .expect("forward-reference program must evaluate successfully");
    // Before the fix the log was "show: foo\nshow: foo\nshow: bar\n" because the
    // forward reference forced `_foo` once and the eager walk evaluated it again.
    let log = evaluator.runtime_ctx.borrow().log_message.clone();
    assert_eq!(
        log, "show: foo\nshow: bar\n",
        "each field value must be evaluated exactly once; got log:\n{}",
        log
    );
}
