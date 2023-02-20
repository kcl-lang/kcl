use chrono::{Local, TimeZone};
use indexmap::IndexSet;
use kclvm_tools::lint::lint_files;
use kclvm_tools::util::lsp::kcl_diag_to_lsp_diags;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use kclvm_error::Diagnostic as KCLDiagnostic;

#[derive(Debug)]
struct Backend {
    client: Client,
}

struct TextDocumentItem {
    uri: Url,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
        })
        .await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
        })
        .await;
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
        })
        .await;

        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Get request: {} ",
                    Local
                        .timestamp_millis_opt(Local::now().timestamp_millis())
                        .unwrap()
                ),
            )
            .await;
        let uri = params.uri.clone();
        let file_name = uri.path();
        self.client
            .log_message(MessageType::INFO, "on change")
            .await;

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Start lint: {} ",
                    Local
                        .timestamp_millis_opt(Local::now().timestamp_millis())
                        .unwrap()
                ),
            )
            .await;

        let (errors, warnings) = lint_files(&[file_name], None);

        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "End lint: {} ",
                    Local
                        .timestamp_millis_opt(Local::now().timestamp_millis())
                        .unwrap()
                ),
            )
            .await;
        let diags: IndexSet<KCLDiagnostic> = errors
            .iter()
            .chain(warnings.iter())
            .cloned()
            .collect::<IndexSet<KCLDiagnostic>>();

        let diagnostics = diags
            .iter()
            .map(|diag| kcl_diag_to_lsp_diags(diag, file_name))
            .flatten()
            .collect::<Vec<Diagnostic>>();

        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, None)
            .await;
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Response to client: {} ",
                    Local
                        .timestamp_millis_opt(Local::now().timestamp_millis())
                        .unwrap()
                ),
            )
            .await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });

    Server::new(stdin, stdout, socket).serve(service).await;
}
