// Phobos IDE - Build Panel (compiler + runner output, streaming)
use eframe::egui;
use crate::build::compiler::{Compiler, CompilationResult};
use crate::build::runner::{Runner, RunResult};
use std::path::Path;

pub struct BuildPanel {
    pub visible: bool,
    pub log: Vec<String>,
    pub last_compile: Option<bool>,
}

impl BuildPanel {
    pub fn new() -> Self { Self { visible: false, log: vec!["🔨 Build ready".into()], last_compile: None } }
    pub fn push(&mut self, s: String) {
        self.log.push(s);
        if self.log.len() > 1000 { self.log.remove(0); }
    }
    pub fn build(&mut self, file: &str, lang: &str) {
        let c = Compiler::new(lang);
        self.push(format!("▶ build {} ({})", file, lang));
        match c.compile(Path::new(file)) {
            Ok(CompilationResult { success, output, errors, duration_ms, .. }) => {
                self.last_compile = Some(success);
                if !output.is_empty() { self.push(format!("OUT:\n{}", output)); }
                if !errors.is_empty() { self.push(format!("ERR:\n{}", errors)); }
                self.push(format!("{} in {}ms", if success {"✅ success"} else {"❌ failed"}, duration_ms));
            }
            Err(e) => { self.push(format!("❌ {}", e)); self.last_compile = Some(false); }
        }
    }
    pub fn run(&mut self, file: &str, lang: &str) {
        let mut r = Runner::new(lang);
        r.set_working_dir(".");
        self.push(format!("▶ run {} ({})", file, lang));
        match r.run(Path::new(file)) {
            Ok(RunResult { success, output, errors, exit_code, duration_ms }) => {
                if !output.is_empty() { self.push(output); }
                if !errors.is_empty() { self.push(format!("STDERR:\n{}", errors)); }
                self.push(format!("exit {} {} ({}ms)", exit_code, if success {"✅"} else {"❌"}, duration_ms));
            }
            Err(e) => self.push(format!("❌ {}", e)),
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🔨 Build");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
                if ui.button("🗑").clicked() { self.log.clear(); }
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
