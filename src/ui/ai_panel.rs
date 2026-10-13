// Phobos IDE - AI Panel (wired to GhostTyper/BugFinder/Explainer)
use eframe::egui;
use crate::ai::ghost_typer::GhostTyper;
use crate::ai::bug_finder::BugFinder;
use crate::ai::code_explainer::CodeExplainer;

pub struct AiPanel {
    pub visible: bool,
    pub tab: usize,
    pub explainer: CodeExplainer,
    pub finder: BugFinder,
    pub typer: GhostTyper,
    pub explanation: String,
    pub language: String,
}

impl AiPanel {
    pub fn new() -> Self {
        Self { visible: false, tab: 0,
            explainer: CodeExplainer::new("rust"), finder: BugFinder::new("rust"),
            typer: GhostTyper::new("rust"), explanation: String::new(),
            language: "rust".into() }
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
            for (i, t) in ["Explain", "Bugs", "Complete"].iter().enumerate() {
                if ui.selectable_label(self.tab == i, *t).clicked() { self.tab = i; }
            }
        });
        ui.separator();
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
            _ => {
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
