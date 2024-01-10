use crate::gpyrpc::{CliConfig, Error, KeyValuePair, LoadSettingsFilesResult, Message, Position};
use kclvm_config::settings::SettingsFile;
use kclvm_error::Diagnostic;

pub(crate) trait IntoLoadSettingsFiles {
    /// Convert self into the LoadSettingsFiles structure.
    fn into_load_settings_files(self, files: &[String]) -> LoadSettingsFilesResult;
}

pub(crate) trait IntoError {
    fn into_error(self) -> Error;
}

impl IntoLoadSettingsFiles for SettingsFile {
    fn into_load_settings_files(self, files: &[String]) -> LoadSettingsFilesResult {
        LoadSettingsFilesResult {
            kcl_cli_configs: self.kcl_cli_configs.map(|config| CliConfig {
                files: files.to_vec(),
                output: config.output.unwrap_or_default(),
                overrides: config.overrides.unwrap_or_default(),
                path_selector: config.path_selector.unwrap_or_default(),
                strict_range_check: config.strict_range_check.unwrap_or_default(),
                disable_none: config.disable_none.unwrap_or_default(),
                verbose: config.verbose.unwrap_or_default() as i64,
                debug: config.debug.unwrap_or_default(),
                sort_keys: config.sort_keys.unwrap_or_default(),
                include_schema_type_path: config.include_schema_type_path.unwrap_or_default(),
            }),
            kcl_options: match self.kcl_options {
                Some(opts) => opts
                    .iter()
                    .map(|o| KeyValuePair {
                        key: o.key.to_string(),
                        value: o.value.to_string(),
                    })
                    .collect(),
                None => vec![],
            },
        }
    }
}

impl IntoError for Diagnostic {
    fn into_error(self) -> Error {
        Error {
            level: self.level.to_string(),
            code: format!(
                "{:?}",
                self.code.unwrap_or(kclvm_error::DiagnosticId::Error(
                    kclvm_error::ErrorKind::InvalidSyntax,
                ))
            ),
            messages: self
                .messages
                .iter()
                .map(|m| Message {
                    msg: m.message.clone(),
                    pos: Some(Position {
                        filename: m.range.0.filename.clone(),
                        line: m.range.0.line as i64,
                        column: m.range.0.column.unwrap_or_default() as i64,
                    }),
                })
                .collect(),
        }
    }
}
