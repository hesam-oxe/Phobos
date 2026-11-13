use eframe::egui;
use egui::{Color32, Rounding, Stroke, Visuals};

/// A complete visual identity for Phobos.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub dark_mode: bool,
    /// primary accent (glow, underlines, logo)
    pub accent: Color32,
    /// secondary accent (gradient partner, badges)
    pub accent2: Color32,
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub border: Color32,
    pub selection: Color32,
    pub line_highlight: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
}

impl Theme {
    // ── 💀 Phobos God — deep ember on void black ────────────────────────────
    pub fn god() -> Self {
        Self {
            name: "Phobos God",
            dark_mode: true,
            accent:  Color32::from_rgb(255, 122, 69),   // ember orange
            accent2: Color32::from_rgb(171, 84, 255),   // violet ember
            bg_primary:   Color32::from_rgb(16, 16, 22),
            bg_secondary: Color32::from_rgb(24, 24, 32),
            bg_tertiary:  Color32::from_rgb(34, 34, 45),
            text_primary:   Color32::from_rgb(228, 228, 240),
            text_secondary: Color32::from_rgb(120, 122, 140),
            border: Color32::from_rgb(48, 48, 62),
            selection: Color32::from_rgb(58, 44, 92),
            line_highlight: Color32::from_rgb(30, 28, 42),
            success: Color32::from_rgb(80, 220, 140),
            warning: Color32::from_rgb(240, 190, 70),
            error:   Color32::from_rgb(255, 90, 95),
        }
    }

    // ── Tokyo Night ─────────────────────────────────────────────────────────
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night",
            dark_mode: true,
            accent:  Color32::from_rgb(122, 162, 247),
            accent2: Color32::from_rgb(187, 154, 247),
            bg_primary:   Color32::from_rgb(26, 27, 38),
            bg_secondary: Color32::from_rgb(31, 33, 47),
            bg_tertiary:  Color32::from_rgb(41, 46, 66),
            text_primary:   Color32::from_rgb(169, 177, 214),
            text_secondary: Color32::from_rgb(86, 95, 137),
            border: Color32::from_rgb(42, 46, 66),
            selection: Color32::from_rgb(51, 61, 100),
            line_highlight: Color32::from_rgb(28, 30, 44),
            success: Color32::from_rgb(115, 218, 202),
            warning: Color32::from_rgb(224, 175, 104),
            error:   Color32::from_rgb(247, 118, 142),
        }
    }

    // ── Dracula ─────────────────────────────────────────────────────────────
    pub fn dracula() -> Self {
        Self {
            name: "Dracula",
            dark_mode: true,
            accent:  Color32::from_rgb(189, 147, 249),
            accent2: Color32::from_rgb(255, 121, 198),
            bg_primary:   Color32::from_rgb(40, 42, 54),
            bg_secondary: Color32::from_rgb(45, 47, 62),
            bg_tertiary:  Color32::from_rgb(54, 57, 80),
            text_primary:   Color32::from_rgb(248, 248, 242),
            text_secondary: Color32::from_rgb(139, 143, 172),
            border: Color32::from_rgb(68, 71, 90),
            selection: Color32::from_rgb(68, 71, 90),
            line_highlight: Color32::from_rgb(52, 55, 74),
            success: Color32::from_rgb(80, 250, 123),
            warning: Color32::from_rgb(241, 250, 140),
            error:   Color32::from_rgb(255, 85, 85),
        }
    }

    // ── Nord ────────────────────────────────────────────────────────────────
    pub fn nord() -> Self {
        Self {
            name: "Nord",
            dark_mode: true,
            accent:  Color32::from_rgb(136, 192, 208),
            accent2: Color32::from_rgb(180, 142, 173),
            bg_primary:   Color32::from_rgb(46, 52, 64),
            bg_secondary: Color32::from_rgb(52, 59, 72),
            bg_tertiary:  Color32::from_rgb(59, 66, 82),
            text_primary:   Color32::from_rgb(236, 239, 244),
            text_secondary: Color32::from_rgb(136, 148, 160),
            border: Color32::from_rgb(67, 76, 94),
            selection: Color32::from_rgb(61, 80, 108),
            line_highlight: Color32::from_rgb(56, 62, 76),
            success: Color32::from_rgb(163, 190, 140),
            warning: Color32::from_rgb(235, 203, 139),
            error:   Color32::from_rgb(191, 97, 106),
        }
    }

    /// Classic Dark+ (the historical default).
    pub fn dark() -> Self {
        Self {
            name: "Dark+",
            dark_mode: true,
            accent: Color32::from_rgb(206, 145, 120),
            accent2: Color32::from_rgb(106, 153, 222),
            bg_primary:   Color32::from_rgb(30, 30, 30),
            bg_secondary: Color32::from_rgb(37, 37, 38),
            bg_tertiary:  Color32::from_rgb(45, 45, 45),
            text_primary:   Color32::from_rgb(212, 212, 212),
            text_secondary: Color32::from_rgb(133, 133, 133),
            border: Color32::from_rgb(60, 60, 60),
            selection: Color32::from_rgb(38, 79, 120),
            line_highlight: Color32::from_rgb(42, 45, 50),
            success: Color32::from_rgb(106, 200, 120),
            warning: Color32::from_rgb(220, 180, 80),
            error:   Color32::from_rgb(240, 100, 100),
        }
    }

    pub fn all() -> Vec<Theme> {
        vec![
            Self::god(),
            Self::tokyo_night(),
            Self::dracula(),
            Self::nord(),
            Self::dark(),
        ]
    }

    pub fn names() -> Vec<&'static str> {
        Self::all().into_iter().map(|t| t.name).collect()
    }

    pub fn by_name(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Blend two colors (t in 0..1).
    pub fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        Color32::from_rgba_premultiplied(
            (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
            (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
            (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
            255,
        )
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        // Panel & window backgrounds
        visuals.panel_fill = self.bg_primary;
        visuals.window_fill = self.bg_secondary;

        // Window styling — floating, softly shadowed
        visuals.window_rounding = Rounding::same(10.0);
        visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 10.0),
            blur: 40.0,
            spread: 2.0,
            color: Color32::from_black_alpha(110),
        };
        visuals.window_stroke = Stroke::new(1.0, self.border);
        visuals.menu_rounding = Rounding::same(10.0);
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 6.0),
            blur: 24.0,
            spread: 0.0,
            color: Color32::from_black_alpha(90),
        };

        // Widget styling
        visuals.widgets.noninteractive.bg_fill = self.bg_secondary;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

        visuals.widgets.inactive.bg_fill = self.bg_tertiary;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.inactive.rounding = Rounding::same(6.0);

        visuals.widgets.hovered.bg_fill = self.bg_tertiary;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.2, self.accent);
        visuals.widgets.hovered.rounding = Rounding::same(6.0);

        visuals.widgets.active.bg_fill = self.selection;
        visuals.widgets.active.fg_stroke = Stroke::new(2.0, self.accent);
        visuals.widgets.active.rounding = Rounding::same(6.0);

        visuals.selection.bg_fill = self.selection;
        visuals.selection.stroke = Stroke::new(1.0, self.accent);

        visuals.override_text_color = Some(self.text_primary);
        visuals.hyperlink_color = self.accent;
        visuals.extreme_bg_color = self.bg_primary;

        ctx.style_mut(|style| {
            style.visuals = visuals.clone();
            style.animation_time = 0.12;
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
            style.spacing.menu_margin = egui::Margin::symmetric(10.0, 7.0);
            style.spacing.scroll.bar_width = 10.0;
            style.spacing.scroll.bar_inner_margin = 3.0;
            style.spacing.scroll.bar_outer_margin = 2.0;
            style.spacing.interact_size.y = 26.0;
        });

        ctx.set_visuals(visuals);
        ctx.set_pixels_per_point(1.2);
    }
}

