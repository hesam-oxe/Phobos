// Phobos IDE - Real PTY Terminal UI (polls TerminalShell via tokio runtime)
use eframe::egui;
use crate::services::terminal::shell::{TerminalShell, AnsiParser};

pub struct PtyTerminal {
    pub visible: bool,
    pub shell: Option<TerminalShell>,
    pub buffer: String,
    pub input: String,
    pub parser: AnsiParser,
    rt: Option<tokio::runtime::Runtime>,
    pub status: String,
}

impl PtyTerminal {
    pub fn new() -> Self {
        Self { visible: false, shell: None, buffer: String::from("💀 Phobos PTY — press Connect\n"),
            input: String::new(), parser: AnsiParser::new(true), rt: None,
            status: "disconnected".into() }
    }
    fn rt(&mut self) -> &tokio::runtime::Runtime {
        if self.rt.is_none() {
            self.rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all().build().ok();
        }
        self.rt.as_ref().unwrap()
    }
    pub fn connect(&mut self) {
        // spawn shell synchronously via runtime
        let rt = self.rt();
        let res: anyhow::Result<TerminalShell> = rt.block_on(async {
            crate::services::terminal::shell::create_terminal(None).await
        });
        match res {
            Ok(s) => { self.shell = Some(s); self.status = "connected".into(); self.buffer.push_str("✅ connected\n"); }
            Err(e) => { self.status = format!("err: {}", e); self.buffer.push_str(&format!("❌ {}\n", e)); }
        }
    }
    pub fn poll(&mut self) {
        if self.shell.is_none() { return; }
        if self.rt.is_none() {
            self.rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all().build().ok();
        }
        // Collect outputs with runtime (only borrows shell), then process buffer/parser afterwards
        let outputs: Vec<String> = {
            let rt = self.rt.as_ref().unwrap();
            let shell_ref = self.shell.as_ref().unwrap() as *const TerminalShell;
            rt.block_on(async move {
                let mut out = Vec::new();
                let s: &TerminalShell = unsafe { &*shell_ref };
                for _ in 0..32 {
                    if let Some(o) = s.try_recv_output().await {
                        out.push(o.data);
                    } else { break; }
                }
                out
            })
        };
        for data in outputs {
            let clean = self.parser.parse(&data);
            self.buffer.push_str(&clean);
            if self.buffer.len() > 200_000 {
                self.buffer = self.buffer[self.buffer.len()-200_000..].to_string();
            }
        }
    }
    pub fn send(&mut self, text: &str) {
        if let Some(s) = self.shell.as_mut() {
            let _ = s.write_input(text);
        } else {
            self.buffer.push_str("(not connected)\n");
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("💻 PTY [{}]", self.status));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
                if ui.button("🗑").clicked() { self.buffer.clear(); }
                if ui.button("🔌 Connect").clicked() { self.connect(); }
            });
        });
        ui.separator();
        self.poll();
        egui::ScrollArea::vertical().stick_to_bottom(true).max_height(ui.available_height()-40.0).show(ui, |ui| {
            ui.label(egui::RichText::new(&self.buffer).monospace().size(12.0));
        });
        let resp = ui.add(egui::TextEdit::singleline(&mut self.input)
            .hint_text("$ type + Enter (Shift+Enter for newline)")
            .font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let cmd = format!("{}\n", self.input);
            self.buffer.push_str(&format!("$ {}", self.input));
            self.buffer.push('\n');
            self.send(&cmd);
            self.input.clear();
            // refocus
            resp.request_focus();
        }
    }
}
