use crate::gpyrpc::{
    CliConfig, Error, KeyValuePair, LoadSettingsFilesResult, Message, Position, Scope, ScopeIndex,
    Symbol, SymbolIndex,
};
use crate::service::ty::kcl_ty_to_pb_ty;
use kclvm_config::settings::SettingsFile;
use kclvm_error::Diagnostic;
use kclvm_loader::{ScopeInfo, SymbolInfo};
use kclvm_sema::core::{scope::ScopeRef, symbol::SymbolRef};

pub(crate) trait IntoLoadSettingsFiles {
    /// Convert self into the LoadSettingsFiles structure.
    fn into_load_settings_files(self, files: &[String]) -> LoadSettingsFilesResult;
}

pub(crate) trait IntoError {
    fn into_error(self) -> Error;
}

pub(crate) trait IntoSymbolIndex {
    fn into_symbol_index(self) -> SymbolIndex;
}

pub(crate) trait IntoSymbol {
    fn into_symbol(self) -> Symbol;
}

pub(crate) trait IntoScope {
    fn into_scope(self) -> Scope;
}

pub(crate) trait IntoScopeIndex {
    fn into_scope_index(self) -> ScopeIndex;
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
                show_hidden: config.show_hidden.unwrap_or_default(),
                fast_eval: config.fast_eval.unwrap_or_default(),
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

impl IntoSymbolIndex for SymbolRef {
    fn into_symbol_index(self) -> SymbolIndex {
        let (index, generation) = self.get_id().into_raw_parts();
        SymbolIndex {
            i: index as u64,
            g: generation,
            kind: format!("{:?}", self.get_kind()),
        }
    }
}

impl IntoScopeIndex for ScopeRef {
    fn into_scope_index(self) -> ScopeIndex {
        let (index, generation) = self.get_id().into_raw_parts();
        ScopeIndex {
            i: index as u64,
            g: generation,
            kind: format!("{:?}", self.get_kind()),
        }
    }
}

impl IntoSymbol for SymbolInfo {
    fn into_symbol(self) -> Symbol {
        Symbol {
            ty: Some(kcl_ty_to_pb_ty(&self.ty)),
            name: self.name,
            owner: self.owner.map(|o| o.into_symbol_index()),
            def: self.def.map(|d| d.into_symbol_index()),
            attrs: self.attrs.iter().map(|a| a.into_symbol_index()).collect(),
            is_global: self.is_global,
        }
    }
}

impl IntoScope for ScopeInfo {
    fn into_scope(self) -> Scope {
        Scope {
            kind: format!("{:?}", self.kind),
            parent: self.parent.map(|o| o.into_scope_index()),
            owner: self.owner.map(|o| o.into_symbol_index()),
            children: self.children.iter().map(|a| a.into_scope_index()).collect(),
            defs: self.defs.iter().map(|a| a.into_symbol_index()).collect(),
        }
    }
}
