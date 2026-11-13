use eframe::egui;
use crate::editor::code_editor::CodeEditor;
use crate::ui::theme::{Theme, chip};

pub struct StatusBar {
    pub show_search: bool,
    /// branch name fed from the app (git services)
    pub branch: Option<String>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            show_search: false,
            branch: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, editor: &CodeEditor, lsp_on: bool) {
        let (errors, warnings) = editor.diagnostic_counts();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            // Git branch chip
            match &self.branch {
                Some(b) => { chip(ui, "🌿", b, theme.success); }
                None => { chip(ui, "🌿", "no repo", theme.text_secondary); }
            }

            // Diagnostics — colored, only shown when nonzero
            if errors > 0 {
                chip(ui, "⬤", &format!("{}", errors), theme.error);
            }
            if warnings > 0 {
                chip(ui, "◆", &format!("{}", warnings), theme.warning);
            }
            if errors == 0 && warnings == 0 {
                chip(ui, "✓", "clean", theme.success);
            }

            // LSP status
            if lsp_on {
                chip(ui, "●", "LSP live", theme.success);
            } else {
                chip(ui, "○", "LSP off", theme.text_secondary);
            }

            // Language badge
            chip(ui, "◈", &editor.language, theme.accent2);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                chip(ui, "🔥", "v1.0 GOD", theme.accent);
                chip(ui, "⌨", "UTF-8 · LF · Spaces:4", theme.text_secondary);

                // Line/Column
                let upto = editor.cursor_pos.min(editor.content.len());
                let line = editor.content[..upto].matches('\n').count() + 1;
                let col = upto - editor.content[..upto].rfind('\n').map(|i| i + 1).unwrap_or(0);
                chip(ui, "⌖", &format!("Ln {}, Col {}", line, col + 1), theme.accent);
            });
        });
    }
}
