use eframe::egui;

pub struct CodeEditor {
    pub content: String,
    pub language: String,
    pub cursor_pos: usize,
    pub scroll: f32,
}

impl CodeEditor {
    pub fn new() -> Self {
        Self {
            content: String::from("// 💀 Phobos IDE — The God of Fear\n// Start coding here...\n"),
            language: String::from("plaintext"),
            cursor_pos: 0,
            scroll: 0.0,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut content = self.content.clone();
        let response = ui.add(
            egui::TextEdit::multiline(&mut content)
                .desired_width(f32::INFINITY)
                .desired_rows(30)
                .font(egui::TextStyle::Monospace)
                .code_editor()
        );
        if response.changed() {
            self.content = content;
        }
    }
}
