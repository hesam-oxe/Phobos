// Phobos IDE - Debug Panel (breakpoints, stack, vars)
use eframe::egui;
use crate::build::debugger::{Debugger, DebugState};

pub struct DebugPanel {
    pub visible: bool,
    pub debugger: Debugger,
    pub target: String,
    pub bp_file: String,
    pub bp_line: String,
}

impl DebugPanel {
    pub fn new() -> Self {
        Self { visible: false, debugger: Debugger::new("rust"),
            target: String::new(), bp_file: String::new(), bp_line: String::new() }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🐞 Debug");
            ui.label(format!("{:?}", self.debugger.get_state()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
            });
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Target:");
            ui.text_edit_singleline(&mut self.target);
            if ui.button("▶ Start").clicked() {
                if !self.target.is_empty() {
                    let _ = self.debugger.start(std::path::Path::new(&self.target));
                }
            }
            if ui.button("⏹ Stop").clicked() { let _ = self.debugger.stop(); }
        });
        ui.horizontal(|ui| {
            if ui.button("▶ Continue").clicked() { let _ = self.debugger.continue_execution(); }
            if ui.button("⏭ Over").clicked() { let _ = self.debugger.step_over(); }
            if ui.button("⬇ Into").clicked() { let _ = self.debugger.step_into(); }
            if ui.button("⬆ Out").clicked() { let _ = self.debugger.step_out(); }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("BP file:"); ui.text_edit_singleline(&mut self.bp_file);
            ui.label("line:"); ui.text_edit_singleline(&mut self.bp_line);
            if ui.button("＋ Add").clicked() {
                if let Ok(n) = self.bp_line.parse::<usize>() {
                    self.debugger.add_breakpoint(&self.bp_file.clone(), n, None);
                }
            }
        });
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Breakpoints");
            for bp in self.debugger.get_breakpoints() {
                ui.label(format!("{}:{} {} {}", bp.file, bp.line,
                    if bp.enabled {"🟢"} else {"⚪"},
                    bp.condition.clone().unwrap_or_default()));
            }
            ui.separator();
            ui.heading("Output");
            ui.label(egui::RichText::new(self.debugger.get_output()).monospace().size(11.0));
            if self.debugger.get_state() != &DebugState::NotStarted {
                if ui.button("Refresh stack/vars").clicked() {
                    let _ = self.debugger.get_stack_trace();
                    let _ = self.debugger.get_variables();
                }
            }
        });
    }
}
