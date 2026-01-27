use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::interval;
use tower_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    ExecuteCommandOptions, ExecuteCommandParams, GotoDefinitionParams, GotoDefinitionResponse,
    InitializeParams, InitializeResult, InitializedParams, Location, OneOf, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer};
use tracing::{info, warn};

use crate::analysis::SymbolIndex;
use crate::diagnostics::to_lsp_diagnostic;
use crate::gradle::{compute_gradle_fingerprint, run_gradle_classpath, GradleCacheEntry};
use crate::index::WorkspaceIndex;
use crate::parser;
use crate::text::{DocumentStore, position_to_char_idx};

pub struct Backend {
    client: Client,
    documents: Arc<RwLock<DocumentStore>>,
    workspace_index: Arc<RwLock<WorkspaceIndex>>,
    workspace_roots: Arc<RwLock<Vec<Url>>>,
    gradle_cache: Arc<RwLock<HashMap<Url, GradleCacheEntry>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(DocumentStore::default())),
            workspace_index: Arc::new(RwLock::new(WorkspaceIndex::default())),
            workspace_roots: Arc::new(RwLock::new(Vec::new())),
            gradle_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        let roots = collect_workspace_roots(&params);
        {
            let mut store = self.workspace_roots.write().await;
            *store = roots;
        }

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            definition_provider: Some(OneOf::Left(true)),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec!["kotlin-analyzer.refreshGradle".to_string()],
                ..ExecuteCommandOptions::default()
            }),
            ..ServerCapabilities::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("kotlin-analyzer initialized");

        let roots = {
            let store = self.workspace_roots.read().await;
            store.clone()
        };

        let workspace_index = self.workspace_index.clone();
        tokio::spawn(async move {
            let index = tokio::task::spawn_blocking(move || WorkspaceIndex::build(&roots))
                .await
                .ok();
            if let Some(index) = index {
                let mut store = workspace_index.write().await;
                *store = index;
            }
        });

        let workspace_index = self.workspace_index.clone();
        let workspace_roots = self.workspace_roots.clone();
        let documents = self.documents.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));
            loop {
                ticker.tick().await;
                let roots = {
                    let store = workspace_roots.read().await;
                    store.clone()
                };
                let open_uris = {
                    let store = documents.read().await;
                    store.uris()
                };
                let skip = open_uris.into_iter().collect::<HashSet<_>>();
                let index = workspace_index.clone();
                tokio::task::spawn_blocking(move || {
                    let mut store = index.blocking_write();
                    store.refresh_roots(&roots, &skip);
                })
                .await
                .ok();
            }
        });

        let gradle_cache = self.gradle_cache.clone();
        let roots = {
            let store = self.workspace_roots.read().await;
            store.clone()
        };
        tokio::spawn(async move {
            for root in roots {
                refresh_gradle_root(root, gradle_cache.clone(), true).await;
            }
        });

        let gradle_cache = self.gradle_cache.clone();
        let workspace_roots = self.workspace_roots.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;
                let roots = {
                    let store = workspace_roots.read().await;
                    store.clone()
                };
                for root in roots {
                    refresh_gradle_root(root, gradle_cache.clone(), false).await;
                }
            }
        });
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let text = params.text_document.text;

        let (text_snapshot, rope_snapshot) = {
            let mut store = self.documents.write().await;
            store.open(uri.clone(), version, text);
            if let Some(document) = store.get(&uri) {
                (document.text(), document.rope().clone())
            } else {
                (String::new(), ropey::Rope::new())
            }
        };

        let (ast, parse_diagnostics) = parser::parse_with_ast(&text_snapshot);
        {
            let mut index = self.workspace_index.write().await;
            index.update_from_ast(uri.clone(), ast.as_ref(), &text_snapshot);
        }

        let diagnostics = parse_diagnostics
            .iter()
            .map(|diag| to_lsp_diagnostic(diag, &rope_snapshot))
            .collect::<Vec<_>>();

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let changes = params.content_changes;

        let (text_snapshot, rope_snapshot) = {
            let mut store = self.documents.write().await;
            store.apply_changes(&uri, version, &changes);
            if let Some(document) = store.get(&uri) {
                (document.text(), document.rope().clone())
            } else {
                (String::new(), ropey::Rope::new())
            }
        };

        let (ast, parse_diagnostics) = parser::parse_with_ast(&text_snapshot);
        {
            let mut index = self.workspace_index.write().await;
            index.update_from_ast(uri.clone(), ast.as_ref(), &text_snapshot);
        }

        let diagnostics = parse_diagnostics
            .iter()
            .map(|diag| to_lsp_diagnostic(diag, &rope_snapshot))
            .collect::<Vec<_>>();

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let mut store = self.documents.write().await;
        store.close(&uri);
        drop(store);

        let mut index = self.workspace_index.write().await;
        index.refresh_from_disk(&uri);

        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let (text_snapshot, rope_snapshot) = {
            let store = self.documents.read().await;
            let Some(document) = store.get(&uri) else {
                return Ok(None);
            };
            (document.text(), document.rope().clone())
        };

        let ident = identifier_at_position(&rope_snapshot, position);
        let Some(name) = ident else {
            return Ok(None);
        };

        let workspace_locations = self.workspace_index.read().await.find_all(&name);
        if !workspace_locations.is_empty() {
            let response = if workspace_locations.len() == 1 {
                GotoDefinitionResponse::Scalar(
                    workspace_locations.into_iter().next().expect("non-empty"),
                )
            } else {
                GotoDefinitionResponse::Array(workspace_locations)
            };
            return Ok(Some(response));
        }

        let (ast, _) = parser::parse_with_ast(&text_snapshot);
        let Some(file) = ast else {
            return Ok(None);
        };

        let symbols = SymbolIndex::from_file(&file, &rope_snapshot);
        let mut locations = symbols
            .find_all(&name)
            .into_iter()
            .map(|symbol| Location {
                uri: uri.clone(),
                range: symbol.range,
            })
            .collect::<Vec<_>>();

        if locations.is_empty() {
            Ok(None)
        } else if locations.len() == 1 {
            Ok(Some(GotoDefinitionResponse::Scalar(
                locations.pop().expect("one location"),
            )))
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> tower_lsp::jsonrpc::Result<Option<serde_json::Value>> {
        if params.command != "kotlin-analyzer.refreshGradle" {
            return Ok(None);
        }

        let roots = parse_gradle_roots(&params.arguments, &self.workspace_roots).await;

        let mut refreshed = 0usize;
        for root in roots {
            refresh_gradle_root(root, self.gradle_cache.clone(), true).await;
            refreshed += 1;
        }

        Ok(Some(serde_json::json!({ "refreshed": refreshed })))
    }
}

