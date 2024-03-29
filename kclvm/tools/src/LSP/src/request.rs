use anyhow::anyhow;
use crossbeam_channel::Sender;

use kclvm_sema::info::is_valid_kcl_name;
use lsp_types::{Location, SemanticTokensResult, TextEdit};
use ra_ap_vfs::VfsPath;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::error::LSPError;
use crate::{
    completion::completion,
    db::AnalysisDatabase,
    dispatcher::RequestDispatcher,
    document_symbol::document_symbol,
    find_refs::find_refs,
    formatting::format,
    from_lsp::{self, file_path_from_url, kcl_pos},
    goto_def::goto_definition_with_gs,
    hover, quick_fix,
    semantic_token::semantic_tokens_full,
    state::{log_message, LanguageServerSnapshot, LanguageServerState, Task},
    util::{compile_with_params, Params},
};

impl LanguageServerState {
    /// Handles a language server protocol request
    pub(super) fn on_request(
        &mut self,
        request: lsp_server::Request,
        request_received: Instant,
    ) -> anyhow::Result<()> {
        self.register_request(&request, request_received);

        // If a shutdown was requested earlier, immediately respond with an error
        if self.shutdown_requested {
            self.respond(lsp_server::Response::new_err(
                request.id,
                lsp_server::ErrorCode::InvalidRequest as i32,
                "shutdown was requested".to_owned(),
            ))?;
            return Ok(());
        }

        // Dispatch the event based on the type of event
        RequestDispatcher::new(self, request)
            .on_sync::<lsp_types::request::Shutdown>(|state, _request| {
                state.shutdown_requested = true;
                Ok(())
            })?
            .on::<lsp_types::request::GotoDefinition>(handle_goto_definition)?
            .on::<lsp_types::request::References>(handle_reference)?
            .on::<lsp_types::request::Completion>(handle_completion)?
            .on::<lsp_types::request::HoverRequest>(handle_hover)?
            .on::<lsp_types::request::DocumentSymbolRequest>(handle_document_symbol)?
            .on::<lsp_types::request::CodeActionRequest>(handle_code_action)?
            .on::<lsp_types::request::Formatting>(handle_formatting)?
            .on::<lsp_types::request::RangeFormatting>(handle_range_formatting)?
            .on::<lsp_types::request::Rename>(handle_rename)?
            .on::<lsp_types::request::SemanticTokensFullRequest>(handle_semantic_tokens_full)?
            .finish();

        Ok(())
    }
}

impl LanguageServerSnapshot {
    // defend against non-kcl files
    pub(crate) fn verify_request_path(&self, path: &VfsPath, sender: &Sender<Task>) -> bool {
        self.verify_vfs(path, sender) && self.verify_db(path, sender)
    }

    pub(crate) fn verify_vfs(&self, path: &VfsPath, sender: &Sender<Task>) -> bool {
        let valid = self.vfs.read().file_id(path).is_some();
        if !valid {
            let _ = log_message(
                format!("Vfs not contains: {}, request failed", path),
                sender,
            );
        }
        valid
    }

    pub(crate) fn verify_db(&self, path: &VfsPath, sender: &Sender<Task>) -> bool {
        let valid = self
            .db
            .read()
            .get(&self.vfs.read().file_id(path).unwrap())
            .is_some();
        if !valid {
            let _ = log_message(
                format!(
                    "LSP AnalysisDatabase not contains: {}, request failed",
                    path
                ),
                sender,
            );
        }
        valid
    }

    pub(crate) fn get_db(&self, path: &VfsPath) -> anyhow::Result<Arc<AnalysisDatabase>> {
        let file_id = self.vfs.read().file_id(path);
        match file_id {
            Some(id) => match self.db.read().get(&id) {
                Some(db) => Ok(Arc::clone(db)),
                None => Err(anyhow::anyhow!(LSPError::AnalysisDatabaseNotFound(
                    path.clone()
                ))),
            },
            None => Err(anyhow::anyhow!(LSPError::FileIdNotFound(path.clone()))),
        }
    }

    ///  Attempts to get db in cache, this function does not block.
    pub(crate) fn try_get_db(
        &self,
        path: &VfsPath,
    ) -> anyhow::Result<Option<Arc<AnalysisDatabase>>> {
        match self.vfs.try_read() {
            Some(vfs) => match vfs.file_id(path) {
                Some(file_id) => match self.db.try_read() {
                    Some(db) => match db.get(&file_id) {
                        Some(db) => Ok(Some(Arc::clone(db))),
                        None => Err(anyhow::anyhow!(LSPError::AnalysisDatabaseNotFound(
                            path.clone()
                        ))),
                    },
                    None => Ok(None),
                },
                None => Err(anyhow::anyhow!(LSPError::FileIdNotFound(path.clone()))),
            },
            None => Ok(None),
        }
    }
}

