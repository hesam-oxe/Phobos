//! Phobos IDE - Core Module
//! Centralized state management and message passing architecture

use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::services::lsp::client::LspClient;
use crate::services::terminal::shell::TerminalShell;

/// ============================================================================
/// MESSAGE TYPES - Command & Event System
/// ============================================================================

/// Commands sent from UI to background workers
#[derive(Debug, Clone)]
pub enum Command {
    /// Open a file at the given path
    OpenFile(PathBuf),
    /// Save the current file
    SaveFile(PathBuf, String),
    /// Close a file tab
    CloseTab(PathBuf),
    /// Trigger LSP initialization for a language
    InitializeLsp(String), // language_id
    /// Send document changes to LSP
    LspDidChange {
        uri: String,
        content: String,
        version: i32,
    },
    /// Request completions from LSP
    LspCompletion { uri: String, line: u32, column: u32 },
    /// Request hover info from LSP
    LspHover { uri: String, line: u32, column: u32 },
    /// Run build command
    Build { command: String, cwd: PathBuf },
    /// Run AI completion request
    AiCompletion { context: String, line: usize },
    /// Terminal input
    TerminalInput(String),
    /// Shutdown application
    Shutdown,
}

/// Events sent from background workers to UI
#[derive(Debug, Clone)]
pub enum Event {
    /// File was successfully opened
    FileOpened { path: PathBuf, content: String },
    /// File save result
    FileSaved {
        path: PathBuf,
        success: bool,
        error: Option<String>,
    },
    /// LSP diagnostics received
    LspDiagnostics {
        uri: String,
        diagnostics: Vec<LspDiagnostic>,
    },
    /// LSP completion items received
    LspCompletions { completions: Vec<CompletionItem> },
    /// LSP hover info received
    LspHover { contents: String },
    /// Build output chunk
    BuildOutput { line: String, is_error: bool },
    /// Build finished
    BuildFinished {
        success: bool,
        exit_code: Option<i32>,
    },
    /// AI completion suggestion
    AiSuggestion {
        suggestion: String,
        start_pos: usize,
        end_pos: usize,
    },
    /// Terminal output
    TerminalOutput(String),
    /// Error occurred
    Error { message: String, source: String },
    /// Status update
    StatusUpdate(String),
}

/// LSP Diagnostic wrapper
#[derive(Debug, Clone)]
pub struct LspDiagnostic {
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// Completion item for autocomplete
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
}

/// ============================================================================
/// APP STATE - Centralized State Management
/// ============================================================================

/// Represents an open file tab
#[derive(Debug, Clone)]
pub struct EditorTab {
    pub path: PathBuf,
    pub content: String,
    pub language: String,
    pub cursor_position: (usize, usize), // (line, column)
    pub modified: bool,
    pub diagnostics: Vec<LspDiagnostic>,
}

impl EditorTab {
    pub fn new(path: PathBuf, content: String, language: String) -> Self {
        Self {
            path,
            content,
            language,
            cursor_position: (0, 0),
            modified: false,
            diagnostics: Vec::new(),
        }
    }
}

/// Configuration for the editor
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub theme: String,
    pub font_size: u16,
    pub tab_size: usize,
    pub auto_save: bool,
    pub auto_save_delay_ms: u64,
    pub show_line_numbers: bool,
    pub show_minimap: bool,
    pub word_wrap: bool,
    pub bracket_matching: bool,
    pub lsp_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 14,
            tab_size: 4,
            auto_save: true,
            auto_save_delay_ms: 1000,
            show_line_numbers: true,
            show_minimap: false,
            word_wrap: true,
            bracket_matching: true,
            lsp_enabled: true,
        }
    }
}

/// Main application state - shared across threads
pub struct AppState {
    /// Currently open tabs
    pub tabs: HashMap<PathBuf, EditorTab>,
    /// Currently active tab path
    pub active_tab: Option<PathBuf>,
    /// Recent files history
    pub recent_files: Vec<PathBuf>,
    /// Application configuration
    pub config: AppConfig,
    /// Working directory (project root)
    pub workspace_root: Option<PathBuf>,

    // === Background Services (Arc<Mutex> for thread-safe access) ===
    /// LSP clients by language ID
    pub lsp_clients: HashMap<String, Arc<Mutex<LspClient>>>,
    /// Active terminal shell
    pub terminal: Option<Arc<Mutex<TerminalShell>>>,

    // === Message Channels ===
    /// Command sender (UI -> Workers)
    pub command_tx: Sender<Command>,
    /// Command receiver (Workers receive)
    pub command_rx: Receiver<Command>,
    /// Event sender (Workers -> UI)
    pub event_tx: Sender<Event>,
    /// Event receiver (UI receives)
    pub event_rx: Receiver<Event>,
}

impl AppState {
    /// Create a new AppState with initialized channels
    pub fn new() -> Self {
        let (command_tx, command_rx) = crossbeam_channel::unbounded();
        let (event_tx, event_rx) = crossbeam_channel::unbounded();

        Self {
            tabs: HashMap::new(),
            active_tab: None,
            recent_files: Vec::new(),
            config: AppConfig::default(),
            workspace_root: None,
            lsp_clients: HashMap::new(),
            terminal: None,
            command_tx,
            command_rx,
            event_tx,
            event_rx,
        }
    }

