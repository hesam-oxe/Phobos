// Phobos IDE - Git Panel
use eframe::egui;
use crate::services::git::{git_info, git_log};

pub struct GitPanel {
    pub visible: bool,
    pub path: String,
}

impl GitPanel {
    pub fn new() -> Self { Self { visible: false, path: ".".into() } }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🌿 Git");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
            });
        });
        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut self.path);
        });
        ui.separator();
        let info = git_info(&self.path);
        ui.label(format!("Branch: 🌿 {}", info.branch));
        ui.label(format!("Staged: {}  Unstaged: {}  Untracked: {}", info.staged, info.unstaged, info.untracked));
        ui.separator();
        ui.heading("Recent commits");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for c in git_log(&self.path, 50) {
                ui.label(egui::RichText::new(c).monospace().size(12.0));
            }
        });
    }
}
