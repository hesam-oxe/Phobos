//! Phobos IDE - Terminal Service
//! Integrated terminal using portable-pty

use anyhow::{bail, Context, Result};
use crossbeam_channel::Sender;
use log::{debug, error, info};
use std::io::Read as _;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Terminal output event
#[derive(Debug, Clone)]
pub struct TerminalOutput {
    pub data: String,
    pub is_error: bool,
}

/// Terminal shell wrapper
pub struct TerminalShell {
    pty_pair: Option<(
        Box<dyn portable_pty::MasterPty>,
        Box<dyn portable_pty::SlavePty>,
    )>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    writer: Option<Box<dyn std::io::Write + Send>>,
    reader_handle: Option<tokio::task::JoinHandle<()>>,
    output_tx: mpsc::Sender<TerminalOutput>,
    output_rx: Arc<Mutex<mpsc::Receiver<TerminalOutput>>>,
    size: (u16, u16), // (columns, rows)
}

impl TerminalShell {
    /// Create a new terminal shell
    pub fn new() -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();

        let pair = pty_system.openpty(portable_pty::PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let (output_tx, output_rx) = mpsc::channel(100);

        Ok(Self {
            pty_pair: Some((pair.master, pair.slave)),
            child: None,
            writer: None,
            reader_handle: None,
            output_tx,
            output_rx: Arc::new(Mutex::new(output_rx)),
            size: (80, 24),
        })
    }

    /// Spawn a shell in the terminal
    pub async fn spawn(&mut self, shell: Option<&str>) -> Result<()> {
        let (master, slave) = self
            .pty_pair
            .take()
            .context("PTY already spawned or not initialized")?;

        // Determine shell command
        let shell_cmd: String = match shell {
            Some(s) => s.to_string(),
            None => {
                if cfg!(windows) {
                    "cmd.exe".to_string()
                } else {
                    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
                }
            }
        };

        let cmd_builder = portable_pty::CommandBuilder::new(&shell_cmd);
        let child = slave.spawn_command(cmd_builder)?;

        // Take writer/reader from the master; keep master + child alive in self
        let writer = master.take_writer()?;
        self.writer = Some(writer);

        let mut reader = master.try_clone_reader()?;
        self.child = Some(child);
        self.pty_pair = Some((master, slave));
        let output_tx = self.output_tx.clone();

        // Start reading output in background task
        let handle = tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        if output_tx
                            .send(TerminalOutput {
                                data,
                                is_error: false,
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from PTY: {}", e);
                        break;
                    }
                }
            }
        });

        self.reader_handle = Some(handle);
        self.size = (80, 24);

        info!("Terminal shell spawned: {}", shell_cmd);
        Ok(())
    }

    /// Write input to the terminal
    pub fn write_input(&mut self, input: &str) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            use std::io::Write;
            writer.write_all(input.as_bytes())?;
            writer.flush()?;
            Ok(())
        } else {
            bail!("Terminal not spawned or writer not available")
        }
    }

    /// Send a key press (special handling for control characters)
    pub fn send_key(&mut self, key: char, ctrl: bool) -> Result<()> {
        let input = if ctrl {
            // Convert Ctrl+key to control character
            match key.to_ascii_lowercase() {
                'c' => "\x03".to_string(), // Ctrl+C
                'd' => "\x04".to_string(), // Ctrl+D
                'l' => "\x0c".to_string(), // Ctrl+L (clear)
                'z' => "\x1a".to_string(), // Ctrl+Z
                _ => format!("{}", ((key as u8) & 0x1f) as char),
            }
        } else {
            key.to_string()
        };

        self.write_input(&input)
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.size = (cols, rows);

        if let Some((ref master, _)) = self.pty_pair {
            master.resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        }

        Ok(())
    }

    /// Try to receive terminal output (non-blocking)
    pub async fn try_recv_output(&self) -> Option<TerminalOutput> {
        let mut rx = self.output_rx.lock().await;
        rx.try_recv().ok()
    }

    /// Receive terminal output (blocking)
    pub async fn recv_output(&self) -> Option<TerminalOutput> {
        let mut rx = self.output_rx.lock().await;
        rx.recv().await
    }

    /// Clear terminal history (send clear command)
    pub fn clear(&mut self) -> Result<()> {
        if cfg!(windows) {
            self.write_input("cls\r\n")
        } else {
            self.write_input("\x1b[2J\x1b[H") // ANSI clear sequence
        }
    }

    /// Check if terminal is still running
    pub fn is_running(&self) -> bool {
        self.reader_handle.is_some()
    }

    /// Kill the terminal process
    pub async fn kill(&mut self) {
        if let Some(handle) = self.reader_handle.take() {
            handle.abort();
        }

        if let Some(mut child) = self.child.take() {
            child.kill().ok();
        }

        self.writer = None;
        self.pty_pair = None;

        info!("Terminal killed");
    }
}

impl Default for TerminalShell {
    fn default() -> Self {
        Self::new().expect("Failed to create terminal")
    }
}

impl Drop for TerminalShell {
    fn drop(&mut self) {
        // Sync-only cleanup: abort the reader task and drop handles.
        // (No blocking executor here; async kill() covers the rest.)
        if let Some(handle) = self.reader_handle.take() {
            handle.abort();
        }
        self.writer = None;
        self.child = None;
        self.pty_pair = None;
    }
}

/// ANSI escape code parser/stripper
pub struct AnsiParser {
    strip_codes: bool,
}

impl AnsiParser {
    pub fn new(strip_codes: bool) -> Self {
        Self { strip_codes }
    }

    /// Parse ANSI codes and return plain text (or styled text if supported)
    pub fn parse(&self, input: &str) -> String {
        if self.strip_codes {
            self.strip_ansi_codes(input)
        } else {
            input.to_string()
        }
    }

    /// Strip ANSI escape codes from string
    fn strip_ansi_codes(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                chars.next(); // consume '['

                // Skip until we find the end of the escape sequence
                while let Some(c) = chars.next() {
                    if c.is_ascii_alphabetic() || c == 'm' {
                        break;
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
    }
}

/// Helper function to create and spawn a terminal
pub async fn create_terminal(shell: Option<&str>) -> Result<TerminalShell> {
    let mut terminal = TerminalShell::new()?;
    terminal.spawn(shell).await?;
    Ok(terminal)
}
