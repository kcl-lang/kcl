use anyhow::Result;
use clap::ArgMatches;
use kclvm_config::settings::{load_file, merge_settings, SettingsFile};
use kclvm_error::Handler;
use kclvm_runtime::PanicInfo;

/// Build settings from arg matches.
pub(crate) fn must_build_settings(matches: &ArgMatches) -> SettingsFile {
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
            SettingsFile::default()
        }
    }
}

/// Build settings from arg matches.
pub(crate) fn build_settings(matches: &ArgMatches) -> Result<SettingsFile> {
    let files: Vec<&str> = match matches.values_of("input") {
        Some(files) => files.into_iter().collect::<Vec<&str>>(),
        None => vec![],
    };
    let debug_mode = matches.occurrences_of("debug") > 0;
    let disable_none = matches.occurrences_of("disable_none") > 0;
    let output = matches.value_of("output").map(|v| v.to_string());

    let mut settings = if let Some(files) = matches.values_of("setting") {
        let files: Vec<&str> = files.into_iter().collect::<Vec<&str>>();
        let mut settings = vec![];
        for f in &files {
            settings.push(load_file(f)?);
        }
        merge_settings(&settings)
    } else {
        SettingsFile::new()
    };
    if let Some(config) = &mut settings.kcl_cli_configs {
        if !files.is_empty() {
            config.files = Some(files.iter().map(|f| f.to_string()).collect());
        }
        config.output = output;
        config.debug = Some(debug_mode);
        config.disable_none = Some(disable_none);
    }
    Ok(settings)
}
