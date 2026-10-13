//! Phobos IDE - LSP Client Service (real JSON-RPC over stdio)
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use anyhow::{Context, Result, bail};
use log::{info, warn, error, debug};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, oneshot};
use lsp_types::*;
use serde_json::{json, Value};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LspState { NotStarted, Starting, Initialized, Ready, Error }

#[derive(Debug, Clone)]
pub struct LspServerConfig {
    pub language_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub file_extensions: Vec<String>,
}

impl LspServerConfig {
    pub fn defaults() -> Vec<Self> {
        vec![
            Self { language_id: "rust".into(), command: "rust-analyzer".into(), args: vec![], file_extensions: vec!["rs".into()] },
            Self { language_id: "python".into(), command: "pylsp".into(), args: vec![], file_extensions: vec!["py".into()] },
            Self { language_id: "typescript".into(), command: "typescript-language-server".into(), args: vec!["--stdio".into()], file_extensions: vec!["ts".into(),"tsx".into()] },
            Self { language_id: "javascript".into(), command: "typescript-language-server".into(), args: vec!["--stdio".into()], file_extensions: vec!["js".into(),"jsx".into()] },
            Self { language_id: "go".into(), command: "gopls".into(), args: vec![], file_extensions: vec!["go".into()] },
            Self { language_id: "java".into(), command: "jdtls".into(), args: vec![], file_extensions: vec!["java".into()] },
            Self { language_id: "c".into(), command: "clangd".into(), args: vec![], file_extensions: vec!["c".into(),"h".into()] },
            Self { language_id: "cpp".into(), command: "clangd".into(), args: vec![], file_extensions: vec!["cpp".into(),"hpp".into()] },
        ]
    }
    pub fn for_extension(ext: &str) -> Option<Self> {
        Self::defaults().into_iter().find(|c| c.file_extensions.iter().any(|e| e == ext))
    }
    pub fn for_language(lid: &str) -> Option<Self> {
        Self::defaults().into_iter().find(|c| c.language_id == lid)
    }
}

#[derive(Debug, Clone)]
struct DocumentInfo { uri: Url, version: i32, content: String }

type PendingMap = Arc<Mutex<HashMap<i32, oneshot::Sender<Value>>>>;

pub struct LspClient {
    config: LspServerConfig,
    state: LspState,
    process: Option<Arc<Mutex<Child>>>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    pending: PendingMap,
    document_versions: HashMap<String, DocumentInfo>,
    root_uri: Option<Url>,
    capabilities: Option<ServerCapabilities>,
    request_id: i32,
    diagnostics_tx: Option<tokio::sync::mpsc::Sender<(String, Vec<lsp_types::Diagnostic>)>>,
}

impl LspClient {
    pub fn new(config: LspServerConfig) -> Self {
        Self { config, state: LspState::NotStarted, process: None, stdin: None,
            pending: Arc::new(Mutex::new(HashMap::new())),
            document_versions: HashMap::new(), root_uri: None, capabilities: None,
            request_id: 0, diagnostics_tx: None }
    }

    pub fn set_diagnostics_channel(&mut self, tx: tokio::sync::mpsc::Sender<(String, Vec<lsp_types::Diagnostic>)>) {
        self.diagnostics_tx = Some(tx);
    }

