// Phobos IDE - Completion Module
// Intelligent code completion system with AI integration

use crate::ui::theme::ACCENT;
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

    /// Replace popup contents with external suggestions (e.g. from LSP).
    pub fn set_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
        self.popup_visible = !self.suggestions.is_empty();
        self.selected_index = 0;
    }

    /// Prepend a snippet suggestion (e.g. Emmet expansion) at the top.
    pub fn prepend_snippet(&mut self, text: String, description: String) {
        self.suggestions.insert(
            0,
            Suggestion {
                text,
                kind: SuggestionKind::Snippet,
                confidence: 1.0,
                description,
            },
        );
        self.popup_visible = true;
        self.selected_index = 0;
    }

    pub fn is_visible(&self) -> bool {
        self.popup_visible && !self.suggestions.is_empty()
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
            .fixed_size([340.0, 220.0])
            .frame(Frame {
                fill: Color32::from_rgb(24, 24, 32),
                stroke: egui::Stroke::new(1.0, Color32::from_rgb(70, 70, 92)),
                rounding: Rounding::same(10.0),
                inner_margin: Margin::same(6.0),
                shadow: egui::epaint::Shadow {
                    offset: egui::vec2(0.0, 8.0),
                    blur: 28.0,
                    spread: 1.0,
                    color: Color32::from_black_alpha(100),
                },
                ..Default::default()
            })
            .show(ui.ctx(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    for (idx, suggestion) in self.suggestions.iter().enumerate() {
                        let is_selected = idx == self.selected_index;
                        let (icon, icon_color) = kind_badge(&suggestion.kind);

                        // custom row: selected bar + kind icon + text + description
                        let row_h = 24.0;
                        let row_w = ui.available_width();
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(row_w, row_h),
                            egui::Sense::click(),
                        );
                        if is_selected {
                            ui.painter().rect_filled(
                                rect,
                                Rounding::same(6.0),
                                Color32::from_rgb(58, 44, 92),
                            );
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(rect.left() + 2.0, rect.top() + 4.0),
                                    egui::vec2(2.5, row_h - 8.0),
                                ),
                                Rounding::same(1.5),
                                ACCENT,
                            );
                        } else if response.hovered() {
                            ui.painter().rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(36, 36, 48));
                        }
                        // kind icon
                        ui.painter().text(
                            egui::pos2(rect.left() + 12.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            icon,
                            FontId::monospace(12.0),
                            icon_color,
                        );
                        // text (truncated)
                        let text = truncate_text(&suggestion.text, 38);
                        ui.painter().text(
                            egui::pos2(rect.left() + 34.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &text,
                            FontId::monospace(12.5),
                            if is_selected { Color32::WHITE } else { Color32::from_rgb(200, 200, 215) },
                        );
                        // description right
                        if !suggestion.description.is_empty() {
                            ui.painter().text(
                                egui::pos2(rect.right() - 8.0, rect.center().y),
                                egui::Align2::RIGHT_CENTER,
                                &truncate_text(&suggestion.description, 18),
                                FontId::proportional(9.5),
                                Color32::from_rgb(110, 112, 135),
                            );
                        }
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

/// (icon, color) per suggestion kind.
fn kind_badge(kind: &SuggestionKind) -> (&'static str, egui::Color32) {
    use SuggestionKind::*;
    match kind {
        Keyword => ("KW", egui::Color32::from_rgb(180, 140, 250)),
        Function => ("ƒ", egui::Color32::from_rgb(255, 200, 90)),
        Method => ("m", egui::Color32::from_rgb(120, 200, 250)),
        Variable => ("v", egui::Color32::from_rgb(140, 220, 140)),
        Type => ("T", egui::Color32::from_rgb(110, 180, 250)),
        Snippet => ("✦", egui::Color32::from_rgb(255, 122, 69)),
        Import => ("⇢", egui::Color32::from_rgb(200, 200, 140)),
    }
}

fn truncate_text(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let t: String = s.chars().take(max_chars).collect();
        format!("{}…", t.trim_end())
    }
}
