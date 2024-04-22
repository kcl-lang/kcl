use std::{fs, path::PathBuf};

use super::{r#override::apply_override_on_module, *};
use crate::{path::parse_attribute_path, selector::list_variables};
use kclvm_ast::ast;
use kclvm_parser::parse_file_force_errors;
use pretty_assertions::assert_eq;

const CARGO_FILE_PATH: &str = env!("CARGO_MANIFEST_DIR");

fn get_test_dir(sub: String) -> PathBuf {
    let mut cargo_file_path = PathBuf::from(CARGO_FILE_PATH);
    cargo_file_path.push("src/test_data");
    cargo_file_path.push(sub);
    cargo_file_path
}

/// Test override_file result.
#[test]
fn test_override_file_simple() {
    let specs = vec![
        "config.image=image/image".to_string(),
        ":config.image=\"image/image:v1\"".to_string(),
        ":config.data={id=1,value=\"override_value\"}".to_string(),
        ":dict_config={\"image\": \"image/image:v2\" \"data\":{\"id\":2 \"value2\": \"override_value2\"}}".to_string(),
        ":envs=[{key=\"key1\" value=\"value1\"} {key=\"key2\" value=\"value2\"}]".to_string(),
        ":isfilter=False".to_string(),
        ":count=2".to_string(),
        ":msg=\"Hi World\"".to_string(),
        ":delete-".to_string(),
        ":dict_delete.image-".to_string(),
        ":dict_delete_whole-".to_string(),
        ":insert_config.key=1".to_string(),
        ":uni_config.labels.key1=1".to_string(),
        ":config_unification=Config {\"image\": \"image/image:v4\"}".to_string(),
        ":config_unification_delete-".to_string()
    ];

    let simple_path = get_test_dir("simple.k".to_string());
    let simple_bk_path = get_test_dir("simple.bk.k".to_string());
    let except_path = get_test_dir("except.k".to_string());
    if simple_path.exists() {
        fs::remove_file(simple_path.clone()).unwrap();
    }

    fs::copy(simple_bk_path, simple_path.clone()).unwrap();

    let import_paths = vec![];
    assert_eq!(
        override_file(simple_path.to_str().unwrap(), &specs, &import_paths).unwrap(),
        true
    );

    let simple_content = fs::read_to_string(simple_path).unwrap();
    let expect_content = fs::read_to_string(except_path).unwrap();

    let simple_content = simple_content.replace("\r\n", "").replace("\n", "");
    let expect_content = expect_content.replace("\r\n", "").replace("\n", "");

    assert_eq!(simple_content, expect_content);
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
        override_file(abs_path, &specs, &import_paths).unwrap(),
        true
    )
}

