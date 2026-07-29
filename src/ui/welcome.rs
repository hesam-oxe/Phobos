use eframe::egui;
use egui::{Color32, RichText, Vec2};

pub struct WelcomePage {
    pub visible: bool,
    pub recent_files: Vec<String>,
}

impl WelcomePage {
    pub fn new() -> Self {
        Self {
            visible: true,
            recent_files: Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<String> {
        if !self.visible {
            return None;
        }

        let mut opened_file = None;

        ui.vertical_centered(|ui| {
            ui.add_space(80.0);

            // Logo
            ui.label(
                RichText::new("💀")
                    .size(64.0)
            );
            ui.add_space(10.0);

            ui.label(
                RichText::new("Phobos IDE")
                    .size(32.0)
                    .strong()
                    .color(Color32::from_rgb(206, 145, 120))
            );
            ui.label(
                RichText::new("The God of Fear — Successor to Delphi")
                    .size(14.0)
                    .color(Color32::from_rgb(133, 133, 133))
            );
            ui.add_space(40.0);

            // Quick actions
            ui.label(RichText::new("Start").size(12.0).strong());
            ui.add_space(10.0);

            let button_width = 300.0;
            let button_height = 40.0;

            if ui.add_sized(
                [button_width, button_height],
                egui::Button::new(RichText::new("📄 New File").size(14.0))
            ).clicked() {
                self.visible = false;
            }

            ui.add_space(6.0);

            if ui.add_sized(
                [button_width, button_height],
                egui::Button::new(RichText::new("📂 Open File...").size(14.0))
            ).clicked() {
                self.visible = false;
                opened_file = Some(String::new()); // Will trigger file dialog
            }

            ui.add_space(6.0);

            if ui.add_sized(
                [button_width, button_height],
                egui::Button::new(RichText::new("📁 Open Folder...").size(14.0))
            ).clicked() {
                self.visible = false;
            }

            ui.add_space(30.0);

            // Recent files
            if !self.recent_files.is_empty() {
                ui.label(RichText::new("Recent").size(12.0).strong());
                ui.add_space(10.0);

                for file in &self.recent_files {
                    if ui.add_sized(
                        [button_width, 30.0],
                        egui::Button::new(RichText::new(format!("📄 {}", file)).size(13.0))
                    ).clicked() {
                        opened_file = Some(file.clone());
                    }
                    ui.add_space(3.0);
                }
            }

            ui.add_space(30.0);

            // Keyboard shortcuts help
            ui.label(RichText::new("Keyboard Shortcuts").size(12.0).strong());
            ui.add_space(8.0);

            let shortcuts = [
                ("Ctrl+O", "Open File"),
                ("Ctrl+S", "Save"),
                ("Ctrl+F", "Find"),
                ("Ctrl+H", "Replace"),
                ("Ctrl+P", "Command Palette"),
                ("Ctrl+B", "Toggle Explorer"),
                ("Ctrl+J", "Toggle Terminal"),
                ("Ctrl+K Ctrl+Z", "Zen Mode"),
                ("Ctrl+\\", "Split Editor"),
                ("Ctrl+W", "Close Tab"),
                ("Ctrl+N", "New File"),
            ];

            for (key, desc) in &shortcuts {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(*desc).size(11.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(20.0);
                        ui.label(RichText::new(*key).size(11.0).color(Color32::from_rgb(150, 150, 150)));
                    });
                });
                ui.add_space(3.0);
            }
        });

        opened_file
    }
}
