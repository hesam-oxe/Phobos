// Phobos IDE - Compiler Module
// Multi-language compilation and build system

use std::process::Command;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub language: String,
    pub compiler_path: Option<PathBuf>,
    pub flags: Vec<String>,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerType {
    Rustc,      // rustc
    Gcc,        // gcc/g++
    Clang,      // clang/clang++
    Javac,      // javac
    Python,     // python3 (interpreter)
    Node,       // node (interpreter)
    Dotnet,     // dotnet
    Go,         // go
    Swift,      // swiftc
    Pascal,     // fpc
    Php,        // php
    Ruby,       // ruby
    Custom(String),
}

impl CompilerType {
    pub fn from_language(lang: &str) -> Self {
        match lang.to_lowercase().as_str() {
            "rust" => CompilerType::Rustc,
            "c" | "cpp" | "c++" => CompilerType::Gcc,
            "objc" | "objective-c" => CompilerType::Clang,
            "java" => CompilerType::Javac,
            "python" | "py" => CompilerType::Python,
            "javascript" | "js" | "typescript" | "ts" => CompilerType::Node,
            "csharp" | "cs" | "c#" => CompilerType::Dotnet,
            "go" => CompilerType::Go,
            "swift" => CompilerType::Swift,
            "pascal" | "delphi" => CompilerType::Pascal,
            "php" => CompilerType::Php,
            "ruby" => CompilerType::Ruby,
            _ => CompilerType::Custom(lang.to_string()),
        }
    }

    pub fn default_command(&self) -> &'static str {
        match self {
            CompilerType::Rustc => "rustc",
            CompilerType::Gcc => "g++",
            CompilerType::Clang => "clang++",
            CompilerType::Javac => "javac",
            CompilerType::Python => "python3",
            CompilerType::Node => "node",
            CompilerType::Dotnet => "dotnet",
            CompilerType::Go => "go",
            CompilerType::Swift => "swiftc",
            CompilerType::Pascal => "fpc",
            CompilerType::Php => "php",
            CompilerType::Ruby => "ruby",
            CompilerType::Custom(_) => "",
        }
    }
}

pub struct Compiler {
    config: CompilerConfig,
    compiler_type: CompilerType,
}

impl Compiler {
    pub fn new(language: &str) -> Self {
        let compiler_type = CompilerType::from_language(language);
        Self {
            config: CompilerConfig {
                language: language.to_string(),
                compiler_path: None,
                flags: Vec::new(),
                output_dir: PathBuf::from("./build"),
            },
            compiler_type,
        }
    }

    pub fn set_flags(&mut self, flags: Vec<String>) {
        self.config.flags = flags;
    }

    pub fn set_output_dir(&mut self, path: &str) {
        self.config.output_dir = PathBuf::from(path);
    }

    pub fn compile(&self, source_file: &Path) -> Result<CompilationResult> {
        let command = self.compiler_type.default_command();
        
        if command.is_empty() {
            anyhow::bail!("No compiler configured for language: {}", self.config.language);
        }

        // Ensure output directory exists
        std::fs::create_dir_all(&self.config.output_dir)?;

        // Build command arguments
        let mut args = Vec::new();
        
        match self.compiler_type {
            CompilerType::Rustc => {
                args.push(source_file.to_string_lossy().to_string());
                args.push("-o".to_string());
                let output = self.config.output_dir.join("output");
                args.push(output.to_string_lossy().to_string());
                args.extend(self.config.flags.clone());
            }
            CompilerType::Gcc | CompilerType::Clang => {
                args.push(source_file.to_string_lossy().to_string());
                args.push("-o".to_string());
                let ext = if self.compiler_type == CompilerType::Gcc { "out" } else { "out" };
                let output = self.config.output_dir.join(format!("output.{}", ext));
                args.push(output.to_string_lossy().to_string());
                args.extend(self.config.flags.clone());
            }
            CompilerType::Javac => {
                args.push("-d".to_string());
                args.push(self.config.output_dir.to_string_lossy().to_string());
                args.push(source_file.to_string_lossy().to_string());
            }
            CompilerType::Go => {
                args.push("build".to_string());
                args.push("-o".to_string());
                let output = self.config.output_dir.join("output");
                args.push(output.to_string_lossy().to_string());
                args.push(source_file.to_string_lossy().to_string());
            }
            CompilerType::Dotnet => {
                args.push("build".to_string());
                args.push(source_file.to_string_lossy().to_string());
                args.push("--output".to_string());
                args.push(self.config.output_dir.to_string_lossy().to_string());
            }
            // Interpreted languages don't need compilation
            CompilerType::Python | CompilerType::Node | CompilerType::Php | CompilerType::Ruby => {
                return Ok(CompilationResult {
                    success: true,
                    output: String::from("Interpreter language - no compilation needed"),
                    errors: String::new(),
                    executable_path: Some(source_file.to_path_buf()),
                    duration_ms: 0,
                });
            }
            _ => {
                args.push(source_file.to_string_lossy().to_string());
                args.extend(self.config.flags.clone());
            }
        }

        // Execute compilation (no deadlock: use output() which drains pipes concurrently)
        let start_time = std::time::Instant::now();

        // Cargo workspace awareness: prefer `cargo build` when manifest exists
        if self.config.language == "rust" {
            if let Some(manifest) = find_cargo_manifest(source_file) {
                let out = Command::new("cargo")
                    .arg("build")
                    .arg("--manifest-path").arg(&manifest)
                    .arg("--message-format").arg("short")
                    .current_dir(manifest.parent().unwrap_or(std::path::Path::new(".")))
                    .output();
                match out {
                    Ok(o) => {
                        let duration = start_time.elapsed();
                        let success = o.status.success();
                        return Ok(CompilationResult {
                            success,
                            output: String::from_utf8_lossy(&o.stdout).to_string(),
                            errors: String::from_utf8_lossy(&o.stderr).to_string(),
                            executable_path: if success {
                                Some(self.config.output_dir.join("output"))
                            } else { None },
                            duration_ms: duration.as_millis() as u64,
                        });
                    }
                    Err(_) => {} // fall through to rustc single-file
                }
            }
        }

        let output = Command::new(command)
            .args(&args)
            .output()
            .with_context(|| format!("Failed to spawn compiler: {}", command))?;

        let duration = start_time.elapsed();
        Ok(CompilationResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            errors: String::from_utf8_lossy(&output.stderr).to_string(),
            executable_path: if output.status.success() {
                Some(self.config.output_dir.join("output"))
            } else {
                None
            },
            duration_ms: duration.as_millis() as u64,
        })
    }

    pub fn get_compiler_info(&self) -> CompilerInfo {
        CompilerInfo {
            language: self.config.language.clone(),
            compiler_type: format!("{:?}", self.compiler_type),
            default_command: self.compiler_type.default_command().to_string(),
            flags: self.config.flags.clone(),
        }
    }
}

#[derive(Debug)]
pub struct CompilationResult {
    pub success: bool,
    pub output: String,
    pub errors: String,
    pub executable_path: Option<PathBuf>,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct CompilerInfo {
    pub language: String,
    pub compiler_type: String,
    pub default_command: String,
    pub flags: Vec<String>,
}

fn find_cargo_manifest(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent().map(|p| p.to_path_buf())
    } else {
        Some(start.to_path_buf())
    };
    while let Some(d) = dir {
        let m = d.join("Cargo.toml");
        if m.exists() {
            return Some(m);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}
