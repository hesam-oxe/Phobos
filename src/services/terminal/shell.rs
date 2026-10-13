//! Phobos IDE - Terminal Service
//! Integrated terminal using portable-pty 0.8

use std::io::Read;
use std::sync::Arc;
use anyhow::{Result, Context};
use log::{info, error};
use tokio::sync::{Mutex, mpsc};

/// Terminal output event
#[derive(Debug, Clone)]
pub struct TerminalOutput {
    pub data: String,
    pub is_error: bool,
}

/// Terminal shell wrapper (portable-pty 0.8 API)
pub struct TerminalShell {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    writer: Option<Box<dyn std::io::Write + Send>>,
    reader_handle: Option<tokio::task::JoinHandle<()>>,
    output_tx: mpsc::Sender<TerminalOutput>,
    output_rx: Arc<Mutex<mpsc::Receiver<TerminalOutput>>>,
    size: (u16, u16),
}

impl TerminalShell {
    pub fn new() -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system.openpty(portable_pty::PtySize {
            rows: 24, cols: 80, pixel_width: 0, pixel_height: 0,
        })?;
        let (output_tx, output_rx) = mpsc::channel(256);
        Ok(Self {
            master: Some(pair.master),
            child: None,
            writer: None,
            reader_handle: None,
            output_tx,
            output_rx: Arc::new(Mutex::new(output_rx)),
            size: (80, 24),
        })
    }

    pub async fn spawn(&mut self, shell: Option<&str>) -> Result<()> {
        let shell_cmd = if let Some(s) = shell {
            s.to_string()
        } else if cfg!(windows) {
            "cmd.exe".to_string()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        };
        let master = self.master.take().context("PTY already spawned")?;
        // Keep a master handle for resize; clone reader/writer before move
        let mut master_for_child = master;
        let mut reader = master_for_child.try_clone_reader().context("clone reader")?;
        let writer = master_for_child.take_writer().context("take writer")?;
        self.writer = Some(writer);

        let mut cmd = portable_pty::CommandBuilder::new(&shell_cmd);
        cmd.cwd(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
        // Re-create pair? portable-pty 0.8 spawn needs slave; we consumed pair.master only.
        // We still need slave: reopen PTY and use its slave for spawn, keep master.
        // Simplest: spawn via slave from a fresh pair and keep original master reader.
        // To avoid complexity, spawn child with slave from new pair is wrong (different pty).
        // Correct approach: we should have kept both master and slave.
        // Workaround: recreate everything properly:
        let pty_system = portable_pty::native_pty_system();
        let pair2 = pty_system.openpty(portable_pty::PtySize {
            rows: self.size.1, cols: self.size.0, pixel_width: 0, pixel_height: 0,
        })?;
        let mut master2 = pair2.master;
        let slave = pair2.slave;
        // replace reader/writer with correct pty
        reader = master2.try_clone_reader().context("clone reader2")?;
        self.writer = Some(master2.take_writer().context("writer2")?);
        let child = slave.spawn_command(cmd).context("spawn shell")?;
        self.child = Some(child);
        self.master = Some(master2);
        drop(master_for_child);

        let output_tx = self.output_tx.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        if output_tx.blocking_send(TerminalOutput { data, is_error: false }).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("PTY read error: {}", e);
                        break;
                    }
                }
            }
        });
        self.reader_handle = Some(handle);
        info!("Terminal shell spawned: {}", shell_cmd);
        Ok(())
    }

    pub fn write_input(&mut self, input: &str) -> Result<()> {
        if let Some(w) = self.writer.as_mut() {
            use std::io::Write;
            w.write_all(input.as_bytes())?;
            w.flush()?;
            Ok(())
        } else {
            anyhow::bail!("Terminal not spawned")
        }
    }

    pub fn send_key(&mut self, key: char, ctrl: bool) -> Result<()> {
        let input = if ctrl {
            match key.to_ascii_lowercase() {
                'c' => "\x03".to_string(),
                'd' => "\x04".to_string(),
                'l' => "\x0c".to_string(),
                'z' => "\x1a".to_string(),
                _ => format!("{}", ((key as u8) & 0x1f) as char),
            }
        } else { key.to_string() };
        self.write_input(&input)
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.size = (cols, rows);
        if let Some(m) = self.master.as_ref() {
            m.resize(portable_pty::PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;
        }
        Ok(())
    }

    pub async fn try_recv_output(&self) -> Option<TerminalOutput> {
        self.output_rx.lock().await.try_recv().ok()
    }

    pub async fn recv_output(&self) -> Option<TerminalOutput> {
        self.output_rx.lock().await.recv().await
    }

    pub fn clear(&mut self) -> Result<()> {
        if cfg!(windows) { self.write_input("cls\r\n") } else { self.write_input("\x1b[2J\x1b[H") }
    }

    pub fn is_running(&self) -> bool { self.reader_handle.is_some() }

    pub async fn kill(&mut self) {
        if let Some(h) = self.reader_handle.take() { h.abort(); }
        self.writer = None;
        self.child = None;
        info!("Terminal killed");
    }
}

impl Default for TerminalShell {
    fn default() -> Self { Self::new().expect("Failed to create terminal") }
}

/// ANSI escape code parser/stripper
pub struct AnsiParser { strip_codes: bool }

impl AnsiParser {
    pub fn new(strip_codes: bool) -> Self { Self { strip_codes } }
    pub fn parse(&self, input: &str) -> String {
        if self.strip_codes { self.strip_ansi_codes(input) } else { input.to_string() }
    }
    fn strip_ansi_codes(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                chars.next();
                while let Some(c) = chars.next() {
                    if c.is_ascii_alphabetic() { break; }
                }
            } else { result.push(c); }
        }
        result
    }
}

pub async fn create_terminal(shell: Option<&str>) -> Result<TerminalShell> {
    let mut t = TerminalShell::new()?;
    t.spawn(shell).await?;
    Ok(t)
}
