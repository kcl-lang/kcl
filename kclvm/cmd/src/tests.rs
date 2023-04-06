use crate::{app, settings::build_settings};

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
        "kclvm_cli".to_string(),
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
