use std::{env, fs, path::PathBuf};

use kclvm_config::modfile::KCL_PKG_PATH;

use crate::{
    app, fmt::fmt_command, run::run_command, settings::build_settings, util::hashmaps_from_matches,
    vet::vet_command,
};

const ROOT_CMD: &str = "kclvm_cli";

#[test]
fn test_build_settings() {
    let work_dir = work_dir();
    let matches = app().get_matches_from(settings_arguments(work_dir.join("kcl.yaml")));
    let matches = matches.subcommand_matches("run").unwrap();
    let s = build_settings(matches).unwrap();
    // Testing work directory
    assert_eq!(s.path().as_ref().unwrap().to_str(), work_dir.to_str());
    // Testing CLI configs
    assert_eq!(
        s.settings().kcl_cli_configs.as_ref().unwrap().files,
        Some(vec!["hello.k".to_string()])
    );
    assert_eq!(
        s.settings().kcl_cli_configs.as_ref().unwrap().disable_none,
        Some(true)
    );
    assert_eq!(
        s.settings()
            .kcl_cli_configs
            .as_ref()
            .unwrap()
            .strict_range_check,
        Some(true)
    );
    assert_eq!(
        s.settings().kcl_cli_configs.as_ref().unwrap().overrides,
        Some(vec!["c.a=1".to_string(), "c.b=1".to_string(),])
    );
    assert_eq!(
        s.settings().kcl_cli_configs.as_ref().unwrap().path_selector,
        Some(vec!["a.b.c".to_string()])
    );
    assert_eq!(s.settings().input(), vec!["hello.k".to_string()]);
}

#[test]
fn test_build_settings_fail() {
    let matches = app().get_matches_from(settings_arguments(work_dir().join("error_kcl.yaml")));
    let matches = matches.subcommand_matches("run").unwrap();
    assert!(build_settings(matches).is_err());
}

#[test]
fn test_fmt_cmd() {
    let input = std::path::Path::new(".")
        .join("src")
        .join("test_data")
        .join("fmt")
        .join("test.k");
    let matches = app().get_matches_from(&[ROOT_CMD, "fmt", input.to_str().unwrap()]);
    let matches = matches.subcommand_matches("fmt").unwrap();
    assert!(fmt_command(&matches).is_ok())
}

fn test_vet_cmd() {
    let test_path = std::path::Path::new(".")
        .join("src")
        .join("test_data")
        .join("vet");
    let data_file = test_path.join("data.json");
    let kcl_file = test_path.join("test.k");
    let matches = app().get_matches_from(&[
        ROOT_CMD,
        "vet",
        data_file.to_str().unwrap(),
        kcl_file.to_str().unwrap(),
    ]);
    let matches = matches.subcommand_matches("vet").unwrap();
    assert!(vet_command(&matches).is_ok())
}

fn work_dir() -> std::path::PathBuf {
    std::path::Path::new(".")
        .join("src")
        .join("test_data")
        .join("settings")
}

fn settings_arguments(path: std::path::PathBuf) -> Vec<String> {
    vec![
        ROOT_CMD.to_string(),
        "run".to_string(),
        "-Y".to_string(),
        path.to_str().unwrap().to_string(),
        "-r".to_string(),
        "-O".to_string(),
        "c.a=1".to_string(),
        "-O".to_string(),
        "c.b=1".to_string(),
        "-S".to_string(),
        "a.b.c".to_string(),
    ]
}

#[test]
fn test_external_cmd() {
    let matches = app().get_matches_from(&[ROOT_CMD, "run", "-E", "test_name=test_path"]);
    let matches = matches.subcommand_matches("run").unwrap();
    let pair = hashmaps_from_matches(matches, "package_map")
        .unwrap()
        .unwrap();
    assert_eq!(pair.len(), 1);
    assert!(pair.contains_key("test_name"));
    assert_eq!(pair.get("test_name").unwrap(), "test_path");
}

#[test]
fn test_version_cmd() {
    let matches = app().get_matches_from(&[ROOT_CMD, "version"]);
    assert!(matches.subcommand_matches("version").is_some())
}

