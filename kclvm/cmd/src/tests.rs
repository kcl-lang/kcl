use std::{
    env,
    fs::{self, remove_file},
    path::{Path, PathBuf},
    sync::Arc,
};

use kclvm_config::modfile::KCL_PKG_PATH;
use kclvm_parser::ParseSession;
use kclvm_runner::{exec_program, MapErrorResult};

use crate::{
    app,
    run::run_command,
    settings::{build_settings, must_build_settings},
    util::hashmaps_from_matches,
};

#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file as symlink;

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
    test_run_command_with_import();
    test_run_command_with_konfig();
    test_load_cache_with_different_pkg();
    test_kcl_path_is_sym_link();
    test_compile_two_kcl_mod();
    test_main_pkg_not_found();
    test_multi_mod_file();
    test_instances_with_yaml();
    test_plugin_not_found();
    test_error_message_fuzz_matched();
    test_error_message_fuzz_unmatched();
    test_keyword_argument_error_message();
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

fn test_load_cache_with_different_pkg() {
    let main_path = PathBuf::from("./src/test_data/cache/main/main.k");
    let main_v1_path = PathBuf::from("./src/test_data/cache/main/main.k.v1");
    let main_v2_path = PathBuf::from("./src/test_data/cache/main/main.k.v2");
    let kcl1_v1_path = PathBuf::from("./src/test_data/cache/v1/kcl1");
    let kcl1_v2_path = PathBuf::from("./src/test_data/cache/v2/kcl1");

    // Copy the content from main.k.v1 to main.k
    fs::copy(main_v1_path, &main_path).unwrap();
    let matches = app().get_matches_from(&[
        ROOT_CMD,
        "run",
        main_path.to_str().unwrap(),
        "-E",
        format!("kcl1={}", kcl1_v1_path.display()).as_str(),
    ]);

    let matches = matches.subcommand_matches("run").unwrap();
    let mut buf = Vec::new();
    run_command(matches, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        "The_first_kcl_program: 1\n"
    );

    // Copy the content from main.k.v2 to main.k
    fs::copy(main_v2_path, &main_path).unwrap();
    let matches = app().get_matches_from(&[
        ROOT_CMD,
        "run",
        main_path.to_str().unwrap(),
        "-E",
        format!("kcl1={}", kcl1_v2_path.display()).as_str(),
    ]);

    let matches = matches.subcommand_matches("run").unwrap();
    let mut buf = Vec::new();
    run_command(matches, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        "The_first_kcl_program: 1\nkcl1_schema:\n  name: kcl1\n"
    );
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

fn test_kcl_path_is_sym_link() {
    let origin = "./src/test_data/sym_link/origin";
    let link = "./src/test_data/sym_link/sym_link";

    let origin_k_file_path = PathBuf::from(origin).join("a.k");
    let origin_matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        origin_k_file_path.to_str().unwrap(),
    ]);

    let mut origin_res = Vec::new();
    run_command(
        origin_matches.subcommand_matches("run").unwrap(),
        &mut origin_res,
    )
    .unwrap();

    // Create a symlink
    symlink(
        PathBuf::from(origin).canonicalize().unwrap(),
        Path::new(link),
    )
    .unwrap();

    let sym_link_k_file_path = PathBuf::from(link).join("a.k");
    let mut sym_link_res = Vec::new();
    let sym_link_matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        sym_link_k_file_path.to_str().unwrap(),
    ]);
    run_command(
        sym_link_matches.subcommand_matches("run").unwrap(),
        &mut sym_link_res,
    )
    .unwrap();

    // compare the result from origin kcl path and symlink kcl path.
    assert_eq!(
        String::from_utf8(sym_link_res),
        String::from_utf8(origin_res)
    );

    // clean up the symlink
    remove_file(link).unwrap();
}

fn test_compile_two_kcl_mod() {
    let test_case_path = PathBuf::from("./src/test_data/multimod");

    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.join("kcl1/main.k").display().to_string(),
        "${kcl2:KCL_MOD}/main.k",
        "-E",
        &format!("kcl2={}", test_case_path.join("kcl2").display().to_string()),
    ]);

    let mut buf = Vec::new();
    run_command(matches.subcommand_matches("run").unwrap(), &mut buf).unwrap();

    assert_eq!(
        "kcl1: hello 1\nkcl2: hello 2\n",
        String::from_utf8(buf).unwrap()
    );

    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.join("kcl2/main.k").display().to_string(),
        "${kcl1:KCL_MOD}/main.k",
        "-E",
        &format!("kcl1={}", test_case_path.join("kcl1").display().to_string()),
    ]);

    let mut buf = Vec::new();
    run_command(matches.subcommand_matches("run").unwrap(), &mut buf).unwrap();

    assert_eq!(
        "kcl2: hello 2\nkcl1: hello 1\n",
        String::from_utf8(buf).unwrap()
    );

    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.join("kcl3/main.k").display().to_string(),
        "${kcl4:KCL_MOD}/main.k",
        "-E",
        &format!(
            "kcl4={}",
            test_case_path
                .join("kcl3")
                .join("kcl4")
                .display()
                .to_string()
        ),
    ]);

    let mut buf = Vec::new();
    run_command(matches.subcommand_matches("run").unwrap(), &mut buf).unwrap();

    assert_eq!(
        "k3: Hello World 3\nk4: Hello World 4\n",
        String::from_utf8(buf).unwrap()
    );

    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path
            .join("kcl3/kcl4/main.k")
            .display()
            .to_string(),
        "${kcl3:KCL_MOD}/main.k",
        "-E",
        &format!("kcl3={}", test_case_path.join("kcl3").display().to_string()),
    ]);

    let mut buf = Vec::new();
    run_command(matches.subcommand_matches("run").unwrap(), &mut buf).unwrap();

    assert_eq!(
        "k4: Hello World 4\nk3: Hello World 3\n",
        String::from_utf8(buf).unwrap()
    );
}

