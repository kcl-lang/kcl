use std::fs;

use lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument};

use crate::{dispatcher::NotificationDispatcher, from_lsp, state::LanguageServerState};

impl LanguageServerState {
    pub fn on_notification(
        &mut self,
        notification: lsp_server::Notification,
    ) -> anyhow::Result<()> {
        self.log_message(format!("on notification {:?}", notification.method));
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
            text,
        } = params;

        let path = from_lsp::abs_path(&text_document.uri)?;
        self.log_message(format!("on did save file: {:?}", path));

        let vfs = &mut *self.vfs.write();

        let contents = text.unwrap_or("".to_string()).into_bytes();

        vfs.set_file_contents(path.into(), Some(contents.clone()));
        Ok(())
    }

    /// Called when a `DidChangeTextDocument` notification was received.
    fn on_did_change_text_document(
        &mut self,
        params: lsp_types::DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        let lsp_types::DidChangeTextDocumentParams {
            text_document,
            content_changes: _,
        } = params;

        let path = from_lsp::abs_path(&text_document.uri)?;
        self.log_message(format!("on did_change file: {:?}", path));

        let vfs = &mut *self.vfs.write();
        let file_id = vfs
            .file_id(&path.clone().into())
            .ok_or(anyhow::anyhow!("Already checked that the file_id exists!"))?;

        let vfspath = vfs.file_path(file_id);
        let filename = vfspath
            .as_path()
            .ok_or(anyhow::anyhow!("Already checked that the file_id exists!"))?
            .as_ref()
            .to_str()
            .ok_or(anyhow::anyhow!("Already checked that the file_id exists!"))?;

        // todo: Update the u8 array directly based on `content_changes` instead of
        // reading the file from the file system.
        let contents = fs::read(filename)?;
        vfs.set_file_contents(path.into(), Some(contents.clone()));

        Ok(())
    }
}
