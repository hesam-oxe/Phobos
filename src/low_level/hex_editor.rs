// Phobos IDE - Hex Editor (egui-based, paged, editable)
use eframe::egui;

pub struct HexEditor {
    pub visible: bool,
    pub bytes: Vec<u8>,
    pub cursor: usize,
    pub bytes_per_row: usize,
    pub file_path: Option<String>,
}

impl HexEditor {
    pub fn new() -> Self {
        Self { visible: false, bytes: Vec::new(), cursor: 0, bytes_per_row: 16, file_path: None }
    }
    pub fn open_file(&mut self, path: &str) -> bool {
        if let Ok(b) = std::fs::read(path) {
            self.bytes = b; self.cursor = 0;
            self.file_path = Some(path.to_string()); self.visible = true; true
        } else { false }
    }
    pub fn open_bytes(&mut self, data: Vec<u8>, name: &str) {
        self.bytes = data; self.cursor = 0;
        self.file_path = Some(name.to_string()); self.visible = true;
    }
    pub fn save(&self) -> bool {
        if let Some(p) = &self.file_path {
            return std::fs::write(p, &self.bytes).is_ok();
        }
        false
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !self.visible { return; }
        ui.horizontal(|ui| {
            ui.heading("🔧 Hex Editor");
            if let Some(p) = &self.file_path { ui.label(format!("{} ({} bytes)", p, self.bytes.len())); }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
                if ui.button("💾 Save").clicked() { self.save(); }
            });
        });
        ui.separator();
        let rows = self.bytes.len().div_ceil(self.bytes_per_row);
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("hex_grid").spacing([4.0, 2.0]).show(ui, |ui| {
                for r in 0..rows.min(4096) {
                    let off = r * self.bytes_per_row;
                    ui.label(egui::RichText::new(format!("{:08X}", off)).monospace().color(egui::Color32::GRAY));
                    for c in 0..self.bytes_per_row {
                        let idx = off + c;
                        if idx < self.bytes.len() {
                            let b = self.bytes[idx];
                            let txt = format!("{:02X}", b);
                            let sel = idx == self.cursor;
                            if ui.selectable_label(sel, egui::RichText::new(txt).monospace()).clicked() {
                                self.cursor = idx;
                            }
                        } else { ui.label("  "); }
                    }
                    // ASCII
                    let ascii: String = self.bytes[off..(off+self.bytes_per_row).min(self.bytes.len())]
                        .iter().map(|&b| if (32..=126).contains(&b) { b as char } else { '.' }).collect();
                    ui.label(egui::RichText::new(ascii).monospace().color(egui::Color32::LIGHT_GREEN));
                    ui.end_row();
                }
            });
        });
        // edit selected byte
        ui.horizontal(|ui| {
            ui.label(format!("Offset: 0x{:X}", self.cursor));
            let mut hex_str = if self.cursor < self.bytes.len() { format!("{:02X}", self.bytes[self.cursor]) } else { String::new() };
            if ui.text_edit_singleline(&mut hex_str).changed() {
                if hex_str.len() == 2 {
                    if let Ok(v) = u8::from_str_radix(hex_str.trim(), 16) {
                        if self.cursor < self.bytes.len() { self.bytes[self.cursor] = v; }
                    }
                }
            }
        });
    }
}
