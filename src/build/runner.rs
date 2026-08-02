// Phobos IDE - Runner Module
// Code execution and runtime management

use std::process::{Command, Stdio, Child};
use std::path::{Path, PathBuf};
use std::io::{Read, Write, BufRead, BufReader};
use std::sync::mpsc::{channel, Sender, Receiver};
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

        let mut cmd = Command::new(interpreter);
        
        if !args.is_empty() {
            cmd.args(&args);
        }

        cmd.current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped());

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let start_time = std::time::Instant::now();
        
        let mut child = cmd.spawn()
            .with_context(|| format!("Failed to spawn process: {}", interpreter))?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let Some(mut out) = child.stdout.take() {
            out.read_to_string(&mut stdout)?;
        }
        if let Some(mut err) = child.stderr.take() {
            err.read_to_string(&mut stderr)?;
        }

        let status = child.wait()?;
        let duration = start_time.elapsed();

        Ok(RunResult {
            success: status.success(),
            output: stdout,
            errors: stderr,
            exit_code: status.code().unwrap_or(-1),
            duration_ms: duration.as_millis() as u64,
        })
    }

    pub fn run_async(&self, file_or_executable: &Path) -> Result<AsyncRunner> {
        let (interpreter, args) = self.get_interpreter_and_args(file_or_executable);
        
        if interpreter.is_empty() {
            anyhow::bail!("No runner configured for language: {}", self.language);
        }

        let mut cmd = Command::new(interpreter);
        
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

pub struct AsyncRunner {
    child: Child,
    stdout_sender: Sender<String>,
    stderr_sender: Sender<String>,
    is_running: bool,
}

impl AsyncRunner {
    fn new(child: Child) -> Self {
        let (stdout_sender, _) = channel::<String>();
        let (stderr_sender, _) = channel::<String>();
        
        Self {
            child,
            stdout_sender,
            stderr_sender,
            is_running: true,
        }
    }

    pub fn write_input(&mut self, input: &str) -> Result<()> {
        if let Some(mut stdin) = self.child.stdin.take() {
            stdin.write_all(input.as_bytes())?;
        }
        Ok(())
    }

    pub fn read_output(&mut self) -> Result<String> {
        let mut output = String::new();
        if let Some(stdout) = self.child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            reader.read_line(&mut output)?;
        }
        Ok(output)
    }

    pub fn stop(&mut self) -> Result<()> {
        self.child.kill()?;
        self.is_running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
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
