// Phobos IDE - Register Viewer (x86_64 GPR + flags)
use eframe::egui;
use std::collections::BTreeMap;

pub struct RegisterViewer {
    pub visible: bool,
    pub regs: BTreeMap<String, u64>,
}

impl RegisterViewer {
    pub fn new() -> Self {
        let mut regs = BTreeMap::new();
        for r in ["RAX","RBX","RCX","RDX","RSI","RDI","RSP","RBP","R8","R9","R10","R11","R12","R13","R14","R15","RIP","RFLAGS"] {
            regs.insert(r.to_string(), 0);
        }
        Self { visible: false, regs }
    }
    pub fn set(&mut self, name: &str, val: u64) { self.regs.insert(name.to_uppercase(), val); }
    pub fn parse_gdb_regs(&mut self, text: &str) {
        for line in text.lines() {
            for tok in line.split_whitespace() {
                if let Some((k,v)) = tok.split_once('=') {
                    let k = k.trim().to_uppercase();
                    let v = v.trim().trim_start_matches("0x");
                    if self.regs.contains_key(&k) {
                        if let Ok(n) = u64::from_str_radix(v, 16) { self.regs.insert(k, n); }
                    }
                }
            }
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !self.visible { return; }
        ui.horizontal(|ui| {
            ui.heading("📟 Registers");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
            });
        });
        ui.separator();
        egui::Grid::new("regs").spacing([8.0,4.0]).show(ui, |ui| {
            let keys: Vec<String> = self.regs.keys().cloned().collect();
            for chunk in keys.chunks(2) {
                for k in chunk {
                    let v = self.regs[k];
                    ui.label(egui::RichText::new(k).monospace().strong().color(egui::Color32::LIGHT_BLUE));
                    ui.label(egui::RichText::new(format!("0x{:016X} ({})", v, v as i64)).monospace());
                }
                ui.end_row();
            }
        });
    }
}
