use eframe::egui;

pub struct Theme {
    pub dark_mode: bool,
}

impl Theme {
    pub fn dark() -> Self {
        Self { dark_mode: true }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }
}
