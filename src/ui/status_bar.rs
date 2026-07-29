use eframe::egui;
use crate::editor::code_editor::CodeEditor;

pub struct StatusBar {
    pub show_search: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            show_search: false,
        }
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
            ui.label(format!("🌐 {}", editor.language));
            ui.separator();
            
            // Line/Column
            let line = editor.content[..editor.cursor_pos.min(editor.content.len())]
                .matches('\n').count() + 1;
            let col = editor.cursor_pos.saturating_sub(
                editor.content[..editor.cursor_pos.min(editor.content.len())]
                    .rfind('\n').map(|i| i + 1).unwrap_or(0)
            );
            ui.label(format!("Ln {}, Col {}", line, col + 1));
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