fn test_instances_with_yaml() {
    let test_cases = [
        "test_inst_1",
        "test_inst_2",
        "test_inst_3",
        "test_inst_4",
        "test_inst_5",
        "test_inst_6",
        "test_inst_7",
        "test_inst_8",
        "test_inst_9",
        "test_inst_10",
        "test_inst_11/test_inst_111",
    ];

    for case in &test_cases {
        let expected = format!("{}/expected", case);
        let case_yaml = format!("{}/kcl.yaml", case);
        test_instances(&case_yaml, &expected);
    }
}

fn test_instances(kcl_yaml_path: &str, expected_file_path: &str) {
    let test_case_path = PathBuf::from("./src/test_data/instances");
    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        "-Y",
        &test_case_path.join(kcl_yaml_path).display().to_string(),
    ]);

    let mut buf = Vec::new();
    run_command(matches.subcommand_matches("run").unwrap(), &mut buf).unwrap();
    let expect = fs::read_to_string(
        test_case_path
            .join(expected_file_path)
            .display()
            .to_string(),
    )
    .unwrap();

    assert_eq!(
        expect.replace("\r\n", "\n"),
        String::from_utf8(buf).unwrap()
    );
}

fn test_main_pkg_not_found() {
    let test_case_path = PathBuf::from("./src/test_data/multimod");

    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        "${kcl3:KCL_MOD}/main.k",
        "-E",
        &format!("kcl3={}", test_case_path.join("kcl3").display().to_string()),
    ]);
    let settings = must_build_settings(matches.subcommand_matches("run").unwrap());
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into().unwrap())
        .map_err_to_result()
        .map_err(|e| e.to_string())
    {
        Ok(_) => panic!("unreachable code."),
        Err(msg) => assert_eq!(
            msg,
            "Cannot find the kcl file, please check the file path ${kcl3:KCL_MOD}/main.k"
        ),
    }
}

fn test_multi_mod_file() {
    let test_case_path = PathBuf::from("./src/test_data/multimod");

    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.join("kcl1").display().to_string(),
        &test_case_path.join("kcl2").display().to_string(),
    ]);
    let settings = must_build_settings(matches.subcommand_matches("run").unwrap());
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into().unwrap()) {
        Ok(res) => {
            assert_eq!(res.yaml_result, "kcl1: hello 1\nkcl2: hello 2");
            assert_eq!(
                res.json_result,
                "{\"kcl1\": \"hello 1\", \"kcl2\": \"hello 2\"}"
            );
        }
        Err(_) => panic!("unreachable code."),
    }
}

fn test_plugin_not_found() {
    let test_case_path = PathBuf::from("./src/test_data/plugin/plugin_not_found");
    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        test_case_path.as_path().display().to_string().as_str(),
    ]);
    let settings = must_build_settings(matches.subcommand_matches("run").unwrap());
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into().unwrap()).map_err_to_result().map_err(|e|e.to_string()) {
        Ok(_) => panic!("unreachable code."),
        Err(msg) => assert!(msg.contains("the plugin package `kcl_plugin.not_exist` is not found, please confirm if plugin mode is enabled")),
    }
}

fn test_error_message_fuzz_matched() {
    let test_case_path = PathBuf::from("./src/test_data/fuzz_match/main.k");
    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.canonicalize().unwrap().display().to_string(),
    ]);
    let settings = must_build_settings(matches.subcommand_matches("run").unwrap());
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into().unwrap())
        .map_err_to_result()
        .map_err(|e| e.to_string())
    {
        Ok(_) => panic!("unreachable code."),
        Err(msg) => {
            assert!(msg.contains("attribute 'a' not found in 'Person', did you mean '[\"aa\"]'?"))
        }
    }
}

fn test_error_message_fuzz_unmatched() {
    let test_case_path = PathBuf::from("./src/test_data/fuzz_match/main_unmatched.k");
    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.canonicalize().unwrap().display().to_string(),
    ]);
    let settings = must_build_settings(matches.subcommand_matches("run").unwrap());
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into().unwrap())
        .map_err_to_result()
        .map_err(|e| e.to_string())
    {
        Ok(_) => panic!("unreachable code."),
        Err(msg) => {
            assert!(msg.contains("attribute 'a' not found in 'Person'"))
        }
    }
}

fn test_keyword_argument_error_message() {
    let test_case_path = PathBuf::from("./src/test_data/failed/keyword_argument_error.k");
    let matches = app().arg_required_else_help(true).get_matches_from(&[
        ROOT_CMD,
        "run",
        &test_case_path.canonicalize().unwrap().display().to_string(),
    ]);
    let settings = must_build_settings(matches.subcommand_matches("run").unwrap());
    let sess = Arc::new(ParseSession::default());
    match exec_program(sess.clone(), &settings.try_into().unwrap())
        .map_err_to_result()
        .map_err(|e| e.to_string())
    {
        Ok(_) => panic!("unreachable code."),
        Err(msg) => {
            assert!(msg.contains("keyword argument 'ID' not found"));
        }
    }
}
