use std::time::Instant;

use crossbeam_channel::Sender;

use crate::{
    dispatcher::RequestDispatcher,
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
        log_message(
            format!("on request {:?}", request.method),
            &self.task_sender,
        )?;
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
            .finish();

        Ok(())
    }
}

/// Find definition of location.
/// Response can be single location, multiple Locations ,a link or None
pub(crate) fn handle_goto_definition(
    _snapshot: LanguageServerSnapshot,
    params: lsp_types::GotoDefinitionParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::GotoDefinitionResponse>> {
    let (kcl_pos, program, prog_scope) = parse_param_and_compile(Param {
        url: params.text_document_position_params.text_document.uri,
        pos: params.text_document_position_params.position,
    });
    let res = goto_definition(program, kcl_pos, prog_scope);
    if res.is_none() {
        log_message("Definition not found".to_string(), &sender)?;
    }
    Ok(res)
}
