use lsp_types::notification::{
    Cancel, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
    DidOpenTextDocument, DidSaveTextDocument,
};
use std::collections::HashSet;

use crate::util::apply_document_changes;
use crate::{
    analysis::OpenFileInfo, dispatcher::NotificationDispatcher, from_lsp,
    state::LanguageServerState,
};

impl LanguageServerState {
    pub fn on_notification(
        &mut self,
        notification: lsp_server::Notification,
    ) -> anyhow::Result<()> {
        NotificationDispatcher::new(self, notification)
            .on::<DidOpenTextDocument>(LanguageServerState::on_did_open_text_document)?
            .on::<DidChangeTextDocument>(LanguageServerState::on_did_change_text_document)?
            .on::<DidSaveTextDocument>(LanguageServerState::on_did_save_text_document)?
            .on::<DidCloseTextDocument>(LanguageServerState::on_did_close_text_document)?
            .on::<DidChangeWatchedFiles>(LanguageServerState::on_did_change_watched_files)?
            .on::<Cancel>(LanguageServerState::cancel)?
            .finish();
        Ok(())
    }

    fn cancel(&mut self, params: lsp_types::CancelParams) -> anyhow::Result<()> {
        let id: lsp_server::RequestId = match params.id {
            lsp_types::NumberOrString::Number(id) => id.into(),
            lsp_types::NumberOrString::String(id) => id.into(),
        };
        self.request_queue.incoming.complete(&id);
        Ok(())
    }

    /// Called when a `DidOpenTextDocument` notification was received.
    fn on_did_open_text_document(
        &mut self,
        params: lsp_types::DidOpenTextDocumentParams,
    ) -> anyhow::Result<()> {
        let path = from_lsp::abs_path(&params.text_document.uri)?;
        self.log_message(format!("on did open file: {:?}", path));
        let mut vfs = self.vfs.write();
        vfs.set_file_contents(
            path.clone().into(),
            Some(params.text_document.text.into_bytes()),
        );
        if let Some(id) = vfs.file_id(&path.into()) {
            self.opened_files.write().insert(
                id,
                OpenFileInfo {
                    version: params.text_document.version,
                    workspaces: HashSet::new(),
                },
            );
        }
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

        // Update vfs
        let vfs = &mut *self.vfs.write();
        let file_id = vfs
            .file_id(&path.clone().into())
            .ok_or(anyhow::anyhow!("Already checked that the file_id exists!"))?;

        let mut text = String::from_utf8(vfs.file_contents(file_id).to_vec())?;
        apply_document_changes(&mut text, content_changes);
        vfs.set_file_contents(path.into(), Some(text.clone().into_bytes()));
        let mut opened_files = self.opened_files.write();
        let file_info = opened_files.get_mut(&file_id).unwrap();
        file_info.version = text_document.version;
        drop(opened_files);

        Ok(())
    }

    /// Called when a `DidCloseTextDocument` notification was received.
    fn on_did_close_text_document(
        &mut self,
        params: lsp_types::DidCloseTextDocumentParams,
    ) -> anyhow::Result<()> {
        let path = from_lsp::abs_path(&params.text_document.uri)?;
        self.log_message(format!("on did_close file: {:?}", path));

        if let Some(id) = self.vfs.read().file_id(&path.clone().into()) {
            self.opened_files.write().remove(&id);
        }

        // Update vfs
        let vfs = &mut *self.vfs.write();
        vfs.set_file_contents(path.clone().into(), None);
        self.loader.handle.invalidate(path);

        Ok(())
    }

    /// Called when a `DidChangeWatchedFiles` was received
    fn on_did_change_watched_files(
        &mut self,
        params: lsp_types::DidChangeWatchedFilesParams,
    ) -> anyhow::Result<()> {
        for change in params.changes {
            let path = from_lsp::abs_path(&change.uri)?;
            self.loader.handle.invalidate(path.clone());
        }

        Ok(())
    }
}
