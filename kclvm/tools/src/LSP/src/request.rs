use std::{sync::Arc, time::Instant};

use compiler_base_session::Session;
use crossbeam_channel::Sender;
use kclvm_driver::lookup_compile_unit;
use kclvm_parser::load_program;
use kclvm_sema::resolver::resolve_program;

use crate::{
    dispatcher::RequestDispatcher,
    from_lsp::kcl_pos,
    goto_def::goto_definition,
    state::{log_message, LanguageServerSnapshot, LanguageServerState, Task},
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

pub(crate) fn handle_goto_definition(
    _snapshot: LanguageServerSnapshot,
    params: lsp_types::GotoDefinitionParams,
    sender: Sender<Task>,
) -> anyhow::Result<Option<lsp_types::GotoDefinitionResponse>> {
    let file = params
        .text_document_position_params
        .text_document
        .uri
        .path();
    let pos = params.text_document_position_params.position;
    let kcl_pos = kcl_pos(file, pos);

    let (files, cfg) = lookup_compile_unit(file);
    let files: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

    let mut program = load_program(Arc::new(Session::default()), &files, cfg).unwrap();
    let prog_scope = resolve_program(&mut program);

    let res = goto_definition(program, kcl_pos, prog_scope);
    if res.is_none() {
        log_message("Definition not found".to_string(), &sender)?;
    }
    Ok(res)
}