/// Test override_file result with the expected modified AST.
#[test]
fn test_override_file_config() {
    let specs = vec![
        "appConfiguration.image=\"kcl/kcl:{}\".format(version)".to_string(),
        "appConfiguration.mainContainer.name=override_name".to_string(),
        "appConfiguration.labels.key.key=\"override_value\"".to_string(),
        "appConfiguration.labels.key.str-key=\"override_value\"".to_string(),
        "appConfiguration.labels.key['dot.key']=\"override_value\"".to_string(),
        "appConfiguration.overQuota=False".to_string(),
        "appConfiguration.probe={periodSeconds=20}".to_string(),
        "appConfiguration.resource-".to_string(),
        "appConfigurationUnification.image=\"kcl/kcl:v0.1\"".to_string(),
        "appConfigurationUnification.mainContainer.name=\"override_name\"".to_string(),
        "appConfigurationUnification.labels.key.key=\"override_value\"".to_string(),
        "appConfigurationUnification.overQuota=False".to_string(),
        "appConfigurationUnification.resource.cpu-".to_string(),
    ];
    let overrides = specs
        .iter()
        .map(|s| parse_override_spec(s))
        .filter_map(Result::ok)
        .collect::<Vec<ast::OverrideSpec>>();
    let import_paths = vec![];

    let mut cargo_file_path = PathBuf::from(CARGO_FILE_PATH);
    cargo_file_path.push("src/test_data/config.k");
    let abs_path = cargo_file_path.to_str().unwrap();

    let mut module = parse_file_force_errors(abs_path, None).unwrap();
    for o in &overrides {
        apply_override_on_module(&mut module, o, &import_paths).unwrap();
    }
    let expected_code = print_ast_module(&module);
    assert_eq!(
        expected_code,
        r#"schema Main:
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
    probe: {periodSeconds = 20}
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
        ("a", "1"),
        ("a1", "2"),
        ("a3", "3m"),
        ("b1", "True"),
        ("b2", "False"),
        ("s1", "\"Hello\""),
        ("array1", "[1, 2, 3]"),
        ("dict1", "{\"a\": 1, \"b\": 2}"),
        ("dict1.a", "1"),
        ("dict1.b", "2"),
        (
            "dict2",
            r#"{
    "a": 1
    "b": {
        "c": 2
        "d": 3
    }
}"#,
        ),
        ("dict2.a", "1"),
        (
            "dict2.b",
            r#"{
    "c": 2
    "d": 3
}"#,
        ),
        ("dict2.b.c", "2"),
        ("dict2.b.d", "3"),
        (
            "sha",
            r#"A {
    name: "Hello"
    ids: [1, 2, 3]
    data: {
        "a": {
            "b": {"c": 2}
        }
    }
}"#,
        ),
        ("sha.name", "\"Hello\""),
        ("sha.ids", "[1, 2, 3]"),
        (
            "sha.data",
            r#"{
    "a": {
        "b": {"c": 2}
    }
}"#,
        ),
        (
            "sha.data.a",
            r#"{
    "b": {"c": 2}
}"#,
        ),
        ("sha.data.a.b", r#"{"c": 2}"#),
        ("sha.data.a.b.c", "2"),
        (
            "shb",
            r#"B {
    a: {
        name: "HelloB"
        ids: [4, 5, 6]
        data: {
            "d": {
                "e": {"f": 3}
            }
        }
    }
}"#,
        ),
        (
            "shb.a",
            r#"{
    name: "HelloB"
    ids: [4, 5, 6]
    data: {
        "d": {
            "e": {"f": 3}
        }
    }
}"#,
        ),
        ("shb.a.name", "\"HelloB\""),
        ("shb.a.ids", "[4, 5, 6]"),
        (
            "shb.a.data",
            r#"{
    "d": {
        "e": {"f": 3}
    }
}"#,
        ),
        (
            "shb.a.data.d",
            r#"{
    "e": {"f": 3}
}"#,
        ),
        ("shb.a.data.d.e", "{\"f\": 3}"),
        ("uconfa.name", "\"b\""),
        ("c.a", "{ids: [7, 8, 9]}"),
        ("job.name", r#""{}-{}".format("app", "test").lower()"#),
    ];

    for (spec, expected) in test_cases {
        let specs = vec![spec.to_string()];
        let result = list_variables(file.clone(), specs).unwrap();
        assert_eq!(result.select_result.get(spec).unwrap(), expected);
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
        ("a", "1"),
        ("a1", "2"),
        ("a3", "3m"),
        ("b1", "True"),
        ("b2", "False"),
        ("s1", "\"Hello\""),
        ("array1", "[1, 2, 3]"),
        ("dict1", "{\"a\": 1, \"b\": 2}"),
        (
            "dict2",
            r#"{
    "a": 1
    "b": {
        "c": 2
        "d": 3
    }
}"#,
        ),
        (
            "sha",
            r#"A {
    name: "Hello"
    ids: [1, 2, 3]
    data: {
        "a": {
            "b": {"c": 2}
        }
    }
}"#,
        ),
        (
            "shb",
            r#"B {
    a: {
        name: "HelloB"
        ids: [4, 5, 6]
        data: {
            "d": {
                "e": {"f": 3}
            }
        }
    }
}"#,
        ),
        (
            "job",
            r#"Job {name = "{}-{}".format("app", "test").lower()}"#,
        ),
    ];

    for (spec, expected) in test_cases {
        let result = list_variables(file.clone(), vec![]).unwrap();
        assert_eq!(result.select_result.get(spec).unwrap(), expected);
    }
}

#[test]
fn test_list_unsupported_variables() {
    let file = PathBuf::from("./src/test_data/test_list_variables/unsupported.k")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
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
        let result = list_variables(file.clone(), specs).unwrap();
        assert_eq!(result.select_result.get(spec), None);
        assert_eq!(result.unsupported[0].code, expected_code);
    }
}

#[test]
fn test_overridefile_insert() {
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
        r#"_access.iType="kkkkkkk""#.to_string(),
        r#"_access5.iType="dddddd""#.to_string(),
    ];

    let simple_path = get_test_dir("test_override_file/main.k".to_string());
    let simple_bk_path = get_test_dir("test_override_file/main.bk.k".to_string());
    let except_path = get_test_dir("test_override_file/expect.k".to_string());
    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();

    for spec in specs {
        let import_paths = vec![];
        assert_eq!(
            override_file(
                &simple_path.display().to_string(),
                &vec![spec],
                &import_paths
            )
            .unwrap(),
            true
        );
    }
    let simple_content = fs::read_to_string(simple_path.clone()).unwrap();
    let expect_content = fs::read_to_string(except_path.clone()).unwrap();

    let simple_content = simple_content.replace("\r\n", "").replace("\n", "");
    let expect_content = expect_content.replace("\r\n", "").replace("\n", "");

    assert_eq!(simple_content, expect_content);
    fs::copy(simple_bk_path.clone(), simple_path.clone()).unwrap();
}
