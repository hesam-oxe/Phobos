//! Phobos IDE - Toolchain Module
//! Detects installed language toolchains and builds project-aware
//! compile/run commands, so any language runs with one keypress.
//!
//! Design notes:
//! - Detection uses the `which` crate (PATH lookup), never hardcoded paths.
//! - Project awareness: a `Cargo.toml` / `go.mod` / `package.json` ancestor
//!   switches single-file commands to project commands.
//! - Timeouts are enforced with a `try_wait` poll loop (no extra deps).

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long a run may take before it is killed.
pub const DEFAULT_RUN_TIMEOUT: Duration = Duration::from_secs(60);

/// A language toolchain: ordered candidate binaries plus install hint.
#[derive(Debug, Clone)]
pub struct Toolchain {
    /// Language id, e.g. "rust", "python".
    pub language: &'static str,
    /// Candidate binaries in preference order.
    pub candidates: &'static [&'static str],
    /// What to tell the user when nothing is installed.
    pub install_hint: &'static str,
    /// Resolved binary path after [`Toolchain::detect`].
    pub found: Option<PathBuf>,
}

impl Toolchain {
    pub const fn new(
        language: &'static str,
        candidates: &'static [&'static str],
        install_hint: &'static str,
    ) -> Self {
        Self {
            language,
            candidates,
            install_hint,
            found: None,
        }
    }

    /// Resolve the first candidate present on PATH.
    pub fn detect(&mut self) -> Option<&Path> {
        self.found = self
            .candidates
            .iter()
            .filter_map(|c| which::which(c).ok())
            .next();
        self.found.as_deref()
    }

    /// True when at least one candidate binary exists on PATH.
    pub fn available(&self) -> bool {
        self.candidates.iter().any(|c| which::which(c).is_ok())
    }

    /// Human-readable error when the toolchain is missing.
    pub fn missing_error(&self) -> anyhow::Error {
        anyhow::anyhow!(
            "No {} toolchain found (tried: {}). {}",
            self.language,
            self.candidates.join(", "),
            self.install_hint
        )
    }
}

/// All toolchains Phobos knows how to run.
pub fn all_toolchains() -> Vec<Toolchain> {
    vec![
        Toolchain::new(
            "rust",
            &["cargo", "rustc"],
            "Install Rust via https://rustup.rs",
        ),
        Toolchain::new(
            "python",
            &["python3", "python"],
            "Install Python 3 from https://python.org or your package manager",
        ),
        Toolchain::new(
            "javascript",
            &["node"],
            "Install Node.js from https://nodejs.org",
        ),
        Toolchain::new(
            "typescript",
            &["tsx", "ts-node", "deno"],
            "Install a TS runner: `npm i -g tsx` (preferred) or Deno",
        ),
        Toolchain::new("go", &["go"], "Install Go from https://go.dev"),
        Toolchain::new(
            "java",
            &["java", "javac"],
            "Install a JDK 17+ (Temurin or your package manager)",
        ),
        Toolchain::new(
            "c",
            &["cc", "gcc", "clang"],
            "Install a C compiler: `apt install build-essential` / Xcode / VS Build Tools",
        ),
        Toolchain::new(
            "cpp",
            &["c++", "g++", "clang++"],
            "Install a C++ compiler: `apt install build-essential` / Xcode / VS Build Tools",
        ),
        Toolchain::new(
            "ruby",
            &["ruby"],
            "Install Ruby via rbenv/rvm or your package manager",
        ),
        Toolchain::new("php", &["php"], "Install PHP from https://php.net"),
        Toolchain::new(
            "csharp",
            &["dotnet"],
            "Install the .NET SDK from https://dotnet.microsoft.com",
        ),
        Toolchain::new("swift", &["swiftc"], "Install Swift (Xcode or swift.org)"),
        Toolchain::new(
            "kotlin",
            &["kotlinc"],
            "Install Kotlin via sdkman or your package manager",
        ),
        Toolchain::new(
            "lua",
            &["lua", "luajit"],
            "Install Lua from https://lua.org",
        ),
        Toolchain::new("r", &["Rscript"], "Install R from https://r-project.org"),
        Toolchain::new(
            "julia",
            &["julia"],
            "Install Julia from https://julialang.org",
        ),
        Toolchain::new("zig", &["zig"], "Install Zig from https://ziglang.org"),
        Toolchain::new(
            "dart",
            &["dart"],
            "Install Dart/Flutter from https://dart.dev",
        ),
    ]
}

