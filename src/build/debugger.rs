// Phobos IDE - Debugger Module
// Advanced debugging capabilities with breakpoints and step execution

use std::process::{Command, Stdio, Child};
use std::path::{Path, PathBuf};
use std::io::{Read, Write, BufRead, BufReader};
use std::sync::mpsc::{channel, Sender, Receiver};
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub file: String,
    pub line: usize,
    pub condition: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum DebugState {
    NotStarted,
    Running,
    Paused,
    Stepping,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub value: String,
    pub var_type: String,
}

pub struct Debugger {
    language: String,
    debugger_cmd: String,
    breakpoints: Vec<Breakpoint>,
    state: DebugState,
    current_file: Option<PathBuf>,
    stack_frames: Vec<StackFrame>,
    variables: Vec<Variable>,
    output_buffer: String,
}

impl Debugger {
    pub fn new(language: &str) -> Self {
        let debugger_cmd = match language.to_lowercase().as_str() {
            "rust" => "gdb".to_string(),
            "c" | "cpp" | "c++" => "gdb".to_string(),
            "python" | "py" => "pdb".to_string(),
            "javascript" | "js" | "typescript" | "ts" => "node --inspect".to_string(),
            "java" => "jdb".to_string(),
            "go" => "dlv".to_string(),
            _ => "gdb".to_string(),
        };

        Self {
            language: language.to_string(),
            debugger_cmd,
            breakpoints: Vec::new(),
            state: DebugState::NotStarted,
            current_file: None,
            stack_frames: Vec::new(),
            variables: Vec::new(),
            output_buffer: String::new(),
        }
    }

    pub fn add_breakpoint(&mut self, file: &str, line: usize, condition: Option<String>) {
        self.breakpoints.push(Breakpoint {
            file: file.to_string(),
            line,
            condition,
            enabled: true,
        });
    }

    pub fn remove_breakpoint(&mut self, file: &str, line: usize) {
        self.breakpoints.retain(|bp| !(bp.file == file && bp.line == line));
    }

    pub fn toggle_breakpoint(&mut self, file: &str, line: usize) {
        for bp in &mut self.breakpoints {
            if bp.file == file && bp.line == line {
                bp.enabled = !bp.enabled;
                return;
            }
        }
    }

    pub fn start(&mut self, executable: &Path) -> Result<()> {
        if !executable.exists() {
            anyhow::bail!("Executable not found: {:?}", executable);
        }

        self.current_file = Some(executable.to_path_buf());
        
        let mut cmd = Command::new(&self.debugger_cmd);
        
        match self.language.to_lowercase().as_str() {
            "python" => {
                cmd.arg("-m").arg("pdb").arg(executable);
            },
            "rust" | "c" | "cpp" => {
                cmd.arg(executable);
            },
            "java" => {
                cmd.arg("-sourcepath").arg(".");
                cmd.arg("Main"); // Assume Main class
            },
            "go" => {
                cmd.arg("debug");
            },
            _ => {
                cmd.arg(executable);
            },
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped());

        self.child = Some(cmd.spawn()?);
        self.state = DebugState::Running;

        Ok(())
    }

    child: Option<Child>,

    pub fn continue_execution(&mut self) -> Result<String> {
        self.send_command("continue")
    }

    pub fn step_over(&mut self) -> Result<String> {
        self.send_command("next")
    }

    pub fn step_into(&mut self) -> Result<String> {
        self.send_command("step")
    }

    pub fn step_out(&mut self) -> Result<String> {
        self.send_command("finish")
    }

    fn send_command(&mut self, cmd: &str) -> Result<String> {
        if let Some(ref mut child) = self.child {
            if let Some(mut stdin) = child.stdin.take() {
                writeln!(stdin, "{}", cmd)?;
                
                if let Some(stdout) = child.stdout.take() {
                    let mut reader = BufReader::new(stdout);
                    let mut output = String::new();
                    reader.read_line(&mut output)?;
                    self.output_buffer = output.clone();
                    return Ok(output);
                }
            }
        }
        Ok(String::new())
    }

    pub fn get_stack_trace(&mut self) -> Result<Vec<StackFrame>> {
        let output = self.send_command("backtrace")?;
        
        // Parse backtrace output (simplified)
        let frames: Vec<StackFrame> = output.lines()
            .filter_map(|line| {
                if line.contains("#") {
                    Some(StackFrame {
                        function: "unknown".to_string(),
                        file: "unknown".to_string(),
                        line: 0,
                    })
                } else {
                    None
                }
            })
            .collect();

        self.stack_frames = frames.clone();
        Ok(frames)
    }

    pub fn get_variables(&mut self) -> Result<Vec<Variable>> {
        let output = self.send_command("info locals")?;
        
        // Parse variables output (simplified)
        let vars: Vec<Variable> = output.lines()
            .filter_map(|line| {
                if line.contains('=') {
                    let parts: Vec<&str> = line.split('=').collect();
                    if parts.len() >= 2 {
                        Some(Variable {
                            name: parts[0].trim().to_string(),
                            value: parts[1].trim().to_string(),
                            var_type: "unknown".to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        self.variables = vars.clone();
        Ok(vars)
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.child {
            child.kill()?;
        }
        self.child = None;
        self.state = DebugState::Terminated;
        Ok(())
    }

    pub fn get_state(&self) -> &DebugState {
        &self.state
    }

    pub fn get_breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }

    pub fn get_output(&self) -> &str {
        &self.output_buffer
    }
}
