// Phobos IDE - Memory Viewer
use eframe::egui;

pub struct MemoryViewer {
    pub visible: bool,
    pub address: u64,
    pub data: Vec<u8>,
}

impl MemoryViewer {
    pub fn new() -> Self { Self { visible: false, address: 0, data: Vec::new() } }
    pub fn set_data(&mut self, addr: u64, data: Vec<u8>) {
        self.address = addr; self.data = data; self.visible = true;
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !self.visible { return; }
        ui.horizontal(|ui| {
            ui.heading("🧠 Memory Viewer");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
            });
        });
        ui.horizontal(|ui| {
            ui.label("Base (hex):");
            let mut s = format!("{:X}", self.address);
            if ui.text_edit_singleline(&mut s).changed() {
                if let Ok(v) = u64::from_str_radix(s.trim().trim_start_matches("0x"), 16) {
                    self.address = v;
                }
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, chunk) in self.data.chunks(16).enumerate() {
                let addr = self.address + (i*16) as u64;
                let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
                let asc: String = chunk.iter().map(|&b| if (32..=126).contains(&b){b as char}else{'.'}).collect();
                ui.label(egui::RichText::new(format!("{:016X}  {:<47}  {}", addr, hex.join(" "), asc)).monospace().size(12.0));
            }
            if self.data.is_empty() {
                ui.label("No memory loaded. Use debugger to fetch memory.");
            }
        });
    }
}
