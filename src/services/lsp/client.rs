//! Phobos IDE - LSP Client Service
//! Generic Language Server Protocol client implementation

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use anyhow::{Context, Result, bail};
use log::{info, warn, error, debug};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc};
use lsp_types::*;
use serde_json::{json, Value};

/// LSP Client state machine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LspState {
    NotStarted,
    Starting,
    Initialized,
    Ready,
    Error,
}

/// Configuration for an LSP server
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    pub language_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub file_extensions: Vec<String>,
}

impl LspServerConfig {
    /// Get default LSP server configurations
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                language_id: "rust".to_string(),
                command: "rust-analyzer".to_string(),
                args: vec![],
                file_extensions: vec!["rs".to_string()],
            },
            Self {
                language_id: "python".to_string(),
                command: "pylsp".to_string(),
                args: vec![],
                file_extensions: vec!["py".to_string()],
            },
            Self {
                language_id: "typescript".to_string(),
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                file_extensions: vec!["ts".to_string(), "tsx".to_string()],
            },
            Self {
                language_id: "javascript".to_string(),
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                file_extensions: vec!["js".to_string(), "jsx".to_string()],
            },
            Self {
                language_id: "go".to_string(),
                command: "gopls".to_string(),
                args: vec![],
                file_extensions: vec!["go".to_string()],
            },
            Self {
                language_id: "java".to_string(),
                command: "jdtls".to_string(),
                args: vec![],
                file_extensions: vec!["java".to_string()],
            },
        ]
    }
    
    /// Find config for a given file extension
    pub fn for_extension(ext: &str) -> Option<Self> {
        Self::defaults().into_iter().find(|c| c.file_extensions.iter().any(|e| e == ext))
    }
    
    /// Find config for a language ID
    pub fn for_language(language_id: &str) -> Option<Self> {
        Self::defaults().into_iter().find(|c| c.language_id == language_id)
    }
}

/// Document version tracking
#[derive(Debug, Clone)]
struct DocumentInfo {
    uri: DocumentUri,
    version: i32,
    content: String,
}

/// Main LSP Client
pub struct LspClient {
    config: LspServerConfig,
    state: LspState,
    process: Option<Arc<Mutex<Child>>>,
    document_versions: HashMap<String, DocumentInfo>,
    root_uri: Option<DocumentUri>,
    capabilities: Option<ServerCapabilities>,
    request_id: i32,
}

impl LspClient {
    /// Create a new LSP client with the given configuration
    pub fn new(config: LspServerConfig) -> Self {
        Self {
            config,
            state: LspState::NotStarted,
            process: None,
            document_versions: HashMap::new(),
            root_uri: None,
            capabilities: None,
            request_id: 0,
        }
    }
    