    pub async fn initialize(&mut self, workspace_root: PathBuf) -> Result<()> {
        if self.state != LspState::NotStarted { bail!("LSP already initialized"); }
        self.state = LspState::Starting;
        if which::which(&self.config.command).is_err() {
            self.state = LspState::Error;
            bail!("LSP server not found in PATH: {}", self.config.command);
        }
        let mut child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().context(format!("Failed to spawn {}", self.config.command))?;
        let stdin = child.stdin.take().context("stdin")?;
        let stdout = child.stdout.take().context("stdout")?;
        let stderr = child.stderr.take().context("stderr")?;
        let child_arc = Arc::new(Mutex::new(child));
        self.process = Some(child_arc);
        self.stdin = Some(Arc::new(Mutex::new(stdin)));

        let root_path = workspace_root.canonicalize().unwrap_or(workspace_root);
        let root_uri = Url::from_file_path(&root_path).map_err(|_| anyhow::anyhow!("bad workspace path"))?;
        self.root_uri = Some(root_uri.clone());

        // spawn stdout reader
        let pending = self.pending.clone();
        let diag_tx = self.diagnostics_tx.clone();
        tokio::spawn(async move {
            Self::reader_loop(stdout, pending, diag_tx).await;
        });
        // log stderr
        tokio::spawn(async move {
            let mut r = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match r.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => warn!("LSP stderr: {}", line.trim()),
                    Err(e) => { error!("LSP stderr read: {}", e); break; }
                }
            }
        });

        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: None,
            root_uri: Some(root_uri.clone()),
            initialization_options: None,
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities {
                        dynamic_registration: Some(false), will_save: Some(false),
                        will_save_wait_until: Some(false), did_save: Some(true) }),
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(false),
                        completion_item: Some(CompletionItemCapability { snippet_support: Some(true), ..Default::default() }),
                        ..Default::default() }),
                    hover: Some(HoverClientCapabilities {
                        dynamic_registration: Some(false),
                        content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]) }),
                    publish_diagnostics: Some(PublishDiagnosticsClientCapabilities {
                        related_information: Some(true), version_support: Some(false), ..Default::default() }),
                    ..Default::default() }),
                workspace: Some(WorkspaceClientCapabilities { workspace_folders: Some(true), ..Default::default() }),
                ..Default::default() },
            trace: Some(TraceValue::Off),
            workspace_folders: Some(vec![WorkspaceFolder { uri: root_uri.clone(), name: root_path.display().to_string() }]),
            client_info: Some(ClientInfo { name: "Phobos IDE".into(), version: Some(env!("CARGO_PKG_VERSION").into()) }),
            locale: None,
            work_done_progress_params: Default::default(),
        };
        let resp: InitializeResult = self.send_request("initialize", init_params).await?;
        self.capabilities = Some(resp.capabilities);
        self.send_notification("initialized", json!({})).await?;
        self.state = LspState::Ready;
        info!("LSP '{}' ready", self.config.language_id);
        Ok(())
    }

    fn next_id(&mut self) -> i32 { self.request_id += 1; self.request_id }

    async fn write_msg(&self, msg: &Value) -> Result<()> {
        let body = serde_json::to_string(msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        let stdin = self.stdin.as_ref().context("no stdin")?.clone();
        let mut g = stdin.lock().await;
        g.write_all(header.as_bytes()).await?;
        g.write_all(body.as_bytes()).await?;
        g.flush().await?;
        Ok(())
    }

    pub async fn send_notification(&self, method: &str, params: Value) -> Result<()> {
        let msg = json!({"jsonrpc":"2.0","method":method,"params":params});
        debug!("LSP notify {}", method);
        self.write_msg(&msg).await
    }

    pub async fn send_request<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self, method: &str, params: P,
    ) -> Result<R> {
        let id = self.next_id();
        let msg = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        self.write_msg(&msg).await?;
        let val = tokio::time::timeout(std::time::Duration::from_secs(10), rx)
            .await.context("LSP request timeout")?
            .context("LSP channel closed")?;
        if let Some(err) = val.get("error") {
            bail!("LSP error: {}", err);
        }
        let result = val.get("result").cloned().unwrap_or(Value::Null);
        Ok(serde_json::from_value(result)?)
    }

    async fn send_request_value(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id();
        let msg = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        self.write_msg(&msg).await?;
        let val = tokio::time::timeout(std::time::Duration::from_secs(10), rx)
            .await.context("timeout")?.context("closed")?;
        if let Some(err) = val.get("error") { bail!("LSP error: {}", err); }
        Ok(val.get("result").cloned().unwrap_or(Value::Null))
    }

    async fn reader_loop(
        stdout: tokio::process::ChildStdout,
        pending: PendingMap,
        diag_tx: Option<tokio::sync::mpsc::Sender<(String, Vec<lsp_types::Diagnostic>)>>,
    ) {
        let mut reader = BufReader::new(stdout);
        loop {
            // read headers
            let mut content_length: Option<usize> = None;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => return,
                    Ok(_) => {},
                    Err(_) => return,
                }
                let t = line.trim();
                if t.is_empty() { break; }
                if let Some(v) = t.strip_prefix("Content-Length:") {
                    content_length = v.trim().parse().ok();
                }
            }
            let len = match content_length { Some(l) => l, None => continue };
            let mut buf = vec![0u8; len];
            let mut read = 0;
            while read < len {
                use tokio::io::AsyncReadExt;
                match reader.read(&mut buf[read..]).await {
                    Ok(0) => return,
                    Ok(n) => read += n,
                    Err(_) => return,
                }
            }
            let msg: Value = match serde_json::from_slice(&buf) {
                Ok(v) => v, Err(_) => continue,
            };
            if let Some(id) = msg.get("id").and_then(|v| v.as_i64()) {
                let mut p = pending.lock().await;
                if let Some(tx) = p.remove(&(id as i32)) { let _ = tx.send(msg); }
            } else if let Some(method) = msg.get("method").and_then(|v| v.as_str()) {
                if method == "textDocument/publishDiagnostics" {
                    if let Some(tx) = &diag_tx {
                        let params = msg.get("params").cloned().unwrap_or(Value::Null);
                        let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let diags: Vec<lsp_types::Diagnostic> = params.get("diagnostics")
                            .and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
                        let _ = tx.send((uri, diags)).await;
                    }
                }
            }
        }
    }

    fn uri_for(&self, path: &str) -> Result<Url> {
        if path.starts_with("file://") { Ok(Url::parse(path)?) }
        else {
            let p = std::path::Path::new(path);
            Url::from_file_path(p).map_err(|_| anyhow::anyhow!("bad path"))
        }
    }

    pub async fn did_open(&mut self, path: &str, content: &str, language_id: &str) -> Result<()> {
        if self.state != LspState::Ready { bail!("not ready"); }
        let uri = self.uri_for(path)?;
        let key = uri.to_string();
        let ver = self.document_versions.get(&key).map(|d| d.version + 1).unwrap_or(1);
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem { uri: uri.clone(), language_id: language_id.into(), version: ver, text: content.into() },
        };
        self.document_versions.insert(key, DocumentInfo { uri, version: ver, content: content.into() });
        self.send_notification("textDocument/didOpen", serde_json::to_value(params)?).await
    }

    pub async fn did_change(&mut self, path: &str, content: &str) -> Result<()> {
        if self.state != LspState::Ready { bail!("not ready"); }
        let uri = self.uri_for(path)?;
        let key = uri.to_string();
        let ver = self.document_versions.get(&key).map(|d| d.version + 1).unwrap_or(1);
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri: uri.clone(), version: ver },
            content_changes: vec![TextDocumentContentChangeEvent { range: None, range_length: None, text: content.into() }],
        };
        if let Some(d) = self.document_versions.get_mut(&key) { d.version = ver; d.content = content.into(); }
        else { self.document_versions.insert(key, DocumentInfo { uri, version: ver, content: content.into() }); }
        self.send_notification("textDocument/didChange", serde_json::to_value(params)?).await
    }

    pub async fn completion(&mut self, path: &str, line: u32, col: u32) -> Result<Vec<CompletionItem>> {
        if self.state != LspState::Ready { bail!("not ready"); }
        let uri = self.uri_for(path)?;
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri }, position: Position { line, character: col } },
            work_done_progress_params: Default::default(), partial_result_params: Default::default(), context: None,
        };
        let v = self.send_request_value("textDocument/completion", serde_json::to_value(params)?).await?;
        if v.is_null() { return Ok(vec![]); }
        if let Some(arr) = v.as_array() {
            Ok(arr.iter().filter_map(|x| serde_json::from_value(x.clone()).ok()).collect())
        } else if let Some(items) = v.get("items") {
            Ok(serde_json::from_value(items.clone()).unwrap_or_default())
        } else { Ok(vec![]) }
    }

    pub async fn hover(&mut self, path: &str, line: u32, col: u32) -> Result<Option<Hover>> {
        if self.state != LspState::Ready { bail!("not ready"); }
        let uri = self.uri_for(path)?;
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri }, position: Position { line, character: col } },
            work_done_progress_params: Default::default(),
        };
        let v = self.send_request_value("textDocument/hover", serde_json::to_value(params)?).await?;
        if v.is_null() { return Ok(None); }
        Ok(serde_json::from_value(v).ok())
    }

    pub async fn did_close(&mut self, path: &str) -> Result<()> {
        let uri = self.uri_for(path)?;
        let key = uri.to_string();
        self.document_versions.remove(&key);
        let params = DidCloseTextDocumentParams { text_document: TextDocumentIdentifier { uri } };
        self.send_notification("textDocument/didClose", serde_json::to_value(params)?).await
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if self.state == LspState::Ready {
            let _ = self.send_request_value("shutdown", Value::Null).await;
            let _ = self.send_notification("exit", Value::Null).await;
        }
        if let Some(p) = &self.process {
            let mut g = p.lock().await;
            // kill is async via start_kill
            let _ = g.start_kill();
        }
        self.state = LspState::NotStarted;
        Ok(())
    }

    pub fn state(&self) -> LspState { self.state }
    pub fn capabilities(&self) -> Option<&ServerCapabilities> { self.capabilities.as_ref() }
}

pub async fn create_lsp_client(language_id: &str, workspace_root: PathBuf) -> Result<LspClient> {
    let config = LspServerConfig::for_language(language_id)
        .context(format!("No LSP config for {}", language_id))?;
    let mut client = LspClient::new(config);
    client.initialize(workspace_root).await?;
    Ok(client)
}
