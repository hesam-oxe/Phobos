// Phobos IDE - Runner Module
// Code execution and runtime management

use std::process::{Command, Stdio, Child};
use std::path::{Path, PathBuf};
use std::io::{Write, BufRead, BufReader};
use std::sync::mpsc::{channel, Receiver};
use anyhow::{Result, Context};

pub struct Runner {
    language: String,
    working_dir: PathBuf,
    env_vars: Vec<(String, String)>,
}

impl Runner {
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            working_dir: PathBuf::from("."),
            env_vars: Vec::new(),
        }
    }

    pub fn set_working_dir(&mut self, path: &str) {
        self.working_dir = PathBuf::from(path);
    }

    pub fn add_env_var(&mut self, key: &str, value: &str) {
        self.env_vars.push((key.to_string(), value.to_string()));
    }

    pub fn run(&self, file_or_executable: &Path) -> Result<RunResult> {
        let (interpreter, args) = self.get_interpreter_and_args(file_or_executable);

        if interpreter.is_empty() {
            anyhow::bail!("No runner configured for language: {}", self.language);
        }

        let mut cmd = Command::new(&interpreter);

        if !args.is_empty() {
            cmd.args(&args);
        }

        cmd.current_dir(&self.working_dir)
            .stdin(Stdio::null());

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Timeout guard: 30s
        let start_time = std::time::Instant::now();

        let output = cmd.output()
            .with_context(|| format!("Failed to spawn process: {}", interpreter))?;

        let duration = start_time.elapsed();

        Ok(RunResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            errors: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            duration_ms: duration.as_millis() as u64,
        })
    }

    pub fn run_async(&self, file_or_executable: &Path) -> Result<AsyncRunner> {
        let (interpreter, args) = self.get_interpreter_and_args(file_or_executable);
        
        if interpreter.is_empty() {
            anyhow::bail!("No runner configured for language: {}", self.language);
        }

        let mut cmd = Command::new(&interpreter);
        
        if !args.is_empty() {
            cmd.args(&args);
        }

        cmd.current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped());

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let child = cmd.spawn()
            .with_context(|| format!("Failed to spawn process: {}", interpreter))?;

        Ok(AsyncRunner::new(child))
    }

    fn get_interpreter_and_args(&self, file: &Path) -> (String, Vec<String>) {
        match self.language.to_lowercase().as_str() {
            "python" | "py" => ("python3".to_string(), vec![file.to_string_lossy().to_string()]),
            "javascript" | "js" => ("node".to_string(), vec![file.to_string_lossy().to_string()]),
            "typescript" | "ts" => ("ts-node".to_string(), vec![file.to_string_lossy().to_string()]),
            "ruby" => ("ruby".to_string(), vec![file.to_string_lossy().to_string()]),
            "php" => ("php".to_string(), vec![file.to_string_lossy().to_string()]),
            "java" => {
                // For Java, we need to extract the class name
                let class_name = file.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Main");
                ("java".to_string(), vec![class_name.to_string()])
            },
            "go" => ("go".to_string(), vec!["run".to_string(), file.to_string_lossy().to_string()]),
            "rust" => {
                // Rust executables are already compiled
                (file.to_string_lossy().to_string(), vec![])
            },
            "c" | "cpp" | "c++" => {
                // C/C++ executables are already compiled
                (file.to_string_lossy().to_string(), vec![])
            },
            _ => (file.to_string_lossy().to_string(), vec![]),
        }
    }
}

/// A spawned child with line-streaming stdout/stderr.
/// Reader threads push lines into channels; the UI drains them every frame.
pub struct AsyncRunner {
    child: Option<Child>,
    out_rx: Receiver<String>,
    err_rx: Receiver<String>,
}

impl AsyncRunner {
    fn new(mut child: Child) -> Self {
        let (out_tx, out_rx) = channel::<String>();
        let (err_tx, err_rx) = channel::<String>();
        if let Some(out) = child.stdout.take() {
            std::thread::spawn(move || {
                let r = BufReader::new(out);
                for line in r.lines().flatten() {
                    if out_tx.send(line).is_err() {
                        break;
                    }
                }
            });
        }
        if let Some(err) = child.stderr.take() {
            std::thread::spawn(move || {
                let r = BufReader::new(err);
                for line in r.lines().flatten() {
                    if err_tx.send(line).is_err() {
                        break;
                    }
                }
            });
        }
        Self { child: Some(child), out_rx, err_rx }
    }

    pub fn write_input(&mut self, input: &str) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(input.as_bytes())?;
                stdin.flush()?;
                return Ok(());
            }
        }
        anyhow::bail!("stdin not available")
    }

    /// Drain available lines without blocking: (stdout, stderr).
    pub fn drain(&mut self) -> (Vec<String>, Vec<String>) {
        let mut out = Vec::new();
        let mut err = Vec::new();
        while let Ok(l) = self.out_rx.try_recv() {
            out.push(l);
        }
        while let Ok(l) = self.err_rx.try_recv() {
            err.push(l);
        }
        (out, err)
    }

    /// Non-blocking exit check. Returns Some(code) once the child exited.
    pub fn try_exit(&mut self) -> Result<Option<i32>> {
        if let Some(child) = self.child.as_mut() {
            if let Some(status) = child.try_wait()? {
                return Ok(Some(status.code().unwrap_or(-1)));
            }
            return Ok(None);
        }
        Ok(Some(-1))
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        match self.try_exit() {
            Ok(None) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct RunResult {
    pub success: bool,
    pub output: String,
    pub errors: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

impl RunResult {
    pub fn formatted_output(&self) -> String {
        let mut result = String::new();
        
        if !self.output.is_empty() {
            result.push_str("📤 Output:\n");
            result.push_str(&self.output);
        }
        
        if !self.errors.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str("❌ Errors:\n");
            result.push_str(&self.errors);
        }
        
        result.push_str(&format!(
            "\n⏱️  Execution time: {}ms | Exit code: {}",
            self.duration_ms, self.exit_code
        ));
        
        result
    }
}
