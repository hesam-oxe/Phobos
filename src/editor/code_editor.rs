use eframe::egui;
use egui::Color32;

pub struct CodeEditor {
    pub content: String,
    pub language: String,
    pub cursor_pos: usize,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub minimap: bool,
    pub bracket_matching: bool,
    pub auto_indent: bool,
    pub multi_cursor_positions: Vec<usize>,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
}

impl CodeEditor {
    pub fn new() -> Self {
        Self {
            content: String::from(
                "// 💀 Phobos IDE — The God of Fear\n// 25 Languages. Zero Plugins. Pure Power.\n\nfn main() {\n    println!(\"Phobos crushes VS Code!\");\n}\n"
            ),
            language: String::from("rust"),
            cursor_pos: 0,
            line_numbers: true,
            word_wrap: true,
            minimap: true,
            bracket_matching: true,
            auto_indent: true,
            multi_cursor_positions: Vec::new(),
            selection_start: None,
            selection_end: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let editor_height = ui.available_height() - 30.0;
        
        // Main editor area with line numbers
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut content = self.content.clone();
                
                // Line numbers panel
                if self.line_numbers {
                    ui.horizontal(|ui| {
                        // Line numbers gutter
                        let line_count = content.lines().count();
                        let mut line_numbers_text = String::new();
                        for i in 1..=line_count.max(1) {
                            line_numbers_text.push_str(&format!("{}\n", i));
                        }
                        
                        ui.add_sized([50.0, editor_height],
                            egui::TextEdit::multiline(&mut line_numbers_text)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false)
                                .desired_width(50.0)
                        );
                        
                        ui.separator();
                        
                        // Code editor
                        let response = ui.add_sized([ui.available_width(), editor_height],
                            egui::TextEdit::multiline(&mut content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                        );
                        
                        if response.changed() {
                            self.content = content;
                            // Auto-indent
                            if self.auto_indent {
                                self.auto_indent_content();
                            }
                        }
                    });
                } else {
                    let response = ui.add_sized([ui.available_width(), editor_height],
                        egui::TextEdit::multiline(&mut content)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                    );
                    
                    if response.changed() {
                        self.content = content;
                        if self.auto_indent {
                            self.auto_indent_content();
                        }
                    }
                }
            });
        
        // Status bar info
        ui.horizontal(|ui| {
            let lines = self.content.lines().count();
            let chars = self.content.len();
            
            ui.label(format!("📝 {} lines", lines));
            ui.separator();
            ui.label(format!("🔤 {} chars", chars));
            ui.separator();
            ui.label(format!("🌐 {}", self.language));
            ui.separator();
            
            // Bracket match indicator
            if self.bracket_matching {
                if let Some(pos) = self.find_matching_bracket() {
                    ui.label(format!("✅ Bracket matched at {}", pos));
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("UTF-8 | LF | Rust 🦀");
            });
        });
    }

    fn auto_indent_content(&mut self) {
        let mut result = String::new();
        let mut indent_level: usize = 0;
        let indent_str = "    "; // 4 spaces
        
        for line in self.content.lines() {
            let trimmed = line.trim();
            
            // Decrease indent for closing brackets
            if trimmed.starts_with('}') || trimmed.starts_with(')') || trimmed.starts_with(']') {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // Add indent
            result.push_str(&indent_str.repeat(indent_level));
            result.push_str(trimmed);
            result.push('\n');
            
            // Increase indent for opening brackets
            let open_count = trimmed.matches('{').count() 
                + trimmed.matches('(').count() 
                + trimmed.matches('[').count();
            let close_count = trimmed.matches('}').count() 
                + trimmed.matches(')').count() 
                + trimmed.matches(']').count();
            
            if open_count > close_count {
                indent_level += open_count - close_count;
            }
        }
        
        // Remove trailing newline if original didn't have one
        if !self.content.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        
        if result != self.content {
            self.content = result;
        }
    }

    fn find_matching_bracket(&self) -> Option<usize> {
        let brackets = [('{', '}'), ('(', ')'), ('[', ']')];
        
        if self.cursor_pos >= self.content.len() {
            return None;
        }
        
        let current_char = self.content[self.cursor_pos..].chars().next()?;
        
        for (open, close) in &brackets {
            if current_char == *open {
                // Find matching close
                let mut depth = 1;
                for (i, c) in self.content[self.cursor_pos + 1..].char_indices() {
                    if c == *open { depth += 1; }
                    if c == *close { depth -= 1; }
                    if depth == 0 {
                        return Some(self.cursor_pos + 1 + i);
                    }
                }
            } else if current_char == *close {
                // Find matching open
                let mut depth = 1;
                for (i, c) in self.content[..self.cursor_pos].char_indices().rev() {
                    if c == *close { depth += 1; }
                    if c == *open { depth -= 1; }
                    if depth == 0 {
                        return Some(i);
                    }
                }
            }
        }
        
        None
    }

    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
    }
    
    pub fn insert_text(&mut self, text: &str) {
        self.content.insert_str(self.cursor_pos.min(self.content.len()), text);
        self.cursor_pos += text.len();
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_pos < self.content.len() {
            self.content.remove(self.cursor_pos);
        }
    }
    
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.content.remove(self.cursor_pos);
        }
    }
}
