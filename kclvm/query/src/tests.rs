use std::{fs, path::PathBuf};

use super::{r#override::apply_override_on_module, *};
use crate::{
    path::parse_attribute_path, r#override::parse_override_spec, selector::list_variables,
};
use kclvm_error::{DiagnosticId, ErrorKind, Level};
use kclvm_parser::parse_file_force_errors;
use kclvm_utils::path::PathPrefix;
use pretty_assertions::assert_eq;
use selector::ListOptions;

const CARGO_FILE_PATH: &str = env!("CARGO_MANIFEST_DIR");

fn get_test_dir(sub: String) -> PathBuf {
    let mut cargo_file_path = PathBuf::from(CARGO_FILE_PATH);
    cargo_file_path.push("src");
    cargo_file_path.push("test_data");
    // Split unix and windows path
    for part in sub.split("/") {
        for p in part.split("\\") {
            cargo_file_path.push(p);
        }
    }
    cargo_file_path
}

/// Test override_file result.
#[test]
fn test_override_file_simple() {
    let specs = vec![
        "config.image=image/image".to_string(),
        "config.image=\"image/image:v1\"".to_string(),
        "config.data={id=1,value=\"override_value\"}".to_string(),
        "dict_config={\"image\": \"image/image:v2\" \"data\":{\"id\":2 \"value2\": \"override_value2\"}}".to_string(),
        "envs=[{key=\"key1\" value=\"value1\"} {key=\"key2\" value=\"value2\"}]".to_string(),
        "isfilter=False".to_string(),
        "count=2".to_string(),
        "msg=\"Hi World\"".to_string(),
        "delete-".to_string(),
        "dict_delete.image-".to_string(),
        "dict_delete_whole-".to_string(),
        "insert_config.key=1".to_string(),
        "uni_config.labels.key1=1".to_string(),
        "config_unification=Config {\"image\": \"image/image:v4\"}".to_string(),
        "config_unification:Config {\"env\": {\"aaa\": \"aaa\"}}".to_string(),
        "config_unification.env: {\"bbb\": \"bbb\"}}".to_string(),
        "config_unification_delete-".to_string()
    ];

    let simple_path = get_test_dir("simple.k".to_string());
    let simple_bk_path = get_test_dir("simple.bk.k".to_string());
    let expect_path = get_test_dir("expect.k".to_string());
    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();
    if simple_path.exists() {
        fs::remove_file(simple_path.clone()).unwrap();
    }

    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();

    let import_paths = vec![];
    assert_eq!(
        override_file(simple_path.clone().to_str().unwrap(), &specs, &import_paths)
            .unwrap()
            .result,
        true
    );

    let simple_content = fs::read_to_string(simple_path.clone()).unwrap();
    let expect_content = fs::read_to_string(expect_path).unwrap();

    let simple_content = simple_content.replace("\r\n", "\n");
    let expect_content = expect_content.replace("\r\n", "\n");

    assert_eq!(simple_content, expect_content);

    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();
}
/// Test override_file result.
#[test]
fn test_override_file_import_paths() {
    let specs = vec!["data.value=\"override_value\"".to_string()];
    let import_paths = vec![
        "pkg".to_string(),
        "pkg.pkg".to_string(),
        "pkg.pkg as alias_pkg1".to_string(),
        "pkg.pkg as alias_pkg2".to_string(),
    ];

    let mut cargo_file_path = PathBuf::from(CARGO_FILE_PATH);
    cargo_file_path.push("src/test_data/import_paths.k");
    let abs_path = cargo_file_path.to_str().unwrap();

    assert_eq!(
        override_file(abs_path, &specs, &import_paths)
            .unwrap()
            .result,
        true
    )
}

/// Test override_file result with the expected modified AST.
#[test]
fn test_override_file_config() {
    let specs = vec![
        "appConfiguration.image=\"kcl/kcl:{}\".format(version)".to_string(),
        r#"appConfiguration.mainContainer.name="override_name""#.to_string(),
        "appConfiguration.labels.key.key=\"override_value\"".to_string(),
        "appConfiguration.labels.key.str-key=\"override_value\"".to_string(),
        "appConfiguration.labels.key['dot.key']=\"override_value\"".to_string(),
        "appConfiguration.overQuota=False".to_string(),
        "appConfiguration.probe={periodSeconds=20}".to_string(),
        "appConfiguration.resource-".to_string(),
        "appConfiguration.svc=s.Service {}".to_string(),
        "appConfigurationUnification.image=\"kcl/kcl:v0.1\"".to_string(),
        r#"appConfigurationUnification.mainContainer.name="override_name""#.to_string(),
        "appConfigurationUnification.labels.key.key=\"override_value\"".to_string(),
        "appConfigurationUnification.overQuota=False".to_string(),
        "appConfigurationUnification.resource.cpu-".to_string(),
        "appConfigurationUnification.svc=s.Service {}".to_string(),
        "appConfigurationUnification:{name=\"name\"}".to_string(),
        "config.x:{a:1}".to_string(),
        "config.x:{b:2}".to_string(),
        "config.x:{b:3}".to_string(),
        "config.x:{c.d:4}".to_string(),
        "config.y=1".to_string(),
        "config.z+=[1,2,3]".to_string(),
        "config.z+=[4,5,6]".to_string(),
        "var1:1".to_string(),
        "var2=1".to_string(),
        "var3+=[1,2,3]".to_string(),
        "var3+=[4,5,6]".to_string(),
        "var4:AppConfiguration {image:'image'}".to_string(),
    ];
    let import_paths = vec!["service as s".to_string()];

    let mut cargo_file_path = PathBuf::from(CARGO_FILE_PATH);
    cargo_file_path.push("src/test_data/config.k");
    let abs_path = cargo_file_path.to_str().unwrap();

    let mut module = parse_file_force_errors(abs_path, None).unwrap();
    for s in &specs {
        apply_override_on_module(&mut module, s, &import_paths).unwrap();
    }
    let expected_code = print_ast_module(&module);
    assert_eq!(
        expected_code,
        r#"import service as s

schema Main:
    name?: str
    env?: [{str:}]

schema Probe:
    initialDelaySeconds?: int
    timeoutSeconds?: int
    periodSeconds?: int = 10
    successThreshold?: int
    failureThreshold?: int

schema AppConfiguration:
    appName: str
    image: str
    overQuota: bool = False
    resource: {str:}
    mainContainer?: Main
    labels: {str:}
    probe?: Probe

appConfiguration = AppConfiguration {
    appName: "kclvm"
    image: "kcl/kcl:{}".format(version)
    labels: {
        key: {
            key: "override_value"
            "str-key" = "override_value"
            "dot.key" = "override_value"
        }
    }
    mainContainer: Main {name: "override_name"}
    overQuota = False
    overQuota = False
    probe: {
        periodSeconds = 20
    }
    svc = s.Service {}
}

appConfigurationUnification: AppConfiguration {
    appName: "kclvm"
    image: "kcl/kcl:v0.1"
    resource: {
        disk: "50Gi"
        memory: "12Gi"
    }
    labels: {
        key: {key: "override_value"}
    }
    mainContainer: Main {name: "override_name"}
    overQuota: False
    svc = s.Service {}
    name = "name"
}
config = {
    x: {
        a: 1
        b: 3
        c: {
            d: 4
        }
    }
    y = 1
    z += [
        1
        2
        3
        4
        5
        6
    ]
}
var1 = 1
var2 = 1
var3 += [
    1
    2
    3
    4
    5
    6
]
var4: AppConfiguration {
    image: 'image'
}
"#
    );
}

/// Test override spec parser.
#[test]
fn test_parse_override_spec_invalid() {
    let specs = vec![":a:", "=a=", ":a", "a-1"];
    for spec in specs {
        assert!(parse_override_spec(spec).is_err(), "{spec} test failed");
    }
}

#[test]
fn test_parse_property_path() {
    assert_eq!(parse_attribute_path("a.b.c").unwrap(), vec!["a", "b", "c"]);
    assert_eq!(
        parse_attribute_path(r#"a["b"].c"#).unwrap(),
        vec!["a", "b", "c"]
    );
    assert_eq!(
        parse_attribute_path(r#"a.["b"].c"#).unwrap(),
        vec!["a", "b", "c"]
    );
    assert_eq!(
        parse_attribute_path(r#"a['b'].c"#).unwrap(),
        vec!["a", "b", "c"]
    );
    assert_eq!(
        parse_attribute_path(r#"a.b['c.d']"#).unwrap(),
        vec!["a", "b", "c.d"]
    );
    assert_eq!(
        parse_attribute_path(r#"a.b.['c.d']"#).unwrap(),
        vec!["a", "b", "c.d"]
    );
    assert_eq!(
        parse_attribute_path(r#"a.b['c.d'].e"#).unwrap(),
        vec!["a", "b", "c.d", "e"]
    );
    assert_eq!(
        parse_attribute_path(r#"a.b.['c.d'].e"#).unwrap(),
        vec!["a", "b", "c.d", "e"]
    );
    assert_eq!(
        parse_attribute_path(r#"a.b.c-d.e"#).unwrap(),
        vec!["a", "b", "c-d", "e"]
    );
    assert!(parse_attribute_path(r#"a.[b.c-d.e"#).is_err(),);
    assert!(parse_attribute_path(r#"a.[b.c]-d.e"#).is_err(),);
}

#[test]
fn test_list_variables() {
    let file = PathBuf::from("./src/test_data/test_list_variables/supported.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let test_cases = vec![
        ("a", "1", "", "="),
        ("a1", "2", "", "="),
        ("a3", "3m", "", "="),
        ("b1", "True", "", "="),
        ("b2", "False", "", "="),
        ("s1", "\"Hello\"", "", "="),
        (
            "array1",
            r#"[
    1
    2
    3
]"#,
            "",
            "=",
        ),
        (
            "dict1",
            r#"{
    "a": 1
    "b": 2
}"#,
            "",
            "=",
        ),
        ("dict1.a", "1", "", ":"),
        ("dict1.b", "2", "", ":"),
        (
            "dict2",
            r#"{
    "a": 1
    "b": {
        "c": 2
        "d": 3
    }
}"#,
            "",
            "=",
        ),
        ("dict2.a", "1", "", ":"),
        (
            "dict2.b",
            r#"{
    "c": 2
    "d": 3
}"#,
            "",
            ":",
        ),
        ("dict2.b.c", "2", "", ":"),
        ("dict2.b.d", "3", "", ":"),
        (
            "sha",
            r#"A {
    name: "Hello"
    ids: [1, 2, 3]
    data: {
        "a": {
            "b": {
                "c": 2
            }
        }
    }
}"#,
            "A",
            "=",
        ),
        ("sha.name", "\"Hello\"", "", ":"),
        (
            "sha.ids",
            r#"[
    1
    2
    3
]"#,
            "",
            ":",
        ),
        (
            "sha.data",
            r#"{
    "a": {
        "b": {
            "c": 2
        }
    }
}"#,
            "",
            ":",
        ),
        (
            "sha.data.a",
            r#"{
    "b": {
        "c": 2
    }
}"#,
            "",
            ":",
        ),
        (
            "sha.data.a.b",
            r#"{
    "c": 2
}"#,
            "",
            ":",
        ),
        ("sha.data.a.b.c", "2", "", ":"),
        (
            "shb",
            r#"B {
    a: {
        name: "HelloB"
        ids: [4, 5, 6]
        data: {
            "d": {
                "e": {
                    "f": 3
                }
            }
        }
    }
}"#,
            "B",
            "=",
        ),
        (
            "shb.a",
            r#"{
    name: "HelloB"
    ids: [4, 5, 6]
    data: {
        "d": {
            "e": {
                "f": 3
            }
        }
    }
}"#,
            "",
            ":",
        ),
        ("shb.a.name", "\"HelloB\"", "", ":"),
        (
            "shb.a.ids",
            r#"[
    4
    5
    6
]"#,
            "",
            ":",
        ),
        (
            "shb.a.data",
            r#"{
    "d": {
        "e": {
            "f": 3
        }
    }
}"#,
            "",
            ":",
        ),
        (
            "shb.a.data.d",
            r#"{
    "e": {
        "f": 3
    }
}"#,
            "",
            ":",
        ),
        (
            "shb.a.data.d.e",
            r#"{
    "f": 3
}"#,
            "",
            ":",
        ),
        ("uconfa.name", "\"b\"", "", "="),
        (
            "c.a",
            r#"{
    name: "Hello"
}"#,
            "",
            ":",
        ),
        (
            "job.name",
            r#""{}-{}".format("app", "test").lower()"#,
            "",
            "=",
        ),
        (
            "union_list",
            r#"[
    *_list0
    *_list1
]"#,
            "",
            "=",
        ),
        (
            "a_dict",
            r#"{
    **_part1
    **_part2
}"#,
            "",
            "=",
        ),
    ];

    for (spec, expected, expected_name, op_sym) in test_cases {
        let specs = vec![spec.to_string()];
        let result = list_variables(vec![file.clone()], specs, None).unwrap();
        assert_eq!(
            result.variables.get(spec).unwrap().get(0).unwrap().value,
            expected,
            "{spec}"
        );
        assert_eq!(
            result
                .variables
                .get(spec)
                .unwrap()
                .get(0)
                .unwrap()
                .type_name,
            expected_name,
            "{spec}"
        );
        assert_eq!(
            result.variables.get(spec).unwrap().get(0).unwrap().op_sym,
            op_sym,
            "{spec}"
        );

        let got_json = serde_json::to_string_pretty(&result.variables).unwrap();
        insta::assert_snapshot!(got_json);
    }
}