#[test]
fn test_multi_external_cmd() {
    let matches = app().get_matches_from(&[
        ROOT_CMD,
        "run",
        "-E",
        "test_name=test_path",
        "-E",
        "test_name1=test_path1",
    ]);
    let matches = matches.subcommand_matches("run").unwrap();
    let pair = hashmaps_from_matches(matches, "package_map")
        .unwrap()
        .unwrap();

    assert_eq!(pair.len(), 2);
    assert!(pair.contains_key("test_name"));
    assert!(pair.contains_key("test_name1"));
    assert_eq!(pair.get("test_name").unwrap(), "test_path");
    assert_eq!(pair.get("test_name1").unwrap(), "test_path1");
}

#[test]
fn test_multi_external_with_same_key_cmd() {
    let matches = app().get_matches_from(&[
        ROOT_CMD,
        "run",
        "-E",
        "test_name=test_path",
        "-E",
        "test_name=test_path1",
    ]);
    let matches = matches.subcommand_matches("run").unwrap();
    let pair = hashmaps_from_matches(matches, "package_map")
        .unwrap()
        .unwrap();
    assert_eq!(pair.len(), 1);
    assert!(pair.contains_key("test_name"));
    assert_eq!(pair.get("test_name").unwrap(), "test_path1");
}

#[test]
fn test_external_cmd_invalid() {
    let invalid_cases: [&str; 5] = [
        "test_nametest_path",
        "test_name=test_path=test_suffix",
        "=test_path",
        "test_name=",
        "=test_name=test_path=",
    ];
    for case in invalid_cases {
        let matches = app().get_matches_from(&[ROOT_CMD, "run", "-E", case]);
        let matches = matches.subcommand_matches("run").unwrap();
        match hashmaps_from_matches(matches, "package_map").unwrap() {
            Ok(_) => {
                panic!("unreachable code.")
            }
            Err(err) => {
                assert!(format!("{:?}", err).contains("Invalid value for top level arguments"));
            }
        };
    }
}

#[test]
// All the unit test cases in [`test_run_command`] can not be executed concurrently.
fn test_run_command() {
    test_vet_cmd();
    test_run_command_with_import();
    test_run_command_with_konfig();
}

fn test_run_command_with_import() {
    let vendor_path = PathBuf::from("./src/test_data/cases/vendor");

    let test_cases = vec!["import_1"];
    let test_case_root = PathBuf::from("./src/test_data/cases")
        .canonicalize()
        .unwrap();

    for test_case in test_cases {
        check_run_command_with_env(
            test_case_root.join(test_case),
            vendor_path.canonicalize().unwrap().display().to_string(),
        );
    }
}

fn test_run_command_with_konfig() {
    let vendor_path = PathBuf::from("../../test/integration");

    let test_cases = vec!["import_konfig_1"];
    let test_case_root = PathBuf::from("./src/test_data/cases")
        .canonicalize()
        .unwrap();

    for test_case in test_cases {
        check_run_command_with_env(
            test_case_root.join(test_case),
            vendor_path.canonicalize().unwrap().display().to_string(),
        );
    }
}

/// rust crate [`gag`]: https://crates.io/crates/gag
/// allows redirecting stderr or stdout either to a file or to nothing,
/// but it only works on unix systems.
/// After [`gag`] can better support windows in the future, it may be considered to test the `println!`.
fn check_run_command_with_env(test_case_path: PathBuf, kcl_pkg_path_env: String) {
    env::set_var(KCL_PKG_PATH, kcl_pkg_path_env);

    let test_case_expect_file = test_case_path.join("stdout").display().to_string();
    let expect = fs::read_to_string(test_case_expect_file).expect("Unable to read file");

    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.join("main.k").display().to_string(),
    ]);

    let mut buf = Vec::new();
    run_command(matches.subcommand_matches("run").unwrap(), &mut buf).unwrap();

    #[cfg(target_os = "windows")]
    let expect = expect.replace("\r\n", "\n");
    assert_eq!(String::from_utf8(buf).unwrap(), expect);
}
