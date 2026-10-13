use crate::editor::code_editor::CodeEditor;
use eframe::egui;

pub struct StatusBar {
    pub show_search: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self { show_search: false }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, editor: &CodeEditor) {
        ui.horizontal(|ui| {
            // Left side — Git info
            ui.label("🌿 main");
            ui.separator();

            // Errors/Warnings
            ui.label("🔴 0");
            ui.label("🟡 0");
            ui.separator();

            // Language
            ui.label(format!("🌐 {}", editor.language()));
            ui.separator();

            // Line/Column (char-safe: cursor is a byte offset on a boundary)
            let cursor = editor.cursor_offset().min(editor.content().len());
            let head = &editor.content()[..cursor];
            let line = head.matches('\n').count() + 1;
            let col_chars = head
                .rfind('\n')
                .map(|i| head[i + 1..].chars().count())
                .unwrap_or_else(|| head.chars().count());
            ui.label(format!("Ln {}, Col {}", line, col_chars + 1));
            ui.separator();

            // Spaces/Tab
            ui.label("Spaces: 4");
            ui.separator();
            ui.label("UTF-8");
            ui.separator();

            // Encoding
            ui.label("LF");
            ui.separator();

            // Language specific
            ui.label(format!("🦀 Rust"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Feedback smiley
                if ui.button("😊").clicked() {}

                // Notification bell
                if ui.button("🔔").clicked() {}
            });
        });
    }
}
