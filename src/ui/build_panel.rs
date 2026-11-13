// Phobos IDE - Build Panel (compiler + runner output, streaming)
use eframe::egui;
use crate::build::compiler::{Compiler, CompilationResult};
use crate::build::runner::{Runner, RunResult, AsyncRunner};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, Receiver};

enum JobMsg {
    Line { text: String, err: bool },
    CompileDone(CompilationResult),
    RunDone(RunResult),
    Failed(String),
}

enum JobCmd {
    Stop,
}

struct ActiveJob {
    desc: String,
    rx: Receiver<JobMsg>,
    cmd_tx: Sender<JobCmd>,
    handle: Option<std::thread::JoinHandle<()>>,
    finished: bool,
}

pub struct BuildPanel {
    pub visible: bool,
    pub log: Vec<String>,
    pub last_compile: Option<bool>,
    job: Option<ActiveJob>,
}

impl BuildPanel {
    pub fn new() -> Self { Self { visible: false, log: vec!["🔨 Build ready".into()], last_compile: None, job: None } }
    pub fn push(&mut self, s: String) {
        // cap each entry so one giant line can't blow up the UI
        let mut s = s;
        if s.len() > 4000 {
            s.truncate(4000);
            s.push_str("…");
        }
        self.log.push(s);
        if self.log.len() > 1000 { self.log.remove(0); }
    }
    fn spawn_job(&mut self, desc: String, lang: &str, file: &str, run_after: bool) {
        // stop previous job
        if let Some(j) = self.job.as_mut() {
            let _ = j.cmd_tx.send(JobCmd::Stop);
        }
        let (tx, rx) = channel::<JobMsg>();
        let (cmd_tx, cmd_rx) = channel::<JobCmd>();
        let lang_owned = lang.to_string();
        let file = PathBuf::from(file);
        let workdir = file.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
        let handle = std::thread::spawn(move || {
            build_thread(&lang_owned, &file, &workdir, run_after, &tx, &cmd_rx);
        });
        self.visible = true;
        self.push(format!("▶ {} ({})", desc, lang));
        self.job = Some(ActiveJob { desc, rx, cmd_tx, handle: Some(handle), finished: false });
    }
    pub fn build(&mut self, file: &str, lang: &str) {
        self.spawn_job(format!("build {}", file), lang, file, false);
    }
    pub fn run(&mut self, file: &str, lang: &str) {
        self.spawn_job(format!("run {}", file), lang, file, true);
    }
    pub fn stop(&mut self) {
        if let Some(j) = self.job.as_mut() {
            let _ = j.cmd_tx.send(JobCmd::Stop);
            self.push("⏹ stop requested".into());
        }
    }
    pub fn is_running(&self) -> bool {
        self.job.as_ref().map(|j| !j.finished).unwrap_or(false)
    }
    fn poll(&mut self) {
        let mut drain = Vec::new();
        if let Some(job) = self.job.as_mut() {
            while let Ok(msg) = job.rx.try_recv() {
                drain.push(msg);
            }
        }
        for msg in drain {
            match msg {
                JobMsg::Line { text, err } => {
                    if err {
                        self.push(format!("ERR: {}", text));
                    } else {
                        self.push(text);
                    }
                }
                JobMsg::CompileDone(r) => {
                    self.last_compile = Some(r.success);
                    for l in r.output.lines() {
                        self.push(format!("OUT: {}", l));
                    }
                    for l in r.errors.lines() {
                        self.push(format!("ERR: {}", l));
                    }
                    self.push(format!("{} in {}ms",
                        if r.success { "✅ success" } else { "❌ failed" }, r.duration_ms));
                    if let Some(j) = self.job.as_mut() {
                        j.finished = true;
                    }
                }
                JobMsg::RunDone(r) => {
                    self.push(format!("exit {} {} ({}ms)",
                        r.exit_code, if r.success { "✅" } else { "❌" }, r.duration_ms));
                    if let Some(j) = self.job.as_mut() {
                        j.finished = true;
                    }
                }
                JobMsg::Failed(e) => {
                    self.push(format!("❌ {}", e));
                    self.last_compile = Some(false);
                    if let Some(j) = self.job.as_mut() {
                        j.finished = true;
                    }
                }
            }
        }
        // reap finished thread handle
        if let Some(job) = self.job.as_mut() {
            if job.finished && job.handle.is_some() {
                // non-blocking: only take if actually done to avoid blocking UI
                // (JoinHandle has no try_join; the worker exits right after last send,
                // so a brief yield here is safe and rare)
                let _ = job.handle.take().map(|h| { let _ = h.join(); });
            }
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.poll();
        ui.horizontal(|ui| {
            ui.heading("🔨 Build");
            if self.is_running() {
                ui.label(egui::RichText::new("● running").color(egui::Color32::GREEN));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
                if ui.button("🗑").clicked() { self.log.clear(); }
                if self.is_running() && ui.button("⏹ Stop").clicked() { self.stop(); }
            });
        });
        ui.separator();
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            for l in &self.log {
                ui.label(egui::RichText::new(l).monospace().size(12.0));
            }
        });
    }
}