#[test]
fn test_list_all_variables() {
    let file = PathBuf::from("./src/test_data/test_list_variables/supported.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let test_cases = vec![
        ("a", "1", "", "="),
        ("a1", "2", "", "="),
        ("a3", "3m", "", "="),
        ("b1", "True", "", "="),
        ("b2", "False", "", "="),
        ("s1", "\"Hello\"", "", "="),
        (
            "array1",
            r#"[
    1
    2
    3
]"#,
            "",
            "=",
        ),
        (
            "dict1",
            r#"{
    "a": 1
    "b": 2
}"#,
            "",
            "=",
        ),
        (
            "dict2",
            r#"{
    "a": 1
    "b": {
        "c": 2
        "d": 3
    }
}"#,
            "",
            "=",
        ),
        (
            "sha",
            r#"A {
    name: "Hello"
    ids: [1, 2, 3]
    data: {
        "a": {
            "b": {
                "c": 2
            }
        }
    }
}"#,
            "A",
            "=",
        ),
        (
            "shb",
            r#"B {
    a: {
        name: "HelloB"
        ids: [4, 5, 6]
        data: {
            "d": {
                "e": {
                    "f": 3
                }
            }
        }
    }
}"#,
            "B",
            "=",
        ),
        (
            "job",
            r#"Job {
    name = "{}-{}".format("app", "test").lower()
}"#,
            "Job",
            "=",
        ),
        (
            "select",
            r#"a.b.c {
    a: 1
}"#,
            "a.b.c",
            "=",
        ),
        (
            "union_list",
            r#"[
    *_list0
    *_list1
]"#,
            "",
            "=",
        ),
        (
            "a_dict",
            r#"{
    **_part1
    **_part2
}"#,
            "",
            "=",
        ),
    ];

    for (spec, expected, expected_name, op_sym) in test_cases {
        println!("{:?}", spec);
        let result = list_variables(vec![file.clone()], vec![], None).unwrap();
        assert_eq!(
            result.variables.get(spec).unwrap().get(0).unwrap().value,
            expected,
            "{spec}"
        );
        assert_eq!(
            result
                .variables
                .get(spec)
                .unwrap()
                .get(0)
                .unwrap()
                .type_name,
            expected_name,
            "{spec}"
        );
        assert_eq!(
            result.variables.get(spec).unwrap().get(0).unwrap().op_sym,
            op_sym,
            "{spec}"
        );
        assert_eq!(result.parse_errors.len(), 0);

        let got_json = serde_json::to_string_pretty(&result.variables).unwrap();
        insta::assert_snapshot!(got_json);
    }
}