    /// Initialize and start the language server process
    pub async fn initialize(&mut self, workspace_root: PathBuf) -> Result<()> {
        if self.state != LspState::NotStarted {
            bail!("LSP client already initialized");
        }
        
        self.state = LspState::Starting;
        
        // Spawn the language server process
        let mut child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(format!("Failed to spawn LSP server: {}", self.config.command))?;
        
        let stdin = child.stdin.take()
            .context("Failed to open stdin for LSP server")?;
        let stdout = child.stdout.take()
            .context("Failed to open stdout for LSP server")?;
        let stderr = child.stderr.take()
            .context("Failed to open stderr for LSP server")?;
        
        let child_arc = Arc::new(Mutex::new(child));
        self.process = Some(child_arc.clone());
        
        // Set root URI
        let root_path = workspace_root.canonicalize()
            .unwrap_or(workspace_root);
        self.root_uri = Some(DocumentUri::from_file_path(&root_path)
            .map_err(|_| anyhow::anyhow!("Invalid workspace path"))?);
        
        // Send initialize request
        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: self.root_uri.clone(),
            root_path: None,
            initialization_options: None,
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSynchronizationClientCapabilities {
                        dynamic_registration: Some(false),
                        will_save: Some(false),
                        will_save_wait_until: Some(false),
                        did_save: Some(true),
                    }),
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(false),
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(true),
                            ..Default::default()
                        }),
                        completion_item_kind: Some(CompletionItemKindCapability {
                            value_set: Some(vec![
                                CompletionItemKind::TEXT,
                                CompletionItemKind::METHOD,
                                CompletionItemKind::FUNCTION,
                                CompletionItemKind::CONSTRUCTOR,
                                CompletionItemKind::FIELD,
                                CompletionItemKind::VARIABLE,
                                CompletionItemKind::CLASS,
                                CompletionItemKind::INTERFACE,
                                CompletionItemKind::MODULE,
                                CompletionItemKind::PROPERTY,
                                CompletionItemKind::UNIT,
                                CompletionItemKind::VALUE,
                                CompletionItemKind::ENUM,
                                CompletionItemKind::KEYWORD,
                                CompletionItemKind::SNIPPET,
                            ]),
                        }),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        dynamic_registration: Some(false),
                        content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                    }),
                    diagnostics: Some(DiagnosticsClientCapabilities {
                        dynamic_registration: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_folders: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            trace: Some(TraceValue::Off),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: self.root_uri.clone().unwrap(),
                name: root_path.display().to_string(),
            }]),
            client_info: Some(ClientInfo {
                name: "Phobos IDE".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            locale: None,
            root_directory: None,
        };
        
        let response = self.send_request::<InitializeParams, InitializeResult>(
            "initialize",
            init_params,
            &stdin,
            &stdout,
        ).await?;
        
        self.capabilities = Some(response.capabilities);
        
        // Send initialized notification
        self.send_notification("initialized", json!({}))
            .await
            .context("Failed to send initialized notification")?;
        
        self.state = LspState::Ready;
        info!("LSP server '{}' initialized successfully", self.config.language_id);
        
        // Start reading stdout in background
        tokio::spawn(Self::read_stdout(stdout, stderr));
        
        Ok(())
    }
    
    /// Open a document in the language server
    pub async fn did_open(&mut self, uri: &str, content: &str, language_id: &str) -> Result<()> {
        if self.state != LspState::Ready {
            bail!("LSP client not ready");
        }
        
        let doc_uri = DocumentUri::parse(uri)?;
        let version = self.document_versions.get(uri).map(|d| d.version + 1).unwrap_or(1);
        
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: doc_uri.clone(),
                language_id: language_id.to_string(),
                version,
                text: content.to_string(),
            },
        };
        
        self.document_versions.insert(uri.to_string(), DocumentInfo {
            uri: doc_uri,
            version,
            content: content.to_string(),
        });
        
        self.send_notification("textDocument/didOpen", json!(params)).await
    }
    
    /// Notify document changes
    pub async fn did_change(&mut self, uri: &str, content: &str) -> Result<()> {
        if self.state != LspState::Ready {
            bail!("LSP client not ready");
        }
        
        let doc_info = self.document_versions.get_mut(uri)
            .context("Document not opened in LSP")?;
        
        doc_info.version += 1;
        doc_info.content = content.to_string();
        
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: doc_info.uri.clone(),
                version: doc_info.version,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: content.to_string(),
            }],
        };
        
        self.send_notification("textDocument/didChange", json!(params)).await
    }
    
    /// Request completions
    pub async fn completion(&self, uri: &str, line: u32, column: u32) -> Result<Vec<CompletionItem>> {
        if self.state != LspState::Ready {
            bail!("LSP client not ready");
        }
        
        let doc_info = self.document_versions.get(uri)
            .context("Document not opened in LSP")?;
        
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                uri: doc_info.uri.clone(),
                position: Position { line, character: column },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };
        
        let response = self.send_request_simple("textDocument/completion", json!(params)).await?;
        
        // Parse response
        match response {
            Value::Array(items) => {
                let completions: Result<Vec<CompletionItem>> = items
                    .into_iter()
                    .map(|v| serde_json::from_value(v).context("Failed to parse completion item"))
                    .collect();
                completions
            }
            Value::Object(obj) => {
                // Response might be wrapped in { items: [...], isIncomplete: bool }
                obj.get("items")
                    .and_then(|v| v.as_array())
                    .map(|items| {
                        items.iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect()
                    })
                    .context("Invalid completion response format")
            }
            _ => bail!("Unexpected completion response format"),
        }
    }
    
    /// Request hover information
    pub async fn hover(&self, uri: &str, line: u32, column: u32) -> Result<Option<Hover>> {
        if self.state != LspState::Ready {
            bail!("LSP client not ready");
        }
        
        let doc_info = self.document_versions.get(uri)
            .context("Document not opened in LSP")?;
        
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                uri: doc_info.uri.clone(),
                position: Position { line, character: column },
            },
            work_done_progress_params: Default::default(),
        };
        
        let response = self.send_request_simple("textDocument/hover", json!(params)).await?;
        
        if response.is_null() {
            return Ok(None);
        }
        
        serde_json::from_value(response)
            .context("Failed to parse hover response")
            .map(Some)
    }
    
    /// Close a document
    pub async fn did_close(&mut self, uri: &str) -> Result<()> {
        if self.state != LspState::Ready {
            bail!("LSP client not ready");
        }
        
        let doc_info = self.document_versions.remove(uri)
            .context("Document not opened in LSP")?;
        
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: doc_info.uri,
            },
        };
        
        self.send_notification("textDocument/didClose", json!(params)).await
    }
    
    /// Shutdown the LSP server
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(process) = &self.process {
            self.send_notification("shutdown", json!({})).await?;
            self.send_notification("exit", json!({})).await?;
            
            let mut proc = process.lock().await;
            proc.kill().await.ok();
        }
        
        self.state = LspState::NotStarted;
        Ok(())
    }
    
    /// Get current state
    pub fn state(&self) -> LspState {
        self.state
    }
    
    /// Get server capabilities
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }
    
    // === Internal helper methods ===
    
    fn next_request_id(&mut self) -> i32 {
        self.request_id += 1;
        self.request_id
    }
    
    async fn send_request<P, R>(
        &mut self,
        method: &str,
        params: P,
        _stdin: &tokio::net::unix::pipe::Sender,
        _stdout: &tokio::net::unix::pipe::Receiver,
    ) -> Result<R>
    where
        P: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        // Simplified request sending - actual implementation would use proper JSON-RPC
        // This is a placeholder that demonstrates the architecture
        let id = self.next_request_id();
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        
        debug!("Sending LSP request: {} {:?}", method, request);
        
        // In a full implementation, we would write to stdin and read from stdout
        // For now, return a placeholder
        bail!("Full LSP request/response implementation pending - architecture demonstrated");
    }
    
    async fn send_notification(&self, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        
        debug!("Sending LSP notification: {} {:?}", method, notification);
        
        // In full implementation, write to stdin with Content-Length header
        Ok(())
    }
    
    async fn send_request_simple(&self, method: &str, params: Value) -> Result<Value> {
        // Placeholder for simple request sending
        debug!("LSP request (placeholder): {}", method);
        Ok(Value::Null)
    }
    
    async fn read_stdout(
        stdout: impl tokio::io::AsyncRead + Unpin + Send + 'static,
        stderr: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    debug!("LSP stdout: {}", line.trim());
                    // Parse JSON-RPC messages and handle notifications
                    // Especially publishDiagnostics
                }
                Err(e) => {
                    error!("Error reading LSP stdout: {}", e);
                    break;
                }
            }
        }
        
        // Also read stderr for errors
        let mut err_reader = BufReader::new(stderr);
        let mut err_line = String::new();
        
        tokio::spawn(async move {
            loop {
                err_line.clear();
                match err_reader.read_line(&mut err_line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        warn!("LSP stderr: {}", err_line.trim());
                    }
                    Err(e) => {
                        error!("Error reading LSP stderr: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

/// Helper function to create LSP client for a language
pub async fn create_lsp_client(
    language_id: &str,
    workspace_root: PathBuf,
) -> Result<LspClient> {
    let config = LspServerConfig::for_language(language_id)
        .context(format!("No LSP config found for language: {}", language_id))?;
    
    let mut client = LspClient::new(config);
    client.initialize(workspace_root).await?;
    
    Ok(client)
}