fn stopped(cmd_rx: &Receiver<JobCmd>) -> bool {
    matches!(cmd_rx.try_recv(), Ok(JobCmd::Stop))
}

fn stream_runner(runner: &mut AsyncRunner, tx: &Sender<JobMsg>, cmd_rx: &Receiver<JobCmd>) -> Option<i32> {
    let start = std::time::Instant::now();
    loop {
        if stopped(cmd_rx) {
            let _ = runner.stop();
            let _ = tx.send(JobMsg::Line { text: "⏹ stopped".into(), err: false });
            return None;
        }
        let (out, err) = runner.drain();
        for l in out {
            let _ = tx.send(JobMsg::Line { text: l, err: false });
        }
        for l in err {
            let _ = tx.send(JobMsg::Line { text: l, err: true });
        }
        match runner.try_exit() {
            Ok(Some(code)) => {
                let (out, err) = runner.drain();
                for l in out {
                    let _ = tx.send(JobMsg::Line { text: l, err: false });
                }
                for l in err {
                    let _ = tx.send(JobMsg::Line { text: l, err: true });
                }
                let _ = start; // duration measured by caller via RunResult
                return Some(code);
            }
            Ok(None) => {}
            Err(e) => {
                let _ = tx.send(JobMsg::Failed(format!("run: {}", e)));
                return None;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}

fn build_thread(
    lang: &str,
    file: &Path,
    workdir: &Path,
    run_after: bool,
    tx: &Sender<JobMsg>,
    cmd_rx: &Receiver<JobCmd>,
) {
    let interpreted = matches!(
        lang.to_lowercase().as_str(),
        "python" | "py" | "javascript" | "js" | "typescript" | "ts" | "php" | "ruby"
    );
    if interpreted {
        let mut r = Runner::new(lang);
        r.set_working_dir(&workdir.to_string_lossy());
        let mut runner = match r.run_async(file) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(JobMsg::Failed(e.to_string()));
                return;
            }
        };
        let start = std::time::Instant::now();
        let code = stream_runner(&mut runner, tx, cmd_rx);
        if let Some(code) = code {
            let _ = tx.send(JobMsg::RunDone(RunResult {
                success: code == 0,
                output: String::new(),
                errors: String::new(),
                exit_code: code,
                duration_ms: start.elapsed().as_millis() as u64,
            }));
        }
        return;
    }
    // compiled: build first (blocking, in worker thread so UI stays live)
    let mut c = Compiler::new(lang);
    // output goes next to the source file, not the IDE process cwd
    if let Some(parent) = file.parent() {
        c.set_output_dir(parent.join("build").to_string_lossy().as_ref());
    }
    let start = std::time::Instant::now();
    let result = match c.compile(file) {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(JobMsg::Failed(e.to_string()));
            return;
        }
    };
    let exe = result.executable_path.clone().unwrap_or_else(|| file.to_path_buf());
    let compile_ok = result.success;
    let _ = tx.send(JobMsg::CompileDone(result));
    if !run_after || !compile_ok {
        return;
    }
    if stopped(cmd_rx) {
        return;
    }
    let mut r = Runner::new(lang);
    r.set_working_dir(&workdir.to_string_lossy());
    let mut runner = match r.run_async(&exe) {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(JobMsg::Failed(e.to_string()));
            return;
        }
    };
    let run_start = std::time::Instant::now();
    let _ = start;
    let code = stream_runner(&mut runner, tx, cmd_rx);
    if let Some(code) = code {
        let _ = tx.send(JobMsg::RunDone(RunResult {
            success: code == 0,
            output: String::new(),
            errors: String::new(),
            exit_code: code,
            duration_ms: run_start.elapsed().as_millis() as u64,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn have(cmd: &str) -> bool {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// End-to-end: exactly what a user gets when they hit Run on an interpreted file.
    #[test]
    fn python_run_job_streams_output_and_finishes() {
        if !have("python3") {
            eprintln!("skip: python3 not found");
            return;
        }
        let dir = std::env::temp_dir().join(format!("phobos-job-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("job.py");
        std::fs::write(&file, "print('job-output-ok')\n").unwrap();

        let (tx, rx) = channel::<JobMsg>();
        let (_cmd_tx, cmd_rx) = channel::<JobCmd>();
        let d2 = dir.clone();
        let handle = std::thread::spawn(move || {
            build_thread("python", &file, &d2, true, &tx, &cmd_rx);
        });

        let mut saw_output = false;
        let mut saw_success_done = false;
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    JobMsg::Line { text, err: false } => {
                        if text.contains("job-output-ok") {
                            saw_output = true;
                        }
                    }
                    JobMsg::RunDone(r) => saw_success_done = r.success,
                    JobMsg::Failed(e) => panic!("job failed: {}", e),
                    _ => {}
                }
            }
            if saw_output && saw_success_done {
                break;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        let _ = handle.join();
        assert!(saw_output, "did not see program output in stream");
        assert!(saw_success_done, "RunDone did not report success");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Compiled-language job: rust single file → compile → run → output.
    #[test]
    fn rust_build_then_run_job_produces_output() {
        if !have("rustc") {
            eprintln!("skip: rustc not found");
            return;
        }
        let dir = std::env::temp_dir().join(format!("phobos-job-rs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("job.rs");
        std::fs::write(&file, "fn main() { println!(\"rust-job-ok\"); }\n").unwrap();

        let (tx, rx) = channel::<JobMsg>();
        let (_cmd_tx, cmd_rx) = channel::<JobCmd>();
        let d2 = dir.clone();
        let handle = std::thread::spawn(move || {
            build_thread("rust", &file, &d2, true, &tx, &cmd_rx);
        });

        let mut saw_compile_ok = false;
        let mut saw_output = false;
        let mut saw_success_done = false;
        let deadline = Instant::now() + Duration::from_secs(90);
        while Instant::now() < deadline {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    JobMsg::CompileDone(r) => saw_compile_ok = r.success,
                    JobMsg::Line { text, err: false } => {
                        if text.contains("rust-job-ok") {
                            saw_output = true;
                        }
                    }
                    JobMsg::Line { .. } => {}
                    JobMsg::RunDone(r) => saw_success_done = r.success,
                    JobMsg::Failed(e) => panic!("job failed: {}", e),
                }
            }
            if saw_compile_ok && saw_output && saw_success_done {
                break;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        let _ = handle.join();
        assert!(saw_compile_ok, "compile did not succeed");
        assert!(saw_output, "did not see program output in stream");
        assert!(saw_success_done, "RunDone did not report success");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Stop signal terminates a long-running job.
    #[test]
    fn stop_terminates_running_job() {
        if !have("python3") {
            eprintln!("skip: python3 not found");
            return;
        }
        let dir = std::env::temp_dir().join(format!("phobos-stop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("loop.py");
        std::fs::write(&file, "import time\nwhile True: time.sleep(0.1)\n").unwrap();

        let (tx, rx) = channel::<JobMsg>();
        let (cmd_tx, cmd_rx) = channel::<JobCmd>();
        let d2 = dir.clone();
        let handle = std::thread::spawn(move || {
            build_thread("python", &file, &d2, true, &tx, &cmd_rx);
        });

        std::thread::sleep(Duration::from_millis(400));
        let _ = cmd_tx.send(JobCmd::Stop);
        // thread must finish promptly after stop
        let (send, recv) = channel();
        std::thread::spawn(move || {
            let _ = handle.join();
            let _ = send.send(());
        });
        assert!(recv.recv_timeout(Duration::from_secs(5)).is_ok(), "job thread hung after Stop");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
