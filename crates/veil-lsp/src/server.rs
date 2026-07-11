pub const SERVER_NAME: &str = "veil-lsp";

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::code_actions::code_actions;
use crate::diagnostics::{findings_to_diagnostics, max_file_size_diagnostic};
use crate::document_store::{DocumentState, DocumentStore};
use anyhow::Result;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    CodeActionParams, CodeActionProviderCapability, CodeActionResponse, Diagnostic,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};
use veil_config::Config;
use veil_core::{scan_content, try_get_all_rules, Rule, DEFAULT_MAX_FILE_SIZE_BYTES};

const CHANGE_SCAN_DEBOUNCE: Duration = Duration::from_millis(200);

pub struct Backend {
    client: Client,
    documents: Arc<tokio::sync::Mutex<DocumentStore>>,
    pending_scans: Arc<tokio::sync::Mutex<PendingScanMap>>,
    config: Arc<Config>,
    rules: Arc<Vec<Rule>>,
}

impl Backend {
    fn with_config_and_rules(client: Client, config: Arc<Config>, rules: Arc<Vec<Rule>>) -> Self {
        Self {
            client,
            documents: Arc::new(tokio::sync::Mutex::new(DocumentStore::default())),
            pending_scans: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            config,
            rules,
        }
    }

    async fn schedule_document_diagnostics(&self, document: DocumentState, debounce: Duration) {
        let uri = document.uri.clone();
        let task_uri = uri.clone();
        let scan_revision = document.scan_revision;
        let client = self.client.clone();
        let documents = Arc::clone(&self.documents);
        let pending_scans = Arc::clone(&self.pending_scans);
        let config = Arc::clone(&self.config);
        let rules = Arc::clone(&self.rules);

        let handle = tokio::spawn(async move {
            tokio::time::sleep(debounce).await;

            let Some(document) =
                current_document_for_revision(&documents, &task_uri, scan_revision).await
            else {
                clear_pending_scan_if_current(&pending_scans, &task_uri, scan_revision).await;
                return;
            };

            let diagnostics = diagnostics_for_text(
                &document.text,
                &path_for_uri(&document.uri),
                &rules,
                &config,
            );

            if current_document_for_revision(&documents, &task_uri, scan_revision)
                .await
                .is_some()
            {
                client
                    .publish_diagnostics(task_uri.clone(), diagnostics, Some(document.version))
                    .await;
            }

            clear_pending_scan_if_current(&pending_scans, &task_uri, scan_revision).await;
        });

        replace_pending_scan(&self.pending_scans, uri, scan_revision, handle).await;
    }

    async fn cancel_pending_scan(&self, uri: &Url) {
        let mut pending_scans = self.pending_scans.lock().await;
        if let Some(pending_scan) = pending_scans.remove(uri) {
            pending_scan.handle.abort();
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
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
            documents.open(
                text_document.uri,
                text_document.language_id,
                text_document.text,
                text_document.version,
            )
        };

        self.schedule_document_diagnostics(document, Duration::ZERO)
            .await;
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
            self.schedule_document_diagnostics(document, CHANGE_SCAN_DEBOUNCE)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.cancel_pending_scan(&uri).await;
        {
            let mut documents = self.documents.lock().await;
            documents.close(&uri);
        }

        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let document = {
            let documents = self.documents.lock().await;
            documents.get(&uri)
        };
        let Some(document) = document else {
            return Ok(None);
        };

        let actions = code_actions(
            &uri,
            &document.language_id,
            &document.text,
            &params.context.diagnostics,
        );
        if actions.is_empty() {
            return Ok(None);
        }

        Ok(Some(actions))
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }
}

pub async fn run_stdio() -> Result<()> {
    run_stdio_with_config(Config::default()).await
}

