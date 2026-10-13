//! Phobos IDE - Debug Adapter Protocol (DAP) client
//! Spawns a debug adapter (debugpy, lldb-dap, cpptools, …), talks DAP over
//! stdio with Content-Length framing, and forwards events to the UI.
//! The UI never blocks: [`DapSession`] owns a worker thread + tokio runtime.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use anyhow::{Result, Context, bail};
use log::{info, warn};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct DapThread {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DapFrame {
    pub id: i64,
    pub name: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct DapScope {
    pub ref_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DapVar {
    pub name: String,
    pub value: String,
    pub var_type: String,
}

/// Events delivered to the UI thread (drained every frame).
#[derive(Debug, Clone)]
pub enum DapEvent {
    Output(String),
    Stopped { reason: String, thread_id: i64 },
    Continued { thread_id: i64 },
    Terminated,
    Exited(i64),
    Threads(Vec<DapThread>),
    StackFrames { thread: i64, frames: Vec<DapFrame> },
    Scopes { frame: i64, scopes: Vec<DapScope> },
    Variables { parent: i64, vars: Vec<DapVar> },
    Status(String),
}

enum DapCmd {
    Launch { adapter: String, args: Vec<String>, program: String, cwd: String },
    Breakpoints { path: String, lines: Vec<u32> },
    Continue { thread: Option<i64> },
    Next { thread: Option<i64> },
    StepIn { thread: Option<i64> },
    Disconnect,
}

/// Sync facade for the UI thread. All methods are non-blocking.
pub struct DapSession {
    tx: std::sync::mpsc::Sender<DapCmd>,
    rx: std::sync::mpsc::Receiver<DapEvent>,
}

impl DapSession {
    pub fn launch(adapter: &str, args: Vec<String>, program: &str, cwd: &str) -> Self {
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<DapCmd>();
        let (ev_tx, ev_rx) = std::sync::mpsc::channel::<DapEvent>();
        let adapter = adapter.to_string();
        let program = program.to_string();
        let cwd = cwd.to_string();
        std::thread::spawn(move || {
            Self::worker(cmd_rx, ev_tx, &adapter, &args, &program, &cwd);
        });
        Self { tx: cmd_tx, rx: ev_rx }
    }

    pub fn set_breakpoints(&self, path: &str, lines: Vec<u32>) {
        let _ = self.tx.send(DapCmd::Breakpoints {
            path: path.to_string(),
            lines,
        });
    }

    pub fn continue_exec(&self, thread: Option<i64>) {
        let _ = self.tx.send(DapCmd::Continue { thread });
    }

    pub fn step_over(&self, thread: Option<i64>) {
        let _ = self.tx.send(DapCmd::Next { thread });
    }

    pub fn step_into(&self, thread: Option<i64>) {
        let _ = self.tx.send(DapCmd::StepIn { thread });
    }

    pub fn disconnect(&self) {
        let _ = self.tx.send(DapCmd::Disconnect);
    }

    /// Drain pending events (non-blocking).
    pub fn poll(&self) -> Vec<DapEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = self.rx.try_recv() {
            out.push(ev);
        }
        out
    }

    fn worker(
        cmd_rx: std::sync::mpsc::Receiver<DapCmd>,
        ev_tx: std::sync::mpsc::Sender<DapEvent>,
        adapter: &str,
        args: &[String],
        program: &str,
        cwd: &str,
    ) {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all().build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = ev_tx.send(DapEvent::Status(format!("DAP runtime: {}", e)));
                return;
            }
        };
        let conn = match rt.block_on(DapConn::spawn(adapter, args, ev_tx.clone())) {
            Ok(c) => c,
            Err(e) => {
                let _ = ev_tx.send(DapEvent::Status(format!("DAP spawn: {}", e)));
                return;
            }
        };
        // initialize
        let init_args = json!({
            "adapterID": "phobos",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "pathFormat": "path",
        });
        if let Err(e) = rt.block_on(conn.request("initialize", init_args)) {
            let _ = ev_tx.send(DapEvent::Status(format!("DAP initialize: {}", e)));
            return;
        }
        // launch
        let launch_args = json!({ "program": program, "cwd": cwd });
        if let Err(e) = rt.block_on(conn.request("launch", launch_args)) {
            let _ = ev_tx.send(DapEvent::Status(format!("DAP launch: {}", e)));
            return;
        }
        let _ = ev_tx.send(DapEvent::Status(format!("DAP launched {}", program)));
        if let Err(e) = rt.block_on(conn.request("configurationDone", json!({}))) {
            let _ = ev_tx.send(DapEvent::Status(format!("DAP configDone: {}", e)));
        }
        let mut breakpoints: HashMap<String, Vec<u32>> = HashMap::new();
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                DapCmd::Launch { .. } => {}
                DapCmd::Breakpoints { path, lines } => {
                    breakpoints.insert(path.clone(), lines.clone());
                    let args = json!({
                        "source": { "path": path },
                        "breakpoints": lines.iter().map(|l| json!({ "line": l })).collect::<Vec<_>>(),
                    });
                    match rt.block_on(conn.request("setBreakpoints", args)) {
                        Ok(_) => {
                            let _ = ev_tx.send(DapEvent::Status(format!(
                                "breakpoints: {} @ {}",
                                lines.len(),
                                path
                            )));
                        }
                        Err(e) => {
                            let _ = ev_tx.send(DapEvent::Status(format!("setBreakpoints: {}", e)));
                        }
                    }
                }
                DapCmd::Continue { thread } => {
                    let args = json!({ "threadId": thread.unwrap_or(1) });
                    if let Err(e) = rt.block_on(conn.request("continue", args)) {
                        let _ = ev_tx.send(DapEvent::Status(format!("continue: {}", e)));
                    }
                }
                DapCmd::Next { thread } => {
                    let args = json!({ "threadId": thread.unwrap_or(1) });
                    if let Err(e) = rt.block_on(conn.request("next", args)) {
                        let _ = ev_tx.send(DapEvent::Status(format!("next: {}", e)));
                    }
                }
                DapCmd::StepIn { thread } => {
                    let args = json!({ "threadId": thread.unwrap_or(1) });
                    if let Err(e) = rt.block_on(conn.request("stepIn", args)) {
                        let _ = ev_tx.send(DapEvent::Status(format!("stepIn: {}", e)));
                    }
                }
                DapCmd::Disconnect => {
                    let _ = rt.block_on(conn.request("disconnect", json!({ "terminateDebuggee": true })));
                    let _ = ev_tx.send(DapEvent::Terminated);
                    break;
                }
            }
        }
        info!("DAP worker exit");
    }
}

