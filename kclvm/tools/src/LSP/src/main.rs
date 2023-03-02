use chrono::{Local, TimeZone};
use indexmap::IndexSet;
use kclvm_tools::lint::lint_files;
use kclvm_tools::util::lsp::{get_project_stack, kcl_diag_to_lsp_diags};
use semantic_token::{
    get_imcomplete_semantic_tokens, imcomplete_semantic_tokens_to_semantic_tokens, LEGEND_TYPE,
};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use kclvm_error::Diagnostic as KCLDiagnostic;
mod semantic_token;

#[cfg(test)]
mod tests;

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
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("KCL".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.clone().into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(false),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),

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

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let file = params.text_document.uri.path();
        let (files, ops) = get_project_stack(file);

        self.client
            .log_message(
                MessageType::INFO,
                format!("semantic_token_full filename:{}", file),
            )
            .await;

        let mut tokens = get_imcomplete_semantic_tokens(&files, ops, file);

        let semantic_tokens = imcomplete_semantic_tokens_to_semantic_tokens(&mut tokens);
        return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: semantic_tokens,
        })));
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
