use eframe::egui;

pub struct TabManager {
    tabs: Vec<Tab>,
    active: usize,
}

pub struct Tab {
    name: String,
    modified: bool,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab { name: "Untitled-1".into(), modified: false }],
            active: 0,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (i, tab) in self.tabs.iter().enumerate() {
                let label = if tab.modified {
                    format!("● {}", tab.name)
                } else {
                    tab.name.clone()
                };
                if ui.selectable_label(i == self.active, label).clicked() {
                    self.active = i;
                }
            }
            if ui.button("➕").clicked() {
                self.tabs.push(Tab { name: format!("Untitled-{}", self.tabs.len() + 1), modified: false });
            }
        });
    }
}
