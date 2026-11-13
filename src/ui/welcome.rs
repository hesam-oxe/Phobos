use eframe::egui;
use egui::{Color32, RichText, pos2, Align2, Rounding, Stroke};
use crate::ui::theme::{Theme, gradient_rect, glow_line, ACCENT};

pub struct WelcomePage {
    pub visible: bool,
    pub recent_files: Vec<String>,
}

impl WelcomePage {
    pub fn new() -> Self {
        Self {
            visible: true,
            recent_files: Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) -> Option<String> {
        if !self.visible {
            return None;
        }

        let mut opened_file = None;
        let full_rect = ui.available_rect_before_wrap();

        // ── Hero gradient backdrop ───────────────────────────────────────────
        let painter = ui.painter();
        gradient_rect(
            painter,
            full_rect,
            theme.bg_primary,
            Theme::lerp(theme.bg_primary, theme.accent, 0.08),
            false,
        );
        // violet halo bottom-right
        gradient_rect(
            painter,
            egui::Rect::from_two_pos(
                pos2(full_rect.center().x, full_rect.center().y),
                full_rect.right_bottom(),
            ),
            Color32::TRANSPARENT,
            Color32::from_rgba_unmultiplied(theme.accent2.r(), theme.accent2.g(), theme.accent2.b(), 26),
            false,
        );
        // ember halo top-left
        gradient_rect(
            painter,
            egui::Rect::from_two_pos(full_rect.left_top(), egui::pos2(full_rect.center().x, full_rect.center().y)),
            Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 22),
            Color32::TRANSPARENT,
            false,
        );

        ui.vertical(|ui| {
            ui.add_space(46.0);
            ui.vertical_centered(|ui| {
                // Glowing logo
                let logo_pos = ui.next_widget_position() + egui::vec2(ui.available_width() / 2.0, 24.0);
                glow_line(
                    ui.painter(),
                    pos2(logo_pos.x - 44.0, logo_pos.y),
                    pos2(logo_pos.x + 44.0, logo_pos.y),
                    theme.accent,
                    2.0,
                );
                ui.label(RichText::new("💀").size(72.0));
                ui.add_space(4.0);
                ui.label(
                    RichText::new("PHOBOS")
                        .size(40.0)
                        .strong()
                        .color(theme.accent),
                );
                ui.label(
                    RichText::new("The God of Fear — True Successor to Delphi")
                        .size(13.0)
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("🔥 v1.0 GOD  ·  Rust-native  ·  Polyglot  ·  Emmet  ·  LSP  ·  DAP"))
                        .size(11.0)
                        .color(theme.accent2),
                );
            });

            ui.add_space(30.0);

            // ── Action cards ──────────────────────────────────────────────────
            let cards = [
                ("📝", "New File", "Ctrl+N", "Start something new"),
                ("📂", "Open File", "Ctrl+O", "Jump back into code"),
                ("📁", "Open Folder", "Ctrl+K", "Bring in a workspace"),
                ("🔌", "Plugins", "", "Marketplace & themes"),
            ];
            ui.vertical_centered(|ui| {
                ui.columns(4, |cols| {
                    for (i, (icon, title, key, desc)) in cards.iter().enumerate() {
                        let ui = &mut cols[i];
                        let card_w = ui.available_width() - 12.0;
                        let card_h = 74.0;
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(card_w, card_h),
                            egui::Sense::click(),
                        );
                        let hovered = resp.hovered();
                        let bg = if hovered {
                            Theme::lerp(theme.bg_secondary, theme.accent, 0.10)
                        } else {
                            theme.bg_secondary
                        };
                        ui.painter().rect_filled(rect, Rounding::same(10.0), bg);
                        ui.painter().rect_stroke(
                            rect,
                            Rounding::same(10.0),
                            Stroke::new(if hovered { 1.4 } else { 1.0 }, theme.border),
                        );
                        if hovered {
                            ui.painter().rect_stroke(rect.shrink(2.0), Rounding::same(8.0), Stroke::new(1.0, theme.accent.gamma_multiply(0.5)));
                        }
                        // icon
                        ui.painter().text(
                            pos2(rect.left() + 18.0, rect.top() + 20.0),
                            Align2::LEFT_CENTER,
                            *icon,
                            egui::FontId::proportional(20.0),
                            theme.accent,
                        );
                        // title
                        ui.painter().text(
                            pos2(rect.left() + 46.0, rect.top() + 20.0),
                            Align2::LEFT_CENTER,
                            *title,
                            egui::FontId::proportional(15.0),
                            theme.text_primary,
                        );
                        // shortcut badge
                        if !key.is_empty() {
                            ui.painter().text(
                                pos2(rect.right() - 10.0, rect.top() + 20.0),
                                Align2::RIGHT_CENTER,
                                *key,
                                egui::FontId::monospace(10.0),
                                theme.text_secondary,
                            );
                        }
                        // description
                        ui.painter().text(
                            pos2(rect.left() + 18.0, rect.bottom() - 16.0),
                            Align2::LEFT_CENTER,
                            *desc,
                            egui::FontId::proportional(11.0),
                            theme.text_secondary,
                        );
                        if resp.clicked() {
                            match i {
                                0 | 1 => {
                                    opened_file = Some(String::new());
                                    self.visible = false;
                                }
                                2 => {
                                    self.visible = false;
                                }
                                3 => {
                                    // plugins handled by caller toggling; no-op here
                                }
                                _ => {}
                            }
                        }
                    }
                });
            });

            ui.add_space(26.0);

            // ── Two columns: Recent files + Shortcuts ─────────────────────────
            ui.columns(2, |cols| {
                // Recent files
                let ui = &mut cols[0];
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("⏱ RECENT")
                            .size(11.0)
                            .strong()
                            .color(theme.text_secondary),
                    );
                });
                ui.add_space(6.0);
                if self.recent_files.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("No files yet — open one above ↑")
                                .size(12.0)
                                .color(theme.text_secondary),
                        );
                    });
                } else {
                    for file in self.recent_files.iter().take(6) {
                        let name = file.rsplit('/').next().unwrap_or(file);
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 26.0),
                            egui::Sense::click(),
                        );
                        let hovered = resp.hovered();
                        if hovered {
                            ui.painter().rect_filled(rect, Rounding::same(6.0), theme.bg_tertiary);
                        }
                        ui.painter().text(
                            pos2(rect.left() + 10.0, rect.center().y),
                            Align2::LEFT_CENTER,
                            format!("📄 {}", name),
                            egui::FontId::monospace(12.0),
                            if hovered { theme.accent } else { theme.text_primary },
                        );
                        if resp.clicked() {
                            opened_file = Some(file.clone());
                            self.visible = false;
                        }
                        ui.add_space(2.0);
                    }
                }

                // Shortcuts
                let ui = &mut cols[1];
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("⌨ SHORTCUTS")
                            .size(11.0)
                            .strong()
                            .color(theme.text_secondary),
                    );
                });
                ui.add_space(6.0);
                let shortcuts = [
                    ("Ctrl+P", "Command Palette"),
                    ("Ctrl+Space", "Complete / Emmet preview"),
                    ("Tab", "Expand Emmet"),
                    ("Ctrl+F / H", "Find / Replace"),
                    ("F5", "Run · Shift+F5 Debug"),
                    ("Ctrl+J", "Terminal"),
                    ("Ctrl+\\", "Split"),
                    ("Ctrl+W", "Close Tab"),
                ];
                for (key, desc) in &shortcuts {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 20.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().text(
                        pos2(rect.left() + 10.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        *key,
                        egui::FontId::monospace(10.5),
                        ACCENT,
                    );
                    ui.painter().text(
                        pos2(rect.left() + 96.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        *desc,
                        egui::FontId::proportional(11.0),
                        theme.text_secondary,
                    );
                }
            });
            ui.add_space(20.0);
        });

        opened_file
    }
}
