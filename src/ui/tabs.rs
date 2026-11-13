use eframe::egui;
use egui::{Color32, Frame, Margin, Rounding, Stroke, pos2};
use crate::ui::theme::{accent_of, bg_active_of, bg_hover_of, border_of, glow_line};

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active: usize,
}

pub struct Tab {
    pub name: String,
    pub modified: bool,
    pub path: Option<String>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab { name: "Untitled-1".into(), modified: false, path: None }],
            active: 0,
        }
    }

    /// Custom tab strip: compact tabs, accent underline on active, hover close ✕.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let bar_rect = ui.available_rect_before_wrap();
        let bar = egui::TopBottomPanel::top("tab_strip")
            .frame(Frame {
                fill: ui.ctx().style().visuals.extreme_bg_color,
                inner_margin: Margin { left: 4.0, right: 4.0, top: 2.0, bottom: 0.0 },
                ..Default::default()
            })
            .height_range(egui::Rangef::new(28.0, 28.0));
        bar.show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let mut close_idx = None;
                let mut activate_idx = None;

                for (i, tab) in self.tabs.iter().enumerate() {
                    let selected = i == self.active;
                    let label = if tab.modified {
                        format!("● {}", tab.name)
                    } else {
                        tab.name.clone()
                    };

                    // tab body — width from text layout + close button padding
                    let text_width = ui.painter().layout_no_wrap(
                        label.clone(),
                        egui::TextStyle::Body.resolve(ui.style()),
                        Color32::WHITE,
                    ).size().x;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2((text_width + 34.0).min(240.0), 24.0),
                        egui::Sense::click(),
                    );
                    let response = ui.interact(rect, egui::Id::new(("tab", i)), egui::Sense::click());

                    // tab background — from live theme
                    let bg = if selected {
                        bg_active_of(ui.ctx())
                    } else if response.hovered() {
                        bg_hover_of(ui.ctx())
                    } else {
                        Color32::TRANSPARENT
                    };
                    let rounding = Rounding {
                        nw: 6.0, ne: 6.0, sw: 0.0, se: 0.0,
                    };
                    ui.painter().rect_filled(rect, rounding, bg);
                    if selected {
                        ui.painter().rect_stroke(rect, rounding, Stroke::new(1.0, border_of(ui.ctx())));
                        // glowing accent underline
                        glow_line(
                            ui.painter(),
                            pos2(rect.left() + 4.0, rect.bottom() - 1.0),
                            pos2(rect.right() - 4.0, rect.bottom() - 1.0),
                            accent_of(ui.ctx()),
                            1.6,
                        );
                    }

                    // label
                    ui.painter().text(
                        pos2(rect.left() + 10.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &label,
                        egui::TextStyle::Body.resolve(ui.style()),
                        if selected { ui.ctx().style().visuals.text_color() } else { ui.ctx().style().visuals.weak_text_color() },
                    );

                    // close ✕ (visible on hover or when active)
                    let close_rect = egui::Rect::from_center_size(
                        pos2(rect.right() - 12.0, rect.center().y),
                        egui::vec2(16.0, 16.0),
                    );
                    if selected || response.hovered() {
                        let close_resp = ui.interact(close_rect, egui::Id::new(("tab_close", i)), egui::Sense::click());
                        let x_color = if close_resp.hovered() {
                            ui.ctx().style().visuals.text_color()
                        } else {
                            ui.ctx().style().visuals.weak_text_color()
                        };
                        if close_resp.hovered() {
                            ui.painter().rect_filled(close_rect, Rounding::same(3.0), bg_hover_of(ui.ctx()));
                        }
                        ui.painter().text(
                            close_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "✕",
                            {
                                let f = egui::TextStyle::Body.resolve(ui.style());
                                egui::FontId::new(f.size * 0.8, f.family.clone())
                            },
                            x_color,
                        );
                        if close_resp.clicked() {
                            close_idx = Some(i);
                        }
                    }

                    if response.clicked() {
                        activate_idx = Some(i);
                    }
                }

                // ➕ new tab
                if ui.small_button("➕").clicked() {
                    self.tabs.push(Tab {
                        name: format!("Untitled-{}", self.tabs.len() + 1),
                        modified: false,
                        path: None,
                    });
                    self.active = self.tabs.len() - 1;
                }

                if let Some(i) = activate_idx {
                    self.active = i;
                }
                if let Some(idx) = close_idx {
                    self.close_tab(idx);
                }
            });
        });
        let _ = bar_rect;
    }

    pub fn open_tab(&mut self, name: &str) {
        self.tabs.push(Tab {
            name: name.to_string(),
            modified: false,
            path: Some(name.to_string()),
        });
        self.active = self.tabs.len() - 1;
    }

    fn close_tab(&mut self, idx: usize) {
        if self.tabs.len() <= 1 { return; }
        self.tabs.remove(idx);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
    }
}
