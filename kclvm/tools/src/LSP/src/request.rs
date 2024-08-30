use anyhow::anyhow;
use crossbeam_channel::Sender;

use kclvm_sema::info::is_valid_kcl_name;
use lsp_types::{Location, SemanticTokensResult, TextEdit};
use ra_ap_vfs::VfsPath;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::{
    analysis::{AnalysisDatabase, DBState},
    completion::completion,
    dispatcher::RequestDispatcher,
    document_symbol::document_symbol,
    error::LSPError,
    find_refs::find_refs,
    formatting::format,
    from_lsp::{self, file_path_from_url, kcl_pos},
    goto_def::goto_def,
    hover,
    inlay_hints::inlay_hints,
    quick_fix,
    semantic_token::semantic_tokens_full,
    signature_help::signature_help,
    state::{log_message, LanguageServerSnapshot, LanguageServerState, Task},
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
            .on::<lsp_types::request::HoverRequest>(handle_hover)?
            .on::<lsp_types::request::DocumentSymbolRequest>(handle_document_symbol)?
            .on::<lsp_types::request::CodeActionRequest>(handle_code_action)?
            .on::<lsp_types::request::Formatting>(handle_formatting)?
            .on::<lsp_types::request::RangeFormatting>(handle_range_formatting)?
            .on::<lsp_types::request::Rename>(handle_rename)?
            .on::<lsp_types::request::SemanticTokensFullRequest>(handle_semantic_tokens_full)?
            .on::<lsp_types::request::InlayHintRequest>(handle_inlay_hint)?
            .on::<lsp_types::request::SignatureHelpRequest>(handle_signature_help)?
            .on_maybe_retry::<lsp_types::request::Completion>(handle_completion)?
            .finish();

        Ok(())
    }
}

