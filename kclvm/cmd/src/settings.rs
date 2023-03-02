use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use kclvm_config::settings::{
    load_file, merge_settings, SettingsFile, SettingsPathBuf, DEFAULT_SETTING_FILE,
};
use kclvm_error::Handler;
use kclvm_runtime::PanicInfo;

/// Build settings from arg matches.
pub(crate) fn must_build_settings(matches: &ArgMatches) -> SettingsPathBuf {
    match build_settings(matches) {
        Ok(settings) => settings,
        Err(err) => {
            // New an error handler.
            let mut handler = Handler::default();
            handler
                .add_panic_info(&PanicInfo {
                    message: err.to_string(),
                    ..Default::default()
                })
                .abort_if_any_errors();
            SettingsPathBuf::default()
        }
    }
}

/// Build settings from arg matches.
pub(crate) fn build_settings(matches: &ArgMatches) -> Result<SettingsPathBuf> {
    let files: Vec<&str> = match matches.values_of("input") {
        Some(files) => files.into_iter().collect::<Vec<&str>>(),
        None => vec![],
    };
    let output = matches.value_of("output").map(|v| v.to_string());

    let mut path = None;
    let mut settings = if let Some(files) = matches.values_of("setting") {
        let files: Vec<&str> = files.into_iter().collect::<Vec<&str>>();
        let mut settings = vec![];
        for file in &files {
            let s = load_file(file)?;
            if !s.input().is_empty() {
                path = Some(
                    PathBuf::from(file)
                        .parent()
                        .map(|p| p.to_path_buf())
                        .ok_or(anyhow::anyhow!("The parent path of {file} is not found"))?,
                )
            }
            settings.push(s);
        }
        merge_settings(&settings)
    // If exists default kcl.yaml, load it.
    } else if std::fs::metadata(DEFAULT_SETTING_FILE).is_ok() {
        path = Some(
            PathBuf::from(DEFAULT_SETTING_FILE)
                .parent()
                .map(|p| p.to_path_buf())
                .ok_or(anyhow::anyhow!(
                    "The parent path of {DEFAULT_SETTING_FILE} is not found"
                ))?,
        );
        load_file(DEFAULT_SETTING_FILE)?
    } else {
        SettingsFile::default()
    };
    if let Some(config) = &mut settings.kcl_cli_configs {
        if !files.is_empty() {
            config.files = Some(files.iter().map(|f| f.to_string()).collect());
        }
        config.output = output;
        if matches.occurrences_of("debug") > 0 {
            config.debug = Some(true);
        }
        if matches.occurrences_of("disable_none") > 0 {
            config.disable_none = Some(true);
        }
    }
    Ok(SettingsPathBuf::new(path, settings))
}