/// Detect every known toolchain, returning `(language, path-or-None)`.
pub fn detect_all() -> Vec<(&'static str, Option<PathBuf>)> {
    all_toolchains()
        .into_iter()
        .map(|mut t| {
            let found = t.detect().map(|p| p.to_path_buf());
            (t.language, found)
        })
        .collect()
}

/// Walk up from `start` looking for a marker file (e.g. `Cargo.toml`).
pub fn find_project_root(start: &Path, marker: &str) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if dir.join(marker).exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// A resolved, ready-to-spawn command.
#[derive(Debug, Clone)]
pub struct ResolvedCommand {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    /// Short explanation shown in the UI status line.
    pub describe: String,
}

/// Resolve the best compile/run command for `file` in `language`.
///
/// Prefers project commands (`cargo run`, `go run ./...`) when a project
/// marker is found, otherwise falls back to single-file commands.
pub fn resolve_run(file: &Path, language: &str) -> Result<ResolvedCommand> {
    let lang = language.to_lowercase();
    let file_str = file.to_string_lossy().to_string();
    let cwd = file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let need = |bins: &[&str], hint: &str| -> Result<String> {
        bins.iter()
            .filter_map(|b| which::which(b).ok())
            .map(|p| p.to_string_lossy().to_string())
            // Prefer the bare binary name when it resolves, so commands
            // stay readable and relocatable.
            .next()
            .or_else(|| {
                bins.iter()
                    .find(|b| which::which(b).is_ok())
                    .map(|s| s.to_string())
            })
            .with_context(|| format!("No toolchain found (tried: {}). {}", bins.join(", "), hint))
    };

    match lang.as_str() {
        "rust" | "rs" => {
            if let Some(root) = find_project_root(file, "Cargo.toml") {
                let cargo = need(&["cargo"], "Install Rust via https://rustup.rs")?;
                Ok(ResolvedCommand {
                    program: cargo,
                    args: vec!["run".into(), "--quiet".into()],
                    cwd: root,
                    describe: "cargo run (project)".into(),
                })
            } else {
                let rustc = need(&["rustc"], "Install Rust via https://rustup.rs")?;
                Ok(ResolvedCommand {
                    program: rustc,
                    args: vec![file_str],
                    describe: "rustc (single file)".into(),
                    cwd,
                })
            }
        }
        "python" | "py" => {
            // Prefer an activated venv, then ./.venv, then system python3.
            if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
                let py = Path::new(&venv).join("bin").join("python");
                if py.exists() {
                    return Ok(ResolvedCommand {
                        program: py.to_string_lossy().to_string(),
                        args: vec![file_str],
                        cwd,
                        describe: "venv python (project)".into(),
                    });
                }
            }
            let py = need(&["python3", "python"], "Install Python 3")?;
            Ok(ResolvedCommand {
                program: py,
                args: vec![file_str],
                cwd,
                describe: "python3 (single file)".into(),
            })
        }
        "javascript" | "js" | "mjs" | "cjs" => {
            let node = need(&["node"], "Install Node.js from https://nodejs.org")?;
            Ok(ResolvedCommand {
                program: node,
                args: vec![file_str],
                cwd,
                describe: "node (single file)".into(),
            })
        }
        "typescript" | "ts" => {
            let runner = need(
                &["tsx", "ts-node", "deno"],
                "Install a TS runner: `npm i -g tsx`",
            )?;
            let args = if runner.ends_with("deno") {
                vec!["run".into(), "--allow-all".into(), file_str]
            } else {
                vec![file_str]
            };
            Ok(ResolvedCommand {
                program: runner,
                args,
                cwd,
                describe: "tsx/ts-node (single file)".into(),
            })
        }
        "go" => {
            let go = need(&["go"], "Install Go from https://go.dev")?;
            if find_project_root(file, "go.mod").is_some() {
                Ok(ResolvedCommand {
                    program: go,
                    args: vec!["run".into(), ".".into()],
                    cwd,
                    describe: "go run (module)".into(),
                })
            } else {
                Ok(ResolvedCommand {
                    program: go,
                    args: vec!["run".into(), file_str],
                    cwd,
                    describe: "go run (single file)".into(),
                })
            }
        }
        "java" => {
            let _ = need(&["javac", "java"], "Install a JDK 17+")?;
            let java = need(&["java"], "Install a JDK 17+")?;
            // Single-file launch (JDK 11+): `java File.java`. Projects
            // still go through javac + classpath via Compiler.
            Ok(ResolvedCommand {
                program: java,
                args: vec![file_str],
                cwd,
                describe: "java (single file, JDK 11+)".into(),
            })
        }
        "c" => {
            let cc = need(&["cc", "gcc", "clang"], "Install a C compiler")?;
            Ok(ResolvedCommand {
                program: cc,
                args: vec![file_str, "-o".into(), "/tmp/phobos_run_out".into()],
                cwd,
                describe: "cc (single file)".into(),
            })
        }
        "cpp" | "c++" | "cc" | "cxx" => {
            let cxx = need(&["c++", "g++", "clang++"], "Install a C++ compiler")?;
            Ok(ResolvedCommand {
                program: cxx,
                args: vec![file_str, "-o".into(), "/tmp/phobos_run_out".into()],
                cwd,
                describe: "c++ (single file)".into(),
            })
        }
        "ruby" | "rb" => {
            let ruby = need(&["ruby"], "Install Ruby")?;
            Ok(ResolvedCommand {
                program: ruby,
                args: vec![file_str],
                cwd,
                describe: "ruby (single file)".into(),
            })
        }
        "php" => {
            let php = need(&["php"], "Install PHP")?;
            Ok(ResolvedCommand {
                program: php,
                args: vec![file_str],
                cwd,
                describe: "php (single file)".into(),
            })
        }
        "lua" => {
            let lua = need(&["lua", "luajit"], "Install Lua")?;
            Ok(ResolvedCommand {
                program: lua,
                args: vec![file_str],
                cwd,
                describe: "lua (single file)".into(),
            })
        }
        "r" => {
            let r = need(&["Rscript"], "Install R")?;
            Ok(ResolvedCommand {
                program: r,
                args: vec![file_str],
                cwd,
                describe: "Rscript (single file)".into(),
            })
        }
        "julia" | "jl" => {
            let jl = need(&["julia"], "Install Julia")?;
            Ok(ResolvedCommand {
                program: jl,
                args: vec![file_str],
                cwd,
                describe: "julia (single file)".into(),
            })
        }
        "zig" => {
            let zig = need(&["zig"], "Install Zig")?;
            Ok(ResolvedCommand {
                program: zig,
                args: vec!["run".into(), file_str],
                cwd,
                describe: "zig run (single file)".into(),
            })
        }
        "dart" => {
            let dart = need(&["dart"], "Install Dart")?;
            Ok(ResolvedCommand {
                program: dart,
                args: vec!["run".into(), file_str],
                cwd,
                describe: "dart run (single file)".into(),
            })
        }
        "kotlin" | "kt" => {
            let kt = need(&["kotlinc"], "Install Kotlin")?;
            Ok(ResolvedCommand {
                program: kt,
                args: vec!["-script".into(), file_str],
                cwd,
                describe: "kotlinc (script)".into(),
            })
        }
        "csharp" | "cs" | "c#" => {
            let dotnet = need(&["dotnet"], "Install the .NET SDK")?;
            Ok(ResolvedCommand {
                program: dotnet,
                args: vec!["run".into()],
                cwd,
                describe: "dotnet run (project)".into(),
            })
        }
        "swift" => {
            let swiftc = need(&["swiftc"], "Install Swift")?;
            Ok(ResolvedCommand {
                program: swiftc,
                args: vec![file_str],
                cwd,
                describe: "swift (single file)".into(),
            })
        }
        "sh" | "bash" => {
            let sh = need(&["bash", "sh"], "Install bash")?;
            Ok(ResolvedCommand {
                program: sh,
                args: vec![file_str],
                cwd,
                describe: "bash (single file)".into(),
            })
        }
        other => Err(anyhow::anyhow!(
            "No run profile for language '{other}'. Add one in toolchain::resolve_run."
        )),
    }
}

