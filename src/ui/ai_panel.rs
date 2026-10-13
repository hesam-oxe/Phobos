// Phobos IDE - AI Panel (wired to GhostTyper/BugFinder/Explainer + remote LLM)
use eframe::egui;
use crate::ai::ghost_typer::GhostTyper;
use crate::ai::bug_finder::BugFinder;
use crate::ai::code_explainer::CodeExplainer;
use crate::ai::provider::{ChatConfig, chat_complete};

pub struct AiPanel {
    pub visible: bool,
    pub tab: usize,
    pub explainer: CodeExplainer,
    pub finder: BugFinder,
    pub typer: GhostTyper,
    pub explanation: String,
    pub language: String,
    pub remote_cfg: ChatConfig,
    pub remote_prompt: String,
    pub remote_answer: String,
    pub remote_busy: bool,
    remote_rx: Option<std::sync::mpsc::Receiver<anyhow::Result<String>>>,
}

impl AiPanel {
    pub fn new() -> Self {
        let mut p = Self { visible: false, tab: 0,
            explainer: CodeExplainer::new("rust"), finder: BugFinder::new("rust"),
            typer: GhostTyper::new("rust"), explanation: String::new(),
            language: "rust".into(), remote_cfg: ChatConfig::default(),
            remote_prompt: String::new(), remote_answer: String::new(),
            remote_busy: false, remote_rx: None };
        p.load_remote_settings();
        p
    }

    fn remote_settings_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("phobos").join("ai.toml"))
    }

    /// Load endpoint+model only. The API key is never persisted.
    fn load_remote_settings(&mut self) {
        if let Some(p) = Self::remote_settings_path() {
            if let Ok(s) = std::fs::read_to_string(&p) {
                if let Ok(v) = toml::from_str::<toml::Value>(&s) {
                    if let Some(e) = v.get("endpoint").and_then(|x| x.as_str()) {
                        self.remote_cfg.endpoint = e.to_string();
                    }
                    if let Some(m) = v.get("model").and_then(|x| x.as_str()) {
                        self.remote_cfg.model = m.to_string();
                    }
                }
            }
        }
    }

    fn save_remote_settings(&self) {
        if let Some(p) = Self::remote_settings_path() {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let body = format!(
                "endpoint = {:?}\nmodel = {:?}\n# api key is intentionally NOT stored\n",
                self.remote_cfg.endpoint, self.remote_cfg.model,
            );
            let _ = std::fs::write(&p, body);
        }
    }
    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.into();
        self.explainer = CodeExplainer::new(lang);
        self.finder = BugFinder::new(lang);
        self.typer = GhostTyper::new(lang);
    }
    pub fn show(&mut self, ui: &mut egui::Ui, code: &str, cursor: usize) {
        ui.horizontal(|ui| {
            ui.heading("🧠 AI Assistant");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").clicked() { self.visible = false; }
            });
        });
        ui.horizontal(|ui| {
            for (i, t) in ["Explain", "Bugs", "Complete", "Remote"].iter().enumerate() {
                if ui.selectable_label(self.tab == i, *t).clicked() { self.tab = i; }
            }
        });
        ui.separator();
        // poll in-flight remote answer without blocking
        if let Some(rx) = self.remote_rx.as_ref() {
            if let Ok(res) = rx.try_recv() {
                self.remote_busy = false;
                self.remote_rx = None;
                match res {
                    Ok(text) => self.remote_answer = text,
                    Err(e) => self.remote_answer = format!("❌ {}", e),
                }
            }
        }
        match self.tab {
            0 => {
                if ui.button("✨ Explain code").clicked() {
                    let e = self.explainer.explain(code);
                    self.explanation = format!("{}\n\n{:?}\n\nConcepts: {}\n\nTips:\n- {}",
                        e.summary, e.complexity, e.key_concepts.join(", "),
                        e.suggestions.join("\n- "));
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(egui::RichText::new(&self.explanation).size(12.0));
                });
            }
            1 => {
                let reports = self.finder.analyze(code);
                ui.label(format!("{} issues", reports.len()));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for r in reports.iter().take(100) {
                        ui.label(format!("L{} [{:?}] {} → {}", r.line_number, r.severity, r.message, r.suggestion));
                        ui.separator();
                    }
                });
            }
            other => {
                if other == 3 {
                    // Remote LLM tab (OpenAI-compatible; key kept in memory only)
                    ui.horizontal(|ui| {
                        ui.label("Endpoint:");
                        if ui.text_edit_singleline(&mut self.remote_cfg.endpoint).changed() {
                            self.save_remote_settings();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Model:");
                        if ui.text_edit_singleline(&mut self.remote_cfg.model).changed() {
                            self.save_remote_settings();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("API key:");
                        ui.add(egui::TextEdit::singleline(&mut self.remote_cfg.api_key).password(true));
                    });
                    ui.label(egui::RichText::new("key is session-only, never saved").size(10.0));
                    ui.separator();
                    ui.label("Ask about the current code:");
                    ui.text_edit_multiline(&mut self.remote_prompt);
                    let can_ask = !self.remote_busy;
                    if ui.add_enabled(can_ask, egui::Button::new(if self.remote_busy { "⏳ asking…" } else { "🌐 Ask" })).clicked() {
                        let cfg = self.remote_cfg.clone();
                        let prompt = if self.remote_prompt.trim().is_empty() {
                            format!("Explain this {} code:\n{}", self.language, code)
                        } else {
                            format!("{}\n\n```{}\n{}\n```", self.remote_prompt, self.language, code)
                        };
                        let (tx, rx) = std::sync::mpsc::channel();
                        self.remote_rx = Some(rx);
                        self.remote_busy = true;
                        std::thread::spawn(move || {
                            let r = chat_complete(&cfg, "You are a concise coding assistant inside an IDE.", &prompt);
                            let _ = tx.send(r);
                        });
                    }
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(egui::RichText::new(&self.remote_answer).size(12.0));
                    });
                    return;
                }
                let suggs = self.typer.get_suggestions(code, cursor);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for s in suggs.iter().take(30) {
                        if ui.button(format!("[{:.0}%] {}", s.confidence*100.0, s.text.lines().next().unwrap_or(""))).clicked() {}
                        ui.label(&s.description);
                    }
                });
            }
        }
    }
}
