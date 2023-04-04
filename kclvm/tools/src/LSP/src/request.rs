use std::time::Instant;

use anyhow::Ok;
use crossbeam_channel::Sender;

use crate::{
    completion::completion,
    dispatcher::RequestDispatcher,
    from_lsp::kcl_pos,
    goto_def::goto_definition,
    state::{log_message, LanguageServerSnapshot, LanguageServerState, Task},
    util::{parse_param_and_compile, Param},
};

impl LanguageServerState {
    /// Handles a language server protocol request
    pub(super) fn on_request(
        &mut self,
        request: lsp_server::Request,
        request_received: Instant,
    ) -> anyhow::Result<()> {
        log_message(format!("on request {:?}", request), &self.task_sender)?;
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
            .on::<lsp_types::request::Completion>(handle_completion)?
            .finish();

        Ok(())
    }
}

/// Called when a `GotoDefinition` request was received.
pub(crate) fn handle_goto_definition(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::GotoDefinitionParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::GotoDefinitionResponse>> {
    let file = params
        .text_document_position_params
        .text_document
        .uri
        .path();

    let (program, prog_scope, _) = parse_param_and_compile(
        Param {
            file: file.to_string(),
        },
        Some(snapshot.vfs),
    )
    .unwrap();
    let kcl_pos = kcl_pos(file, params.text_document_position_params.position);
    let res = goto_definition(&program, &kcl_pos, &prog_scope);
    if res.is_none() {
        log_message("Definition not found".to_string(), &sender)?;
    }
    Ok(res)
}

/// Called when a `Completion` request was received.
pub(crate) fn handle_completion(
    snapshot: LanguageServerSnapshot,
    params: lsp_types::CompletionParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::CompletionResponse>> {
    let file = params.text_document_position.text_document.uri.path();

    let (program, prog_scope, _) = parse_param_and_compile(
        Param {
            file: file.to_string(),
        },
        Some(snapshot.vfs),
    )
    .unwrap();
    let kcl_pos = kcl_pos(file, params.text_document_position.position);
    log_message(
        format!(
            "handle_completion {:?}",
            params.text_document_position.position
        ),
        &sender,
    )?;
    let completion_trigger_character = params
        .context
        .and_then(|ctx| ctx.trigger_character)
        .and_then(|s| s.chars().next());

    let res = completion(
        completion_trigger_character,
        &program,
        &kcl_pos,
        &prog_scope,
    );
    Ok(res)
}