/// Shared accent color used across custom widgets (tab underline, gutter, etc.)
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(255, 122, 69);
/// Subtle border color shared across custom widgets
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(48, 48, 62);

// ─────────────────────────────────────────────────────────────────────────────
// God-UI painting helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Paint a smooth linear gradient rectangle (horizontal or vertical).
pub fn gradient_rect(
    painter: &egui::Painter,
    rect: egui::Rect,
    from: Color32,
    to: Color32,
    horizontal: bool,
) {
    use egui::epaint::{Mesh, Vertex, Pos2};
    let mut mesh = Mesh::default();
    let (a, b) = if horizontal {
        (rect.left_top(), rect.right_top())
    } else {
        (rect.left_top(), rect.left_bottom())
    };
    let c1 = Color32::from_rgba_unmultiplied(from.r(), from.g(), from.b(), from.a());
    let c2 = Color32::from_rgba_unmultiplied(to.r(), to.g(), to.b(), to.a());
    let v = |p: egui::Pos2, c: Color32| Vertex {
        pos: p,
        uv: egui::epaint::WHITE_UV,
        color: c,
    };
    mesh.vertices.push(v(a, c1));
    mesh.vertices.push(v(b, c2));
    mesh.vertices.push(v(egui::pos2(b.x, rect.bottom()), c2));
    mesh.vertices.push(v(egui::pos2(a.x, rect.bottom()), c1));
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    painter.add(mesh);
}

/// Draw a rounded chip (badge) with icon + text, returns the response.
pub fn chip(
    ui: &mut egui::Ui,
    icon: &str,
    text: &str,
    color: Color32,
) -> egui::Response {
    let galley_icon = ui.painter().layout_no_wrap(
        icon.to_string(),
        egui::FontId::monospace(12.0),
        color,
    );
    let galley_text = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::monospace(12.0),
        ui.style().visuals.text_color(),
    );
    let w = galley_icon.size().x + galley_text.size().x + 18.0;
    let h = 20.0f32.max(galley_text.size().y + 6.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            Rounding::same(10.0),
            Color32::from_black_alpha(40),
        );
    }
    ui.painter().rect_stroke(rect, Rounding::same(10.0), Stroke::new(1.0, color.gamma_multiply(0.45)));
    let icon_w = galley_icon.size().x;
    let icon_h = galley_icon.size().y;
    let text_h = galley_text.size().y;
    let mut x = rect.left() + 8.0;
    let cy = rect.center().y;
    ui.painter().galley(egui::pos2(x, cy - icon_h / 2.0), galley_icon, color);
    x += icon_w + 4.0;
    ui.painter().galley(egui::pos2(x, cy - text_h / 2.0), galley_text, ui.style().visuals.text_color());
    response
}

/// Soft radial-ish glow behind a rect region (few layered strokes).
pub fn glow_line(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    color: Color32,
    thickness: f32,
) {
    for (i, alpha) in [25u8, 55, 110, 200].iter().enumerate() {
        let w = thickness + (3.0 - i as f32) * 2.5;
        painter.line_segment(
            [from, to],
            Stroke::new(w, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), *alpha)),
        );
    }
}