#[test]
fn test_list_unsupported_variables() {
    let file = PathBuf::from("./src/test_data/test_list_variables/unsupported.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();

    // test unsupport code
    let test_cases = vec![
        ("list", "[_x for _x in range(20) if _x % 2 == 0]"),
        ("list1", "[i if i > 2 else i + 1 for i in [1, 2, 3]]"),
        ("dict", "{str(i): 2 * i for i in range(3)}"),
        (
            "func",
            r#"lambda x: int, y: int -> int {
    x + y
}"#,
        ),
        (
            "if_schema.falseValue",
            "if True:\n    trueValue: 1\nelse:\n    falseValue: 2",
        ),
        (
            "if_schema.trueValue",
            "if True:\n    trueValue: 1\nelse:\n    falseValue: 2",
        ),
    ];

    for (spec, expected_code) in test_cases {
        let specs = vec![spec.to_string()];
        let result = list_variables(vec![file.clone()], specs, None).unwrap();
        assert_eq!(result.variables.get(spec), None);
        assert_eq!(result.unsupported[0].code, expected_code);
        assert_eq!(result.parse_errors.len(), 0);
    }

    // test list variables from unsupported code
    let test_cases = vec![
        ("if_schema.name", r#""name""#),
        ("if_schema.age", "1"),
        (
            "if_schema.inner",
            r#"IfSchemaInner {
    innerValue: 1
}"#,
        ),
        ("if_schema.inner.innerValue", "1"),
        (
            "if_schema.inner2",
            r#"{
    innerValue: 2
}"#,
        ),
        ("if_schema.inner2.innerValue", "2"),
        ("if_schema1.name", r#""name""#),
        ("if_schema1.age", "1"),
        (
            "if_schema1.inner",
            r#"IfSchemaInner {
    innerValue: 1
}"#,
        ),
        ("if_schema1.inner.innerValue", "1"),
        (
            "if_schema1.inner2",
            r#"{
    innerValue: 2
}"#,
        ),
        ("if_schema1.inner2.innerValue", "2"),
    ];

    for (spec, expected_code) in test_cases {
        let specs = vec![spec.to_string()];
        let result = list_variables(vec![file.clone()], specs, None).unwrap();
        assert_eq!(
            result.variables.get(spec).unwrap().get(0).unwrap().value,
            expected_code,
            "{spec}",
        );
    }
}