    /// Get the command sender for dispatching commands
    pub fn command_sender(&self) -> Sender<Command> {
        self.command_tx.clone()
    }

    /// Get the event receiver for polling events in UI loop
    pub fn event_receiver(&self) -> &Receiver<Event> {
        &self.event_rx
    }

    /// Try to receive next event (non-blocking)
    pub fn try_recv_event(&self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }

    /// Add or update a tab
    pub fn open_tab(&mut self, path: PathBuf, content: String, language: String) {
        let tab = EditorTab::new(path.clone(), content, language);
        self.tabs.insert(path.clone(), tab);
        self.active_tab = Some(path.clone());

        // Add to recent files (max 20)
        if let Some(pos) = self.recent_files.iter().position(|p| p == &path) {
            self.recent_files.remove(pos);
        }
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 20 {
            self.recent_files.pop();
        }
    }

    /// Close a tab
    pub fn close_tab(&mut self, path: &PathBuf) -> Option<EditorTab> {
        if let Some(tab) = self.tabs.remove(path) {
            if self.active_tab.as_ref() == Some(path) {
                self.active_tab = self.tabs.keys().next().cloned();
            }
            Some(tab)
        } else {
            None
        }
    }

    /// Get the active tab
    pub fn get_active_tab(&self) -> Option<&EditorTab> {
        self.active_tab.as_ref().and_then(|p| self.tabs.get(p))
    }

    /// Get mutable reference to active tab
    pub fn get_active_tab_mut(&mut self) -> Option<&mut EditorTab> {
        self.active_tab.as_ref().and_then(|p| self.tabs.get_mut(p))
    }

    /// Update content of active tab
    pub fn update_active_content(&mut self, new_content: String) {
        if let Some(tab) = self.get_active_tab_mut() {
            tab.content = new_content;
            tab.modified = true;
        }
    }

    /// Register an LSP client
    pub fn register_lsp_client(&mut self, language_id: String, client: LspClient) {
        self.lsp_clients
            .insert(language_id, Arc::new(Mutex::new(client)));
    }

    /// Get LSP client for a language
    pub fn get_lsp_client(&self, language_id: &str) -> Option<Arc<Mutex<LspClient>>> {
        self.lsp_clients.get(language_id).cloned()
    }

    /// Set terminal instance
    pub fn set_terminal(&mut self, terminal: TerminalShell) {
        self.terminal = Some(Arc::new(Mutex::new(terminal)));
    }

    /// Get terminal instance
    pub fn get_terminal(&self) -> Option<Arc<Mutex<TerminalShell>>> {
        self.terminal.clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// Conversion implementations
impl From<lsp_types::DiagnosticSeverity> for DiagnosticSeverity {
    fn from(severity: lsp_types::DiagnosticSeverity) -> Self {
        match severity {
            lsp_types::DiagnosticSeverity::ERROR => DiagnosticSeverity::Error,
            lsp_types::DiagnosticSeverity::WARNING => DiagnosticSeverity::Warning,
            lsp_types::DiagnosticSeverity::INFORMATION => DiagnosticSeverity::Information,
            lsp_types::DiagnosticSeverity::HINT => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Information,
        }
    }
}

impl From<lsp_types::CompletionItemKind> for CompletionItemKind {
    fn from(kind: lsp_types::CompletionItemKind) -> Self {
        match kind {
            lsp_types::CompletionItemKind::TEXT => CompletionItemKind::Text,
            lsp_types::CompletionItemKind::METHOD => CompletionItemKind::Method,
            lsp_types::CompletionItemKind::FUNCTION => CompletionItemKind::Function,
            lsp_types::CompletionItemKind::CONSTRUCTOR => CompletionItemKind::Constructor,
            lsp_types::CompletionItemKind::FIELD => CompletionItemKind::Field,
            lsp_types::CompletionItemKind::VARIABLE => CompletionItemKind::Variable,
            lsp_types::CompletionItemKind::CLASS => CompletionItemKind::Class,
            lsp_types::CompletionItemKind::INTERFACE => CompletionItemKind::Interface,
            lsp_types::CompletionItemKind::MODULE => CompletionItemKind::Module,
            lsp_types::CompletionItemKind::PROPERTY => CompletionItemKind::Property,
            lsp_types::CompletionItemKind::UNIT => CompletionItemKind::Unit,
            lsp_types::CompletionItemKind::VALUE => CompletionItemKind::Value,
            lsp_types::CompletionItemKind::ENUM => CompletionItemKind::Enum,
            lsp_types::CompletionItemKind::KEYWORD => CompletionItemKind::Keyword,
            lsp_types::CompletionItemKind::SNIPPET => CompletionItemKind::Snippet,
            lsp_types::CompletionItemKind::COLOR => CompletionItemKind::Color,
            lsp_types::CompletionItemKind::FILE => CompletionItemKind::File,
            lsp_types::CompletionItemKind::REFERENCE => CompletionItemKind::Reference,
            _ => CompletionItemKind::Text,
        }
    }
}
