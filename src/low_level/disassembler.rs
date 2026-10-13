// Phobos IDE - Disassembler (iced-x86, x86_64)
use eframe::egui;
use iced_x86::{Decoder, DecoderOptions, Formatter, IntelFormatter};

pub struct Disassembler {
    pub visible: bool,
    pub output: Vec<String>,
}

impl Disassembler {
    pub fn new() -> Self { Self { visible: false, output: Vec::new() } }
    pub fn disassemble(&mut self, bytes: &[u8], rip: u64) {
        self.output.clear();
        let mut decoder = Decoder::with_ip(64, bytes, rip, DecoderOptions::NONE);
        let mut formatter = IntelFormatter::new();
        let mut out = String::new();
        for instr in &mut decoder {
            out.clear();
            formatter.format(&instr, &mut out);
            self.output.push(format!("{:016X}: {}", instr.ip(), out));
            if self.output.len() > 5000 { break; }
        }
        self.visible = true;
    }
    pub fn disassemble_file(&mut self, path: &str) -> bool {
        if let Ok(b) = std::fs::read(path) {
            // skip ELF header heuristically: disassemble from 0x1000 or raw
            let start = if b.len() > 0x1000 { 0x1000 } else { 0 };
            self.disassemble(&b[start..], 0x1000 + start as u64);
            true
        } else { false }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !self.visible { return; }
        ui.horizontal(|ui| {
            ui.heading("⚙️ Disassembler (x86_64 Intel)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
            });
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for line in &self.output {
                ui.label(egui::RichText::new(line).monospace().size(12.0));
            }
        });
    }
}
