pub const SERVER_NAME: &str = "veil-lsp";

use std::path::{Path, PathBuf};

use crate::diagnostics::findings_to_diagnostics;
use crate::document_store::{DocumentState, DocumentStore};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};
use veil_config::Config;
use veil_core::{scan_content, try_get_all_rules, Rule};

pub struct Backend {
    client: Client,
    documents: tokio::sync::Mutex<DocumentStore>,
    config: Config,
    rules: Vec<Rule>,
}

impl Backend {
    fn new(client: Client) -> Self {
        let config = Config::default();
        let rules = try_get_all_rules(&config, Vec::new())
            .unwrap_or_else(|_| veil_core::get_default_rules());

        Self {
            client,
            documents: tokio::sync::Mutex::new(DocumentStore::default()),
            config,
            rules,
        }
    }

    async fn publish_document_diagnostics(&self, document: DocumentState) {
        let diagnostics = diagnostics_for_text(
            &document.text,
            &path_for_uri(&document.uri),
            &self.rules,
            &self.config,
        );

        self.client
            .publish_diagnostics(document.uri, diagnostics, Some(document.version))
            .await;
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let text_document = params.text_document;
        let document = {
            let mut documents = self.documents.lock().await;
            documents.open(text_document.uri, text_document.text, text_document.version)
        };

        self.publish_document_diagnostics(document).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let (changed_document, change_error) = {
            let mut documents = self.documents.lock().await;
            match documents.apply_changes(
                &params.text_document.uri,
                params.text_document.version,
                params.content_changes,
            ) {
                Ok(document) => (document, None),
                Err(error) => (None, Some(error.to_string())),
            }
        };

        if let Some(error) = change_error {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("failed to apply text document change: {error}"),
                )
                .await;
        }

        if let Some(document) = changed_document {
            self.publish_document_diagnostics(document).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        {
            let mut documents = self.documents.lock().await;
            documents.close(&uri);
        }

        self.client.publish_diagnostics(uri, Vec::new(), None).await;
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

pub fn diagnostics_for_text(
    text: &str,
    path: &Path,
    rules: &[Rule],
    config: &Config,
) -> Vec<Diagnostic> {
    findings_to_diagnostics(&scan_content(text, path, rules, config))
}

fn path_for_uri(uri: &Url) -> PathBuf {
    uri.to_file_path()
        .unwrap_or_else(|_| PathBuf::from(uri.path()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, TextDocumentSyncCapability};

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

    #[test]
    fn diagnostics_for_text_scans_content_with_default_rules() {
        let config = Config::default();
        let rules = try_get_all_rules(&config, Vec::new()).expect("rules");
        let diagnostics = diagnostics_for_text(
            "🙂 contact test@example.com\n",
            Path::new("fixture.txt"),
            &rules,
            &config,
        );

        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| {
                matches!(
                    diagnostic.code.as_ref(),
                    Some(NumberOrString::String(code)) if code == "pii.net.email"
                )
            })
            .expect("email diagnostic");

        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diagnostic.range.start.line, 0);
        assert_eq!(diagnostic.range.start.character, 11);

        let data = diagnostic.data.as_ref().expect("diagnostic data");
        let data_text = data.to_string();
        assert!(!data_text.contains("test@example.com"));
        assert!(!data_text.contains("🙂 contact test@example.com"));
    }
}