type PendingMap = Arc<Mutex<HashMap<i64, oneshot::Sender<Value>>>>;

#[derive(Clone)]
struct DapConn {
    stdin: Arc<Mutex<ChildStdin>>,
    pending: PendingMap,
    seq: Arc<AtomicI64>,
}

impl DapConn {
    async fn spawn(
        adapter: &str,
        args: &[String],
        ev_tx: std::sync::mpsc::Sender<DapEvent>,
    ) -> Result<Self> {
        if which::which(adapter).is_err() {
            bail!("adapter not found in PATH: {}", adapter);
        }
        let mut child = Command::new(adapter)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(format!("spawn {}", adapter))?;
        let stdin = child.stdin.take().context("stdin")?;
        let stdout = child.stdout.take().context("stdout")?;
        let stderr = child.stderr.take().context("stderr")?;
        let conn = Self {
            stdin: Arc::new(Mutex::new(stdin)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            seq: Arc::new(AtomicI64::new(1)),
        };
        // stderr drain (adapter logs)
        tokio::spawn(async move {
            let mut r = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match r.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => warn!("DAP stderr: {}", line.trim()),
                    Err(_) => break,
                }
            }
        });
        // stdout reader: routes responses + converts events (with auto-fetch chain)
        let reader = conn.clone();
        tokio::spawn(async move {
            reader.reader_loop(stdout, ev_tx).await;
        });
        Ok(conn)
    }

    async fn request(&self, command: &str, args: Value) -> Result<Value> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let msg = json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": args,
        });
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(seq, tx);
        self.write_msg(&msg).await?;
        let val = tokio::time::timeout(std::time::Duration::from_secs(20), rx)
            .await
            .context("DAP request timeout")?
            .context("DAP channel closed")?;
        if val.get("success").and_then(|v| v.as_bool()) == Some(false) {
            let m = val.get("message").and_then(|v| v.as_str()).unwrap_or("request failed");
            bail!("DAP {}: {}", command, m);
        }
        Ok(val.get("body").cloned().unwrap_or(Value::Null))
    }

    async fn write_msg(&self, msg: &Value) -> Result<()> {
        let body = serde_json::to_string(msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        let mut g = self.stdin.lock().await;
        g.write_all(header.as_bytes()).await?;
        g.write_all(body.as_bytes()).await?;
        g.flush().await?;
        Ok(())
    }

    async fn reader_loop(
        &self,
        stdout: tokio::process::ChildStdout,
        ev_tx: std::sync::mpsc::Sender<DapEvent>,
    ) {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut content_length: Option<usize> = None;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => return,
                    Ok(_) => {}
                    Err(_) => return,
                }
                let t = line.trim();
                if t.is_empty() {
                    break;
                }
                if let Some(v) = t.strip_prefix("Content-Length:") {
                    content_length = v.trim().parse().ok();
                }
            }
            let len = match content_length {
                Some(l) => l,
                None => continue,
            };
            let mut buf = vec![0u8; len];
            let mut read = 0;
            while read < len {
                match reader.read(&mut buf[read..]).await {
                    Ok(0) => return,
                    Ok(n) => read += n,
                    Err(_) => return,
                }
            }
            let msg: Value = match serde_json::from_slice(&buf) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let typ = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if typ == "response" {
                if let Some(rs) = msg.get("request_seq").and_then(|v| v.as_i64()) {
                    let mut p = self.pending.lock().await;
                    if let Some(tx) = p.remove(&rs) {
                        let _ = tx.send(msg);
                    }
                }
            } else if typ == "event" {
                self.handle_event(&msg, &ev_tx).await;
            }
        }
    }

    async fn handle_event(&self, msg: &Value, ev_tx: &std::sync::mpsc::Sender<DapEvent>) {
        let event = msg.get("event").and_then(|v| v.as_str()).unwrap_or("");
        let body = msg.get("body").cloned().unwrap_or(Value::Null);
        match event {
            "stopped" => {
                let reason = body.get("reason").and_then(|v| v.as_str()).unwrap_or("?").to_string();
                let thread = body.get("threadId").and_then(|v| v.as_i64()).unwrap_or(1);
                let _ = ev_tx.send(DapEvent::Stopped { reason, thread_id: thread });
                // auto-fetch: threads → stack → scopes → variables (first of each)
                if let Ok(Value::Array(threads)) = self
                    .request("threads", json!({}))
                    .await
                    .map(|b| b.get("threads").cloned().unwrap_or(Value::Null))
                {
                    let list: Vec<DapThread> = threads
                        .iter()
                        .map(|t| DapThread {
                            id: t.get("id").and_then(|v| v.as_i64()).unwrap_or(0),
                            name: t.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                        })
                        .collect();
                    let _ = ev_tx.send(DapEvent::Threads(list));
                }
                let stack = self
                    .request("stackTrace", json!({ "threadId": thread }))
                    .await
                    .ok()
                    .and_then(|b| b.get("stackFrames").cloned())
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();
                let frames: Vec<DapFrame> = stack
                    .iter()
                    .map(|f| DapFrame {
                        id: f.get("id").and_then(|v| v.as_i64()).unwrap_or(0),
                        name: f.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                        file: f.get("source")
                            .and_then(|s| s.get("path"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("?")
                            .to_string(),
                        line: f.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    })
                    .collect();
                let _ = ev_tx.send(DapEvent::StackFrames { thread, frames: frames.clone() });
                if let Some(f) = frames.first() {
                    if let Ok(Value::Array(scopes)) = self
                        .request("scopes", json!({ "frameId": f.id }))
                        .await
                        .map(|b| b.get("scopes").cloned().unwrap_or(Value::Null))
                    {
                        let list: Vec<DapScope> = scopes
                            .iter()
                            .map(|s| DapScope {
                                ref_id: s.get("variablesReference").and_then(|v| v.as_i64()).unwrap_or(0),
                                name: s.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                            })
                            .collect();
                        let _ = ev_tx.send(DapEvent::Scopes { frame: f.id, scopes: list.clone() });
                        if let Some(s) = list.first() {
                            if let Ok(Value::Array(vars)) = self
                                .request("variables", json!({ "variablesReference": s.ref_id }))
                                .await
                                .map(|b| b.get("variables").cloned().unwrap_or(Value::Null))
                            {
                                let vlist: Vec<DapVar> = vars
                                    .iter()
                                    .map(|v| DapVar {
                                        name: v.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string(),
                                        value: v.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        var_type: v.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    })
                                    .collect();
                                let _ = ev_tx.send(DapEvent::Variables { parent: s.ref_id, vars: vlist });
                            }
                        }
                    }
                }
            }
            "continued" => {
                let thread = body.get("threadId").and_then(|v| v.as_i64()).unwrap_or(0);
                let _ = ev_tx.send(DapEvent::Continued { thread_id: thread });
            }
            "terminated" => {
                let _ = ev_tx.send(DapEvent::Terminated);
            }
            "exited" => {
                let code = body.get("exitCode").and_then(|v| v.as_i64()).unwrap_or(0);
                let _ = ev_tx.send(DapEvent::Exited(code));
            }
            "output" => {
                let text = body.get("output").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if !text.is_empty() {
                    let _ = ev_tx.send(DapEvent::Output(text));
                }
            }
            _ => {}
        }
    }
}

/// Adapter suggestions per language for the DAP section.
pub fn suggested_adapter(language: &str) -> &'static str {
    match language.to_lowercase().as_str() {
        "python" | "py" => "debugpy-adapter",
        "rust" | "c" | "cpp" | "c++" => "lldb-dap",
        "go" => "dlv dap",
        "java" => "java-debug-adapter",
        _ => "lldb-dap",
    }
}