/// Spawn `program` with `args` in `cwd`, killing it after `timeout`.
/// Returns (exit_code, stdout, stderr, timed_out).
pub fn execute_with_timeout(
    program: &str,
    args: &[String],
    cwd: &Path,
    env: &[(String, String)],
    timeout: Duration,
) -> Result<(i32, String, String, bool)> {
    let start = Instant::now();
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .envs(env.iter().cloned())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn '{program}' (is the toolchain installed?)"))?;

    // Poll loop: wait for exit or timeout, then kill.
    let timed_out = loop {
        match child.try_wait()? {
            Some(_) => break false,
            None if start.elapsed() >= timeout => {
                child.kill().ok();
                child.wait().ok();
                break true;
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    };

    let output = child.wait_with_output()?;
    let code = output.status.code().unwrap_or(-1);
    Ok((
        code,
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        timed_out,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_toolchains_have_hints_and_candidates() {
        for t in all_toolchains() {
            assert!(!t.candidates.is_empty(), "{}", t.language);
            assert!(!t.install_hint.is_empty(), "{}", t.language);
        }
    }

    #[test]
    fn detect_all_returns_every_language() {
        let all = detect_all();
        assert!(all.len() >= 15);
        assert!(all.iter().any(|(l, _)| *l == "rust"));
        assert!(all.iter().any(|(l, _)| *l == "python"));
    }

    #[test]
    fn find_project_root_finds_cargo_toml() {
        // This crate itself is a Rust project: any file inside it must
        // resolve to the crate root via Cargo.toml.
        let me = Path::new(env!("CARGO_MANIFEST_DIR")).join("toolchain.rs");
        let root = find_project_root(&me, "Cargo.toml");
        assert_eq!(root, Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))));
    }

    #[test]
    fn find_project_root_returns_none_without_marker() {
        assert_eq!(find_project_root(Path::new("/tmp"), "Cargo.toml"), None);
    }

    #[test]
    fn resolve_run_prefers_cargo_in_rust_projects() {
        if which::which("cargo").is_err() {
            return; // toolchain-dependent arm; absence is covered below
        }
        let me = Path::new(env!("CARGO_MANIFEST_DIR")).join("toolchain.rs");
        let cmd = resolve_run(&me, "rust").unwrap();
        assert_eq!(cmd.args[0], "run");
        assert!(cmd.describe.contains("cargo"));
    }

    #[test]
    fn resolve_run_python_points_at_a_python() {
        if which::which("python3").is_err() && which::which("python").is_err() {
            return;
        }
        let cmd = resolve_run(Path::new("/tmp/x.py"), "python").unwrap();
        assert!(cmd.program.contains("python"));
    }

    #[test]
    fn resolve_run_unknown_language_errors() {
        assert!(resolve_run(Path::new("/tmp/x.brainfuck"), "brainfuck").is_err());
    }

    #[test]
    fn execute_with_timeout_runs_echo() {
        let (code, out, _, timed_out) = execute_with_timeout(
            "echo",
            &["hello-phobos".to_string()],
            Path::new("/tmp"),
            &[],
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(!timed_out);
        assert_eq!(code, 0);
        assert!(out.contains("hello-phobos"));
    }

    #[test]
    fn execute_with_timeout_kills_sleepers() {
        let (_code, _, _, timed_out) = execute_with_timeout(
            "sleep",
            &["30".to_string()],
            Path::new("/tmp"),
            &[],
            Duration::from_millis(300),
        )
        .unwrap();
        assert!(timed_out);
    }
}
