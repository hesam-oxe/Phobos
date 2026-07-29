use eframe::egui;

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

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mut close_idx = None;
            
            for (i, tab) in self.tabs.iter().enumerate() {
                let label = if tab.modified {
                    format!("● {}", tab.name)
                } else {
                    tab.name.clone()
                };
                
                let response = ui.selectable_label(i == self.active, &label);
                
                if response.clicked() {
                    self.active = i;
                }
                
                // Close button on hover
                if response.hovered() {
                    if ui.small_button("✕").clicked() {
                        close_idx = Some(i);
                    }
                }
            }
            
            if let Some(idx) = close_idx {
                self.close_tab(idx);
            }
            
            if ui.button("➕").clicked() {
                self.tabs.push(Tab {
                    name: format!("Untitled-{}", self.tabs.len() + 1),
                    modified: false,
                    path: None,
                });
                self.active = self.tabs.len() - 1;
            }
        });
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
