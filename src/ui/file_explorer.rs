use eframe::egui;
use std::path::Path;

pub struct FileExplorer {
    pub current_dir: String,
    entries: Vec<FileEntry>,
    selected: Option<usize>,
    pub on_file_open: Option<String>,
}

#[derive(Clone)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    icon: String,
    size: String,
}

impl FileExplorer {
    pub fn new() -> Self {
        let mut explorer = Self {
            current_dir: String::from("."),
            entries: Vec::new(),
            selected: None,
            on_file_open: None,
        };
        explorer.refresh();
        explorer
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let icon = if is_dir {
                    "📁"
                } else {
                    self.file_icon(&name)
                };
                let size = if !is_dir {
                    if let Ok(meta) = entry.metadata() {
                        self.format_size(meta.len())
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                self.entries.push(FileEntry {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir,
                    icon: icon.to_string(),
                    size,
                });
            }
        }
        self.entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
    }

    fn file_icon(&self, name: &str) -> &str {
        match name.split('.').last().unwrap_or("") {
            "rs" => "🦀",
            "py" => "🐍",
            "js" => "🟨",
            "ts" => "🟦",
            "html" => "🌐",
            "css" => "🎨",
            "json" => "📋",
            "toml" => "⚙️",
            "md" => "📘",
            "c" | "h" => "🔵",
            "cpp" | "hpp" => "🔵",
            "java" => "☕",
            "go" => "🔷",
            "php" => "🐘",
            "sh" | "bash" => "🐚",
            "sql" => "🗄️",
            _ => "📄",
        }
    }

    fn format_size(&self, bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> &mut Self {
        ui.heading("📁 Explorer");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("⬆").clicked() {
                if let Some(parent) = Path::new(&self.current_dir).parent() {
                    self.current_dir = parent.to_string_lossy().to_string();
                    self.refresh();
                }
            }
            if ui.button("🔄").clicked() {
                self.refresh();
            }
            ui.label(&self.current_dir);
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, entry) in self.entries.clone().iter().enumerate() {
                let selected = self.selected == Some(i);
                let response = ui.selectable_label(
                    selected,
                    format!("{} {}  {}", entry.icon, entry.name, entry.size),
                );

                if response.clicked() {
                    self.selected = Some(i);
                    if entry.is_dir {
                        self.current_dir = entry.path.clone();
                        self.refresh();
                    } else {
                        // Fire file open event
                        self.on_file_open = Some(entry.path.clone());
                    }
                }
            }
        });

        self
    }
}
