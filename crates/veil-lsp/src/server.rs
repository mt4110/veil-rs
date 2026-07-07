pub const SERVER_NAME: &str = "veil-lsp";

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct Backend {
    client: Client,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(ServerInfo {
                name: SERVER_NAME.to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "veil-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub async fn run_stdio() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        ..ServerCapabilities::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::TextDocumentSyncCapability;

    #[test]
    fn server_name_matches_binary_name() {
        assert_eq!(SERVER_NAME, "veil-lsp");
    }

    #[test]
    fn capabilities_advertise_incremental_text_sync() {
        let capabilities = server_capabilities();

        assert_eq!(
            capabilities.text_document_sync,
            Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL
            ))
        );
        assert!(capabilities.diagnostic_provider.is_none());
        assert!(capabilities.code_action_provider.is_none());
    }
}
