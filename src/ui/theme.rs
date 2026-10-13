use eframe::egui;
use egui::{Color32, Rounding, Stroke, Visuals};

pub struct Theme {
    pub dark_mode: bool,
    pub accent: Color32,
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub border: Color32,
    pub selection: Color32,
    pub line_highlight: Color32,
}

impl Theme {
    // VS Code Dark+ inspired
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            accent: Color32::from_rgb(206, 145, 120), // 🍊 نارنجی VS Code
            bg_primary: Color32::from_rgb(30, 30, 30), // دودی تیره
            bg_secondary: Color32::from_rgb(37, 37, 38), // دودی متوسط
            bg_tertiary: Color32::from_rgb(45, 45, 45), // دودی روشن
            text_primary: Color32::from_rgb(212, 212, 212), // سفید دودی
            text_secondary: Color32::from_rgb(133, 133, 133), // خاکستری
            border: Color32::from_rgb(60, 60, 60),    // خطوط
            selection: Color32::from_rgb(38, 79, 120), // آبی انتخاب
            line_highlight: Color32::from_rgb(42, 45, 50), // خط فعال
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        // Panel backgrounds
        visuals.window_fill = self.bg_primary;
        visuals.panel_fill = self.bg_primary;

        // Window styling
        visuals.window_rounding = Rounding::same(0.0);
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.window_stroke = Stroke::new(1.0, self.border);

        // Widget styling
        visuals.widgets.noninteractive.bg_fill = self.bg_secondary;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.noninteractive.rounding = Rounding::same(4.0);

        visuals.widgets.inactive.bg_fill = self.bg_tertiary;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        visuals.widgets.inactive.rounding = Rounding::same(4.0);

        visuals.widgets.hovered.bg_fill = self.bg_tertiary;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, self.accent);
        visuals.widgets.hovered.rounding = Rounding::same(4.0);

        visuals.widgets.active.bg_fill = self.selection;
        visuals.widgets.active.fg_stroke = Stroke::new(2.0, self.accent);
        visuals.widgets.active.rounding = Rounding::same(4.0);

        // Selection
        visuals.selection.bg_fill = self.selection;
        visuals.selection.stroke = Stroke::new(1.0, self.accent);

        // Text
        visuals.override_text_color = Some(self.text_primary);

        // Hyperlinks
        visuals.hyperlink_color = self.accent;

        // Menu
        visuals.menu_rounding = Rounding::same(6.0);

        // Apply
        ctx.set_visuals(visuals);
        ctx.set_pixels_per_point(1.2); // Scale for readability
    }
}
