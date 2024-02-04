use std::path::PathBuf;

use super::{r#override::apply_override_on_module, *};
use crate::path::parse_attribute_path;
use kclvm_ast::ast;
use kclvm_parser::parse_file_force_errors;
use pretty_assertions::assert_eq;

const CARGO_FILE_PATH: &str = env!("CARGO_MANIFEST_DIR");

/// Test override_file result.
#[test]
fn test_override_file_simple() {
    let specs = vec![
        "config.image=image/image".to_string(),
        ":config.image=\"image/image:v1\"".to_string(),
        ":config.data={id=1,value=\"override_value\"}".to_string(),
    ];

    let mut cargo_file_path = PathBuf::from(CARGO_FILE_PATH);
    cargo_file_path.push("src/test_data/simple.k");
    let abs_path = cargo_file_path.to_str().unwrap();

    let import_paths = vec![];
    assert_eq!(
        override_file(abs_path, &specs, &import_paths).unwrap(),
        true
    )
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
