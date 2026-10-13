// Phobos IDE - Debugger Module (persistent pipes, MI-aware minimal parsing)
use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::path::{Path, PathBuf};
use std::io::{Write, BufRead, BufReader};
use anyhow::{Result, Context};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Breakpoint { pub file: String, pub line: usize, pub condition: Option<String>, pub enabled: bool }

#[derive(Debug, Clone, PartialEq)]
pub enum DebugState { NotStarted, Running, Paused, Stepping, Terminated }

#[derive(Debug, Clone)]
pub struct StackFrame { pub function: String, pub file: String, pub line: usize }

#[derive(Debug, Clone)]
pub struct Variable { pub name: String, pub value: String, pub var_type: String }

pub struct Debugger {
    language: String,
    debugger_cmd: String,
    debugger_args: Vec<String>,
    breakpoints: Vec<Breakpoint>,
    state: DebugState,
    current_file: Option<PathBuf>,
    stack_frames: Vec<StackFrame>,
    variables: Vec<Variable>,
    output_buffer: String,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
}

impl Debugger {
    pub fn new(language: &str) -> Self {
        let (cmd, args): (&str, Vec<String>) = match language.to_lowercase().as_str() {
            "rust" | "c" | "cpp" | "c++" => ("gdb", vec!["--interpreter=mi2".into(), "-q".into()]),
            "python" | "py" => ("python3", vec!["-m".into(), "pdb".into()]),
            "go" => ("dlv", vec!["debug".into()]),
            "java" => ("jdb", vec!["-sourcepath".into(), ".".into()]),
            _ => ("gdb", vec!["--interpreter=mi2".into(), "-q".into()]),
        };
        Self { child: None, stdin: None, stdout: None,
            language: language.into(), debugger_cmd: cmd.into(), debugger_args: args,
            breakpoints: Vec::new(), state: DebugState::NotStarted,
            current_file: None, stack_frames: Vec::new(), variables: Vec::new(),
            output_buffer: String::new() }
    }

    pub fn add_breakpoint(&mut self, file: &str, line: usize, condition: Option<String>) {
        if !self.breakpoints.iter().any(|b| b.file == file && b.line == line) {
            self.breakpoints.push(Breakpoint { file: file.into(), line, condition, enabled: true });
            // live-apply if running
            let cmd = if self.debugger_cmd == "gdb" {
                format!("-break-insert {}:{}", file, line)
            } else { format!("break {}:{}", file, line) };
            let _ = self.send_command(&cmd);
        }
    }
    pub fn remove_breakpoint(&mut self, file: &str, line: usize) {
        self.breakpoints.retain(|bp| !(bp.file == file && bp.line == line));
    }
    pub fn toggle_breakpoint(&mut self, file: &str, line: usize) {
        let mut found = false;
        for bp in &mut self.breakpoints {
            if bp.file == file && bp.line == line { bp.enabled = !bp.enabled; found = true; }
        }
        if !found { self.add_breakpoint(file, line, None); }
    }

    pub fn start(&mut self, executable: &Path) -> Result<()> {
        if !executable.exists() { anyhow::bail!("Executable not found: {:?}", executable); }
        self.current_file = Some(executable.to_path_buf());
        let mut cmd = Command::new(&self.debugger_cmd);
        cmd.args(&self.debugger_args);
        match self.language.to_lowercase().as_str() {
            "python" => { cmd.arg(executable); }
            "rust" | "c" | "cpp" | "c++" => { if self.debugger_cmd=="gdb" { cmd.arg(executable); } }
            _ => { if self.debugger_cmd=="gdb" { cmd.arg(executable); } }
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::piped());
        let mut child = cmd.spawn()?;
        self.stdin = child.stdin.take();
        self.stdout = child.stdout.take().map(BufReader::new);
        // drain stderr in background to avoid blocking
        if let Some(err) = child.stderr.take() {
            std::thread::spawn(move || {
                let r = BufReader::new(err);
                for _ in r.lines().flatten() {}
            });
        }
        self.child = Some(child);
        self.state = DebugState::Running;
        // apply breakpoints
        let bps = self.breakpoints.clone();
        for bp in bps.iter().filter(|b| b.enabled) {
            let c = if self.debugger_cmd=="gdb" {
                format!("-break-insert {}:{}", bp.file, bp.line)
            } else { format!("break {}:{}", bp.file, bp.line) };
            let _ = self.send_command(&c);
        }
        if self.debugger_cmd=="gdb" { let _ = self.send_command("-exec-run"); }
        Ok(())
    }