pub(crate) fn handle_semantic_tokens_full(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::SemanticTokensParams,
    _sender: Sender<Task>,
) -> anyhow::Result<Option<SemanticTokensResult>> {
    let file = file_path_from_url(&params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document.uri)?;
    let db = match snapshot.try_get_db(&path.clone().into())? {
        Some(db) => db,
        None => return Err(anyhow!(LSPError::Retry)),
    };
    let res = semantic_tokens_full(&file, &db.gs);
    Ok(res)
}

pub(crate) fn handle_formatting(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::DocumentFormattingParams,
    _sender: Sender<Task>,
) -> anyhow::Result<Option<Vec<TextEdit>>> {
    let file = file_path_from_url(&params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document.uri)?;
    let src = {
        let vfs = snapshot.vfs.read();
        let file_id = vfs
            .file_id(&path.into())
            .ok_or(anyhow::anyhow!("Already checked that the file_id exists!"))?;

        String::from_utf8(vfs.file_contents(file_id).to_vec())?
    };

    format(file, src, None)
}

pub(crate) fn handle_range_formatting(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::DocumentRangeFormattingParams,
    _sender: Sender<Task>,
) -> anyhow::Result<Option<Vec<TextEdit>>> {
    let file = file_path_from_url(&params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document.uri)?;
    let vfs = &*snapshot.vfs.read();

    let file_id = vfs
        .file_id(&path.clone().into())
        .ok_or(anyhow::anyhow!("Already checked that the file_id exists!"))?;

    let text = String::from_utf8(vfs.file_contents(file_id).to_vec())?;
    let range = from_lsp::text_range(&text, params.range);
    if let Some(src) = text.get(range) {
        format(file, src.to_owned(), Some(params.range))
    } else {
        Ok(None)
    }
}

