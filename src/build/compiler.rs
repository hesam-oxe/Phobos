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
        // The actual produced executable, tracked per toolchain
        let mut produced: Option<PathBuf> = None;
        
        match self.compiler_type {
            CompilerType::Rustc => {
                args.push(source_file.to_string_lossy().to_string());
                args.push("-o".to_string());
                let output = self.config.output_dir.join("output");
                args.push(output.to_string_lossy().to_string());
                args.extend(self.config.flags.clone());
                produced = Some(output);
            }
            CompilerType::Gcc | CompilerType::Clang => {
                args.push(source_file.to_string_lossy().to_string());
                args.push("-o".to_string());
                let output = self.config.output_dir.join("output.out");
                args.push(output.to_string_lossy().to_string());
                args.extend(self.config.flags.clone());
                produced = Some(output);
            }
            CompilerType::Javac => {
                args.push("-d".to_string());
                args.push(self.config.output_dir.to_string_lossy().to_string());
                args.push(source_file.to_string_lossy().to_string());
                // javac emits .class files; Runner invokes `java <Class>` instead
            }
            CompilerType::Go => {
                args.push("build".to_string());
                args.push("-o".to_string());
                let output = self.config.output_dir.join("output");
                args.push(output.to_string_lossy().to_string());
                args.push(source_file.to_string_lossy().to_string());
                produced = Some(output);
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
                        // locate target/debug/<crate-name> next to the manifest
                        let exe = if success {
                            let name = std::fs::read_to_string(&manifest)
                                .ok()
                                .and_then(|s| toml::from_str::<toml::Value>(&s).ok())
                                .and_then(|v| {
                                    v.get("package")
                                        .and_then(|p| p.get("name"))
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string())
                                });
                            name
                                .and_then(|n| {
                                    manifest.parent().map(|d| {
                                        d.join("target").join("debug").join(&n)
                                    })
                                })
                                .filter(|p| p.exists())
                                .map(|p| std::fs::canonicalize(&p).unwrap_or(p))
                        } else {
                            None
                        };
                        return Ok(CompilationResult {
                            success,
                            output: String::from_utf8_lossy(&o.stdout).to_string(),
                            errors: String::from_utf8_lossy(&o.stderr).to_string(),
                            executable_path: exe,
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
        // Always report an absolute executable path so the runner can spawn it
        // regardless of its own working directory.
        let exe_abs = produced
            .filter(|_| output.status.success())
            .map(|p| {
                std::fs::canonicalize(&p).unwrap_or_else(|_| {
                    std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(&p)
                })
            });
        Ok(CompilationResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            errors: String::from_utf8_lossy(&output.stderr).to_string(),
            executable_path: exe_abs,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn have(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("phobos-cc-{}-{}", tag, std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn rust_single_file_compile_then_run() {
        if !have("rustc") {
            eprintln!("skip: rustc not found");
            return;
        }
        let dir = temp_dir("rust");
        let src = dir.join("hello.rs");
        std::fs::write(&src, "fn main() { println!(\"rust-compile-ok\"); }\n").unwrap();
        let mut c = Compiler::new("rust");
        c.set_output_dir(dir.join("out").to_str().unwrap());
        let r = c.compile(&src).expect("compile");
        assert!(r.success, "compile errors: {}", r.errors);
        let exe = r.executable_path.expect("executable path");
        assert!(exe.exists(), "no executable at {:?}", exe);
        let out = Command::new(&exe).output().expect("run compiled exe");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("rust-compile-ok"), "stdout: {:?}", stdout);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn c_file_compiles_with_gcc() {
        if !have("g++") {
            eprintln!("skip: g++ not found");
            return;
        }
        let dir = temp_dir("c");
        let src = dir.join("hello.c");
        std::fs::write(&src, "#include <stdio.h>\nint main(void){ printf(\"c-compile-ok\\n\"); return 0; }\n").unwrap();
        let mut c = Compiler::new("c");
        c.set_output_dir(dir.join("out").to_str().unwrap());
        let r = c.compile(&src).expect("compile");
        assert!(r.success, "compile errors: {}", r.errors);
        let exe = r.executable_path.expect("executable path");
        assert!(exe.exists());
        let out = Command::new(&exe).output().expect("run");
        assert!(String::from_utf8_lossy(&out.stdout).contains("c-compile-ok"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_language_errors() {
        let c = Compiler::new("no-such-lang");
        let res = c.compile(Path::new("/tmp/nonexistent.q"));
        assert!(res.is_err(), "unknown language must not silently succeed");
    }

    #[test]
    fn compiler_type_mapping() {
        assert!(matches!(CompilerType::from_language("rust"), CompilerType::Rustc));
        assert!(matches!(CompilerType::from_language("Python"), CompilerType::Python));
        assert!(matches!(CompilerType::from_language("go"), CompilerType::Go));
        assert!(matches!(CompilerType::from_language("weird"), CompilerType::Custom(_)));
    }
}
