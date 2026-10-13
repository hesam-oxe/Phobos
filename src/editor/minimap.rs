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

    pub fn show(&mut self, ui: &mut egui::Ui, content: &str, cursor_line: usize) -> Option<usize> {
        if !self.visible {
            return None;
        }
        let total = content.lines().count().max(1);
        let h = ui.available_height().max(60.0);
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(self.width, h), egui::Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(22, 22, 28));
        let line_h = h / total as f32;
        for (i, line) in content.lines().enumerate() {
            let w = ((line.len() as f32 / 100.0) * (self.width - 8.0))
                .clamp(2.0, self.width - 8.0);
            let y = rect.min.y + i as f32 * line_h;
            let c = if i == cursor_line {
                Color32::from_rgb(120, 170, 255)
            } else {
                Color32::from_rgb(90, 90, 110)
            };
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + 4.0, y + 0.5),
                    egui::vec2(w, (line_h - 1.0).max(1.0)),
                ),
                0.0,
                c,
            );
        }
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let line = ((pos.y - rect.min.y) / h * total as f32) as usize;
                return Some(line.min(total.saturating_sub(1)));
            }
        }
        None
    }
}
