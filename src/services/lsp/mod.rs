//! Phobos IDE - LSP Service Module

pub mod client;

pub use client::{LspClient, LspServerConfig, LspState, create_lsp_client};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use crossbeam_channel::{Sender, Receiver, unbounded};

use crate::core::{
    CompletionItem as CoreCompletionItem, CompletionItemKind as CoreKind,
    DiagnosticSeverity as CoreSeverity, LspDiagnostic as CoreDiagnostic,
};

/// Events from the LSP background thread to the UI (polled every frame).
#[derive(Debug, Clone)]
pub enum LspEvent {
    Diagnostics { path: String, diagnostics: Vec<CoreDiagnostic> },
    Completions { id: u64, items: Vec<CoreCompletionItem> },
    Hover { id: u64, text: String },
    Status(String),
}

enum LspCmd {
    Open { language: String, root: PathBuf, path: String, content: String },
    Change { language: String, path: String, content: String },
    Complete { id: u64, language: String, path: String, line: u32, col: u32 },
    Hover { id: u64, language: String, path: String, line: u32, col: u32 },
}

/// LSP manager: owns one background thread with its own tokio runtime.
/// All methods are non-blocking; results arrive via [`LspManager::poll`].
pub struct LspManager {
    tx: Sender<LspCmd>,
    rx: Receiver<LspEvent>,
    next_id: u64,
}

