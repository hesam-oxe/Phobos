use eframe::egui;
use egui::Color32;

pub struct TerminalPanel {
    pub visible: bool,
    pub input: String,
    pub output: Vec<String>,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub current_dir: String,
}

impl TerminalPanel {
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::new(),
            output: vec![
                "💀 Phobos Terminal v1.0".into(),
                "Type 'help' for available commands".into(),
                String::new(),
            ],
            history: Vec::new(),
            history_index: None,
            current_dir: String::from("~"),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("💻 Terminal");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑️ Clear").clicked() {
                    self.output.clear();
                }
                if ui.button("❌").clicked() {
                    self.visible = false;
                }
            });
        });
        ui.separator();

        // Output area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &self.output {
                    if line.starts_with("$ ") {
                        ui.label(
                            egui::RichText::new(line)
                                .color(Color32::from_rgb(0, 255, 0))
                                .size(12.0)
                        );
                    } else if line.starts_with("Error:") {
                        ui.label(
                            egui::RichText::new(line)
                                .color(Color32::from_rgb(255, 80, 80))
                                .size(12.0)
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(line)
                                .color(Color32::from_rgb(200, 200, 200))
                                .size(12.0)
                        );
                    }
                }
            });

        // Input
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.input)
                .hint_text(format!("{} $ ", self.current_dir))
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
        );

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.execute_command();
        }
    }

    fn execute_command(&mut self) {
        let cmd = self.input.trim().to_string();
        if cmd.is_empty() {
            return;
        }

        self.output.push(format!("$ {}", cmd));
        self.history.push(cmd.clone());
        self.history_index = None;

        match cmd.as_str() {
            "help" => {
                self.output.push("Available commands:".into());
                self.output.push("  help     - Show this message".into());
                self.output.push("  clear    - Clear terminal".into());
                self.output.push("  ls       - List files".into());
                self.output.push("  pwd      - Print working directory".into());
                self.output.push("  date     - Show current date".into());
                self.output.push("  whoami   - Show current user".into());
                self.output.push("  version  - Show Phobos version".into());
            }
            "clear" => {
                self.output.clear();
            }
            "ls" => {
                if let Ok(entries) = std::fs::read_dir(".") {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let icon = if entry.path().is_dir() { "📁" } else { "📄" };
                        self.output.push(format!("  {} {}", icon, name));
                    }
                }
            }
            "pwd" => {
                if let Ok(path) = std::env::current_dir() {
                    self.output.push(path.to_string_lossy().to_string());
                }
            }
            "date" => {
                self.output.push(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            }
            "whoami" => {
                self.output.push("💀 Phobos User".into());
            }
            "version" => {
                self.output.push("Phobos IDE v0.3.0 — The God of Fear".into());
            }
            _ => {
                self.output.push(format!("Error: command not found: {}", cmd));
            }
        }

        self.input.clear();
    }
}
