// Phobos IDE - Hover Module
// Intelligent hover information and tooltips

use crate::ai::code_explainer::{CodeExplainer, Explanation};
use eframe::egui;
use egui::{Color32, FontId, Frame, Margin, Pos2, Rounding, Ui, Vec2};

pub struct HoverProvider {
    explainer: CodeExplainer,
    last_hover_pos: Option<Pos2>,
    current_word: Option<String>,
    explanation: Option<Explanation>,
}

impl HoverProvider {
    pub fn new(language: &str) -> Self {
        Self {
            explainer: CodeExplainer::new(language),
            last_hover_pos: None,
            current_word: None,
            explanation: None,
        }
    }

    pub fn update_hover(&mut self, code: &str, cursor_pos: usize, mouse_pos: Pos2) {
        self.last_hover_pos = Some(mouse_pos);

        // Extract word at cursor position
        if let Some(word) = self.extract_word_at_position(code, cursor_pos) {
            if self.current_word.as_ref() != Some(&word) {
                self.current_word = Some(word.clone());
                // Generate explanation for the context around the word
                self.explanation = Some(self.explainer.explain(code));
            }
        } else {
            self.current_word = None;
            self.explanation = None;
        }
    }

    fn extract_word_at_position(&self, code: &str, pos: usize) -> Option<String> {
        if pos >= code.len() {
            return None;
        }

        let before = &code[..pos];
        let after = &code[pos..];

        let mut start = pos;
        let mut end = pos;

        // Find word boundaries
        for (i, c) in before.chars().rev().enumerate() {
            if c.is_alphanumeric() || c == '_' {
                start = pos - i - 1;
            } else {
                break;
            }
        }

        for (i, c) in after.char_indices() {
            if c.is_alphanumeric() || c == '_' {
                end = pos + i + 1;
            } else {
                break;
            }
        }

        if start < end && start < code.len() {
            Some(code[start..end.min(code.len())].to_string())
        } else {
            None
        }
    }

    pub fn render_tooltip(&self, ui: &mut Ui) {
        if let Some(explanation) = &self.explanation {
            if let Some(pos) = self.last_hover_pos {
                egui::Window::new("hover_tooltip")
                    .id(egui::Id::new("hover_tooltip"))
                    .collapsible(false)
                    .resizable(false)
                    .title_bar(false)
                    .fixed_size([350.0, 250.0])
                    .current_pos(pos + Vec2::new(20.0, 20.0))
                    .frame(Frame {
                        fill: Color32::from_rgb(25, 25, 35),
                        stroke: egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 120)),
                        rounding: Rounding::same(8.0),
                        inner_margin: Margin::same(12.0),
                        ..Default::default()
                    })
                    .show(ui.ctx(), |ui| {
                        ui.vertical(|ui| {
                            // Summary
                            ui.label(
                                egui::RichText::new("📖 Code Overview")
                                    .font(FontId::proportional(14.0))
                                    .strong()
                                    .color(Color32::LIGHT_BLUE),
                            );

                            ui.separator();

                            ScrollArea::vertical().show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&explanation.summary)
                                        .font(FontId::proportional(11.0))
                                        .color(Color32::WHITE),
                                );

                                ui.add_space(8.0);

                                // Complexity
                                ui.label(
                                    egui::RichText::new(format!(
                                        "📊 Complexity: {:?}",
                                        explanation.complexity
                                    ))
                                    .font(FontId::monospace(10.0))
                                    .color(Color32::YELLOW),
                                );

                                ui.add_space(8.0);

                                // Key concepts
                                if !explanation.key_concepts.is_empty() {
                                    ui.label(
                                        egui::RichText::new("🔑 Key Concepts:")
                                            .font(FontId::proportional(11.0))
                                            .strong()
                                            .color(Color32::LIGHT_GREEN),
                                    );

                                    for concept in &explanation.key_concepts {
                                        ui.label(
                                            egui::RichText::new(format!("  • {}", concept))
                                                .font(FontId::monospace(10.0))
                                                .color(Color32::GRAY),
                                        );
                                    }
                                }

                                ui.add_space(8.0);

                                // Suggestions
                                if !explanation.suggestions.is_empty() {
                                    ui.label(
                                        egui::RichText::new("💡 Suggestions:")
                                            .font(FontId::proportional(11.0))
                                            .strong()
                                            .color(Color32::LIGHT_YELLOW),
                                    );

                                    for suggestion in &explanation.suggestions {
                                        ui.label(
                                            egui::RichText::new(format!("  → {}", suggestion))
                                                .font(FontId::monospace(10.0))
                                                .color(Color32::LIGHT_BLUE),
                                        );
                                    }
                                }
                            });
                        });
                    });
            }
        }
    }

    pub fn set_language(&mut self, language: &str) {
        self.explainer = CodeExplainer::new(language);
        self.explanation = None;
    }

    pub fn get_current_word(&self) -> Option<&String> {
        self.current_word.as_ref()
    }
}

use egui::ScrollArea;
