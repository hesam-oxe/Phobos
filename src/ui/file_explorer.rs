use eframe::egui;

pub struct FileExplorer {
    current_dir: String,
    files: Vec<String>,
}

impl FileExplorer {
    pub fn new() -> Self {
        Self {
            current_dir: String::from("."),
            files: vec!["main.rs".into(), "app.rs".into(), "Cargo.toml".into()],
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("📁 Explorer");
        ui.separator();
        for file in &self.files {
            if ui.selectable_label(false, format!("📄 {}", file)).clicked() {
                // Open file
            }
        }
    }
}
