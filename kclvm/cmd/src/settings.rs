use anyhow::Result;
use clap::ArgMatches;
use kclvm_config::settings::{build_settings_pathbuf, SettingsPathBuf};
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
    let setting_files = if let Some(files) = matches.values_of("setting") {
        Some(files.into_iter().collect::<Vec<&str>>())
    } else {
        None
    };
    build_settings_pathbuf(
        files.as_slice(),
        output,
        setting_files,
        matches.occurrences_of("debug") > 0,
        matches.occurrences_of("disable_none") > 0,
    )
}
