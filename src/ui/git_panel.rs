// Phobos IDE - Git Panel
use eframe::egui;
use crate::services::git::{git_info, git_log};

pub struct GitPanel {
    pub visible: bool,
    pub path: String,
    branch_cache: Option<String>,
    branch_checked_at: Option<std::time::Instant>,
}

impl GitPanel {
    pub fn new() -> Self { Self { visible: false, path: ".".into(), branch_cache: None, branch_checked_at: None } }

    /// Current branch for the status bar (cached 5s, cheap git2 read).
    pub fn branch_for_display(&mut self) -> Option<String> {
        // cache for 5s: no git churn every frame
        let fresh = self
            .branch_checked_at
            .map(|t| t.elapsed().as_secs_f32() < 5.0)
            .unwrap_or(false);
        if fresh {
            return self.branch_cache.clone();
        }
        let info = crate::services::git::git_info(&self.path);
        self.branch_cache = if info.in_repo {
            let b = info.branch.trim();
            if b.is_empty() { None } else { Some(b.to_string()) }
        } else {
            None
        };
        self.branch_checked_at = Some(std::time::Instant::now());
        self.branch_cache.clone()
    }
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
