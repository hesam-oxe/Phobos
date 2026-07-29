use eframe::egui;
use crate::editor::code_editor::CodeEditor;

pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&self, ui: &mut egui::Ui, editor: &CodeEditor) {
        ui.horizontal(|ui| {
            ui.label(format!("💀 Phobos | {} | Ln {}, Col {} | UTF-8 | {}",
                editor.language,
                editor.content[..editor.cursor_pos.min(editor.content.len())].lines().count(),
                editor.cursor_pos.saturating_sub(
                    editor.content[..editor.cursor_pos.min(editor.content.len())]
                        .rfind('\n').map(|i| i + 1).unwrap_or(0)
                ),
                if cfg!(target_os = "linux") { "🐧 Linux" }
                else if cfg!(target_os = "windows") { "🪟 Windows" }
                else { "🍎 macOS" }
            ));
        });
    }
}
