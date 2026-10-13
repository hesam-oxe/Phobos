// Phobos IDE - Debug Panel (GDB breakpoints + DAP session)
use eframe::egui;
use crate::build::debugger::{Debugger, DebugState};
use crate::build::dap::{
    DapSession, DapEvent, DapThread, DapFrame, DapScope, DapVar,
    suggested_adapter, split_adapter, workspace_of,
};

pub struct DebugPanel {
    pub visible: bool,
    pub debugger: Debugger,
    pub target: String,
    pub bp_file: String,
    pub bp_line: String,
    dap: Option<DapSession>,
    dap_adapter: String,
    dap_program: String,
    dap_threads: Vec<DapThread>,
    dap_thread: Option<i64>,
    dap_frames: Vec<DapFrame>,
    dap_scopes: Vec<DapScope>,
    dap_vars: Vec<DapVar>,
    dap_output: Vec<String>,
    dap_status: String,
}

impl DebugPanel {
    pub fn new() -> Self {
        Self { visible: false, debugger: Debugger::new("rust"),
            target: String::new(), bp_file: String::new(), bp_line: String::new(),
            dap: None, dap_adapter: "lldb-dap".into(), dap_program: String::new(),
            dap_threads: Vec::new(), dap_thread: None, dap_frames: Vec::new(),
            dap_scopes: Vec::new(), dap_vars: Vec::new(), dap_output: Vec::new(),
            dap_status: "idle".into() }
    }

    fn poll_dap(&mut self) {
        let events = match self.dap.as_ref() {
            Some(s) => s.poll(),
            None => return,
        };
        for ev in events {
            match ev {
                DapEvent::Output(t) => {
                    for line in t.lines() {
                        self.dap_output.push(line.to_string());
                    }
                    if self.dap_output.len() > 500 {
                        let n = self.dap_output.len() - 500;
                        self.dap_output.drain(..n);
                    }
                }
                DapEvent::Stopped { reason, thread_id } => {
                    self.dap_thread = Some(thread_id);
                    self.dap_status = format!("stopped: {}", reason);
                }
                DapEvent::Continued { .. } => {
                    self.dap_status = "running".into();
                }
                DapEvent::Terminated | DapEvent::Exited(_) => {
                    self.dap_status = "terminated".into();
                    self.dap = None;
                }
                DapEvent::Threads(list) => {
                    if self.dap_thread.is_none() {
                        self.dap_thread = list.first().map(|t| t.id);
                    }
                    self.dap_threads = list;
                }
                DapEvent::StackFrames { frames, .. } => {
                    self.dap_frames = frames;
                }
                DapEvent::Scopes { scopes, .. } => {
                    self.dap_scopes = scopes;
                }
                DapEvent::Variables { vars, .. } => {
                    self.dap_vars = vars;
                }
                DapEvent::Status(s) => {
                    self.dap_status = s;
                }
            }
        }
    }