pub async fn run_stdio_with_config(config: Config) -> Result<()> {
    let remote_rules = remote_rules_for_config(&config).await?;
    let rules = Arc::new(try_get_all_rules(&config, remote_rules)?);
    let config = Arc::new(config);
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::build(|client| {
        Backend::with_config_and_rules(client, Arc::clone(&config), Arc::clone(&rules))
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        ..ServerCapabilities::default()
    }
}

pub fn diagnostics_for_text(
    text: &str,
    path: &Path,
    rules: &[Rule],
    config: &Config,
) -> Vec<Diagnostic> {
    if is_ignored_by_config(path, config) {
        return Vec::new();
    }

    let max_size_bytes = config
        .core
        .max_file_size
        .unwrap_or(DEFAULT_MAX_FILE_SIZE_BYTES);
    let file_size_bytes = text.len() as u64;
    if file_size_bytes > max_size_bytes {
        return vec![max_file_size_diagnostic(file_size_bytes, max_size_bytes)];
    }

    findings_to_diagnostics(&scan_content(text, path, rules, config))
}

fn is_ignored_by_config(path: &Path, config: &Config) -> bool {
    let path_str = path.to_string_lossy();
    config
        .core
        .ignore
        .iter()
        .any(|pattern| path_str.contains(pattern))
}

async fn remote_rules_for_config(config: &Config) -> Result<Vec<Rule>> {
    let remote_url = std::env::var("VEIL_REMOTE_RULES_URL")
        .ok()
        .or_else(|| config.core.remote_rules_url.clone());

    let Some(url) = remote_url else {
        return Ok(Vec::new());
    };

    let fetch_url = url.clone();
    let rules = match tokio::task::spawn_blocking(move || {
        veil_core::remote::fetch_remote_rules(&fetch_url, 3)
    })
    .await?
    {
        Ok(rules) => rules,
        Err(error) => {
            eprintln!(
                "Warning: Failed to fetch remote rules from {}: {}. Continuing with local rules only.",
                url, error
            );
            Vec::new()
        }
    };

    Ok(rules)
}

fn path_for_uri(uri: &Url) -> PathBuf {
    uri.to_file_path()
        .unwrap_or_else(|_| PathBuf::from(uri.path()))
}

type PendingScanMap = HashMap<Url, PendingScan>;

struct PendingScan {
    scan_revision: u64,
    handle: tokio::task::JoinHandle<()>,
}

async fn replace_pending_scan(
    pending_scans: &tokio::sync::Mutex<PendingScanMap>,
    uri: Url,
    scan_revision: u64,
    handle: tokio::task::JoinHandle<()>,
) {
    let mut pending_scans = pending_scans.lock().await;
    if let Some(previous_scan) = pending_scans.insert(
        uri,
        PendingScan {
            scan_revision,
            handle,
        },
    ) {
        previous_scan.handle.abort();
    }
}

async fn clear_pending_scan_if_current(
    pending_scans: &tokio::sync::Mutex<PendingScanMap>,
    uri: &Url,
    scan_revision: u64,
) {
    let mut pending_scans = pending_scans.lock().await;
    let should_clear = pending_scans
        .get(uri)
        .is_some_and(|pending_scan| pending_scan.scan_revision == scan_revision);
    if should_clear {
        pending_scans.remove(uri);
    }
}

async fn current_document_for_revision(
    documents: &tokio::sync::Mutex<DocumentStore>,
    uri: &Url,
    scan_revision: u64,
) -> Option<DocumentState> {
    let documents = documents.lock().await;
    document_for_revision(&documents, uri, scan_revision)
}

fn document_for_revision(
    documents: &DocumentStore,
    uri: &Url,
    scan_revision: u64,
) -> Option<DocumentState> {
    if documents.has_revision(uri, scan_revision) {
        return documents.get(uri);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tower_lsp::lsp_types::{
        CodeActionProviderCapability, DiagnosticSeverity, NumberOrString,
        TextDocumentSyncCapability,
    };
    use veil_config::RuleConfig;

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
        assert_eq!(
            capabilities.code_action_provider,
            Some(CodeActionProviderCapability::Simple(true))
        );
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

    #[test]
    fn diagnostics_for_text_returns_max_file_size_skip_before_scanning() {
        let mut config = Config::default();
        config.core.max_file_size = Some(8);
        let rules = try_get_all_rules(&config, Vec::new()).expect("rules");

        let diagnostics = diagnostics_for_text(
            "contact test@example.com\n",
            Path::new("fixture.txt"),
            &rules,
            &config,
        );

        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert!(matches!(
            diagnostic.code.as_ref(),
            Some(NumberOrString::String(code)) if code == veil_core::RULE_ID_MAX_FILE_SIZE
        ));
        assert_eq!(diagnostic.range.start.line, 0);
        assert_eq!(diagnostic.range.start.character, 0);
        assert_eq!(diagnostic.range.end.line, 0);
        assert_eq!(diagnostic.range.end.character, 0);

        let data = diagnostic.data.as_ref().expect("diagnostic data");
        let data_text = data.to_string();
        assert!(!data_text.contains("test@example.com"));
        assert!(!data_text.contains("contact test@example.com"));
    }

    #[test]
    fn diagnostics_for_text_honors_config_ignores() {
        let mut config = Config::default();
        config.core.ignore.push("generated".to_string());
        let rules = try_get_all_rules(&config, Vec::new()).expect("rules");

        let diagnostics = diagnostics_for_text(
            "contact test@example.com\n",
            Path::new("src/generated/secrets.txt"),
            &rules,
            &config,
        );

        assert!(diagnostics.is_empty());
    }

    #[tokio::test]
    async fn run_stdio_with_config_rejects_rule_loading_errors() {
        let config = Config {
            rules: HashMap::from([(
                "custom.invalid_validator".to_string(),
                RuleConfig {
                    enabled: true,
                    enabled_is_set: true,
                    severity: None,
                    pattern: Some("SECRET".to_string()),
                    score: None,
                    category: None,
                    tags: None,
                    base_score: None,
                    context_lines_before: None,
                    context_lines_after: None,
                    validator: Some("unknown_validator".to_string()),
                    description: None,
                    placeholder: None,
                },
            )]),
            ..Config::default()
        };

        let error = run_stdio_with_config(config).await.unwrap_err();

        assert!(error
            .to_string()
            .contains("Unknown validator 'unknown_validator'"));
    }

    #[test]
    fn change_scan_debounce_matches_design_window() {
        assert_eq!(CHANGE_SCAN_DEBOUNCE, Duration::from_millis(200));
    }

    #[test]
    fn document_for_revision_requires_latest_scan_revision() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let mut documents = DocumentStore::default();
        documents.open(uri.clone(), "text".to_string(), "before".to_string(), 1);
        let current = documents
            .apply_changes(
                &uri,
                2,
                vec![tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: "after".to_string(),
                }],
            )
            .expect("change")
            .expect("document");

        assert!(document_for_revision(&documents, &uri, current.scan_revision).is_some());
        assert!(document_for_revision(&documents, &uri, 0).is_none());
    }
}
