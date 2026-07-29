use eframe::egui;

pub struct SearchPanel {
    pub visible: bool,
    pub search_query: String,
    pub replace_query: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub matches: Vec<usize>,
    pub current_match: usize,
}

impl SearchPanel {
    pub fn new() -> Self {
        Self {
            visible: false,
            search_query: String::new(),
            replace_query: String::new(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            matches: Vec::new(),
            current_match: 0,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, content: &str) -> Option<String> {
        if !self.visible {
            return None;
        }
        
        let mut result = None;
        
        ui.horizontal(|ui| {
            ui.label("🔍");
            
            let search_response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search...")
                    .desired_width(200.0)
            );
            
            if search_response.changed() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.find_matches(content);
            }
            
            ui.label(format!("{} matches", self.matches.len()));
            
            if ui.button("⬆").clicked() && !self.matches.is_empty() {
                self.current_match = (self.current_match + self.matches.len() - 1) % self.matches.len();
            }
            if ui.button("⬇").clicked() && !self.matches.is_empty() {
                self.current_match = (self.current_match + 1) % self.matches.len();
            }
            
            ui.separator();
            
            // Replace
            ui.label("🔄");
            ui.add(egui::TextEdit::singleline(&mut self.replace_query)
                .hint_text("Replace with...")
                .desired_width(150.0));
            
            if ui.button("Replace").clicked() {
                if let Some(&pos) = self.matches.get(self.current_match) {
                    let mut new_content = content.to_string();
                    new_content.replace_range(pos..pos + self.search_query.len(), &self.replace_query);
                    result = Some(new_content);
                }
            }
            
            if ui.button("Replace All").clicked() {
                let mut new_content = content.to_string();
                for &pos in self.matches.iter().rev() {
                    new_content.replace_range(pos..pos + self.search_query.len(), &self.replace_query);
                }
                result = Some(new_content);
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.case_sensitive, "Aa");
                ui.checkbox(&mut self.whole_word, "W");
                ui.checkbox(&mut self.regex, ".*");
                if ui.button("❌").clicked() {
                    self.visible = false;
                }
            });
        });
        
        result
    }
    
    fn find_matches(&mut self, content: &str) {
        self.matches.clear();
        self.current_match = 0;
        
        if self.search_query.is_empty() {
            return;
        }
        
        let search = if self.case_sensitive {
            self.search_query.clone()
        } else {
            self.search_query.to_lowercase()
        };
        
        let content_lower = if self.case_sensitive {
            content.to_string()
        } else {
            content.to_lowercase()
        };
        
        let mut start = 0;
        while let Some(pos) = content_lower[start..].find(&search) {
            let abs_pos = start + pos;
            if self.whole_word {
                let before = abs_pos == 0 || !content.as_bytes().get(abs_pos - 1).map_or(false, |&b| b.is_ascii_alphanumeric());
                let after = abs_pos + search.len() >= content.len() || !content.as_bytes().get(abs_pos + search.len()).map_or(false, |&b| b.is_ascii_alphanumeric());
                if before && after {
                    self.matches.push(abs_pos);
                }
            } else {
                self.matches.push(abs_pos);
            }
            start = abs_pos + search.len();
            if start >= content.len() {
                break;
            }
        }
    }
}
