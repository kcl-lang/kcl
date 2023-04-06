use lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument};

use crate::{
    dispatcher::NotificationDispatcher, from_lsp, state::LanguageServerState,
    util::apply_document_changes,
};

impl LanguageServerState {
    pub fn on_notification(
        &mut self,
        notification: lsp_server::Notification,
    ) -> anyhow::Result<()> {
        self.log_message(format!("on notification {:?}", notification));
        NotificationDispatcher::new(self, notification)
            .on::<DidOpenTextDocument>(LanguageServerState::on_did_open_text_document)?
            .on::<DidChangeTextDocument>(LanguageServerState::on_did_change_text_document)?
            .on::<DidSaveTextDocument>(LanguageServerState::on_did_save_text_document)?
            // .on::<DidCloseTextDocument>(LanguageServerState::on_did_close_text_document)?
            // .on::<DidChangeWatchedFiles>(LanguageServerState::on_did_change_watched_files)?
            .finish();
        Ok(())
    }

    /// Called when a `DidOpenTextDocument` notification was received.
    fn on_did_open_text_document(
        &mut self,
        params: lsp_types::DidOpenTextDocumentParams,
    ) -> anyhow::Result<()> {
        let path = from_lsp::abs_path(&params.text_document.uri)?;
        self.log_message(format!("on did open file: {:?}", path));
        self.vfs
            .write()
            .set_file_contents(path.into(), Some(params.text_document.text.into_bytes()));
        Ok(())
    }

    /// Called when a `DidChangeTextDocument` notification was received.
    fn on_did_save_text_document(
        &mut self,
        params: lsp_types::DidSaveTextDocumentParams,
    ) -> anyhow::Result<()> {
        let lsp_types::DidSaveTextDocumentParams {
            text_document,
            text: _,
        } = params;

        let path = from_lsp::abs_path(&text_document.uri)?;
        self.log_message(format!("on did save file: {:?}", path));
        Ok(())
    }

    /// Called when a `DidChangeTextDocument` notification was received.
    fn on_did_change_text_document(
        &mut self,
        params: lsp_types::DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        let lsp_types::DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;

        let path = from_lsp::abs_path(&text_document.uri)?;
        self.log_message(format!("on did_change file: {:?}", path));

        let vfs = &mut *self.vfs.write();
        let file_id = vfs
            .file_id(&path.clone().into())
            .ok_or(anyhow::anyhow!("Already checked that the file_id exists!"))?;

        let mut text = String::from_utf8(vfs.file_contents(file_id).to_vec())?;
        apply_document_changes(&mut text, content_changes);
        vfs.set_file_contents(path.into(), Some(text.into_bytes()));

        Ok(())
    }
}