impl LspManager {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = unbounded::<LspCmd>();
        let (ev_tx, ev_rx) = unbounded::<LspEvent>();
        std::thread::spawn(move || Self::worker(cmd_rx, ev_tx));
        Self { tx: cmd_tx, rx: ev_rx, next_id: 0 }
    }

    /// Language id for a file path, if a server is configured.
    pub fn language_for(path: &str) -> Option<String> {
        let ext = std::path::Path::new(path).extension()?.to_str()?.to_lowercase();
        LspServerConfig::for_extension(&ext).map(|c| c.language_id)
    }

    pub fn supports(path: &str) -> bool {
        Self::language_for(path).is_some()
    }

    pub fn open_file(&self, path: &str, content: &str, root: PathBuf) {
        if let Some(language) = Self::language_for(path) {
            let _ = self.tx.send(LspCmd::Open {
                language, root, path: path.to_string(), content: content.to_string(),
            });
        }
    }

    pub fn change(&self, path: &str, content: &str) {
        if let Some(language) = Self::language_for(path) {
            let _ = self.tx.send(LspCmd::Change {
                language, path: path.to_string(), content: content.to_string(),
            });
        }
    }

    pub fn request_completion(&mut self, path: &str, line: u32, col: u32) -> Option<u64> {
        let language = Self::language_for(path)?;
        self.next_id += 1;
        let id = self.next_id;
        let _ = self.tx.send(LspCmd::Complete {
            id, language, path: path.to_string(), line, col,
        });
        Some(id)
    }

    pub fn request_hover(&mut self, path: &str, line: u32, col: u32) -> Option<u64> {
        let language = Self::language_for(path)?;
        self.next_id += 1;
        let id = self.next_id;
        let _ = self.tx.send(LspCmd::Hover {
            id, language, path: path.to_string(), line, col,
        });
        Some(id)
    }

    /// Drain pending events (non-blocking).
    pub fn poll(&self) -> Vec<LspEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = self.rx.try_recv() {
            out.push(ev);
        }
        out
    }

    fn worker(cmd_rx: Receiver<LspCmd>, ev_tx: Sender<LspEvent>) {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all().build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = ev_tx.send(LspEvent::Status(format!("LSP runtime failed: {}", e)));
                return;
            }
        };
        let mut clients: HashMap<String, LspClient> = HashMap::new();
        let mut opened: HashSet<String> = HashSet::new();
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                LspCmd::Open { language, root, path, content } => {
                    let client = match Self::ensure_client(
                        &rt, &mut clients, &ev_tx, &language, &root,
                    ) {
                        Some(c) => c,
                        None => continue,
                    };
                    let res = if opened.contains(&path) {
                        rt.block_on(client.did_change(&path, &content))
                    } else {
                        match rt.block_on(client.did_open(&path, &content, &language)) {
                            Ok(()) => {
                                opened.insert(path.clone());
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    };
                    if let Err(e) = res {
                        let _ = ev_tx.send(LspEvent::Status(format!("LSP {}: {}", language, e)));
                    }
                }
                LspCmd::Change { language, path, content } => {
                    let client = match Self::ensure_client(
                        &rt, &mut clients, &ev_tx, &language,
                        &std::path::Path::new(&path).parent()
                            .map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from(".")),
                    ) {
                        Some(c) => c,
                        None => continue,
                    };
                    let res = if opened.contains(&path) {
                        rt.block_on(client.did_change(&path, &content))
                    } else {
                        match rt.block_on(client.did_open(&path, &content, &language)) {
                            Ok(()) => {
                                opened.insert(path.clone());
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    };
                    if let Err(e) = res {
                        let _ = ev_tx.send(LspEvent::Status(format!("LSP {}: {}", language, e)));
                    }
                }
                LspCmd::Complete { id, language, path, line, col } => {
                    if let Some(client) = clients.get_mut(&language) {
                        match rt.block_on(client.completion(&path, line, col)) {
                            Ok(items) => {
                                let v = items.into_iter().map(convert_completion).collect();
                                let _ = ev_tx.send(LspEvent::Completions { id, items: v });
                            }
                            Err(e) => {
                                let _ = ev_tx.send(LspEvent::Status(format!("complete: {}", e)));
                            }
                        }
                    }
                }
                LspCmd::Hover { id, language, path, line, col } => {
                    if let Some(client) = clients.get_mut(&language) {
                        match rt.block_on(client.hover(&path, line, col)) {
                            Ok(Some(h)) => {
                                let _ = ev_tx.send(LspEvent::Hover { id, text: hover_to_string(h) });
                            }
                            Ok(None) => {
                                let _ = ev_tx.send(LspEvent::Hover { id, text: String::new() });
                            }
                            Err(e) => {
                                let _ = ev_tx.send(LspEvent::Status(format!("hover: {}", e)));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get or initialize the client for a language. Returns None (after a
    /// Status event) when no server is configured or it failed to start.
    /// The failure is reported only once per language per session.
    fn ensure_client<'a>(
        rt: &tokio::runtime::Runtime,
        clients: &'a mut HashMap<String, LspClient>,
        ev_tx: &Sender<LspEvent>,
        language: &str,
        root: &PathBuf,
    ) -> Option<&'a mut LspClient> {
        if !clients.contains_key(language) {
            let cfg = match LspServerConfig::for_language(language) {
                Some(c) => c,
                None => return None,
            };
            let mut c = LspClient::new(cfg);
            let (dtx, mut drx) = tokio::sync::mpsc::channel::<(
                String, Vec<lsp_types::Diagnostic>,
            )>(64);
            c.set_diagnostics_channel(dtx);
            match rt.block_on(c.initialize(root.clone())) {
                Ok(()) => {
                    let _ = ev_tx.send(LspEvent::Status(format!("LSP {} ready", language)));
                    let ev = ev_tx.clone();
                    rt.spawn(async move {
                        while let Some((uri, diags)) = drx.recv().await {
                            let items = diags.into_iter().map(convert_diagnostic).collect();
                            let _ = ev.send(LspEvent::Diagnostics {
                                path: uri_to_path(&uri),
                                diagnostics: items,
                            });
                        }
                    });
                }
                Err(e) => {
                    let _ = ev_tx.send(LspEvent::Status(
                        format!("LSP {} unavailable: {}", language, e),
                    ));
                    return None;
                }
            }
            clients.insert(language.to_string(), c);
        }
        clients.get_mut(language)
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

fn uri_to_path(uri: &str) -> String {
    if let Ok(u) = url::Url::parse(uri) {
        if let Ok(p) = u.to_file_path() {
            return p.to_string_lossy().to_string();
        }
    }
    uri.to_string()
}

fn convert_diagnostic(d: lsp_types::Diagnostic) -> CoreDiagnostic {
    CoreDiagnostic {
        line: d.range.start.line,
        column: d.range.start.character,
        end_line: d.range.end.line,
        end_column: d.range.end.character,
        severity: d.severity.map(CoreSeverity::from).unwrap_or(CoreSeverity::Information),
        message: d.message,
        source: d.source,
        code: d.code.map(|c| match c {
            lsp_types::NumberOrString::Number(n) => n.to_string(),
            lsp_types::NumberOrString::String(s) => s,
        }),
    }
}

fn convert_completion(c: lsp_types::CompletionItem) -> CoreCompletionItem {
    let doc = c.documentation.map(|d| match d {
        lsp_types::Documentation::String(s) => s,
        lsp_types::Documentation::MarkupContent(m) => m.value,
    });
    let label = c.label.clone();
    CoreCompletionItem {
        label,
        kind: c.kind.map(CoreKind::from).unwrap_or(CoreKind::Text),
        detail: c.detail,
        documentation: doc,
        insert_text: c.insert_text.or(Some(c.label)),
    }
}

fn marked_to_string(m: lsp_types::MarkedString) -> String {
    match m {
        lsp_types::MarkedString::String(s) => s,
        lsp_types::MarkedString::LanguageString(ls) => ls.value,
    }
}

fn hover_to_string(h: lsp_types::Hover) -> String {
    match h.contents {
        lsp_types::HoverContents::Scalar(m) => marked_to_string(m),
        lsp_types::HoverContents::Array(v) => v
            .into_iter()
            .map(marked_to_string)
            .collect::<Vec<_>>()
            .join("\n---\n"),
        lsp_types::HoverContents::Markup(m) => m.value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        Diagnostic, DiagnosticSeverity, Hover, HoverContents, MarkedString, NumberOrString,
        Position, Range,
    };

    fn pos(line: u32, ch: u32) -> Position {
        Position { line, character: ch }
    }

    #[test]
    fn diagnostic_conversion() {
        let d = Diagnostic {
            range: Range { start: pos(3, 5), end: pos(3, 9) },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::Number(42)),
            code_description: None,
            source: Some("rustc".into()),
            message: "boom".into(),
            related_information: None,
            tags: None,
            data: None,
        };
        let c = convert_diagnostic(d);
        assert_eq!((c.line, c.column, c.end_line, c.end_column), (3, 5, 3, 9));
        assert!(matches!(c.severity, CoreSeverity::Error));
        assert_eq!(c.code.as_deref(), Some("42"));
    }

    #[test]
    fn hover_scalars_and_markup() {
        let h = Hover {
            contents: HoverContents::Scalar(MarkedString::String("hi".into())),
            range: None,
        };
        assert_eq!(hover_to_string(h), "hi");
        let h = Hover {
            contents: HoverContents::Array(vec![
                MarkedString::String("a".into()),
                MarkedString::LanguageString(lsp_types::LanguageString {
                    language: "rust".into(),
                    value: "b".into(),
                }),
            ]),
            range: None,
        };
        assert_eq!(hover_to_string(h), "a\n---\nb");
    }

    #[test]
    fn uri_file_path() {
        assert_eq!(uri_to_path("file:///tmp/a.rs"), "/tmp/a.rs");
        assert_eq!(uri_to_path("notauri"), "notauri");
    }

    #[test]
    fn completion_falls_back_to_label() {
        let c = lsp_types::CompletionItem {
            label: "foo".into(),
            insert_text: None,
            ..Default::default()
        };
        let out = convert_completion(c);
        assert_eq!(out.insert_text.as_deref(), Some("foo"));
        assert!(matches!(out.kind, CoreKind::Text));
    }
}