#[test]
fn test_override_file_insert() {
    let specs = vec![
        r#"b={
            "c": 2
        }"#
        .to_string(),
        r#"c.b={"a": "b"}"#.to_string(),
        r#"d.e.f.g=3"#.to_string(),
        r#"_access3=test.ServiceAccess {
    iType = "kkkkkkk"
    sType = dsType
    TestStr = ["${test_str}"]
    ports = [80, 443]
    booltest = True
}"#
        .to_string(),
        r#"_access4=test.ServiceAccess {
    iType = "kkkkkkk"
    sType = dsType
    TestStr = ["${test_str}"]
    ports = [80, 443]
    booltest = True
}"#
        .to_string(),
        r#"_access.iType="kkkkkkk""#.to_string(),
        r#"_access5.iType="dddddd""#.to_string(),
        r#"a=b"#.to_string(),
        r#"_access6      ="a6""#.to_string(),
        r#"_access.mergedattr=1"#.to_string(),
        r#"_access.a.b.c=2"#.to_string(),
        r#"_access.a.c.b=3"#.to_string(),
    ];

    let simple_path = get_test_dir("test_override_file/main.k".to_string());
    let simple_bk_path = get_test_dir("test_override_file/main.bk.k".to_string());
    let expect_path = get_test_dir("test_override_file/expect.k".to_string());
    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();
    let import_paths = vec![
        "base.pkg.kusion_models.app".to_string(),
        "base.pkg.kusion_models.app.vip as vip".to_string(),
        "base.pkg.kusion_models.app.container".to_string(),
        "base.pkg.kusion_models.app.resource as res".to_string(),
        "base.pkg.kusion_models.app.sidecar".to_string(),
        ".values".to_string(),
    ];

    // test insert multiple times
    for _ in 1..=5 {
        assert_eq!(
            override_file(&simple_path.display().to_string(), &specs, &import_paths)
                .unwrap()
                .result,
            true
        );

        let simple_content = fs::read_to_string(simple_path.clone()).unwrap();
        let expect_content = fs::read_to_string(expect_path.clone()).unwrap();

        let simple_content = simple_content.replace("\r\n", "\n");
        let expect_content = expect_content.replace("\r\n", "\n");

        assert_eq!(simple_content, expect_content);
    }

    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();
}

