// Phobos IDE - Completion Module
// Intelligent code completion system with AI integration

use eframe::egui;
use egui::{Color32, FontId, Frame, Margin, Rounding, ScrollArea, Ui, Vec2};
use crate::ai::ghost_typer::{GhostTyper, Suggestion, SuggestionKind};

pub struct CompletionProvider {
    ghost_typer: GhostTyper,
    popup_visible: bool,
    selected_index: usize,
    suggestions: Vec<Suggestion>,
    popup_position: Option<egui::Pos2>,
}

impl CompletionProvider {
    pub fn new(language: &str) -> Self {
        Self {
            ghost_typer: GhostTyper::new(language),
            popup_visible: false,
            selected_index: 0,
            suggestions: Vec::new(),
            popup_position: None,
        }
    }

    pub fn update_suggestions(&mut self, code: &str, cursor_pos: usize) {
        self.suggestions = self.ghost_typer.get_suggestions(code, cursor_pos);
        self.popup_visible = !self.suggestions.is_empty();
        self.selected_index = 0;
    }

    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.suggestions.is_empty() {
            self.selected_index = self.suggestions.len() - 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.suggestions.len();
        }
    }

    pub fn accept_completion(&mut self) -> Option<String> {
        if self.suggestions.is_empty() {
            return None;
        }
        let suggestion = &self.suggestions[self.selected_index];
        self.popup_visible = false;
        Some(suggestion.text.clone())
    }

    pub fn dismiss(&mut self) {
        self.popup_visible = false;
    }

    pub fn render_popup(&mut self, ui: &mut Ui) -> Option<usize> {
        if !self.popup_visible || self.suggestions.is_empty() {
            return None;
        }

        let mut accepted_index = None;

        egui::Window::new("completion_popup")
            .id(egui::Id::new("completion_popup"))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .fixed_size([300.0, 200.0])
            .frame(Frame {
                fill: Color32::from_rgb(30, 30, 40),
                stroke: egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 150)),
                rounding: Rounding::same(8.0),
                inner_margin: Margin::same(8.0),
                ..Default::default()
            })
            .show(ui.ctx(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for (idx, suggestion) in self.suggestions.iter().enumerate() {
                        let is_selected = idx == self.selected_index;
                        let bg_color = if is_selected {
                            Color32::from_rgb(60, 60, 100)
                        } else {
                            Color32::TRANSPARENT
                        };

                        let response = ui.add(
                            egui::Label::new(
                                egui::RichText::new(&suggestion.text)
                                    .font(FontId::monospace(14.0))
                                    .color(if is_selected { Color32::WHITE } else { Color32::LIGHT_GRAY })
                                    .background_color(bg_color)
                            )
                            .sense(egui::Sense::click())
                        );

                        if response.clicked() {
                            self.selected_index = idx;
                            accepted_index = Some(idx);
                        }
                    }
                });
            });

        if accepted_index.is_some() {
            self.popup_visible = false;
        }

        accepted_index
    }

    pub fn set_language(&mut self, language: &str) {
        self.ghost_typer = GhostTyper::new(language);
    }
}
