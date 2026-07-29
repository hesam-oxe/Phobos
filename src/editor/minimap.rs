use eframe::egui;
use egui::{Color32, Stroke};

pub struct Minimap {
    pub visible: bool,
    pub width: f32,
}

impl Minimap {
    pub fn new() -> Self {
        Self {
            visible: true,
            width: 80.0,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, content: &str) {
        if !self.visible {
            return;
        }

        let minimap_content = content.lines()
            .map(|line| if line.len() > 100 { &line[..100] } else { line })
            .collect::<Vec<_>>()
            .join("\n");

        ui.add_sized(
            [self.width, ui.available_height()],
            egui::TextEdit::multiline(&mut minimap_content.to_string())
                .font(egui::TextStyle::Monospace)
                .interactive(false)
                .desired_width(self.width)
        );
    }
}
