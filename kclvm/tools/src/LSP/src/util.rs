use std::{fs, sync::Arc};

use indexmap::IndexSet;
use kclvm_ast::ast::Program;
use kclvm_driver::lookup_compile_unit;
use kclvm_error::Diagnostic;
use kclvm_parser::{load_program, ParseSession};
use kclvm_sema::resolver::{resolve_program, scope::ProgramScope};
use lsp_types::Url;
use parking_lot::{RwLock, RwLockReadGuard};
use ra_ap_vfs::{FileId, Vfs};
use serde::{de::DeserializeOwned, Serialize};

use crate::from_lsp;

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
    pub file: String,
}

pub(crate) fn parse_param_and_compile(
    param: Param,
    vfs: Option<Arc<RwLock<Vfs>>>,
) -> anyhow::Result<(Program, ProgramScope, IndexSet<Diagnostic>)> {
    let (files, opt) = lookup_compile_unit(&param.file);
    let files: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let mut opt = opt.unwrap_or_default();

    // update opt.k_code_list
    if let Some(vfs) = vfs {
        let mut k_code_list = load_files_code_from_vfs(&files, vfs)?;
        opt.k_code_list.append(&mut k_code_list);
    }
    let mut program = load_program(Arc::new(ParseSession::default()), &files, Some(opt)).unwrap();
    let prog_scope = resolve_program(&mut program);

    Ok((
        program,
        prog_scope.clone(),
        prog_scope.handler.diagnostics.clone(),
    ))
}

/// Update text with TextDocumentContentChangeEvent param
pub(crate) fn apply_document_changes(
    old_text: &mut String,
    content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
) {
    for change in content_changes {
        match change.range {
            Some(range) => {
                let range = from_lsp::text_range(&old_text, range);
                old_text.replace_range(range, &change.text);
            }
            None => {
                *old_text = change.text;
            }
        }
    }
}

fn load_files_code_from_vfs(files: &[&str], vfs: Arc<RwLock<Vfs>>) -> anyhow::Result<Vec<String>> {
    let mut res = vec![];
    let vfs = &mut vfs.read();
    for file in files {
        let url = Url::from_file_path(file)
            .map_err(|_| anyhow::anyhow!("can't convert file to url: {}", file))?;
        let path = from_lsp::abs_path(&url)?;
        match vfs.file_id(&path.clone().into()) {
            Some(id) => {
                // Load code from vfs if exist
                res.push(String::from_utf8(vfs.file_contents(id).to_vec()).unwrap());
            }
            None => {
                // In order to ensure that k_file corresponds to k_code, load the code from the file system if not exist
                res.push(
                    fs::read_to_string(path)
                        .map_err(|_| anyhow::anyhow!("can't convert file to url: {}", file))?,
                );
            }
        }
    }
    Ok(res)
}