/// Called when a `textDocument/codeAction` request was received.
pub(crate) fn handle_code_action(
    _snapshot: LanguageServerSnapshot,
    params: lsp_types::CodeActionParams,
    _sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::CodeActionResponse>> {
    let mut code_actions: Vec<lsp_types::CodeActionOrCommand> = vec![];
    code_actions.extend(quick_fix::quick_fix(
        &params.text_document.uri,
        &params.context.diagnostics,
    ));
    Ok(Some(code_actions))
}

/// Called when a `textDocument/definition` request was received.
pub(crate) fn handle_goto_definition(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::GotoDefinitionParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::GotoDefinitionResponse>> {
    let file = file_path_from_url(&params.text_document_position_params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document_position_params.text_document.uri)?;
    if !snapshot.verify_request_path(&path.clone().into(), &sender) {
        return Ok(None);
    }
    let db = snapshot.get_db(&path.clone().into())?;
    let kcl_pos = kcl_pos(&file, params.text_document_position_params.position);
    let res = goto_definition_with_gs(&db.prog, &kcl_pos, &db.gs);
    if res.is_none() {
        log_message("Definition item not found".to_string(), &sender)?;
    }
    Ok(res)
}

/// Called when a `textDocument/references` request was received
pub(crate) fn handle_reference(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::ReferenceParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<Vec<Location>>> {
    let include_declaration = params.context.include_declaration;
    let file = file_path_from_url(&params.text_document_position.text_document.uri)?;

    let path = from_lsp::abs_path(&params.text_document_position.text_document.uri)?;

    if !snapshot.verify_request_path(&path.clone().into(), &sender) {
        return Ok(None);
    }
    let db = snapshot.get_db(&path.clone().into())?;
    let pos = kcl_pos(&file, params.text_document_position.position);
    let log = |msg: String| log_message(msg, &sender);
    let module_cache = snapshot.module_cache.clone();
    let scope_cache = snapshot.scope_cache.clone();
    match find_refs(
        &db.prog,
        &pos,
        include_declaration,
        snapshot.word_index_map.clone(),
        Some(snapshot.vfs.clone()),
        log,
        &db.gs,
        module_cache,
        scope_cache,
    ) {
        core::result::Result::Ok(locations) => Ok(Some(locations)),
        Err(msg) => {
            log(format!("Find references failed: {msg}"))?;
            Ok(None)
        }
    }
}

/// Called when a `textDocument/completion` request was received.
pub(crate) fn handle_completion(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::CompletionParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::CompletionResponse>> {
    let file = file_path_from_url(&params.text_document_position.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document_position.text_document.uri)?;
    if !snapshot.verify_request_path(&path.clone().into(), &sender) {
        return Ok(None);
    }

    let kcl_pos = kcl_pos(&file, params.text_document_position.position);
    let completion_trigger_character = params
        .context
        .and_then(|ctx| ctx.trigger_character)
        .and_then(|s| s.chars().next());

    let db = match completion_trigger_character {
        // Some trigger characters need to re-compile
        Some(ch) => match ch {
            '=' | ':' => {
                match compile_with_params(Params {
                    file: file.clone(),
                    module_cache: snapshot.module_cache,
                    scope_cache: None,
                    vfs: Some(snapshot.vfs.clone()),
                }) {
                    Ok((prog, diags, gs)) => Arc::new(AnalysisDatabase { prog, diags, gs }),
                    Err(_) => return Ok(None),
                }
            }
            _ => snapshot.get_db(&path.clone().into())?,
        },

        None => snapshot.get_db(&path.clone().into())?,
    };

    let res = completion(completion_trigger_character, &db.prog, &kcl_pos, &db.gs);

    if res.is_none() {
        log_message("Completion item not found".to_string(), &sender)?;
    }
    Ok(res)
}

/// Called when a `textDocument/hover` request was received.
pub(crate) fn handle_hover(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::HoverParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::Hover>> {
    let file = file_path_from_url(&params.text_document_position_params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document_position_params.text_document.uri)?;
    if !snapshot.verify_request_path(&path.clone().into(), &sender) {
        return Ok(None);
    }
    let db = snapshot.get_db(&path.clone().into())?;
    let kcl_pos = kcl_pos(&file, params.text_document_position_params.position);
    let res = hover::hover(&db.prog, &kcl_pos, &db.gs);
    if res.is_none() {
        log_message("Hover definition not found".to_string(), &sender)?;
    }
    Ok(res)
}

/// Called when a `textDocument/documentSymbol` request was received.
pub(crate) fn handle_document_symbol(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::DocumentSymbolParams,
    _sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::DocumentSymbolResponse>> {
    let file = file_path_from_url(&params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document.uri)?;
    let db = match snapshot.try_get_db(&path.clone().into())? {
        Some(db) => db,
        None => return Err(anyhow!(LSPError::Retry)),
    };
    let res = document_symbol(&file, &db.gs);
    Ok(res)
}

/// Called when a `textDocument/rename` request was received.
pub(crate) fn handle_rename(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::RenameParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::WorkspaceEdit>> {
    // 1. check the new name validity
    let new_name = params.new_name;
    if !is_valid_kcl_name(new_name.as_str()) {
        return Err(anyhow!("Can not rename to: {new_name}, invalid name"));
    }

    // 2. find all the references of the symbol
    let file = file_path_from_url(&params.text_document_position.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document_position.text_document.uri)?;
    if !snapshot.verify_request_path(&path.clone().into(), &sender) {
        return Ok(None);
    }
    let db = snapshot.get_db(&path.clone().into())?;
    let kcl_pos = kcl_pos(&file, params.text_document_position.position);
    let log = |msg: String| log_message(msg, &sender);
    let references = find_refs(
        &db.prog,
        &kcl_pos,
        true,
        snapshot.word_index_map.clone(),
        Some(snapshot.vfs.clone()),
        log,
        &db.gs,
        snapshot.module_cache.clone(),
        snapshot.scope_cache.clone(),
    );
    match references {
        Result::Ok(locations) => {
            if locations.is_empty() {
                let _ = log("Symbol not found".to_string());
                anyhow::Ok(None)
            } else {
                // 3. return the workspaceEdit to rename all the references with the new name
                let mut workspace_edit = lsp_types::WorkspaceEdit::default();

                let changes = locations.into_iter().fold(
                    HashMap::new(),
                    |mut map: HashMap<lsp_types::Url, Vec<TextEdit>>, location| {
                        let uri = location.uri;
                        map.entry(uri.clone()).or_default().push(TextEdit {
                            range: location.range,
                            new_text: new_name.clone(),
                        });
                        map
                    },
                );
                workspace_edit.changes = Some(changes);
                anyhow::Ok(Some(workspace_edit))
            }
        }
        Err(msg) => {
            let err_msg = format!("Can not rename symbol: {msg}");
            log(err_msg.clone())?;
            Err(anyhow!(err_msg))
        }
    }
}