/// Split "dlv dap"-style suggestion into (program, extra args).
pub fn split_adapter(s: &str) -> (String, Vec<String>) {
    let mut parts = s.split_whitespace();
    let prog = parts.next().unwrap_or("lldb-dap").to_string();
    let args = parts.map(|x| x.to_string()).collect();
    (prog, args)
}

pub fn workspace_of(program: &str) -> PathBuf {
    std::path::Path::new(program)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_plain_adapter() {
        let (p, a) = split_adapter("lldb-dap");
        assert_eq!(p, "lldb-dap");
        assert!(a.is_empty());
    }

    #[test]
    fn split_adapter_with_args() {
        let (p, a) = split_adapter("dlv dap --listen 127.0.0.1:0");
        assert_eq!(p, "dlv");
        assert_eq!(a, vec!["dap", "--listen", "127.0.0.1:0"]);
    }

    #[test]
    fn adapter_suggestions() {
        assert_eq!(suggested_adapter("python"), "debugpy-adapter");
        assert_eq!(suggested_adapter("rust"), "lldb-dap");
        assert_eq!(suggested_adapter("brainfuck"), "lldb-dap");
    }

    #[test]
    fn workspace_from_program() {
        assert_eq!(workspace_of("/a/b/prog"), PathBuf::from("/a/b"));
        assert_eq!(workspace_of("prog"), PathBuf::from("."));
    }
}
