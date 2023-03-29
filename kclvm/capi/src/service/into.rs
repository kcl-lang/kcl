use crate::model::gpyrpc::{CliConfig, KeyValuePair, LoadSettingsFiles_Result};
use kclvm_config::settings::SettingsFile;
use protobuf::MessageField;

pub(crate) trait IntoLoadSettingsFiles {
    /// Convert self into the LoadSettingsFiles structure.
    fn into_load_settings_files(self, files: &[String]) -> LoadSettingsFiles_Result;
}

impl IntoLoadSettingsFiles for SettingsFile {
    fn into_load_settings_files(self, files: &[String]) -> LoadSettingsFiles_Result {
        LoadSettingsFiles_Result {
            kcl_cli_configs: match self.kcl_cli_configs {
                Some(config) => MessageField::some(CliConfig {
                    files: files.to_vec(),
                    output: config.output.unwrap_or_default(),
                    overrides: config.overrides.unwrap_or_default(),
                    path_selector: config.path_selector.unwrap_or_default(),
                    strict_range_check: config.strict_range_check.unwrap_or_default(),
                    disable_none: config.disable_none.unwrap_or_default(),
                    verbose: config.verbose.unwrap_or_default() as i64,
                    debug: config.debug.unwrap_or_default(),
                    ..Default::default()
                }),
                None => MessageField::none(),
            },
            kcl_options: match self.kcl_options {
                Some(opts) => opts
                    .iter()
                    .map(|o| KeyValuePair {
                        key: o.key.to_string(),
                        value: o.value.to_string(),
                        ..Default::default()
                    })
                    .collect(),
                None => vec![],
            },
            ..Default::default()
        }
    }
}
