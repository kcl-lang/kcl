use crate::settings::build_settings;

#[test]
fn test_build_settings() {
    let work_dir = work_dir();
    let matches = mock_clap_app().get_matches_from(settings_arguments(work_dir.join("kcl.yaml")));
    let s = build_settings(&matches).unwrap();
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
    assert_eq!(s.settings().input(), vec!["hello.k".to_string()]);
}

#[test]
fn test_build_settings_fail() {
    assert!(build_settings(
        &mock_clap_app().get_matches_from(settings_arguments(work_dir().join("error_kcl.yaml")))
    )
    .is_err());
}

fn work_dir() -> std::path::PathBuf {
    std::path::Path::new(".")
        .join("src")
        .join("test_data")
        .join("settings")
}

fn settings_arguments<'a>(path: std::path::PathBuf) -> Vec<String> {
    vec![
        "kcl".to_string(),
        "-Y".to_string(),
        path.to_str().unwrap().to_string(),
    ]
}

fn mock_clap_app<'ctx>() -> clap::App<'ctx> {
    clap_app!(kcl =>
        (@arg input: ... "Sets the input file to use")
        (@arg output: -o --output +takes_value "Sets the LLVM IR/BC output file path")
        (@arg setting: ... -Y --setting +takes_value "Sets the input file to use")
        (@arg verbose: -v --verbose "Print test information verbosely")
        (@arg disable_none: -n --disable-none "Disable dumping None values")
        (@arg debug: -d --debug "Run in debug mode (for developers only)")
        (@arg sort_key: -k --sort "Sort result keys")
        (@arg argument: ... -D --argument "Specify the top-level argument")
    )
}