#[test]
fn test_list_variable_with_invalid_kcl() {
    let file = PathBuf::from("./src/test_data/test_list_variables/invalid.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let specs = vec!["a".to_string()];
    let result = list_variables(vec![file.clone()], specs, None).unwrap();
    assert_eq!(result.variables.get("a"), None);
    assert_eq!(result.parse_errors.len(), 2);
    assert_eq!(result.parse_errors[0].level, Level::Error);
    assert_eq!(
        result.parse_errors[0].code,
        Some(DiagnosticId::Error(ErrorKind::InvalidSyntax))
    );
    assert_eq!(
        result.parse_errors[0].messages[0].message,
        "expected one of [\"=\"] got eof",
    );
    assert_eq!(result.parse_errors[0].messages[0].range.0.filename, file);
    assert_eq!(result.parse_errors[0].messages[0].range.0.line, 1);
    assert_eq!(result.parse_errors[0].messages[0].range.0.column, Some(8));
}

#[test]
fn test_overridefile_with_invalid_kcl() {
    let simple_path = get_test_dir("test_override_file/invalid.k".to_string());
    let simple_bk_path = get_test_dir("invalid.bk.k".to_string());
    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();

    let result = override_file(
        &simple_path.display().to_string(),
        &vec!["a=b".to_string()],
        &vec![],
    )
    .unwrap();

    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();
    assert_eq!(result.result, true);
    assert_eq!(result.parse_errors.len(), 2);
    assert_eq!(result.parse_errors[0].level, Level::Error);
    assert_eq!(
        result.parse_errors[0].code,
        Some(DiagnosticId::Error(ErrorKind::InvalidSyntax))
    );
    assert_eq!(
        result.parse_errors[0].messages[0].message,
        "expected one of [\"=\"] got eof"
    );
    assert_eq!(
        result.parse_errors[0].messages[0]
            .range
            .0
            .filename
            .adjust_canonicalization(),
        simple_path.display().to_string().adjust_canonicalization()
    );
    assert_eq!(result.parse_errors[0].messages[0].range.0.line, 1);
    assert_eq!(result.parse_errors[0].messages[0].range.0.column, Some(8));
}

#[test]
fn test_list_variables_with_file_noexist() {
    let file = PathBuf::from("./src/test_data/test_list_variables/noexist.k")
        .display()
        .to_string();
    let specs = vec!["a".to_string()];
    let result = list_variables(vec![file.clone()], specs, None);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.to_string(), "Cannot find the kcl file, please check the file path ./src/test_data/test_list_variables/noexist.k");
}