    fn forward_breakpoints(&self) {
        let session = match self.dap.as_ref() {
            Some(s) => s,
            None => return,
        };
        let mut by_file: std::collections::HashMap<String, Vec<u32>> =
            std::collections::HashMap::new();
        for bp in self.debugger.get_breakpoints() {
            if bp.enabled {
                by_file.entry(bp.file.clone()).or_default().push(bp.line as u32);
            }
        }
        for (path, mut lines) in by_file {
            lines.sort_unstable();
            lines.dedup();
            session.set_breakpoints(&path, lines);
        }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.poll_dap();
        ui.horizontal(|ui| {
            ui.heading("🐞 Debug");
            ui.label(format!("{:?}", self.debugger.get_state()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
            });
        });
        ui.separator();
        // ---- DAP session (Debug Adapter Protocol) ----
        ui.heading("🔌 DAP");
        ui.horizontal(|ui| {
            ui.label("Adapter:");
            ui.text_edit_singleline(&mut self.dap_adapter);
            if ui.button("✨ suggest").clicked() {
                self.dap_adapter = suggested_adapter(self.debugger.language()).into();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Program:");
            ui.text_edit_singleline(&mut self.dap_program);
            if self.dap.is_none() {
                if ui.button("▶ DAP Launch").clicked() && !self.dap_program.is_empty() {
                    let (prog, args) = split_adapter(&self.dap_adapter.clone());
                    let cwd = workspace_of(&self.dap_program).to_string_lossy().to_string();
                    self.dap = Some(DapSession::launch(&prog, args, &self.dap_program.clone(), &cwd));
                    self.dap_status = "launching…".into();
                    self.forward_breakpoints();
                }
            } else if ui.button("⏹ Disconnect").clicked() {
                if let Some(s) = self.dap.as_ref() {
                    s.disconnect();
                }
                self.dap = None;
                self.dap_status = "idle".into();
            }
            ui.label(format!("⌨ {}", self.dap_status));
        });
        if self.dap.is_some() {
            ui.horizontal(|ui| {
                if ui.button("▶ Continue").clicked() {
                    if let Some(s) = self.dap.as_ref() { s.continue_exec(self.dap_thread); }
                }
                if ui.button("⏭ Over").clicked() {
                    if let Some(s) = self.dap.as_ref() { s.step_over(self.dap_thread); }
                }
                if ui.button("⬇ Into").clicked() {
                    if let Some(s) = self.dap.as_ref() { s.step_into(self.dap_thread); }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Threads:");
                for t in self.dap_threads.clone() {
                    if ui.selectable_label(self.dap_thread == Some(t.id), &t.name).clicked() {
                        self.dap_thread = Some(t.id);
                    }
                }
            });
            egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                ui.label(egui::RichText::new("Stack").strong());
                for f in &self.dap_frames {
                    ui.label(format!("#{} {} ({}:{})", f.id, f.name, f.file, f.line));
                }
                ui.separator();
                ui.label(egui::RichText::new("Variables").strong());
                for s in &self.dap_scopes {
                    ui.label(format!("▸ {}", s.name));
                }
                for v in &self.dap_vars {
                    let t = if v.var_type.is_empty() { String::new() } else { format!(": {}", v.var_type) };
                    ui.label(format!("{} = {}{}", v.name, v.value, t));
                }
            });
            egui::ScrollArea::vertical().max_height(80.0).stick_to_bottom(true).show(ui, |ui| {
                for l in &self.dap_output {
                    ui.label(egui::RichText::new(l).monospace().size(11.0));
                }
            });
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Target:");
            ui.text_edit_singleline(&mut self.target);
            if ui.button("▶ Start").clicked() {
                if !self.target.is_empty() {
                    let _ = self.debugger.start(std::path::Path::new(&self.target));
                }
            }
            if ui.button("⏹ Stop").clicked() { let _ = self.debugger.stop(); }
        });
        ui.horizontal(|ui| {
            if ui.button("▶ Continue").clicked() { let _ = self.debugger.continue_execution(); }
            if ui.button("⏭ Over").clicked() { let _ = self.debugger.step_over(); }
            if ui.button("⬇ Into").clicked() { let _ = self.debugger.step_into(); }
            if ui.button("⬆ Out").clicked() { let _ = self.debugger.step_out(); }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("BP file:"); ui.text_edit_singleline(&mut self.bp_file);
            ui.label("line:"); ui.text_edit_singleline(&mut self.bp_line);
            if ui.button("＋ Add").clicked() {
                if let Ok(n) = self.bp_line.parse::<usize>() {
                    self.debugger.add_breakpoint(&self.bp_file.clone(), n, None);
                    self.forward_breakpoints();
                }
            }
        });
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Breakpoints");
            for bp in self.debugger.get_breakpoints() {
                ui.label(format!("{}:{} {} {}", bp.file, bp.line,
                    if bp.enabled {"🟢"} else {"⚪"},
                    bp.condition.clone().unwrap_or_default()));
            }
            ui.separator();
            ui.heading("Output");
            ui.label(egui::RichText::new(self.debugger.get_output()).monospace().size(11.0));
            if self.debugger.get_state() != &DebugState::NotStarted {
                if ui.button("Refresh stack/vars").clicked() {
                    let _ = self.debugger.get_stack_trace();
                    let _ = self.debugger.get_variables();
                }
            }
        });
    }
}
