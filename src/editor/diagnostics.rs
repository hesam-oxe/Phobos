// Phobos IDE - Diagnostics Module
// Real-time code diagnostics and error reporting

use crate::ai::bug_finder::{BugCategory, BugFinder, BugReport, Severity};
use eframe::egui;
use egui::{Color32, FontId, Frame, Margin, Rounding, ScrollArea, Ui};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl From<&Severity> for DiagnosticSeverity {
    fn from(severity: &Severity) -> Self {
        match severity {
            Severity::Critical | Severity::High => DiagnosticSeverity::Error,
            Severity::Medium => DiagnosticSeverity::Warning,
            Severity::Low => DiagnosticSeverity::Info,
            Severity::Info => DiagnosticSeverity::Hint,
        }
    }
}

pub struct DiagnosticsPanel {
    finder: BugFinder,
    diagnostics: Vec<Diagnostic>,
    visible: bool,
    filter_severity: DiagnosticSeverity,
}

impl DiagnosticsPanel {
    pub fn new(language: &str) -> Self {
        Self {
            finder: BugFinder::new(language),
            diagnostics: Vec::new(),
            visible: true,
            filter_severity: DiagnosticSeverity::Hint,
        }
    }

    pub fn analyze(&mut self, code: &str) {
        let reports = self.finder.analyze(code);
        self.diagnostics = reports
            .iter()
            .map(|r| Diagnostic {
                line: r.line_number,
                column: 0,
                severity: DiagnosticSeverity::from(&r.severity),
                message: r.message.clone(),
                source: "AI Bug Finder".to_string(),
                code: Some(r.suggestion.clone()),
            })
            .collect();
    }

    pub fn set_language(&mut self, language: &str) {
        self.finder = BugFinder::new(language);
        self.diagnostics.clear();
    }

    pub fn get_diagnostics_for_line(&self, line: usize) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.line == line).collect()
    }

    pub fn render_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("🔍 Problems")
                    .font(FontId::proportional(16.0))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    // Trigger re-analysis
                }
            });
        });

        ui.separator();

        // Filter options
        ui.horizontal(|ui| {
            ui.label("Filter:");
            egui::ComboBox::from_id_source("severity_filter")
                .selected_text(format!("{:?}", self.filter_severity))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.filter_severity,
                        DiagnosticSeverity::Error,
                        "Errors",
                    );
                    ui.selectable_value(
                        &mut self.filter_severity,
                        DiagnosticSeverity::Warning,
                        "Warnings",
                    );
                    ui.selectable_value(
                        &mut self.filter_severity,
                        DiagnosticSeverity::Info,
                        "Info",
                    );
                    ui.selectable_value(
                        &mut self.filter_severity,
                        DiagnosticSeverity::Hint,
                        "Hints",
                    );
                });
        });

        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            if self.diagnostics.is_empty() {
                ui.label(egui::RichText::new("✅ No problems found").color(Color32::GREEN));
                return;
            }

            for diagnostic in &self.diagnostics {
                let color = match diagnostic.severity {
                    DiagnosticSeverity::Error => Color32::RED,
                    DiagnosticSeverity::Warning => Color32::YELLOW,
                    DiagnosticSeverity::Info => Color32::BLUE,
                    DiagnosticSeverity::Hint => Color32::GRAY,
                };

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(match diagnostic.severity {
                            DiagnosticSeverity::Error => "❌",
                            DiagnosticSeverity::Warning => "⚠️",
                            DiagnosticSeverity::Info => "ℹ️",
                            DiagnosticSeverity::Hint => "💡",
                        })
                        .color(color),
                    );

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&diagnostic.message)
                                .color(color)
                                .font(FontId::monospace(12.0)),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Line {} | {}",
                                diagnostic.line, diagnostic.source
                            ))
                            .color(Color32::DARK_GRAY)
                            .font(FontId::proportional(10.0)),
                        );

                        if let Some(code) = &diagnostic.code {
                            if !code.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!("💡 Suggestion: {}", code))
                                        .color(Color32::LIGHT_BLUE)
                                        .font(FontId::monospace(11.0)),
                                );
                            }
                        }
                    });
                });

                ui.add_space(4.0);
                ui.separator();
            }
        });
    }

    pub fn count_by_severity(&self) -> (usize, usize, usize) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut info = 0;

        for d in &self.diagnostics {
            match d.severity {
                DiagnosticSeverity::Error => errors += 1,
                DiagnosticSeverity::Warning => warnings += 1,
                DiagnosticSeverity::Info | DiagnosticSeverity::Hint => info += 1,
            }
        }

        (errors, warnings, info)
    }
}
