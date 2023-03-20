use std::sync::Arc;

use kclvm_ast::ast::Program;
use kclvm_driver::lookup_compile_unit;
use kclvm_error::Position as KCLPos;
use kclvm_parser::{load_program, ParseSession};
use kclvm_sema::resolver::{resolve_program, scope::ProgramScope};
use lsp_types::{Position, Url};
use parking_lot::RwLockReadGuard;
use ra_ap_vfs::{FileId, Vfs};
use serde::{de::DeserializeOwned, Serialize};

use crate::from_lsp::kcl_pos;

#[allow(unused)]
/// Deserializes a `T` from a json value.
pub(crate) fn from_json<T: DeserializeOwned>(
    what: &'static str,
    json: serde_json::Value,
) -> anyhow::Result<T> {
    T::deserialize(&json)
        .map_err(|e| anyhow::anyhow!("could not deserialize {}: {}: {}", what, e, json))
}

/// Converts the `T` to a json value
pub(crate) fn to_json<T: Serialize>(value: T) -> anyhow::Result<serde_json::Value> {
    serde_json::to_value(value).map_err(|e| anyhow::anyhow!("could not serialize to json: {}", e))
}

pub fn get_file_name(vfs: RwLockReadGuard<Vfs>, file_id: FileId) -> anyhow::Result<String> {
    if let Some(path) = vfs.file_path(file_id).as_path() {
        Ok(path
            .as_ref()
            .to_str()
            .ok_or(anyhow::anyhow!("Failed to get file file"))?
            .to_string())
    } else {
        Err(anyhow::anyhow!(
            "{} isn't on the file system.",
            vfs.file_path(file_id)
        ))
    }
}

pub(crate) struct Param {
    pub url: Url,
    pub pos: Position,
}

pub(crate) fn parse_param_and_compile(param: Param) -> (KCLPos, Program, ProgramScope) {
    let file = param.url.path();
    let kcl_pos = kcl_pos(file, param.pos);
    let (files, cfg) = lookup_compile_unit(file);
    let files: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    let mut program = load_program(Arc::new(ParseSession::default()), &files, cfg).unwrap();
    let prog_scope = resolve_program(&mut program);
    (kcl_pos, program, prog_scope)
}