#[test]
fn test_override_file_with_invalid_spec() {
    let specs = vec!["....".to_string()];
    let import_paths = vec![];
    let file = PathBuf::from("./src/test_data/test_override_file/main.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    let result = override_file(&file, &specs, &import_paths);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.to_string(), "Invalid spec format '....', expected <field_path>=filed_value>, <field_path>:filed_value>, <field_path>+=filed_value> or <field_path>-");
}

#[test]
fn test_list_merged_variables() {
    let file = PathBuf::from("./src/test_data/test_list_variables/test_list_merged_variables")
        .canonicalize()
        .unwrap();

    file.join("path").display().to_string();

    let test_cases = vec![
        (
            vec![
                file.join("merge_1/base.k").display().to_string(),
                file.join("merge_1/main.k").display().to_string(),
            ],
            vec!["appConfiguration.resource".to_string()],
            vec![r#"res.Resource {cpu = "2", disk = "35Gi", memory = "4Gi"}"#.to_string()],
        ),
                (
                    vec![
                        file.join("merge_2/base.k").display().to_string(),
                        file.join("merge_2/main.k").display().to_string(),
                    ],
                    vec!["appConfiguration.resource".to_string()],
                    vec![r#"res.Resource {
    cpu = "2"
    memory = "4Gi"
}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_3/base.k").display().to_string(),
                        file.join("merge_3/main.k").display().to_string(),
                    ],
                    vec!["appConfiguration.resource".to_string()],
                    vec![r#"res.Resource {cpu = "2", disk = "35Gi", memory = "4Gi"}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_4/base.k").display().to_string(),
                        file.join("merge_4/main.k").display().to_string(),
                    ],
                    vec!["appConfiguration.resource".to_string()],
                    vec![r#"res.Resource {
    cpu = "2"
    memory = "4Gi"
}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_5/base.k").display().to_string(),
                        file.join("merge_5/main.k").display().to_string(),
                    ],
                    vec!["appConfiguration.resource".to_string()],
                    vec![r#"res.Resource {cpu = {limit: "200m", limit_plus: "20000m", request: "100m"}, disk = "35Gi", memory = "4Gi"}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_6/main.k").display().to_string(),
                        file.join("merge_6/base.k").display().to_string(),
                    ],
                    vec!["config".to_string()],
                    vec![r#"Config {
    name = "config2"
    args: ["kcl", "main.k"]
    labels: {key1: "value1"}
}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_7/main.k").display().to_string(),
                        file.join("merge_7/base.k").display().to_string(),
                    ],
                    vec!["config".to_string()],
                    vec![r#"Config {
    name = _NAME
    args: ["kcl", "main.k"]
    labels: {key1: "value1"}
}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_8/main.k").display().to_string(),
                    ],
                    vec!["config".to_string()],
                    vec![r#"Config {
    args: [
        "kcl"
        "main.k"
    ]
    labels: {
        key1: "value1"
    }
    labels: {
        key2: "value2"
    }
    "labels": {
        "key3": "value3"
    }
}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_9/main.k").display().to_string(),
                    ],
                    vec!["config".to_string()],
                    vec![r#"Config {
    args: [
        "kcl"
        "main.k"
    ]
    labels: {
        key1: "value1"
    }
    labels: {
        key2: "value2"
    }
    name: {
        name: {
            name: "name"
        }
    }
    name: {
        name: Name0 {data: 1}
    }
}"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_10/main.k").display().to_string(),
                    ],
                    vec!["alice.hc".to_string()],
                    vec![r#"[
    2
]"#.to_string()],
                ),
                (
                    vec![
                        file.join("merge_11/main.k").display().to_string(),
                    ],
                    vec!["config.main".to_string()],
                    vec![r#"Main {args: ["1"], env: ["789", "456"]}"#.to_string()],
                ),
    ];

    for (files, specs, expected_values) in test_cases {
        println!("{:?}", files);
        let result = list_variables(
            files,
            specs.clone(),
            Some(&ListOptions {
                merge_program: true,
            }),
        )
        .unwrap();
        for (i, expected_value) in expected_values.iter().enumerate() {
            let variables = result
                .variables
                .get(&specs.get(i).unwrap().to_string())
                .unwrap();
            assert_eq!(variables.len(), 1);
            for variable in variables {
                assert_eq!(variable.value.to_string(), expected_value.to_string());
            }
        }
    }
}