fn collect_workspace_roots(params: &InitializeParams) -> Vec<Url> {
    if let Some(folders) = &params.workspace_folders {
        let mut roots = folders.iter().map(|f| f.uri.clone()).collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        return roots;
    }

    params.root_uri.iter().cloned().collect::<Vec<_>>()
}

async fn parse_gradle_roots(
    args: &[serde_json::Value],
    workspace_roots: &Arc<RwLock<Vec<Url>>>,
) -> Vec<Url> {
    if args.is_empty() {
        let store = workspace_roots.read().await;
        return store.clone();
    }

    if let Some(first) = args.first() {
        if let Some(value) = first.as_str() {
            if let Ok(uri) = Url::parse(value) {
                return vec![uri];
            }
            let path = std::path::PathBuf::from(value);
            if let Ok(uri) = Url::from_file_path(path) {
                return vec![uri];
            }
        }
    }

    let store = workspace_roots.read().await;
    store.clone()
}

async fn refresh_gradle_root(
    root: Url,
    cache: Arc<RwLock<HashMap<Url, GradleCacheEntry>>>,
    force: bool,
) {
    let Ok(path) = root.to_file_path() else {
        return;
    };

    let Some(fingerprint) = compute_gradle_fingerprint(&path) else {
        return;
    };

    if !force {
        let cached = {
            let store = cache.read().await;
            store.get(&root).map(|entry| entry.fingerprint)
        };
        if cached == Some(fingerprint) {
            return;
        }
    }

    let root_clone = root.clone();
    let cache_clone = cache.clone();
    let result = tokio::task::spawn_blocking(move || run_gradle_classpath(&path)).await;
    match result {
        Ok(Ok(classpath)) => {
            let entry = GradleCacheEntry {
                classpath,
                fingerprint,
            };
            let mut store = cache_clone.write().await;
            store.insert(root_clone, entry);
        }
        Ok(Err(err)) => {
            warn!(?err, "gradle classpath task failed");
        }
        Err(err) => {
            warn!(?err, "gradle classpath task panicked");
        }
    }
}

fn identifier_at_position(
    rope: &ropey::Rope,
    position: tower_lsp::lsp_types::Position,
) -> Option<String> {
    let mut index = position_to_char_idx(rope, position);
    let len = rope.len_chars();
    if len == 0 {
        return None;
    }

    if index >= len {
        index = len.saturating_sub(1);
    }

    let current = rope.char(index);
    if !is_ident_char(current) {
        if index == 0 {
            return None;
        }
        let prev = rope.char(index - 1);
        if !is_ident_char(prev) {
            return None;
        }
        index -= 1;
    }

    let mut start = index;
    while start > 0 && is_ident_char(rope.char(start - 1)) {
        start -= 1;
    }

    let mut end = index + 1;
    while end < len && is_ident_char(rope.char(end)) {
        end += 1;
    }

    if start >= end {
        return None;
    }

    Some(slice_rope(rope, start, end))
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn slice_rope(rope: &ropey::Rope, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    let end = end.min(rope.len_chars());
    let start = start.min(end);
    rope.slice(start..end).to_string()
}