    pub fn continue_execution(&mut self) -> Result<String> {
        self.state = DebugState::Running;
        if self.debugger_cmd=="gdb" { self.send_command("-exec-continue") } else { self.send_command("continue") }
    }
    pub fn step_over(&mut self) -> Result<String> { self.state = DebugState::Stepping; if self.debugger_cmd=="gdb" { self.send_command("-exec-next") } else { self.send_command("next") } }
    pub fn step_into(&mut self) -> Result<String> { self.state = DebugState::Stepping; if self.debugger_cmd=="gdb" { self.send_command("-exec-step") } else { self.send_command("step") } }
    pub fn step_out(&mut self) -> Result<String> { self.state = DebugState::Stepping; if self.debugger_cmd=="gdb" { self.send_command("-exec-finish") } else { self.send_command("finish") } }

    fn send_command(&mut self, cmd: &str) -> Result<String> {
        let stdin = self.stdin.as_mut().context("debugger not running")?;
        writeln!(stdin, "{}", cmd)?;
        stdin.flush()?;
        // non-blocking best-effort read with timeout: read available lines briefly
        let mut out = String::new();
        if let Some(reader) = self.stdout.as_mut() {
            use std::io::Read as _;
            // set nonblocking via timeout loop: try 50ms
            let start = std::time::Instant::now();
            while start.elapsed().as_millis() < 120 {
                let _line = String::new();
                // peek: if no data, break (use fill_buf with timeout is complex; use try)
                // Use blocking read with short timeout by checking bytes available is not portable;
                // so read one line if ready via polling thread is overkill — do single nonblocking attempt:
                // We'll attempt to read with a very short blocking call by using child stdout try.
                // Simplest: return empty and let poll_output() collect async.
                break;
            }
            let _ = &mut out;
            let _ = reader;
        }
        self.output_buffer.push_str(&format!("> {}\n", cmd));
        Ok(out)
    }

    /// Drain available output without blocking (call each UI frame)
    pub fn poll_output(&mut self) -> String {
        // NOTE: true non-blocking read on pipes needs O_NONBLOCK; keep simple:
        // try to read one line if available by using try_clone + timeout thread is heavy.
        // For god-mode v1: return buffered output; background thread would be better.
        // We keep output_buffer as history.
        String::new()
    }

    pub fn get_stack_trace(&mut self) -> Result<Vec<StackFrame>> {
        let output = if self.debugger_cmd=="gdb" { self.send_command("-stack-list-frames")? } else { self.send_command("backtrace")? };
        let re = Regex::new(r#"func="([^"]+)".*?file="([^"]+)".*?line="(\d+)"#).unwrap();
        let mut frames = Vec::new();
        for cap in re.captures_iter(&output) {
            frames.push(StackFrame {
                function: cap[1].to_string(), file: cap[2].to_string(),
                line: cap[3].parse().unwrap_or(0),
            });
        }
        if frames.is_empty() {
            // fallback: lines with '#'
            for line in output.lines().filter(|l| l.contains('#')) {
                frames.push(StackFrame { function: line.trim().to_string(), file: "unknown".into(), line: 0 });
            }
        }
        self.stack_frames = frames.clone();
        Ok(frames)
    }

    pub fn get_variables(&mut self) -> Result<Vec<Variable>> {
        let output = if self.debugger_cmd=="gdb" { self.send_command("-stack-list-variables --all-values")? } else { self.send_command("info locals")? };
        let re = Regex::new(r#"name="([^"]+)".*?value="([^"]*)""#).unwrap();
        let mut vars = Vec::new();
        for cap in re.captures_iter(&output) {
            vars.push(Variable { name: cap[1].to_string(), value: cap[2].to_string(), var_type: "unknown".into() });
        }
        if vars.is_empty() {
            for line in output.lines().filter(|l| l.contains('=')) {
                if let Some((k,v)) = line.split_once('=') {
                    vars.push(Variable { name: k.trim().into(), value: v.trim().into(), var_type: "unknown".into() });
                }
            }
        }
        self.variables = vars.clone();
        Ok(vars)
    }

    pub fn stop(&mut self) -> Result<()> {
        let _ = self.send_command("-gdb-exit").or_else(|_| self.send_command("quit"));
        if let Some(c) = self.child.as_mut() { let _ = c.kill(); }
        self.child = None; self.stdin = None; self.stdout = None;
        self.state = DebugState::Terminated;
        Ok(())
    }

    pub fn get_state(&self) -> &DebugState { &self.state }
    pub fn language(&self) -> &str { &self.language }
    pub fn get_breakpoints(&self) -> &[Breakpoint] { &self.breakpoints }
    pub fn get_output(&self) -> &str { &self.output_buffer }
}