impl LanguageServerSnapshot {
    // defend against non-kcl files
    pub(crate) fn verify_request_path(&self, path: &VfsPath, sender: &Sender<Task>) -> bool {
        self.verify_vfs(path, sender)
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

    /// Attempts to get db in cache, this function does not block.
    /// return Ok(Some(db)) -> Compile completed
    /// return Ok(None) -> In compiling or rw lock, retry to wait compile completed
    /// return Err(_) ->  Compile failed
    pub(crate) fn try_get_db(
        &self,
        path: &VfsPath,
        sender: &Sender<Task>,
    ) -> anyhow::Result<Option<Arc<AnalysisDatabase>>> {
        match self.try_get_db_state(path) {
            Ok(db) => match db {
                Some(db) => match db {
                    DBState::Ready(db) => Ok(Some(db.clone())),
                    DBState::Compiling(_) | DBState::Init => {
                        log_message(
                            format!("Try get {:?} db state: In compiling, retry", path),
                            sender,
                        )?;
                        Ok(None)
                    }
                    DBState::Failed(e) => {
                        log_message(
                            format!("Try get {:?} db state: Failed: {:?}", path, e),
                            sender,
                        )?;
                        Err(anyhow::anyhow!(e))
                    }
                },
                None => Ok(None),
            },
            Err(e) => Err(e),
        }
    }

    /// Attempts to get db in cache, this function does not block.
    /// return Ok(Some(db)) -> Compile completed
    /// return Ok(None) -> RWlock, retry to unlock
    /// return Err(_) ->  Compile failed
    pub(crate) fn try_get_db_state(&self, path: &VfsPath) -> anyhow::Result<Option<DBState>> {
        match self.vfs.try_read() {
            Some(vfs) => match vfs.file_id(path) {
                Some(file_id) => {
                    let open_file = self.opened_files.read();
                    let file_info = open_file.get(&file_id).unwrap();
                    match self.temporary_workspace.read().get(&file_id) {
                        Some(option_workspace) => match option_workspace {
                            Some(work_space) => match self.workspaces.try_read() {
                                Some(workspaces) => match workspaces.get(work_space) {
                                    Some(db) => Ok(Some(db.clone())),
                                    None => Err(anyhow::anyhow!(
                                        LSPError::AnalysisDatabaseNotFound(path.clone())
                                    )),
                                },
                                None => Ok(None),
                            },
                            None => Ok(None),
                        },

                        None => {
                            if file_info.workspaces.is_empty() {
                                return Err(anyhow::anyhow!(LSPError::WorkSpaceIsEmpty(
                                    path.clone()
                                )));
                            }
                            // todo: now just get first, need get all workspaces
                            let work_space = file_info.workspaces.iter().next().unwrap();
                            match self.workspaces.try_read() {
                                Some(workspaces) => match workspaces.get(work_space) {
                                    Some(db) => Ok(Some(db.clone())),
                                    None => Err(anyhow::anyhow!(
                                        LSPError::AnalysisDatabaseNotFound(path.clone())
                                    )),
                                },
                                None => Ok(None),
                            }
                        }
                    }
                }

                None => Err(anyhow::anyhow!(LSPError::FileIdNotFound(path.clone()))),
            },
            None => Ok(None),
        }
    }
}

pub(crate) fn handle_semantic_tokens_full(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::SemanticTokensParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<SemanticTokensResult>> {
    let file = file_path_from_url(&params.text_document.uri)?;
    let path: VfsPath = from_lsp::abs_path(&params.text_document.uri)?.into();
    let db = match snapshot.try_get_db(&path, &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
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
    };
    let db = match snapshot.try_get_db(&path.clone().into(), &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
    };
    let kcl_pos = kcl_pos(&file, params.text_document_position_params.position);
    let res = goto_def(&kcl_pos, &db.gs);
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
    let file = file_path_from_url(&params.text_document_position.text_document.uri)?;

    let path = from_lsp::abs_path(&params.text_document_position.text_document.uri)?;

    if !snapshot.verify_request_path(&path.clone().into(), &sender) {
        return Ok(None);
    }
    let db = match snapshot.try_get_db(&path.clone().into(), &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
    };
    let pos = kcl_pos(&file, params.text_document_position.position);
    let res = find_refs(&pos, &db.gs);
    Ok(res)
}

/// Called when a `textDocument/completion` request was received.
pub(crate) fn handle_completion(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::CompletionParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::CompletionResponse>> {
    let file = file_path_from_url(&params.text_document_position.text_document.uri)?;
    let path: VfsPath =
        from_lsp::abs_path(&params.text_document_position.text_document.uri)?.into();
    if !snapshot.verify_request_path(&path.clone().into(), &sender) {
        return Ok(None);
    }

    let db_state = match snapshot.try_get_db_state(&path) {
        Ok(option_state) => match option_state {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
    };

    let completion_trigger_character = params
        .context
        .and_then(|ctx| ctx.trigger_character)
        .and_then(|s| s.chars().next());

    if matches!(completion_trigger_character, Some('\n')) {
        match db_state {
            DBState::Compiling(_) | DBState::Init => return Err(anyhow!(LSPError::Retry)),
            DBState::Failed(_) => return Ok(None),
            _ => {}
        }
    }

    let db = match db_state {
        DBState::Ready(db) => db,
        DBState::Compiling(db) => db,
        DBState::Init => return Err(anyhow!(LSPError::Retry)),
        DBState::Failed(_) => return Ok(None),
    };

    let kcl_pos = kcl_pos(&file, params.text_document_position.position);

    let metadata = snapshot
        .entry_cache
        .read()
        .get(&file)
        .and_then(|metadata| metadata.0 .2.clone());

    let res = completion(
        completion_trigger_character,
        &db.prog,
        &kcl_pos,
        &db.gs,
        &*snapshot.tool.read(),
        metadata,
    );

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
    let db = match snapshot.try_get_db(&path.clone().into(), &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
    };
    let kcl_pos = kcl_pos(&file, params.text_document_position_params.position);
    let res = hover::hover(&kcl_pos, &db.gs);
    if res.is_none() {
        log_message("Hover definition not found".to_string(), &sender)?;
    }
    Ok(res)
}

/// Called when a `textDocument/documentSymbol` request was received.
pub(crate) fn handle_document_symbol(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::DocumentSymbolParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::DocumentSymbolResponse>> {
    let file = file_path_from_url(&params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document.uri)?;
    let db = match snapshot.try_get_db(&path.clone().into(), &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
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
    let db = match snapshot.try_get_db(&path.clone().into(), &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
    };
    let kcl_pos = kcl_pos(&file, params.text_document_position.position);
    let references = find_refs(&kcl_pos, &db.gs);
    match references {
        Some(locations) => {
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
            return anyhow::Ok(Some(workspace_edit));
        }
        None => {}
    }
    Ok(None)
}

pub(crate) fn handle_inlay_hint(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::InlayHintParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<Vec<lsp_types::InlayHint>>> {
    let file = file_path_from_url(&params.text_document.uri)?;
    let path = from_lsp::abs_path(&params.text_document.uri)?;
    let db = match snapshot.try_get_db(&path.clone().into(), &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
    };
    let res = inlay_hints(&file, &db.gs);
    Ok(res)
}

pub(crate) fn handle_signature_help(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::SignatureHelpParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::SignatureHelp>> {
    let file = file_path_from_url(&params.text_document_position_params.text_document.uri)?;
    let pos = kcl_pos(&file, params.text_document_position_params.position);
    let trigger_character = params.context.and_then(|ctx| ctx.trigger_character);
    let path = from_lsp::abs_path(&params.text_document_position_params.text_document.uri)?;
    let db = match snapshot.try_get_db(&path.clone().into(), &sender) {
        Ok(option_db) => match option_db {
            Some(db) => db,
            None => return Err(anyhow!(LSPError::Retry)),
        },
        Err(_) => return Ok(None),
    };
    let res = signature_help(&pos, &db.gs, trigger_character);

    Ok(res)
}
